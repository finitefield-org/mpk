//! Canonical generation, import, linked validation, and hashing for VC v1.

use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use std::fmt;

use crate::call_wp::{program_declaration_name, ProgramDeclarationKind};
use crate::canonical_json::{
    canonical_json_bytes_bounded, parse_strict_json, scan_strict_json, serialize_json_bounded,
    BoundedJsonSerializeError, CanonicalJsonError, StrictJsonError, StrictJsonEvent,
    StrictJsonLimits, StrictJsonPathSegment, StrictJsonValue, StrictJsonValueKind,
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
    VerificationLimitId,
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
    scan_vc_stream_limits(input)?;
    let strict = parse_strict_json(input, VC_JSON_LIMITS).map_err(map_transport_error)?;
    validate_root_shape(&strict)?;
    limit_precedence::validate_vc_pre_stream_phases(input)?;

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
    let canonical = canonical_json_bytes_bounded(&strict, canonical_vc_maximum())
        .map_err(map_vc_canonical_error)?;

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

fn scan_vc_stream_limits(input: &[u8]) -> Result<(), VcValidationError> {
    let mut document_members = 0_u64;
    let mut document_nodes = 0_u64;
    let mut member_nodes = BTreeMap::<(u64, u64), u64>::new();
    let mut first_error: Option<StrictJsonError> = None;
    let mut observer = |event: StrictJsonEvent<'_>| -> Result<(), StrictJsonError> {
        if first_error.is_some() {
            return Ok(());
        }
        let result = (|| {
            match event {
                StrictJsonEvent::ArrayElement { path, count }
                    if vc_members_path(path).is_some() =>
                {
                    observed_verification_max(VerificationLimitId::MembersPerFunction, count)?;
                    document_members = observed_verification_add(
                        VerificationLimitId::MembersPerDocument,
                        document_members,
                        1,
                    )?;
                    observed_verification_max(
                        VerificationLimitId::MembersPerDocument,
                        document_members,
                    )?;
                }
                StrictJsonEvent::ArrayElement { path, count }
                    if vc_assumptions_path(path).is_some() =>
                {
                    observed_verification_max(VerificationLimitId::AssumptionsPerMember, count)?;
                }
                StrictJsonEvent::Value {
                    path,
                    kind: StrictJsonValueKind::Object,
                } => {
                    if let Some((owner, depth, expression_node)) = vc_term_location(path) {
                        if expression_node {
                            if let Some(member) = owner {
                                let count = member_nodes.entry(member).or_insert(0);
                                *count = observed_verification_add(
                                    VerificationLimitId::ExpressionNodesPerMember,
                                    *count,
                                    1,
                                )?;
                                observed_verification_max(
                                    VerificationLimitId::ExpressionNodesPerMember,
                                    *count,
                                )?;
                            }
                            document_nodes = observed_verification_add(
                                VerificationLimitId::ExpressionNodesPerDocument,
                                document_nodes,
                                1,
                            )?;
                            observed_verification_max(
                                VerificationLimitId::ExpressionNodesPerDocument,
                                document_nodes,
                            )?;
                        }
                        if owner.is_some() {
                            observed_verification_max(
                                VerificationLimitId::MemberExpressionDepth,
                                depth,
                            )?;
                        }
                    }
                }
                _ => {}
            }
            Ok(())
        })();
        if first_error.is_none() {
            first_error = result.err();
        }
        Ok(())
    };
    scan_strict_json(input, VC_JSON_LIMITS, &mut observer).map_err(map_transport_error)?;
    if let Some(error) = first_error {
        limit_precedence::validate_vc_pre_stream_phases(input)?;
        let limit = match &error {
            StrictJsonError::ObservedLimitExceeded { limit, .. }
            | StrictJsonError::ObservedCounterOverflow { limit } => {
                VerificationLimitId::try_from(*limit)
                    .map_err(|_| map_transport_error(error.clone()))?
            }
            _ => return Err(map_transport_error(error)),
        };
        return Err(VcValidationError::new(
            VcValidationPhase::StreamLimits,
            limit.code(),
            error.to_string(),
        ));
    }
    Ok(())
}

#[allow(dead_code)]
mod limit_precedence {
    use super::*;
    use serde::de::{DeserializeSeed, IgnoredAny};
    use serde_json::value::RawValue;
    use std::marker::PhantomData;

    enum SchemaField {
        String(String),
        Other,
    }

    impl<'de> Deserialize<'de> for SchemaField {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            struct Visitor;

            impl<'de> serde::de::Visitor<'de> for Visitor {
                type Value = SchemaField;

                fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                    formatter.write_str("any JSON value")
                }

                fn visit_borrowed_str<E>(self, value: &'de str) -> Result<Self::Value, E> {
                    Ok(SchemaField::String(value.to_owned()))
                }

                fn visit_str<E>(self, value: &str) -> Result<Self::Value, E> {
                    Ok(SchemaField::String(value.to_owned()))
                }

                fn visit_string<E>(self, value: String) -> Result<Self::Value, E> {
                    Ok(SchemaField::String(value))
                }

                fn visit_bool<E>(self, _value: bool) -> Result<Self::Value, E> {
                    Ok(SchemaField::Other)
                }

                fn visit_i64<E>(self, _value: i64) -> Result<Self::Value, E> {
                    Ok(SchemaField::Other)
                }

                fn visit_u64<E>(self, _value: u64) -> Result<Self::Value, E> {
                    Ok(SchemaField::Other)
                }

                fn visit_f64<E>(self, _value: f64) -> Result<Self::Value, E> {
                    Ok(SchemaField::Other)
                }

                fn visit_unit<E>(self) -> Result<Self::Value, E> {
                    Ok(SchemaField::Other)
                }

