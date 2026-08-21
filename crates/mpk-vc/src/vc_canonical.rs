//! Canonical generation, import, linked validation, and hashing for VC v1.

use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use std::fmt;

use serde::{Deserialize, Serialize};

use crate::call_wp::{program_declaration_name, ProgramDeclarationKind};
use crate::canonical_json::{
    canonical_json_bytes, parse_strict_json, StrictJsonError, StrictJsonLimits, StrictJsonValue,
    MAX_SAFE_JSON_INTEGER,
};
use crate::hash::{hash_canonical_json, HashDomain};
use crate::program_wp::generate_program_vcs;
use crate::semantic_profile::validate_semantic_parameters;
use crate::source_manifest::{SourceManifestStage, ValidatedSourceManifest, ValidatedVcIdentity};
use crate::type_encode::encode_vir_type;
use crate::vc::{
    VcBinder, VcDocument, VcFunction, VcGroup, VcGroupKind, VcMember, VcSourceContext,
    VcSourceFunction, VcTerm, VcTypeTerm, VC_SCHEMA_VERSION, VERIFICATION_LIMIT_PROFILE,
};
use crate::verification_limits::{
    validate_grouped_theorem_limits, validate_vc_stream_limits, validate_verification_limit,
};
use crate::vir::{DecimalInteger, LowercaseSha256, VirModule, VIR_SCHEMA_VERSION};
use crate::vir_canonical::{contract_hash, vir_hash};
use crate::vir_validate::validate_vir;

pub const VC_HASH_DOMAIN: HashDomain = HashDomain::new("MPK-VC-1.0");

const VC_JSON_LIMITS: StrictJsonLimits =
    StrictJsonLimits::new(268_435_456, 268_435_456, 768, 1_048_576);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum VcValidationPhase {
    Transport,
    Shape,
    Scalar,
    StreamLimits,
    Linkage,
    Members,
    Groups,
    Dependencies,
    TheoremLimits,
    CanonicalSize,
    CanonicalTransport,
    Hash,
}

impl VcValidationPhase {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Transport => "transport",
            Self::Shape => "shape",
            Self::Scalar => "scalar",
            Self::StreamLimits => "stream_limits",
            Self::Linkage => "linkage",
            Self::Members => "members",
            Self::Groups => "groups",
            Self::Dependencies => "dependencies",
            Self::TheoremLimits => "theorem_limits",
            Self::CanonicalSize => "canonical_size",
            Self::CanonicalTransport => "canonical_transport",
            Self::Hash => "hash",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VcValidationError {
    phase: VcValidationPhase,
    code: &'static str,
    detail: String,
}

impl VcValidationError {
    pub fn new(phase: VcValidationPhase, code: &'static str, detail: impl Into<String>) -> Self {
        Self {
            phase,
            code,
            detail: detail.into(),
        }
    }

    pub const fn phase(&self) -> VcValidationPhase {
        self.phase
    }

    pub const fn code(&self) -> &'static str {
        self.code
    }

    pub fn detail(&self) -> &str {
        &self.detail
    }
}

impl fmt::Display for VcValidationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{} during {}: {}",
            self.code,
            self.phase.as_str(),
            self.detail
        )
    }
}

impl Error for VcValidationError {}

#[derive(Clone, Debug)]
pub struct ValidatedVcDocument {
    document: VcDocument,
    canonical_bytes: Vec<u8>,
    hash: LowercaseSha256,
}

impl ValidatedVcDocument {
    pub fn document(&self) -> &VcDocument {
        &self.document
    }

    pub fn canonical_bytes(&self) -> &[u8] {
        &self.canonical_bytes
    }

    pub fn hash(&self) -> &LowercaseSha256 {
        &self.hash
    }

    pub fn validated_identity(&self) -> Result<ValidatedVcIdentity, VcValidationError> {
        ValidatedVcIdentity::new(
            self.document.input_set_hash.clone(),
            self.document.source_ir_schema.clone(),
            self.document.source_ir_hash.clone(),
            self.document.semantic_profile,
            self.document.semantic_parameters.clone(),
            self.hash.as_str().to_owned(),
        )
        .map_err(|error| {
            VcValidationError::new(
                VcValidationPhase::Linkage,
                "VC_SOURCE_LINKAGE",
                error.to_string(),
            )
        })
    }
}

