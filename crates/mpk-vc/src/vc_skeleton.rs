//! Canonical grouped theorem-declaration skeletons for VC v1.

use std::collections::BTreeSet;
use std::error::Error;
use std::fmt;

use serde::{Deserialize, Serialize};

use crate::canonical_json::{
    canonical_json_bytes, parse_strict_json, StrictJsonError, StrictJsonLimits, StrictJsonValue,
};
use crate::grouping::{group_body, GroupingError};
use crate::semantic_profile::{SemanticParameters, SemanticProfile};
use crate::vc::{VcBinder, VcGroupKind, VcSourceContext, VcTerm, VC_SCHEMA_VERSION};
use crate::vc_canonical::{
    import_vc_v1_json, is_mpk_name, validate_term, validate_type_term, ValidatedVcDocument,
};
use crate::verification_limits::{validate_grouped_theorem_limits, validate_verification_limit};
use crate::vir::LowercaseSha256;

pub const VC_CERT_SKELETON_V1_SCHEMA_VERSION: &str = "mpk.vc.cert_skeleton.v1";

const SKELETON_JSON_LIMITS: StrictJsonLimits =
    StrictJsonLimits::new(268_435_456, 268_435_456, 768, 1_048_576);

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct VcCertificateSkeletonV1 {
    pub schema: String,
    pub source_vc_schema: String,
    pub source_vc_hash: String,
    pub source_ir_schema: String,
    pub source_ir_hash: String,
    pub input_set_hash: String,
    pub semantic_profile: SemanticProfile,
    pub semantic_parameters: SemanticParameters,
    pub verification_limit_profile: String,
    pub theorem_declarations: Vec<GroupedTheoremDeclaration>,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct GroupedTheoremDeclaration {
    pub name: String,
    pub function_id: String,
    pub group_id: String,
    pub group_kind: VcGroupKind,
    pub member_ids: Vec<String>,
    pub dependencies: Vec<String>,
    pub theorem_type: GroupedTheoremType,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct GroupedTheoremType {
    pub binders: Vec<VcBinder>,
    pub body: VcTerm,
}

#[derive(Clone, Debug)]
pub struct ValidatedVcCertificateSkeleton {
    skeleton: VcCertificateSkeletonV1,
    canonical_bytes: Vec<u8>,
}

impl ValidatedVcCertificateSkeleton {
    pub fn skeleton(&self) -> &VcCertificateSkeletonV1 {
        &self.skeleton
    }

    pub fn canonical_bytes(&self) -> &[u8] {
        &self.canonical_bytes
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum VcSkeletonValidationPhase {
    Transport,
    Shape,
    Scalar,
    StreamLimits,
    VcLinkage,
    Declarations,
    TheoremType,
    TheoremLimits,
    CanonicalSize,
    CanonicalTransport,
}

impl VcSkeletonValidationPhase {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Transport => "transport",
            Self::Shape => "shape",
            Self::Scalar => "scalar",
            Self::StreamLimits => "stream_limits",
            Self::VcLinkage => "vc_linkage",
            Self::Declarations => "declarations",
            Self::TheoremType => "theorem_type",
            Self::TheoremLimits => "theorem_limits",
            Self::CanonicalSize => "canonical_size",
            Self::CanonicalTransport => "canonical_transport",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VcSkeletonValidationError {
    phase: VcSkeletonValidationPhase,
    code: &'static str,
    detail: String,
}

impl VcSkeletonValidationError {
    fn new(
        phase: VcSkeletonValidationPhase,
        code: &'static str,
        detail: impl Into<String>,
    ) -> Self {
        Self {
            phase,
            code,
            detail: detail.into(),
        }
    }

    pub const fn phase(&self) -> VcSkeletonValidationPhase {
        self.phase
    }

    pub const fn code(&self) -> &'static str {
        self.code
    }

    pub fn detail(&self) -> &str {
        &self.detail
    }
}

impl fmt::Display for VcSkeletonValidationError {
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

impl Error for VcSkeletonValidationError {}

/// Emits no bytes until the complete canonical VC has passed its own linked
/// validation and the resulting skeleton has passed every import phase.
pub fn emit_vc_skeleton_v1(
    source_vc_bytes: &[u8],
    source: &VcSourceContext,
) -> Result<ValidatedVcCertificateSkeleton, VcSkeletonValidationError> {
    let vc = import_source_vc(source_vc_bytes, source)?;
    let skeleton = build_skeleton(&vc)?;
    let canonical_bytes = canonical_skeleton_json(&skeleton)?;
    import_vc_skeleton_v1_json(&canonical_bytes, source_vc_bytes, source)
}

/// Emits the grouped declaration skeleton directly from an already linked and
/// canonical VC. This keeps callers from reconstructing a source context after
/// the validated VC boundary has been crossed.
pub fn emit_validated_vc_skeleton_v1(
    vc: &ValidatedVcDocument,
) -> Result<ValidatedVcCertificateSkeleton, VcSkeletonValidationError> {
    let skeleton = build_skeleton(vc)?;
    validate_scalars(&skeleton)?;
    validate_skeleton_stream_limits(&skeleton)?;
    validate_grouped_theorem_limits(vc.document()).map_err(|error| {
        VcSkeletonValidationError::new(
            VcSkeletonValidationPhase::TheoremLimits,
            error.code(),
            error.to_string(),
        )
    })?;
    let canonical_bytes = canonical_skeleton_json(&skeleton)?;
    validate_verification_limit(
        "canonical_skeleton_json_bytes",
        u64::try_from(canonical_bytes.len()).unwrap_or(u64::MAX),
    )
    .map_err(|error| {
        VcSkeletonValidationError::new(
            VcSkeletonValidationPhase::CanonicalSize,
            error.code(),
            error.to_string(),
        )
    })?;
    Ok(ValidatedVcCertificateSkeleton {
        skeleton,
        canonical_bytes,
    })
}

/// Imports a skeleton against complete canonical source VC bytes. Every
/// repeated identity and theorem declaration is reconstructed, never trusted.
pub fn import_vc_skeleton_v1_json(
    input: &[u8],
    source_vc_bytes: &[u8],
    source: &VcSourceContext,
) -> Result<ValidatedVcCertificateSkeleton, VcSkeletonValidationError> {
    let strict = parse_strict_json(input, SKELETON_JSON_LIMITS).map_err(map_transport_error)?;
    validate_root_shape(&strict)?;

    let mut deserializer = serde_json::Deserializer::from_slice(input);
    deserializer.disable_recursion_limit();
    let skeleton = VcCertificateSkeletonV1::deserialize(&mut deserializer).map_err(|error| {
        VcSkeletonValidationError::new(
            VcSkeletonValidationPhase::Shape,
            "VC_SKELETON_SHAPE",
            error.to_string(),
        )
    })?;

    validate_scalars(&skeleton)?;
    validate_skeleton_stream_limits(&skeleton)?;
    let vc = import_source_vc(source_vc_bytes, source)?;
    validate_vc_linkage(&skeleton, &vc)?;
    let expected = build_skeleton(&vc)?;
    validate_declarations(&skeleton, &expected)?;
    validate_theorem_types(&skeleton, &expected)?;
    validate_grouped_theorem_limits(vc.document()).map_err(|error| {
        VcSkeletonValidationError::new(
            VcSkeletonValidationPhase::TheoremLimits,
            error.code(),
            error.to_string(),
        )
    })?;

    let canonical = canonical_json_bytes(&strict).map_err(|error| {
        VcSkeletonValidationError::new(
            VcSkeletonValidationPhase::Shape,
            "VC_SKELETON_SHAPE",
            error.to_string(),
        )
    })?;
    validate_verification_limit(
        "canonical_skeleton_json_bytes",
        u64::try_from(canonical.len()).unwrap_or(u64::MAX),
    )
    .map_err(|error| {
        VcSkeletonValidationError::new(
            VcSkeletonValidationPhase::CanonicalSize,
            error.code(),
            error.to_string(),
        )
    })?;
    if input != canonical {
        return Err(VcSkeletonValidationError::new(
            VcSkeletonValidationPhase::CanonicalTransport,
            "VC_SKELETON_CANONICAL_TRANSPORT",
            "skeleton transport is not byte-identical JCS",
        ));
    }

    Ok(ValidatedVcCertificateSkeleton {
        skeleton,
        canonical_bytes: canonical,
    })
}

pub fn canonical_skeleton_json(
    skeleton: &VcCertificateSkeletonV1,
) -> Result<Vec<u8>, VcSkeletonValidationError> {
    let bytes = serde_json::to_vec(skeleton).map_err(|error| {
        VcSkeletonValidationError::new(
            VcSkeletonValidationPhase::Shape,
            "VC_SKELETON_SHAPE",
            error.to_string(),
        )
    })?;
    let strict = parse_strict_json(&bytes, SKELETON_JSON_LIMITS).map_err(map_transport_error)?;
    canonical_json_bytes(&strict).map_err(|error| {
        VcSkeletonValidationError::new(
            VcSkeletonValidationPhase::Shape,
            "VC_SKELETON_SHAPE",
            error.to_string(),
        )
    })
}

fn import_source_vc(
    source_vc_bytes: &[u8],
    source: &VcSourceContext,
) -> Result<ValidatedVcDocument, VcSkeletonValidationError> {
    import_vc_v1_json(source_vc_bytes, source).map_err(|error| {
        VcSkeletonValidationError::new(
            VcSkeletonValidationPhase::VcLinkage,
            "VC_SKELETON_VC_LINKAGE",
            format!("source VC rejected: {error}"),
        )
    })
}

fn build_skeleton(
    vc: &ValidatedVcDocument,
) -> Result<VcCertificateSkeletonV1, VcSkeletonValidationError> {
    let document = vc.document();
    let mut theorem_declarations = Vec::with_capacity(document.functions.len().saturating_mul(2));
    for function in &document.functions {
        for group in &function.groups {
            theorem_declarations.push(GroupedTheoremDeclaration {
                name: group.declaration_name.clone(),
                function_id: function.function_id.clone(),
                group_id: group.id.clone(),
                group_kind: group.kind,
                member_ids: group.member_ids.clone(),
                dependencies: group.dependencies.clone(),
                theorem_type: GroupedTheoremType {
                    binders: function.parameters.clone(),
                    body: group_body(function, group).map_err(map_grouping_error)?,
                },
            });
        }
    }
    Ok(VcCertificateSkeletonV1 {
        schema: VC_CERT_SKELETON_V1_SCHEMA_VERSION.to_owned(),
        source_vc_schema: document.schema.clone(),
        source_vc_hash: vc.hash().as_str().to_owned(),
        source_ir_schema: document.source_ir_schema.clone(),
        source_ir_hash: document.source_ir_hash.clone(),
        input_set_hash: document.input_set_hash.clone(),
        semantic_profile: document.semantic_profile,
        semantic_parameters: document.semantic_parameters.clone(),
        verification_limit_profile: document.verification_limit_profile.clone(),
        theorem_declarations,
    })
}

fn validate_root_shape(value: &StrictJsonValue) -> Result<(), VcSkeletonValidationError> {
    let object = value.as_object().ok_or_else(|| {
        VcSkeletonValidationError::new(
            VcSkeletonValidationPhase::Shape,
            "VC_SKELETON_SHAPE",
            "skeleton root is not an object",
        )
    })?;
    if value
        .get("schema")
        .is_some_and(|schema| schema.as_str() != Some(VC_CERT_SKELETON_V1_SCHEMA_VERSION))
    {
        return Err(VcSkeletonValidationError::new(
            VcSkeletonValidationPhase::Shape,
            "VC_SKELETON_SCHEMA",
            "wrong skeleton schema discriminator",
        ));
    }
    const FIELDS: [&str; 10] = [
        "schema",
        "source_vc_schema",
        "source_vc_hash",
        "source_ir_schema",
        "source_ir_hash",
        "input_set_hash",
        "semantic_profile",
        "semantic_parameters",
        "verification_limit_profile",
        "theorem_declarations",
    ];
    if object.len() != FIELDS.len()
        || FIELDS
            .iter()
            .any(|field| !object.iter().any(|(name, _)| name == field))
    {
        return Err(VcSkeletonValidationError::new(
            VcSkeletonValidationPhase::Shape,
            "VC_SKELETON_SHAPE",
            "skeleton root fields are not the exact v1 set",
        ));
    }
    Ok(())
}

fn validate_scalars(skeleton: &VcCertificateSkeletonV1) -> Result<(), VcSkeletonValidationError> {
    for (field, hash) in [
        ("source_vc_hash", &skeleton.source_vc_hash),
        ("source_ir_hash", &skeleton.source_ir_hash),
        ("input_set_hash", &skeleton.input_set_hash),
    ] {
        LowercaseSha256::new(hash.clone()).map_err(|error| scalar(field, error.to_string()))?;
    }
    for declaration in &skeleton.theorem_declarations {
        if !is_mpk_name(&declaration.name) {
            return Err(scalar("name", "invalid declaration MPK name"));
        }
        for dependency in &declaration.dependencies {
            if !is_mpk_name(dependency) {
                return Err(scalar("dependencies", "invalid dependency MPK name"));
            }
        }
        let binder_ids = declaration
            .theorem_type
            .binders
            .iter()
            .map(|binder| binder.id.as_str())
            .collect::<BTreeSet<_>>();
        if binder_ids.len() != declaration.theorem_type.binders.len() {
            return Err(scalar("binders", "duplicate outer binder id"));
        }
        for binder in &declaration.theorem_type.binders {
            validate_type_term(&binder.r#type).map_err(map_scalar_from_vc)?;
        }
        // Binder-to-body equality is a theorem-type reconstruction rule, not
        // a scalar rule. Admit the body's variable labels here so removing a
        // binder receives the stable theorem_type diagnostic later.
        let mut body_variables = BTreeSet::new();
        collect_variable_names(&declaration.theorem_type.body, &mut body_variables);
        validate_term(&declaration.theorem_type.body, &body_variables, 0)
            .map_err(map_scalar_from_vc)?;
    }
    Ok(())
}

fn collect_variable_names<'a>(term: &'a VcTerm, names: &mut BTreeSet<&'a str>) {
    match term {
        VcTerm::Var { name } => {
            names.insert(name);
        }
        VcTerm::Apply { args, .. } => {
            for argument in args {
                collect_variable_names(argument, names);
            }
        }
        VcTerm::Convert { value, .. } => collect_variable_names(value, names),
        VcTerm::Forall { body, .. } => collect_variable_names(body, names),
        VcTerm::Bound { .. } | VcTerm::Constant { .. } | VcTerm::BitVecLiteral { .. } => {}
    }
}

fn validate_skeleton_stream_limits(
    skeleton: &VcCertificateSkeletonV1,
) -> Result<(), VcSkeletonValidationError> {
    let declarations = u64::try_from(skeleton.theorem_declarations.len()).unwrap_or(u64::MAX);
    let members = skeleton
        .theorem_declarations
        .iter()
        .try_fold(0_u64, |total, declaration| {
            total.checked_add(u64::try_from(declaration.member_ids.len()).unwrap_or(u64::MAX))
        })
        .ok_or_else(|| {
            VcSkeletonValidationError::new(
                VcSkeletonValidationPhase::StreamLimits,
                "VC_SKELETON_SHAPE",
                "skeleton member counter overflow",
            )
        })?;
    if declarations > 524_288 || members > 262_144 {
        return Err(VcSkeletonValidationError::new(
            VcSkeletonValidationPhase::StreamLimits,
            "VC_SKELETON_SHAPE",
            "skeleton repeats more declarations or members than VC v1 permits",
        ));
    }
    Ok(())
}

fn validate_vc_linkage(
    actual: &VcCertificateSkeletonV1,
    vc: &ValidatedVcDocument,
) -> Result<(), VcSkeletonValidationError> {
    let document = vc.document();
    if actual.source_vc_schema != VC_SCHEMA_VERSION
        || actual.source_vc_schema != document.schema
        || actual.source_vc_hash != vc.hash().as_str()
        || actual.source_ir_schema != document.source_ir_schema
        || actual.source_ir_hash != document.source_ir_hash
        || actual.input_set_hash != document.input_set_hash
        || actual.semantic_profile != document.semantic_profile
        || actual.semantic_parameters != document.semantic_parameters
        || actual.verification_limit_profile != document.verification_limit_profile
    {
        return Err(VcSkeletonValidationError::new(
            VcSkeletonValidationPhase::VcLinkage,
            "VC_SKELETON_VC_LINKAGE",
            "skeleton source VC identity repetition differs from recomputation",
        ));
    }
    Ok(())
}

fn validate_declarations(
    actual: &VcCertificateSkeletonV1,
    expected: &VcCertificateSkeletonV1,
) -> Result<(), VcSkeletonValidationError> {
    let same = actual.theorem_declarations.len() == expected.theorem_declarations.len()
        && actual
            .theorem_declarations
            .iter()
            .zip(&expected.theorem_declarations)
            .all(|(actual, expected)| {
                actual.name == expected.name
                    && actual.function_id == expected.function_id
                    && actual.group_id == expected.group_id
                    && actual.group_kind == expected.group_kind
                    && actual.member_ids == expected.member_ids
                    && actual.dependencies == expected.dependencies
            });
    if same {
        Ok(())
    } else {
        Err(VcSkeletonValidationError::new(
            VcSkeletonValidationPhase::Declarations,
            "VC_SKELETON_DECLARATIONS",
            "declaration order or repeated group metadata differs from source VC",
        ))
    }
}

fn validate_theorem_types(
    actual: &VcCertificateSkeletonV1,
    expected: &VcCertificateSkeletonV1,
) -> Result<(), VcSkeletonValidationError> {
    if actual
        .theorem_declarations
        .iter()
        .zip(&expected.theorem_declarations)
        .all(|(actual, expected)| actual.theorem_type == expected.theorem_type)
    {
        Ok(())
    } else {
        Err(VcSkeletonValidationError::new(
            VcSkeletonValidationPhase::TheoremType,
            "VC_SKELETON_THEOREM_TYPE",
            "theorem binders or canonical balanced proposition differ",
        ))
    }
}

fn map_transport_error(error: StrictJsonError) -> VcSkeletonValidationError {
    match error {
        StrictJsonError::DuplicateObjectName { .. } => VcSkeletonValidationError::new(
            VcSkeletonValidationPhase::Transport,
            "VC_SKELETON_JSON_DUPLICATE_KEY",
            error.to_string(),
        ),
        StrictJsonError::InputBytesExceeded { .. } => VcSkeletonValidationError::new(
            VcSkeletonValidationPhase::Transport,
            "VC_LIMIT_CANONICAL_SKELETON_JSON_BYTES",
            error.to_string(),
        ),
        _ => VcSkeletonValidationError::new(
            VcSkeletonValidationPhase::Transport,
            "VC_SKELETON_JSON_INVALID",
            error.to_string(),
        ),
    }
}

fn scalar(field: &str, detail: impl Into<String>) -> VcSkeletonValidationError {
    VcSkeletonValidationError::new(
        VcSkeletonValidationPhase::Scalar,
        "VC_SKELETON_SHAPE",
        format!("invalid skeleton {field}: {}", detail.into()),
    )
}

fn map_scalar_from_vc(error: crate::vc_canonical::VcValidationError) -> VcSkeletonValidationError {
    scalar("theorem_type", error.to_string())
}

fn map_grouping_error(error: GroupingError) -> VcSkeletonValidationError {
    VcSkeletonValidationError::new(
        VcSkeletonValidationPhase::Declarations,
        "VC_SKELETON_DECLARATIONS",
        error.to_string(),
    )
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PolicyMemberBindingError {
    MissingMember(String),
    AmbiguousMember(String),
    WrongGroup { expected: String, actual: String },
    WrongDeclaration { expected: String, actual: String },
}

impl fmt::Display for PolicyMemberBindingError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingMember(member) => write!(formatter, "missing checked member {member:?}"),
            Self::AmbiguousMember(member) => {
                write!(
                    formatter,
                    "member {member:?} belongs to multiple declarations"
                )
            }
            Self::WrongGroup { expected, actual } => {
                write!(formatter, "member group is {expected:?}, not {actual:?}")
            }
            Self::WrongDeclaration { expected, actual } => write!(
                formatter,
                "member declaration is {expected:?}, not {actual:?}"
            ),
        }
    }
}

impl Error for PolicyMemberBindingError {}

/// Validates the policy-evidence containment tuple against an already checked
/// skeleton. Individual conjuncts never become independent declarations.
pub fn validate_policy_member_binding(
    skeleton: &ValidatedVcCertificateSkeleton,
    member_id: &str,
    group_id: &str,
    declaration_name: &str,
) -> Result<(), PolicyMemberBindingError> {
    let mut containing = skeleton
        .skeleton
        .theorem_declarations
        .iter()
        .filter(|declaration| declaration.member_ids.iter().any(|id| id == member_id));
    let declaration = containing
        .next()
        .ok_or_else(|| PolicyMemberBindingError::MissingMember(member_id.to_owned()))?;
    if containing.next().is_some() {
        return Err(PolicyMemberBindingError::AmbiguousMember(
            member_id.to_owned(),
        ));
    }
    if declaration.group_id != group_id {
        return Err(PolicyMemberBindingError::WrongGroup {
            expected: declaration.group_id.clone(),
            actual: group_id.to_owned(),
        });
    }
    if declaration.name != declaration_name {
        return Err(PolicyMemberBindingError::WrongDeclaration {
            expected: declaration.name.clone(),
            actual: declaration_name.to_owned(),
        });
    }
    Ok(())
}