                fn visit_none<E>(self) -> Result<Self::Value, E> {
                    Ok(SchemaField::Other)
                }

                fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
                where
                    D: serde::Deserializer<'de>,
                {
                    SchemaField::deserialize(deserializer)
                }

                fn visit_seq<A>(self, mut sequence: A) -> Result<Self::Value, A::Error>
                where
                    A: serde::de::SeqAccess<'de>,
                {
                    while sequence.next_element::<IgnoredAny>()?.is_some() {}
                    Ok(SchemaField::Other)
                }

                fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
                where
                    A: serde::de::MapAccess<'de>,
                {
                    while map.next_entry::<IgnoredAny, IgnoredAny>()?.is_some() {}
                    Ok(SchemaField::Other)
                }
            }

            deserializer.deserialize_any(Visitor)
        }
    }

    struct VcSchemaLimitProbe(Option<SchemaField>);

    impl<'de> Deserialize<'de> for VcSchemaLimitProbe {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            struct Visitor;

            impl<'de> serde::de::Visitor<'de> for Visitor {
                type Value = VcSchemaLimitProbe;

                fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                    formatter.write_str("a VC root object")
                }

                fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
                where
                    A: serde::de::MapAccess<'de>,
                {
                    let mut schema = None;
                    while let Some(name) = map.next_key::<String>()? {
                        if name == "schema" {
                            if schema.is_some() {
                                return Err(serde::de::Error::custom("duplicate schema field"));
                            }
                            schema = Some(map.next_value::<SchemaField>()?);
                        } else {
                            map.next_value::<IgnoredAny>()?;
                        }
                    }
                    Ok(VcSchemaLimitProbe(schema))
                }
            }

            deserializer.deserialize_map(Visitor)
        }
    }

    struct SemanticParametersShapeProbe;

    impl<'de> Deserialize<'de> for SemanticParametersShapeProbe {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            struct Visitor;

            impl<'de> serde::de::Visitor<'de> for Visitor {
                type Value = SemanticParametersShapeProbe;

                fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                    formatter.write_str("a closed semantic-parameters object")
                }

                fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
                where
                    A: serde::de::MapAccess<'de>,
                {
                    let mut target_id = false;
                    let mut pointer_width = false;
                    let mut overflow_mode = false;
                    let mut panic_mode = false;
                    while let Some(name) = map.next_key::<String>()? {
                        let seen = match name.as_str() {
                            "target_id" => {
                                map.next_value::<String>()?;
                                &mut target_id
                            }
                            "pointer_width" => {
                                map.next_value::<i64>()?;
                                &mut pointer_width
                            }
                            "overflow_mode" => {
                                map.next_value::<String>()?;
                                &mut overflow_mode
                            }
                            "panic_mode" => {
                                map.next_value::<String>()?;
                                &mut panic_mode
                            }
                            _ => {
                                return Err(serde::de::Error::custom(
                                    "unknown semantic-parameters field",
                                ));
                            }
                        };
                        if *seen {
                            return Err(serde::de::Error::custom(
                                "duplicate semantic-parameters field",
                            ));
                        }
                        *seen = true;
                    }
                    if !target_id || !pointer_width || (overflow_mode != panic_mode) {
                        return Err(serde::de::Error::custom(
                            "incomplete semantic-parameters branch",
                        ));
                    }
                    Ok(SemanticParametersShapeProbe)
                }
            }

            deserializer.deserialize_map(Visitor)
        }
    }

    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct VcLimitShapeRootProbe {
        schema: String,
        source_ir_schema: String,
        source_ir_hash: String,
        input_set_hash: String,
        semantic_profile: String,
        semantic_parameters: SemanticParametersShapeProbe,
        verification_limit_profile: String,
        functions: DiscardingSequence<VcLimitShapeFunctionProbe>,
        vc_hash: String,
    }

    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct VcLimitShapeFunctionProbe {
        function_id: String,
        contract_hash: String,
        parameters: DiscardingSequence<VcLimitShapeBinderProbe>,
        requires: DiscardingSequence<VcLimitShapeTermProbe>,
        members: DiscardingSequence<VcLimitShapeMemberProbe>,
        groups: DiscardingSequence<VcLimitShapeGroupProbe>,
    }

    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct VcLimitShapeBinderProbe {
        id: String,
        #[serde(rename = "type")]
        r#type: VcLimitShapeTypeTermProbe,
    }

    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct VcLimitShapeMemberProbe {
        id: String,
        function_id: String,
        kind: crate::vc::VcMemberKind,
        local_binders: DiscardingSequence<VcLimitShapeTypeTermProbe>,
        assumptions: DiscardingSequence<VcLimitShapeTermProbe>,
        conclusion: VcLimitShapeTermProbe,
        group_id: String,
    }

    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct VcLimitShapeGroupProbe {
        id: String,
        kind: crate::vc::VcGroupKind,
        declaration_name: String,
        member_ids: DiscardingSequence<String>,
        dependencies: DiscardingSequence<String>,
    }

    #[derive(Deserialize)]
    #[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
    enum VcLimitShapeTypeTermProbe {
        Constant {
            name: String,
        },
        Apply {
            function: String,
            args: DiscardingSequence<VcLimitShapeTypeTermProbe>,
        },
        NatLiteral {
            value: i64,
        },
        StringLiteral {
            value: String,
        },
    }

    #[derive(Deserialize)]
    #[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
    enum VcLimitShapeTermProbe {
        Var {
            name: String,
        },
        Bound {
            index: i64,
        },
        Constant {
            name: String,
        },
        BitVecLiteral {
            value: String,
            width: i64,
            signed: bool,
        },
        Apply {
            function: String,
            args: DiscardingSequence<VcLimitShapeTermProbe>,
        },
        Convert {
            value: Box<VcLimitShapeTermProbe>,
            target: VcLimitShapeTypeTermProbe,
        },
        Forall {
            binder_type: VcLimitShapeTypeTermProbe,
            body: Box<VcLimitShapeTermProbe>,
        },
    }

    struct DiscardingSequence<T> {
        count: u64,
        element: PhantomData<fn() -> T>,
    }

    impl<'de, T> Deserialize<'de> for DiscardingSequence<T>
    where
        T: Deserialize<'de>,
    {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            struct SequenceVisitor<T>(PhantomData<fn() -> T>);

            impl<'de, T> serde::de::Visitor<'de> for SequenceVisitor<T>
            where
                T: Deserialize<'de>,
            {
                type Value = DiscardingSequence<T>;

                fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                    formatter.write_str("a JSON array")
                }

                fn visit_seq<A>(self, mut sequence: A) -> Result<Self::Value, A::Error>
                where
                    A: serde::de::SeqAccess<'de>,
                {
                    let mut count = 0_u64;
                    while sequence.next_element::<T>()?.is_some() {
                        count = count
                            .checked_add(1)
                            .ok_or_else(|| serde::de::Error::custom("array length overflow"))?;
                    }
                    Ok(DiscardingSequence {
                        count,
                        element: PhantomData,
                    })
                }
            }

            deserializer.deserialize_seq(SequenceVisitor(PhantomData))
        }
    }

    struct ValidVcSha256;

    impl<'de> Deserialize<'de> for ValidVcSha256 {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            let value = String::deserialize(deserializer)?;
            LowercaseSha256::new(value).map_err(serde::de::Error::custom)?;
            Ok(Self)
        }
    }

    struct ValidMpkName;

    impl<'de> Deserialize<'de> for ValidMpkName {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            let value = String::deserialize(deserializer)?;
            if !is_mpk_name(&value) {
                return Err(serde::de::Error::custom(format!(
                    "invalid MPK name {value:?}"
                )));
            }
            Ok(Self)
        }
    }

    struct NonnegativeU32(u32);

    impl<'de> Deserialize<'de> for NonnegativeU32 {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            let value = i64::deserialize(deserializer)?;
            u32::try_from(value)
                .map(Self)
                .map_err(|_| serde::de::Error::custom("integer is outside the u32 range"))
        }
    }

    struct NonnegativeU64(u64);

    impl<'de> Deserialize<'de> for NonnegativeU64 {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            let value = i64::deserialize(deserializer)?;
            u64::try_from(value)
                .map(Self)
                .map_err(|_| serde::de::Error::custom("integer is negative"))
        }
    }

    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct VcLimitScalarRootProbe {
        schema: IgnoredAny,
        source_ir_schema: IgnoredAny,
        source_ir_hash: ValidVcSha256,
        input_set_hash: ValidVcSha256,
        semantic_profile: crate::SemanticProfile,
        semantic_parameters: crate::SemanticParameters,
        verification_limit_profile: IgnoredAny,
        functions: DiscardingSequence<VcLimitScalarFunctionProbe>,
        vc_hash: ValidVcSha256,
    }

    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct VcLimitScalarFunctionProbe {
        function_id: IgnoredAny,
        contract_hash: ValidVcSha256,
        parameters: DiscardingSequence<VcLimitScalarBinderProbe>,
        requires: DiscardingSequence<VcLimitScalarTermProbe>,
        members: DiscardingSequence<VcLimitScalarMemberProbe>,
        groups: DiscardingSequence<VcLimitScalarGroupProbe>,
    }

    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct VcLimitScalarBinderProbe {
        id: IgnoredAny,
        #[serde(rename = "type")]
        r#type: VcLimitScalarTypeTermProbe,
    }

    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct VcLimitScalarMemberProbe {
        id: IgnoredAny,
        function_id: IgnoredAny,
        kind: IgnoredAny,
        local_binders: DiscardingSequence<VcLimitScalarTypeTermProbe>,
        assumptions: DiscardingSequence<VcLimitScalarTermProbe>,
        conclusion: VcLimitScalarTermProbe,
        group_id: IgnoredAny,
    }

    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct VcLimitScalarGroupProbe {
        id: IgnoredAny,
        kind: IgnoredAny,
        declaration_name: ValidMpkName,
        member_ids: DiscardingSequence<IgnoredAny>,
        dependencies: DiscardingSequence<ValidMpkName>,
    }

    struct VcLimitScalarTypeTermProbe;

    #[derive(Deserialize)]
    #[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
    enum VcLimitScalarTypeTermValue {
        Constant {
            name: ValidMpkName,
        },
        Apply {
            function: ValidMpkName,
            args: DiscardingSequence<VcLimitScalarTypeTermProbe>,
        },
        NatLiteral {
            value: NonnegativeU64,
        },
        StringLiteral {
            value: String,
        },
    }

    impl<'de> Deserialize<'de> for VcLimitScalarTypeTermProbe {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            let value = VcLimitScalarTypeTermValue::deserialize(deserializer)?;
            if let VcLimitScalarTypeTermValue::NatLiteral { value } = value {
                if value.0 > MAX_SAFE_JSON_INTEGER as u64 {
                    return Err(serde::de::Error::custom(
                        "type nat_literal is outside the safe JSON range",
                    ));
                }
            }
            Ok(Self)
        }
    }

    struct VcLimitScalarTermProbe;

    #[derive(Deserialize)]
    #[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
    enum VcLimitScalarTermValue {
        Var {
            name: String,
        },
        Bound {
            index: NonnegativeU32,
        },
        Constant {
            name: ValidMpkName,
        },
        BitVecLiteral {
            value: String,
            width: NonnegativeU32,
            signed: bool,
        },
        Apply {
            function: ValidMpkName,
            args: DiscardingSequence<VcLimitScalarTermProbe>,
        },
        Convert {
            value: Box<VcLimitScalarTermProbe>,
            target: VcLimitScalarTypeTermProbe,
        },
        Forall {
            binder_type: VcLimitScalarTypeTermProbe,
            body: Box<VcLimitScalarTermProbe>,
        },
    }

    impl<'de> Deserialize<'de> for VcLimitScalarTermProbe {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            let value = VcLimitScalarTermValue::deserialize(deserializer)?;
            if let VcLimitScalarTermValue::BitVecLiteral {
                value,
                width,
                signed,
            } = value
            {
                validate_bit_vec_literal(&value, width.0, signed)
                    .map_err(serde::de::Error::custom)?;
            }
            Ok(Self)
        }
    }

    #[derive(Default)]
    struct BoundedNameSet {
        names: BTreeSet<String>,
        overflowed: bool,
    }

    impl BoundedNameSet {
        fn insert(&mut self, name: String) {
            if self.overflowed || self.names.contains(&name) {
                return;
            }
            if self.names.len() == crate::vir_validate::VIR_PARAMS_MAX {
                self.overflowed = true;
                return;
            }
            self.names.insert(name);
        }

        fn merge(&mut self, other: Self) {
            if self.overflowed {
                return;
            }
            if other.overflowed {
                self.overflowed = true;
                return;
            }
            for name in other.names {
                self.insert(name);
                if self.overflowed {
                    break;
                }
            }
        }

        fn contains(&self, name: &str) -> bool {
            self.names.contains(name)
        }
    }

    #[derive(Default)]
    struct VcLimitTermContext {
        variables: BoundedNameSet,
        required_binder_depth: u64,
    }

    impl VcLimitTermContext {
        fn merge(&mut self, other: Self) {
            self.variables.merge(other.variables);
            self.required_binder_depth =
                self.required_binder_depth.max(other.required_binder_depth);
        }
    }

    struct VcLimitContextTermProbe(VcLimitTermContext);

    #[derive(Deserialize)]
    #[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
    enum VcLimitContextTermValue {
        Var {
            name: String,
        },
        Bound {
            index: u32,
        },
        Constant {
            name: IgnoredAny,
        },
        BitVecLiteral {
            value: IgnoredAny,
            width: IgnoredAny,
            signed: IgnoredAny,
        },
        Apply {
            function: IgnoredAny,
            args: VcLimitContextTermSequence,
        },
        Convert {
            value: Box<VcLimitContextTermProbe>,
            target: IgnoredAny,
        },
        Forall {
            binder_type: IgnoredAny,
            body: Box<VcLimitContextTermProbe>,
        },
    }

    impl<'de> Deserialize<'de> for VcLimitContextTermProbe {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            let value = VcLimitContextTermValue::deserialize(deserializer)?;
            let context = match value {
                VcLimitContextTermValue::Var { name } => {
                    let mut variables = BoundedNameSet::default();
                    variables.insert(name);
                    VcLimitTermContext {
                        variables,
                        required_binder_depth: 0,
                    }
                }
                VcLimitContextTermValue::Bound { index } => VcLimitTermContext {
                    variables: BoundedNameSet::default(),
                    required_binder_depth: u64::from(index) + 1,
                },
                VcLimitContextTermValue::Apply { args, .. } => args.context,
                VcLimitContextTermValue::Convert { value, .. } => value.0,
                VcLimitContextTermValue::Forall { body, .. } => VcLimitTermContext {
                    variables: body.0.variables,
                    required_binder_depth: body.0.required_binder_depth.saturating_sub(1),
                },
                VcLimitContextTermValue::Constant { .. }
                | VcLimitContextTermValue::BitVecLiteral { .. } => VcLimitTermContext::default(),
            };
            Ok(Self(context))
        }
    }

    struct VcLimitContextTermSequence {
        context: VcLimitTermContext,
    }

    impl<'de> Deserialize<'de> for VcLimitContextTermSequence {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            struct Visitor;

            impl<'de> serde::de::Visitor<'de> for Visitor {
                type Value = VcLimitContextTermSequence;

                fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                    formatter.write_str("a VC term array")
                }

                fn visit_seq<A>(self, mut sequence: A) -> Result<Self::Value, A::Error>
                where
                    A: serde::de::SeqAccess<'de>,
                {
                    let mut context = VcLimitTermContext::default();
                    while let Some(term) = sequence.next_element::<VcLimitContextTermProbe>()? {
                        context.merge(term.0);
                    }
                    Ok(VcLimitContextTermSequence { context })
                }
            }

            deserializer.deserialize_seq(Visitor)
        }
    }

    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct VcLimitContextParameterValue {
        id: String,
        #[serde(rename = "type")]
        r#type: IgnoredAny,
    }

    struct VcLimitContextParameterSequence(BoundedNameSet);

    impl<'de> Deserialize<'de> for VcLimitContextParameterSequence {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            struct Visitor;

            impl<'de> serde::de::Visitor<'de> for Visitor {
                type Value = VcLimitContextParameterSequence;

                fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                    formatter.write_str("a VC parameter array")
                }

                fn visit_seq<A>(self, mut sequence: A) -> Result<Self::Value, A::Error>
                where
                    A: serde::de::SeqAccess<'de>,
                {
                    let mut names = BoundedNameSet::default();
                    while let Some(parameter) =
                        sequence.next_element::<VcLimitContextParameterValue>()?
                    {
                        names.insert(parameter.id);
                    }
                    Ok(VcLimitContextParameterSequence(names))
                }
            }

            deserializer.deserialize_seq(Visitor)
        }
    }

    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct VcLimitContextMemberValue {
        id: IgnoredAny,
        function_id: IgnoredAny,
        kind: IgnoredAny,
        local_binders: DiscardingSequence<IgnoredAny>,
        assumptions: VcLimitContextTermSequence,
        conclusion: VcLimitContextTermProbe,
        group_id: IgnoredAny,
    }

    struct VcLimitContextMemberProbe(VcLimitTermContext);

    impl<'de> Deserialize<'de> for VcLimitContextMemberProbe {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            let member = VcLimitContextMemberValue::deserialize(deserializer)?;
            let mut context = member.assumptions.context;
            context.merge(member.conclusion.0);
            if context.required_binder_depth > member.local_binders.count {
                return Err(serde::de::Error::custom("open de Bruijn index"));
            }
            context.required_binder_depth = 0;
            Ok(Self(context))
        }
    }

    struct VcLimitContextMemberSequence(VcLimitTermContext);

    impl<'de> Deserialize<'de> for VcLimitContextMemberSequence {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            struct Visitor;

            impl<'de> serde::de::Visitor<'de> for Visitor {
                type Value = VcLimitContextMemberSequence;

                fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                    formatter.write_str("a VC member array")
                }

                fn visit_seq<A>(self, mut sequence: A) -> Result<Self::Value, A::Error>
                where
                    A: serde::de::SeqAccess<'de>,
                {
                    let mut context = VcLimitTermContext::default();
                    while let Some(member) = sequence.next_element::<VcLimitContextMemberProbe>()? {
                        context.merge(member.0);
                    }
                    Ok(VcLimitContextMemberSequence(context))
                }
            }

            deserializer.deserialize_seq(Visitor)
        }
    }

    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct VcLimitContextFunctionValue {
        function_id: IgnoredAny,
        contract_hash: IgnoredAny,
        parameters: VcLimitContextParameterSequence,
        requires: VcLimitContextTermSequence,
        members: VcLimitContextMemberSequence,
        groups: IgnoredAny,
    }

    struct VcLimitContextFunctionProbe;

    impl<'de> Deserialize<'de> for VcLimitContextFunctionProbe {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            let raw = <&RawValue>::deserialize(deserializer)?;
            let function = deserialize_context_raw::<VcLimitContextFunctionValue>(raw)
                .map_err(serde::de::Error::custom)?;
            let mut context = function.requires.context;
            context.merge(function.members.0);
            let invalid_variables = if function.parameters.0.overflowed {
                validate_overflowed_parameter_context(raw, &context.variables)
                    .map_err(serde::de::Error::custom)?
            } else {
                context.variables.overflowed
                    || context
                        .variables
                        .names
                        .iter()
                        .any(|name| !function.parameters.0.contains(name))
            };
            if context.required_binder_depth != 0 || invalid_variables {
                return Err(serde::de::Error::custom(
                    "unbound VC variable or open de Bruijn index",
                ));
            }
            Ok(Self)
        }
    }

    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct VcLimitContextRawFunction<'a> {
        function_id: IgnoredAny,
        contract_hash: IgnoredAny,
        #[serde(borrow)]
        parameters: &'a RawValue,
        #[serde(borrow)]
        requires: &'a RawValue,
        #[serde(borrow)]
        members: &'a RawValue,
        groups: IgnoredAny,
    }

    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct VcLimitContextRawMember<'a> {
        id: IgnoredAny,
        function_id: IgnoredAny,
        kind: IgnoredAny,
        local_binders: IgnoredAny,
        #[serde(borrow)]
        assumptions: &'a RawValue,
        #[serde(borrow)]
        conclusion: &'a RawValue,
        group_id: IgnoredAny,
    }

    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct VcLimitContextRawTerm<'a> {
        kind: String,
        #[serde(default, borrow)]
        name: Option<&'a RawValue>,
        #[serde(default, borrow)]
        index: Option<&'a RawValue>,
        #[serde(default, borrow)]
        function: Option<&'a RawValue>,
        #[serde(default, borrow)]
        args: Option<&'a RawValue>,
        #[serde(default, borrow)]
        value: Option<&'a RawValue>,
        #[serde(default, borrow)]
        width: Option<&'a RawValue>,
        #[serde(default, borrow)]
        signed: Option<&'a RawValue>,
        #[serde(default, borrow)]
        target: Option<&'a RawValue>,
        #[serde(default, borrow)]
        binder_type: Option<&'a RawValue>,
        #[serde(default, borrow)]
        body: Option<&'a RawValue>,
    }

    struct VcLimitVariableBatch<'a> {
        after: Option<&'a str>,
        names: BTreeSet<String>,
    }

    impl<'a> VcLimitVariableBatch<'a> {
        fn new(after: Option<&'a str>) -> Self {
            Self {
                after,
                names: BTreeSet::new(),
            }
        }

        fn insert(&mut self, name: String) {
            if self.after.is_some_and(|after| name.as_str() <= after) || self.names.contains(&name)
            {
                return;
            }
            if self.names.len() < crate::vir_validate::VIR_PARAMS_MAX {
                self.names.insert(name);
                return;
            }
            let replace_largest = self
                .names
                .iter()
                .next_back()
                .is_some_and(|largest| name.as_str() < largest.as_str());
            if replace_largest {
                self.names.pop_last();
                self.names.insert(name);
            }
        }
    }

    // An oversized repeated parameter set is a later linkage failure, so it
    // cannot make an earlier unbound-variable scalar failure disappear. Raw
    // slices avoid retaining the malformed collection. When both sides have
    // more than VIR_PARAMS_MAX distinct names, lexicographic batches make the
    // subset check exact while retaining at most VIR_PARAMS_MAX names.
    fn validate_overflowed_parameter_context(
        raw: &RawValue,
        observed_variables: &BoundedNameSet,
    ) -> Result<bool, String> {
        let function = deserialize_context_raw::<VcLimitContextRawFunction<'_>>(raw)?;
        if !observed_variables.overflowed {
            let mut unresolved = observed_variables.names.clone();
            resolve_parameter_names(function.parameters, &mut unresolved)?;
            return Ok(!unresolved.is_empty());
        }

        let mut after = None::<String>;
        loop {
            let mut batch = VcLimitVariableBatch::new(after.as_deref());
            collect_context_term_sequence(function.requires, &mut batch)?;
            collect_context_member_sequence(function.members, &mut batch)?;
            let Some(next_after) = batch.names.iter().next_back().cloned() else {
                return Ok(false);
            };
            let mut unresolved = batch.names;
            resolve_parameter_names(function.parameters, &mut unresolved)?;
            if !unresolved.is_empty() {
                return Ok(true);
            }
            after = Some(next_after);
        }
    }

    fn resolve_parameter_names(
        raw: &RawValue,
        unresolved: &mut BTreeSet<String>,
    ) -> Result<(), String> {
        let mut deserializer = serde_json::Deserializer::from_str(raw.get());
        deserializer.disable_recursion_limit();
        ParameterMembershipSeed { unresolved }
            .deserialize(&mut deserializer)
            .map_err(|error| error.to_string())?;
        deserializer.end().map_err(|error| error.to_string())
    }

    struct ParameterMembershipSeed<'a> {
        unresolved: &'a mut BTreeSet<String>,
    }

    impl<'de> DeserializeSeed<'de> for ParameterMembershipSeed<'_> {
        type Value = ();

        fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            struct Visitor<'a> {
                unresolved: &'a mut BTreeSet<String>,
            }

            impl<'de> serde::de::Visitor<'de> for Visitor<'_> {
                type Value = ();

                fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                    formatter.write_str("a VC parameter array")
                }

                fn visit_seq<A>(self, mut sequence: A) -> Result<Self::Value, A::Error>
                where
                    A: serde::de::SeqAccess<'de>,
                {
                    while let Some(parameter) =
                        sequence.next_element::<VcLimitContextParameterValue>()?
                    {
                        self.unresolved.remove(&parameter.id);
                    }
                    Ok(())
                }
            }

            deserializer.deserialize_seq(Visitor {
                unresolved: self.unresolved,
            })
        }
    }

    fn collect_context_member_sequence(
        raw: &RawValue,
        batch: &mut VcLimitVariableBatch<'_>,
    ) -> Result<(), String> {
        let mut deserializer = serde_json::Deserializer::from_str(raw.get());
        deserializer.disable_recursion_limit();
        ContextMemberSequenceSeed { batch }
            .deserialize(&mut deserializer)
            .map_err(|error| error.to_string())?;
        deserializer.end().map_err(|error| error.to_string())
    }

    struct ContextMemberSequenceSeed<'a, 'cursor> {
        batch: &'a mut VcLimitVariableBatch<'cursor>,
    }

    impl<'de> DeserializeSeed<'de> for ContextMemberSequenceSeed<'_, '_> {
        type Value = ();

        fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            struct Visitor<'a, 'cursor> {
                batch: &'a mut VcLimitVariableBatch<'cursor>,
            }

            impl<'de> serde::de::Visitor<'de> for Visitor<'_, '_> {
                type Value = ();

                fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                    formatter.write_str("a VC member array")
                }

                fn visit_seq<A>(self, mut sequence: A) -> Result<Self::Value, A::Error>
                where
                    A: serde::de::SeqAccess<'de>,
                {
                    let batch = self.batch;
                    while sequence
                        .next_element_seed(ContextMemberSeed { batch: &mut *batch })?
                        .is_some()
                    {}
                    Ok(())
                }
            }

            deserializer.deserialize_seq(Visitor { batch: self.batch })
        }
    }

    struct ContextMemberSeed<'a, 'cursor> {
        batch: &'a mut VcLimitVariableBatch<'cursor>,
    }

    impl<'de> DeserializeSeed<'de> for ContextMemberSeed<'_, '_> {
        type Value = ();

        fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            let raw = <&RawValue>::deserialize(deserializer)?;
            let member = deserialize_context_raw::<VcLimitContextRawMember<'_>>(raw)
                .map_err(serde::de::Error::custom)?;
            collect_context_term_sequence(member.assumptions, self.batch)
                .map_err(serde::de::Error::custom)?;
            collect_context_term(member.conclusion, self.batch).map_err(serde::de::Error::custom)
        }
    }

    fn collect_context_term_sequence(
        raw: &RawValue,
        batch: &mut VcLimitVariableBatch<'_>,
    ) -> Result<(), String> {
        let mut deserializer = serde_json::Deserializer::from_str(raw.get());
        deserializer.disable_recursion_limit();
        ContextTermSequenceSeed { batch }
            .deserialize(&mut deserializer)
            .map_err(|error| error.to_string())?;
        deserializer.end().map_err(|error| error.to_string())
    }

    struct ContextTermSequenceSeed<'a, 'cursor> {
        batch: &'a mut VcLimitVariableBatch<'cursor>,
    }

    impl<'de> DeserializeSeed<'de> for ContextTermSequenceSeed<'_, '_> {
        type Value = ();

        fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            struct Visitor<'a, 'cursor> {
                batch: &'a mut VcLimitVariableBatch<'cursor>,
            }

            impl<'de> serde::de::Visitor<'de> for Visitor<'_, '_> {
                type Value = ();

                fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                    formatter.write_str("a VC term array")
                }

                fn visit_seq<A>(self, mut sequence: A) -> Result<Self::Value, A::Error>
                where
                    A: serde::de::SeqAccess<'de>,
                {
                    let batch = self.batch;
                    while sequence
                        .next_element_seed(ContextTermSeed { batch: &mut *batch })?
                        .is_some()
                    {}
                    Ok(())
                }
            }

            deserializer.deserialize_seq(Visitor { batch: self.batch })
        }
    }

    struct ContextTermSeed<'a, 'cursor> {
        batch: &'a mut VcLimitVariableBatch<'cursor>,
    }

    impl<'de> DeserializeSeed<'de> for ContextTermSeed<'_, '_> {
        type Value = ();

        fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            let raw = <&RawValue>::deserialize(deserializer)?;
            collect_context_term(raw, self.batch).map_err(serde::de::Error::custom)
        }
    }

    fn collect_context_term(
        raw: &RawValue,
        batch: &mut VcLimitVariableBatch<'_>,
    ) -> Result<(), String> {
        let term = deserialize_context_raw::<VcLimitContextRawTerm<'_>>(raw)?;
        match term.kind.as_str() {
            "var" => batch.insert(deserialize_context_raw(required_raw(term.name, "name")?)?),
            "apply" => collect_context_term_sequence(required_raw(term.args, "args")?, batch)?,
            "convert" => collect_context_term(required_raw(term.value, "value")?, batch)?,
            "forall" => collect_context_term(required_raw(term.body, "body")?, batch)?,
            "bound" | "constant" | "bit_vec_literal" => {}
            _ => return Err("unknown VC term kind".to_owned()),
        }
        Ok(())
    }

    fn required_raw<'a>(value: Option<&'a RawValue>, field: &str) -> Result<&'a RawValue, String> {
        value.ok_or_else(|| format!("VC term is missing {field}"))
    }

    fn deserialize_context_raw<'de, T>(raw: &'de RawValue) -> Result<T, String>
    where
        T: Deserialize<'de>,
    {
        let mut deserializer = serde_json::Deserializer::from_str(raw.get());
        deserializer.disable_recursion_limit();
        let value = T::deserialize(&mut deserializer).map_err(|error| error.to_string())?;
        deserializer.end().map_err(|error| error.to_string())?;
        Ok(value)
    }

    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct VcLimitContextRootProbe {
        schema: IgnoredAny,
        source_ir_schema: IgnoredAny,
        source_ir_hash: IgnoredAny,
        input_set_hash: IgnoredAny,
        semantic_profile: IgnoredAny,
        semantic_parameters: IgnoredAny,
        verification_limit_profile: IgnoredAny,
        functions: DiscardingSequence<VcLimitContextFunctionProbe>,
        vc_hash: IgnoredAny,
    }

    pub(super) fn validate_vc_pre_stream_phases(input: &[u8]) -> Result<(), VcValidationError> {
        let mut schema_deserializer = serde_json::Deserializer::from_slice(input);
        schema_deserializer.disable_recursion_limit();
        let schema =
            VcSchemaLimitProbe::deserialize(&mut schema_deserializer).map_err(|error| {
                VcValidationError::new(VcValidationPhase::Shape, "VC_SHAPE", error.to_string())
            })?;
        match schema.0 {
            None => {
                return Err(VcValidationError::new(
                    VcValidationPhase::Shape,
                    "VC_SHAPE",
                    "VC root is missing its schema field",
                ));
            }
            Some(SchemaField::String(value)) if value == VC_SCHEMA_VERSION => {}
            Some(SchemaField::String(_) | SchemaField::Other) => {
                return Err(VcValidationError::new(
                    VcValidationPhase::Shape,
                    "VC_SCHEMA",
                    "wrong VC schema discriminator",
                ));
            }
        }
        let mut shape_deserializer = serde_json::Deserializer::from_slice(input);
        shape_deserializer.disable_recursion_limit();
        let shape =
            VcLimitShapeRootProbe::deserialize(&mut shape_deserializer).map_err(|error| {
                VcValidationError::new(VcValidationPhase::Shape, "VC_SHAPE", error.to_string())
            })?;
        debug_assert_eq!(shape.schema, VC_SCHEMA_VERSION);
        let mut scalar_deserializer = serde_json::Deserializer::from_slice(input);
        scalar_deserializer.disable_recursion_limit();
        let scalar_probe = VcLimitScalarRootProbe::deserialize(&mut scalar_deserializer)
            .map_err(|error| scalar("stream limit scalar validation", error.to_string()))?;
        if scalar_probe.functions.count == 0 {
            return Err(scalar("functions", "VC document has no functions"));
        }
        let mut context_deserializer = serde_json::Deserializer::from_slice(input);
        context_deserializer.disable_recursion_limit();
        VcLimitContextRootProbe::deserialize(&mut context_deserializer)
            .map_err(|error| scalar("stream limit scalar context", error.to_string()))?;
        Ok(())
    }
}