/// Imports canonical VC bytes against the complete linked source projection.
pub fn import_vc_v1_json(
    input: &[u8],
    source: &VcSourceContext,
) -> Result<ValidatedVcDocument, VcValidationError> {
    let strict = parse_strict_json(input, VC_JSON_LIMITS).map_err(map_transport_error)?;
    validate_root_shape(&strict)?;

    let mut deserializer = serde_json::Deserializer::from_slice(input);
    deserializer.disable_recursion_limit();
    let document = VcDocument::deserialize(&mut deserializer).map_err(|error| {
        VcValidationError::new(VcValidationPhase::Shape, "VC_SHAPE", error.to_string())
    })?;

    validate_scalars(&document)?;
    validate_vc_stream_limits(&document).map_err(|error| {
        VcValidationError::new(
            VcValidationPhase::StreamLimits,
            error.code(),
            error.to_string(),
        )
    })?;
    validate_linkage(&document, source)?;
    validate_members(&document, source)?;
    validate_groups(&document)?;
    validate_dependencies(&document, source)?;
    validate_grouped_theorem_limits(&document).map_err(|error| {
        VcValidationError::new(
            VcValidationPhase::TheoremLimits,
            error.code(),
            error.to_string(),
        )
    })?;
    let canonical = canonical_json_bytes(&strict).map_err(|error| {
        VcValidationError::new(VcValidationPhase::Shape, "VC_SHAPE", error.to_string())
    })?;
    validate_verification_limit(
        "canonical_vc_json_bytes",
        u64::try_from(canonical.len()).unwrap_or(u64::MAX),
    )
    .map_err(|error| {
        VcValidationError::new(
            VcValidationPhase::CanonicalSize,
            error.code(),
            error.to_string(),
        )
    })?;

    if input != canonical {
        return Err(VcValidationError::new(
            VcValidationPhase::CanonicalTransport,
            "VC_CANONICAL_TRANSPORT",
            "VC transport is not byte-identical JCS",
        ));
    }

    let recomputed = vc_hash(&document)?;
    if recomputed.as_str() != document.vc_hash {
        return Err(VcValidationError::new(
            VcValidationPhase::Hash,
            "VC_HASH",
            "vc_hash does not match the MPK-VC-1.0 preimage",
        ));
    }

    Ok(ValidatedVcDocument {
        document,
        canonical_bytes: canonical,
        hash: recomputed,
    })
}

/// Deterministically generates VC v1 from a linked source projection. This is
/// the shared implementation used by production generation and conformance
/// fixtures.
pub fn generate_vc_v1_from_context(
    source: &VcSourceContext,
) -> Result<ValidatedVcDocument, VcValidationError> {
    let functions = source
        .functions
        .iter()
        .map(build_function)
        .collect::<Vec<_>>();
    let mut document = VcDocument {
        schema: VC_SCHEMA_VERSION.to_owned(),
        source_ir_schema: source.source_ir_schema.clone(),
        source_ir_hash: source.source_ir_hash.clone(),
        input_set_hash: source.input_set_hash.clone(),
        semantic_profile: source.semantic_profile,
        semantic_parameters: source.semantic_parameters.clone(),
        verification_limit_profile: source.verification_limit_profile.clone(),
        functions,
        vc_hash: "0".repeat(64),
    };

    // Generation enforces the same pre-canonical phase order as import before
    // it serializes or hashes the completed in-memory document. In particular,
    // an oversized source projection cannot turn a member/node/depth limit
    // into a later canonical-size or hash failure.
    validate_scalars(&document)?;
    validate_vc_stream_limits(&document).map_err(|error| {
        VcValidationError::new(
            VcValidationPhase::StreamLimits,
            error.code(),
            error.to_string(),
        )
    })?;
    validate_linkage(&document, source)?;
    validate_members(&document, source)?;
    validate_groups(&document)?;
    validate_dependencies(&document, source)?;
    validate_grouped_theorem_limits(&document).map_err(|error| {
        VcValidationError::new(
            VcValidationPhase::TheoremLimits,
            error.code(),
            error.to_string(),
        )
    })?;
    let placeholder = canonical_vc_json(&document)?;
    validate_verification_limit(
        "canonical_vc_json_bytes",
        u64::try_from(placeholder.len()).unwrap_or(u64::MAX),
    )
    .map_err(|error| {
        VcValidationError::new(
            VcValidationPhase::CanonicalSize,
            error.code(),
            error.to_string(),
        )
    })?;
    document.vc_hash = vc_hash(&document)?.as_str().to_owned();
    let canonical = canonical_vc_json(&document)?;
    import_vc_v1_json(&canonical, source)
}

/// Production entry point: validates exact VIR/manifest identities, generates
/// program members, and returns no partial document on any failure.
pub fn generate_vc_v1(
    vir: &VirModule,
    manifest: &ValidatedSourceManifest,
) -> Result<ValidatedVcDocument, VcValidationError> {
    validate_vir(vir).map_err(|error| source_error(error.to_string()))?;
    if manifest.stage() != SourceManifestStage::Frontend {
        return Err(source_error(
            "VC generation requires a frontend-stage manifest",
        ));
    }
    let manifest_value = manifest.manifest();
    let recomputed_vir = vir_hash(vir).map_err(|error| source_error(error.to_string()))?;
    if recomputed_vir != vir.vir_hash
        || manifest_value.vir_hash != vir.vir_hash.as_str()
        || manifest_value.semantic_profile != vir.semantic_profile
        || manifest_value.semantic_parameters != vir.semantic_parameters
    {
        return Err(source_error(
            "validated VIR and source-manifest repeated identities differ",
        ));
    }

    let generated = generate_program_vcs(vir).map_err(|error| {
        let (phase, code) = if error.code().starts_with("VC_LIMIT_") {
            (VcValidationPhase::StreamLimits, error.code())
        } else {
            (VcValidationPhase::Members, "VC_MEMBER_SET")
        };
        VcValidationError::new(phase, code, error.to_string())
    })?;
    let mut functions_by_id = BTreeMap::new();
    for unit in &vir.units {
        for function in &unit.functions {
            functions_by_id.insert(function.id.as_str(), (unit, function));
        }
    }

    let mut source_functions = Vec::with_capacity(generated.functions.len());
    for generated_function in &generated.functions {
        let (unit, function) = functions_by_id
            .get(generated_function.function_id.as_str())
            .ok_or_else(|| source_error("generated function is absent from VIR"))?;
        let parameters = function
            .params
            .iter()
            .map(|parameter| {
                encode_vir_type(
                    vir.semantic_profile,
                    &vir.semantic_parameters,
                    &unit.type_decls,
                    &parameter.r#type,
                )
                .map(|encoded| VcBinder {
                    id: parameter.id.clone(),
                    r#type: VcTypeTerm::from(&encoded),
                })
                .map_err(|error| source_error(error.to_string()))
            })
            .collect::<Result<Vec<_>, _>>()?;
        let requires = generated_function
            .requires
            .iter()
            .map(VcTerm::try_from)
            .collect::<Result<Vec<_>, _>>()
            .map_err(|error| source_error(error.to_string()))?;
        let regenerated_members = generated_function
            .members
            .iter()
            .map(VcMember::try_from)
            .collect::<Result<Vec<_>, _>>()
            .map_err(|error| source_error(error.to_string()))?;
        source_functions.push(VcSourceFunction {
            function_id: function.id.clone(),
            contract_hash: contract_hash(&function.contracts)
                .map_err(|error| source_error(error.to_string()))?
                .as_str()
                .to_owned(),
            direct_callees: generated_function.direct_callees.clone(),
            parameters,
            requires,
            regenerated_members,
        });
    }

    let source = VcSourceContext {
        id: "production.validated_vir_manifest".to_owned(),
        source_ir_schema: VIR_SCHEMA_VERSION.to_owned(),
        source_ir_hash: recomputed_vir.as_str().to_owned(),
        input_set_hash: manifest_value.input_set_hash.clone(),
        semantic_profile: vir.semantic_profile,
        semantic_parameters: vir.semantic_parameters.clone(),
        verification_limit_profile: VERIFICATION_LIMIT_PROFILE.to_owned(),
        functions: source_functions,
    };
    generate_vc_v1_from_context(&source)
}