fn observed_verification_add(
    limit: VerificationLimitId,
    current: u64,
    increment: u64,
) -> Result<u64, StrictJsonError> {
    current
        .checked_add(increment)
        .ok_or(StrictJsonError::ObservedCounterOverflow {
            limit: limit.as_str(),
        })
}

fn observed_verification_max(
    limit: VerificationLimitId,
    actual: u64,
) -> Result<(), StrictJsonError> {
    if actual > limit.maximum() {
        Err(StrictJsonError::ObservedLimitExceeded {
            limit: limit.as_str(),
            maximum: limit.maximum(),
            actual,
        })
    } else {
        Ok(())
    }
}

fn vc_members_path(path: &[StrictJsonPathSegment]) -> Option<u64> {
    match path {
        [StrictJsonPathSegment::Key(functions), StrictJsonPathSegment::Index(function), StrictJsonPathSegment::Key(members)]
            if functions == "functions" && members == "members" =>
        {
            Some(*function)
        }
        _ => None,
    }
}

fn vc_assumptions_path(path: &[StrictJsonPathSegment]) -> Option<(u64, u64)> {
    match path {
        [StrictJsonPathSegment::Key(functions), StrictJsonPathSegment::Index(function), StrictJsonPathSegment::Key(members), StrictJsonPathSegment::Index(member), StrictJsonPathSegment::Key(assumptions)]
            if functions == "functions" && members == "members" && assumptions == "assumptions" =>
        {
            Some((*function, *member))
        }
        _ => None,
    }
}

/// Returns the member owner (or document-only ownership for a requirement),
/// the exact branch depth, and whether the current object is an expression
/// rather than an embedded type node.
type VcTermLocation = (Option<(u64, u64)>, u64, bool);

fn vc_term_location(path: &[StrictJsonPathSegment]) -> Option<VcTermLocation> {
    let (owner, suffix) = match path {
        [StrictJsonPathSegment::Key(functions), StrictJsonPathSegment::Index(_), StrictJsonPathSegment::Key(requires), StrictJsonPathSegment::Index(_), suffix @ ..]
            if functions == "functions" && requires == "requires" =>
        {
            (None, suffix)
        }
        [StrictJsonPathSegment::Key(functions), StrictJsonPathSegment::Index(function), StrictJsonPathSegment::Key(members), StrictJsonPathSegment::Index(member), StrictJsonPathSegment::Key(assumptions), StrictJsonPathSegment::Index(_), suffix @ ..]
            if functions == "functions" && members == "members" && assumptions == "assumptions" =>
        {
            (Some((*function, *member)), suffix)
        }
        [StrictJsonPathSegment::Key(functions), StrictJsonPathSegment::Index(function), StrictJsonPathSegment::Key(members), StrictJsonPathSegment::Index(member), StrictJsonPathSegment::Key(conclusion), suffix @ ..]
            if functions == "functions" && members == "members" && conclusion == "conclusion" =>
        {
            (Some((*function, *member)), suffix)
        }
        _ => return None,
    };

    let mut expression = true;
    let mut depth = 1_u64;
    let mut offset = 0;
    while offset < suffix.len() {
        match (&suffix[offset..], expression) {
            ([StrictJsonPathSegment::Key(field), StrictJsonPathSegment::Index(_), ..], _)
                if field == "args" =>
            {
                offset += 2
            }
            ([StrictJsonPathSegment::Key(field), ..], true)
                if matches!(field.as_str(), "value" | "body") =>
            {
                offset += 1;
            }
            ([StrictJsonPathSegment::Key(field), ..], true)
                if matches!(field.as_str(), "target" | "binder_type") =>
            {
                expression = false;
                offset += 1;
            }
            _ => return None,
        }
        depth = depth.checked_add(1)?;
    }
    Some((owner, depth, expression))
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
    let serialized =
        serialize_json_bounded(document, canonical_vc_maximum()).map_err(|error| match error {
            BoundedJsonSerializeError::OutputBytesExceeded { .. } => vc_canonical_size_error(),
            BoundedJsonSerializeError::Serialize(detail) => {
                VcValidationError::new(VcValidationPhase::Shape, "VC_SHAPE", detail)
            }
        })?;
    let strict = parse_strict_json(&serialized, VC_JSON_LIMITS).map_err(|error| {
        VcValidationError::new(VcValidationPhase::Shape, "VC_SHAPE", error.to_string())
    })?;
    canonical_json_bytes_bounded(&strict, canonical_vc_maximum()).map_err(map_vc_canonical_error)
}