pub fn canonical_vc_json(document: &VcDocument) -> Result<Vec<u8>, VcValidationError> {
    let serialized = serde_json::to_vec(document).map_err(|error| {
        VcValidationError::new(VcValidationPhase::Shape, "VC_SHAPE", error.to_string())
    })?;
    validate_verification_limit(
        "canonical_vc_json_bytes",
        u64::try_from(serialized.len()).unwrap_or(u64::MAX),
    )
    .map_err(|error| {
        VcValidationError::new(
            VcValidationPhase::CanonicalSize,
            error.code(),
            error.to_string(),
        )
    })?;
    let strict = parse_strict_json(&serialized, VC_JSON_LIMITS).map_err(|error| {
        VcValidationError::new(VcValidationPhase::Shape, "VC_SHAPE", error.to_string())
    })?;
    canonical_json_bytes(&strict).map_err(|error| {
        VcValidationError::new(VcValidationPhase::Shape, "VC_SHAPE", error.to_string())
    })
}

pub fn canonical_vc_hash_payload(document: &VcDocument) -> Result<Vec<u8>, VcValidationError> {
    let payload = serialized_strict_value(document, VcValidationPhase::Hash, "VC_HASH")?
        .clone_without_fields(&["vc_hash"])
        .map_err(|error| {
            VcValidationError::new(VcValidationPhase::Shape, "VC_SHAPE", error.to_string())
        })?;
    canonical_json_bytes(&payload).map_err(|error| {
        VcValidationError::new(VcValidationPhase::Hash, "VC_HASH", error.to_string())
    })
}

pub fn vc_hash(document: &VcDocument) -> Result<LowercaseSha256, VcValidationError> {
    let payload = serialized_strict_value(document, VcValidationPhase::Hash, "VC_HASH")?
        .clone_without_fields(&["vc_hash"])
        .map_err(|error| {
            VcValidationError::new(VcValidationPhase::Shape, "VC_SHAPE", error.to_string())
        })?;
    let digest = hash_canonical_json(VC_HASH_DOMAIN, &payload).map_err(|error| {
        VcValidationError::new(VcValidationPhase::Hash, "VC_HASH", error.to_string())
    })?;
    LowercaseSha256::new(digest.to_hex()).map_err(|error| {
        VcValidationError::new(VcValidationPhase::Hash, "VC_HASH", error.to_string())
    })
}

fn serialized_strict_value<T: Serialize>(
    value: &T,
    phase: VcValidationPhase,
    code: &'static str,
) -> Result<StrictJsonValue, VcValidationError> {
    let bytes = serde_json::to_vec(value)
        .map_err(|error| VcValidationError::new(phase, code, error.to_string()))?;
    parse_strict_json(&bytes, VC_JSON_LIMITS)
        .map_err(|error| VcValidationError::new(phase, code, error.to_string()))
}

fn map_transport_error(error: StrictJsonError) -> VcValidationError {
    match error {
        StrictJsonError::DuplicateObjectName { .. } => VcValidationError::new(
            VcValidationPhase::Transport,
            "VC_JSON_DUPLICATE_KEY",
            error.to_string(),
        ),
        StrictJsonError::InputBytesExceeded { .. } => VcValidationError::new(
            VcValidationPhase::Transport,
            "VC_LIMIT_CANONICAL_JSON_BYTES",
            error.to_string(),
        ),
        _ => VcValidationError::new(
            VcValidationPhase::Transport,
            "VC_JSON_INVALID",
            error.to_string(),
        ),
    }
}