pub fn canonical_vc_hash_payload(document: &VcDocument) -> Result<Vec<u8>, VcValidationError> {
    let payload = serialized_strict_value(document, VcValidationPhase::Hash, "VC_HASH")?
        .clone_without_fields(&["vc_hash"])
        .map_err(|error| {
            VcValidationError::new(VcValidationPhase::Shape, "VC_SHAPE", error.to_string())
        })?;
    canonical_json_bytes_bounded(&payload, canonical_vc_maximum()).map_err(map_vc_canonical_error)
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
    let bytes =
        serialize_json_bounded(value, canonical_vc_maximum()).map_err(|error| match error {
            BoundedJsonSerializeError::OutputBytesExceeded { .. } => vc_canonical_size_error(),
            BoundedJsonSerializeError::Serialize(detail) => {
                VcValidationError::new(phase, code, detail)
            }
        })?;
    parse_strict_json(&bytes, VC_JSON_LIMITS)
        .map_err(|error| VcValidationError::new(phase, code, error.to_string()))
}

fn canonical_vc_maximum() -> usize {
    usize::try_from(VerificationLimitId::CanonicalVcJsonBytes.maximum()).unwrap_or(usize::MAX)
}

fn vc_canonical_size_error() -> VcValidationError {
    let limit = VerificationLimitId::CanonicalVcJsonBytes;
    VcValidationError::new(
        VcValidationPhase::CanonicalSize,
        limit.code(),
        format!(
            "{} exceeds inclusive maximum {}",
            limit.as_str(),
            limit.maximum()
        ),
    )
}

fn map_vc_canonical_error(error: CanonicalJsonError) -> VcValidationError {
    if matches!(error, CanonicalJsonError::OutputBytesExceeded { .. }) {
        vc_canonical_size_error()
    } else {
        VcValidationError::new(VcValidationPhase::Shape, "VC_SHAPE", error.to_string())
    }
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

pub(crate) fn validate_bit_vec_literal(
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