fn validate_root_shape(value: &StrictJsonValue) -> Result<(), VcValidationError> {
    let object = value.as_object().ok_or_else(|| {
        VcValidationError::new(
            VcValidationPhase::Shape,
            "VC_SHAPE",
            "VC root is not an object",
        )
    })?;
    let schema = value.get("schema");
    if let Some(schema) = schema {
        if schema.as_str() != Some(VC_SCHEMA_VERSION) {
            return Err(VcValidationError::new(
                VcValidationPhase::Shape,
                "VC_SCHEMA",
                "wrong VC schema discriminator",
            ));
        }
    }
    const FIELDS: [&str; 9] = [
        "schema",
        "source_ir_schema",
        "source_ir_hash",
        "input_set_hash",
        "semantic_profile",
        "semantic_parameters",
        "verification_limit_profile",
        "functions",
        "vc_hash",
    ];
    if object.len() != FIELDS.len()
        || FIELDS
            .iter()
            .any(|required| !object.iter().any(|(name, _)| name == required))
    {
        return Err(VcValidationError::new(
            VcValidationPhase::Shape,
            "VC_SHAPE",
            "VC root fields are not the exact v1 set",
        ));
    }
    Ok(())
}

fn validate_scalars(document: &VcDocument) -> Result<(), VcValidationError> {
    for (name, value) in [
        ("source_ir_hash", &document.source_ir_hash),
        ("input_set_hash", &document.input_set_hash),
        ("vc_hash", &document.vc_hash),
    ] {
        LowercaseSha256::new(value.clone()).map_err(|error| scalar(name, error.to_string()))?;
    }
    if document.functions.is_empty() {
        return Err(scalar("functions", "VC document has no functions"));
    }
    for function in &document.functions {
        LowercaseSha256::new(function.contract_hash.clone())
            .map_err(|error| scalar("contract_hash", error.to_string()))?;
        let parameter_names = function
            .parameters
            .iter()
            .map(|parameter| parameter.id.as_str())
            .collect::<BTreeSet<_>>();
        for parameter in &function.parameters {
            validate_type_term(&parameter.r#type)?;
        }
        for requirement in &function.requires {
            validate_term(requirement, &parameter_names, 0)?;
        }
        for member in &function.members {
            for binder in &member.local_binders {
                validate_type_term(binder)?;
            }
            let local_depth = member.local_binders.len();
            for assumption in &member.assumptions {
                validate_term(assumption, &parameter_names, local_depth)?;
            }
            validate_term(&member.conclusion, &parameter_names, local_depth)?;
        }
        for group in &function.groups {
            if !is_mpk_name(&group.declaration_name) {
                return Err(scalar(
                    "declaration_name",
                    format!("invalid MPK name {:?}", group.declaration_name),
                ));
            }
            for dependency in &group.dependencies {
                if !is_mpk_name(dependency) {
                    return Err(scalar(
                        "dependencies",
                        format!("invalid MPK name {dependency:?}"),
                    ));
                }
            }
        }
    }
    Ok(())
}

pub(crate) fn validate_type_term(term: &VcTypeTerm) -> Result<(), VcValidationError> {
    match term {
        VcTypeTerm::Constant { name } => validate_name_scalar(name, "type constant"),
        VcTypeTerm::Apply { function, args } => {
            validate_name_scalar(function, "type application")?;
            for arg in args {
                validate_type_term(arg)?;
            }
            Ok(())
        }
        VcTypeTerm::NatLiteral { value } => {
            if *value > MAX_SAFE_JSON_INTEGER as u64 {
                Err(scalar(
                    "type nat_literal",
                    "value is outside safe JSON range",
                ))
            } else {
                Ok(())
            }
        }
        VcTypeTerm::StringLiteral { .. } => Ok(()),
    }
}

pub(crate) fn validate_term(
    term: &VcTerm,
    parameters: &BTreeSet<&str>,
    binder_depth: usize,
) -> Result<(), VcValidationError> {
    match term {
        VcTerm::Var { name } => {
            if parameters.contains(name.as_str()) {
                Ok(())
            } else {
                Err(scalar(
                    "var",
                    format!("unbound function parameter {name:?}"),
                ))
            }
        }
        VcTerm::Bound { index } => {
            if usize::try_from(*index)
                .ok()
                .is_some_and(|index| index < binder_depth)
            {
                Ok(())
            } else {
                Err(scalar("bound", format!("open de Bruijn index {index}")))
            }
        }
        VcTerm::Constant { name } => validate_name_scalar(name, "constant"),
        VcTerm::BitVecLiteral {
            value,
            width,
            signed,
        } => validate_bit_vec_literal(value, *width, *signed),
        VcTerm::Apply { function, args } => {
            validate_name_scalar(function, "application")?;
            for arg in args {
                validate_term(arg, parameters, binder_depth)?;
            }
            Ok(())
        }
        VcTerm::Convert { value, target } => {
            validate_term(value, parameters, binder_depth)?;
            validate_type_term(target)
        }
        VcTerm::Forall { binder_type, body } => {
            validate_type_term(binder_type)?;
            let nested = binder_depth
                .checked_add(1)
                .ok_or_else(|| scalar("forall", "binder depth overflow"))?;
            validate_term(body, parameters, nested)
        }
    }
}

fn validate_bit_vec_literal(
    value: &str,
    width: u32,
    signed: bool,
) -> Result<(), VcValidationError> {
    DecimalInteger::new(value.to_owned()).map_err(|error| scalar("literal", error.to_string()))?;
    if !matches!(width, 8 | 16 | 32 | 64) {
        return Err(scalar(
            "literal",
            format!("unsupported bit-vector width {width}"),
        ));
    }
    let parsed = value.parse::<i128>().map_err(|_| {
        scalar(
            "literal",
            "bit-vector value exceeds 128-bit validation range",
        )
    })?;
    let fits = if signed {
        let bound = 1_i128 << (width - 1);
        (-bound..bound).contains(&parsed)
    } else {
        parsed >= 0 && parsed < (1_i128 << width)
    };
    if fits {
        Ok(())
    } else {
        Err(scalar(
            "literal",
            "bit-vector value does not fit its declared width",
        ))
    }
}

fn validate_name_scalar(name: &str, field: &str) -> Result<(), VcValidationError> {
    if is_mpk_name(name) {
        Ok(())
    } else {
        Err(scalar(field, format!("invalid MPK name {name:?}")))
    }
}

pub(crate) fn is_mpk_name(name: &str) -> bool {
    !name.is_empty()
        && name.split('.').all(|component| {
            let mut bytes = component.bytes();
            bytes
                .next()
                .is_some_and(|byte| byte.is_ascii_alphabetic() || byte == b'_')
                && bytes.all(|byte| byte.is_ascii_alphanumeric() || byte == b'_' || byte == b'\'')
        })
}

fn validate_linkage(
    document: &VcDocument,
    source: &VcSourceContext,
) -> Result<(), VcValidationError> {
    if document.source_ir_schema != source.source_ir_schema
        || document.source_ir_schema != VIR_SCHEMA_VERSION
        || document.source_ir_hash != source.source_ir_hash
        || document.input_set_hash != source.input_set_hash
    {
        return Err(source_error("source VIR or input-set repetition differs"));
    }
    if document.semantic_profile != source.semantic_profile
        || document.semantic_parameters != source.semantic_parameters
        || document.verification_limit_profile != source.verification_limit_profile
        || document.verification_limit_profile != VERIFICATION_LIMIT_PROFILE
        || validate_semantic_parameters(document.semantic_profile, &document.semantic_parameters)
            .is_err()
    {
        return Err(VcValidationError::new(
            VcValidationPhase::Linkage,
            "VC_PROFILE_LINKAGE",
            "semantic or verification-limit profile repetition differs",
        ));
    }
    if document
        .functions
        .iter()
        .map(|function| function.function_id.as_str())
        .ne(source
            .functions
            .iter()
            .map(|function| function.function_id.as_str()))
    {
        return Err(VcValidationError::new(
            VcValidationPhase::Linkage,
            "VC_FUNCTION_ORDER",
            "function set or callee-first order differs",
        ));
    }
    for (function, expected) in document.functions.iter().zip(&source.functions) {
        if function.contract_hash != expected.contract_hash
            || function.parameters != expected.parameters
            || function.requires != expected.requires
        {
            return Err(source_error(format!(
                "source repetition differs for function {:?}",
                function.function_id
            )));
        }
    }
    Ok(())
}

fn validate_members(
    document: &VcDocument,
    source: &VcSourceContext,
) -> Result<(), VcValidationError> {
    let mut ids = BTreeSet::new();
    for function in &document.functions {
        for member in &function.members {
            if !valid_member_id(member, &function.function_id) || !ids.insert(member.id.as_str()) {
                return Err(VcValidationError::new(
                    VcValidationPhase::Members,
                    "VC_MEMBER_ID",
                    format!(
                        "malformed, duplicate, or wrong-function member ID {:?}",
                        member.id
                    ),
                ));
            }
        }
    }
    for function in &document.functions {
        if function
            .members
            .windows(2)
            .any(|pair| pair[0].id.as_bytes() >= pair[1].id.as_bytes())
        {
            return Err(VcValidationError::new(
                VcValidationPhase::Members,
                "VC_MEMBER_ORDER",
                format!("member order differs for {:?}", function.function_id),
            ));
        }
    }
    for (function, expected) in document.functions.iter().zip(&source.functions) {
        if function.members != expected.regenerated_members {
            return Err(VcValidationError::new(
                VcValidationPhase::Members,
                "VC_MEMBER_SET",
                format!("regenerated members differ for {:?}", function.function_id),
            ));
        }
    }
    Ok(())
}

fn valid_member_id(member: &VcMember, containing_function: &str) -> bool {
    if member.function_id != containing_function || containing_function.contains('#') {
        return false;
    }
    let mut parts = member.id.split('#');
    let (Some(function), Some(kind), Some(ordinal), None) =
        (parts.next(), parts.next(), parts.next(), parts.next())
    else {
        return false;
    };
    function == containing_function
        && kind == member.kind.as_str()
        && ordinal.len() == 6
        && ordinal.bytes().all(|byte| byte.is_ascii_digit())
}

fn validate_groups(document: &VcDocument) -> Result<(), VcValidationError> {
    for function in &document.functions {
        let expected_shape = [
            (
                VcGroupKind::Contract,
                format!("{}.contract", function.function_id),
                program_declaration_name(&function.function_id, ProgramDeclarationKind::Contract),
            ),
            (
                VcGroupKind::PanicFree,
                format!("{}.panic_free", function.function_id),
                program_declaration_name(&function.function_id, ProgramDeclarationKind::PanicFree),
            ),
        ];
        if function.groups.len() != 2
            || function
                .groups
                .iter()
                .zip(expected_shape)
                .any(|(group, (kind, id, declaration))| {
                    group.kind != kind || group.id != id || group.declaration_name != declaration
                })
        {
            return Err(VcValidationError::new(
                VcValidationPhase::Groups,
                "VC_GROUP_SHAPE",
                format!("group shape differs for {:?}", function.function_id),
            ));
        }

        let members = function
            .members
            .iter()
            .map(|member| (member.id.as_str(), member))
            .collect::<BTreeMap<_, _>>();
        let mut counts = BTreeMap::<&str, usize>::new();
        for group in &function.groups {
            for id in &group.member_ids {
                if !members.contains_key(id.as_str()) {
                    return Err(group_partition(
                        function,
                        "group contains an unknown member",
                    ));
                }
                *counts.entry(id).or_default() += 1;
            }
        }
        if members
            .keys()
            .any(|id| counts.get(id).copied().unwrap_or(0) != 1)
        {
            return Err(group_partition(
                function,
                "members are not partitioned exactly once",
            ));
        }

        for group in &function.groups {
            for id in &group.member_ids {
                let member = members[id.as_str()];
                let expected_group_id = format!(
                    "{}.{}",
                    function.function_id,
                    member.kind.required_group().as_str()
                );
                if member.kind.required_group() != group.kind
                    || member.group_id != expected_group_id
                {
                    return Err(VcValidationError::new(
                        VcValidationPhase::Groups,
                        "VC_GROUP_KIND",
                        format!("member {id:?} is assigned to the wrong group"),
                    ));
                }
            }
            let expected_ids = function
                .members
                .iter()
                .filter(|member| member.kind.required_group() == group.kind)
                .map(|member| member.id.as_str())
                .collect::<Vec<_>>();
            if group.member_ids.iter().map(String::as_str).ne(expected_ids) {
                return Err(group_partition(function, "group member array differs"));
            }
        }
    }
    Ok(())
}

fn group_partition(function: &VcFunction, detail: &str) -> VcValidationError {
    VcValidationError::new(
        VcValidationPhase::Groups,
        "VC_GROUP_PARTITION",
        format!("{detail} for {:?}", function.function_id),
    )
}

fn validate_dependencies(
    document: &VcDocument,
    source: &VcSourceContext,
) -> Result<(), VcValidationError> {
    let declarations = document
        .functions
        .iter()
        .flat_map(|function| function.groups.iter())
        .enumerate()
        .map(|(index, group)| (group.declaration_name.as_str(), index))
        .collect::<BTreeMap<_, _>>();

    for (function, source_function) in document.functions.iter().zip(&source.functions) {
        for group in &function.groups {
            for dependency in &group.dependencies {
                if !declarations.contains_key(dependency.as_str()) {
                    return Err(VcValidationError::new(
                        VcValidationPhase::Dependencies,
                        "VC_DEPENDENCY_REFERENCE",
                        format!("unknown dependency {dependency:?}"),
                    ));
                }
            }
            if group
                .dependencies
                .windows(2)
                .any(|pair| pair[0].as_bytes() >= pair[1].as_bytes())
            {
                return Err(VcValidationError::new(
                    VcValidationPhase::Dependencies,
                    "VC_DEPENDENCY_ORDER",
                    format!("dependency order differs for {:?}", group.id),
                ));
            }
            let group_index = declarations[group.declaration_name.as_str()];
            if group
                .dependencies
                .iter()
                .any(|dependency| declarations[dependency.as_str()] >= group_index)
            {
                return Err(VcValidationError::new(
                    VcValidationPhase::Dependencies,
                    "VC_DEPENDENCY_CYCLE",
                    format!("dependency graph is cyclic or forward for {:?}", group.id),
                ));
            }

            let expected =
                expected_dependencies(&function.function_id, source_function, group.kind);
            if group.dependencies != expected {
                return Err(VcValidationError::new(
                    VcValidationPhase::Dependencies,
                    "VC_DEPENDENCY_SET",
                    format!("dependency set differs for {:?}", group.id),
                ));
            }
        }
    }
    Ok(())
}

fn expected_dependencies(
    function_id: &str,
    source: &VcSourceFunction,
    kind: VcGroupKind,
) -> Vec<String> {
    let mut dependencies = BTreeSet::new();
    if kind == VcGroupKind::PanicFree {
        dependencies.insert(program_declaration_name(
            function_id,
            ProgramDeclarationKind::Contract,
        ));
    }
    for callee in &source.direct_callees {
        dependencies.insert(program_declaration_name(
            callee,
            ProgramDeclarationKind::Contract,
        ));
        if kind == VcGroupKind::PanicFree {
            dependencies.insert(program_declaration_name(
                callee,
                ProgramDeclarationKind::PanicFree,
            ));
        }
    }
    dependencies.into_iter().collect()
}

fn build_function(source: &VcSourceFunction) -> VcFunction {
    let groups = [VcGroupKind::Contract, VcGroupKind::PanicFree]
        .into_iter()
        .map(|kind| {
            let declaration_kind = match kind {
                VcGroupKind::Contract => ProgramDeclarationKind::Contract,
                VcGroupKind::PanicFree => ProgramDeclarationKind::PanicFree,
            };
            VcGroup {
                id: format!("{}.{}", source.function_id, kind.as_str()),
                kind,
                declaration_name: program_declaration_name(&source.function_id, declaration_kind),
                member_ids: source
                    .regenerated_members
                    .iter()
                    .filter(|member| member.kind.required_group() == kind)
                    .map(|member| member.id.clone())
                    .collect(),
                dependencies: expected_dependencies(&source.function_id, source, kind),
            }
        })
        .collect();
    VcFunction {
        function_id: source.function_id.clone(),
        contract_hash: source.contract_hash.clone(),
        parameters: source.parameters.clone(),
        requires: source.requires.clone(),
        members: source.regenerated_members.clone(),
        groups,
    }
}

fn scalar(field: &str, detail: impl fmt::Display) -> VcValidationError {
    VcValidationError::new(
        VcValidationPhase::Scalar,
        "VC_SCALAR",
        format!("{field}: {detail}"),
    )
}

fn source_error(detail: impl Into<String>) -> VcValidationError {
    VcValidationError::new(VcValidationPhase::Linkage, "VC_SOURCE_LINKAGE", detail)
}
