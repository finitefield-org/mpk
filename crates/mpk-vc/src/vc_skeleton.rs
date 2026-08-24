//! Canonical grouped theorem-declaration skeletons for VC v1.

use std::collections::BTreeSet;
use std::error::Error;
use std::fmt;

use serde::{Deserialize, Serialize};

use crate::canonical_json::{
    canonical_json_bytes_bounded, parse_strict_json, scan_strict_json, serialize_json_bounded,
    BoundedJsonSerializeError, CanonicalJsonError, StrictJsonError, StrictJsonEvent,
    StrictJsonLimits, StrictJsonPathSegment, StrictJsonValue, MAX_SAFE_JSON_INTEGER,
};
use crate::grouping::{group_body, GroupingError};
use crate::semantic_profile::{SemanticParameters, SemanticProfile};
use crate::vc::{VcBinder, VcGroupKind, VcSourceContext, VcTerm, VC_SCHEMA_VERSION};
use crate::vc_canonical::{
    import_vc_v1_json, is_mpk_name, validate_type_term, ValidatedVcDocument,
};
use crate::verification_limits::{
    validate_grouped_theorem_limits, validate_verification_limit, VerificationLimitId,
};
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
    scan_skeleton_stream_limits(input)?;
    let strict = parse_strict_json(input, SKELETON_JSON_LIMITS).map_err(map_transport_error)?;
    validate_root_shape(&strict)?;
    limit_precedence::validate_skeleton_pre_stream_phases(input)?;

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

    let canonical = canonical_json_bytes_bounded(&strict, canonical_skeleton_maximum())
        .map_err(map_skeleton_canonical_error)?;
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

fn scan_skeleton_stream_limits(input: &[u8]) -> Result<(), VcSkeletonValidationError> {
    let mut members = 0_u64;
    let mut first_error: Option<StrictJsonError> = None;
    let mut observer = |event: StrictJsonEvent<'_>| -> Result<(), StrictJsonError> {
        let result = (|| {
            if let StrictJsonEvent::ArrayElement { path, count } = event {
                if skeleton_declarations_path(path) {
                    observed_skeleton_max("theorem_declarations", count, 524_288)?;
                } else if skeleton_member_ids_path(path) {
                    members =
                        members
                            .checked_add(1)
                            .ok_or(StrictJsonError::ObservedCounterOverflow {
                                limit: "skeleton_members",
                            })?;
                    observed_skeleton_max("skeleton_members", members, 262_144)?;
                }
            }
            Ok(())
        })();
        if first_error.is_none() {
            first_error = result.err();
        }
        Ok(())
    };
    scan_strict_json(input, SKELETON_JSON_LIMITS, &mut observer).map_err(map_transport_error)?;
    if let Some(error) = first_error {
        limit_precedence::validate_skeleton_pre_stream_phases(input)?;
        return Err(VcSkeletonValidationError::new(
            VcSkeletonValidationPhase::StreamLimits,
            "VC_SKELETON_SHAPE",
            error.to_string(),
        ));
    }
    Ok(())
}

#[allow(dead_code)]
mod limit_precedence {
    use super::*;
    use serde::de::IgnoredAny;
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

    struct SkeletonSchemaLimitProbe(Option<SchemaField>);

    impl<'de> Deserialize<'de> for SkeletonSchemaLimitProbe {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            struct Visitor;

            impl<'de> serde::de::Visitor<'de> for Visitor {
                type Value = SkeletonSchemaLimitProbe;

                fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                    formatter.write_str("a skeleton root object")
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
                    Ok(SkeletonSchemaLimitProbe(schema))
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
            struct Visitor<T>(PhantomData<fn() -> T>);

            impl<'de, T> serde::de::Visitor<'de> for Visitor<T>
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

            deserializer.deserialize_seq(Visitor(PhantomData))
        }
    }

    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct SkeletonShapeRootProbe {
        schema: String,
        source_vc_schema: String,
        source_vc_hash: String,
        source_ir_schema: String,
        source_ir_hash: String,
        input_set_hash: String,
        semantic_profile: String,
        semantic_parameters: SemanticParametersShapeProbe,
        verification_limit_profile: String,
        theorem_declarations: DiscardingSequence<SkeletonShapeDeclarationProbe>,
    }

    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct SkeletonShapeDeclarationProbe {
        name: String,
        function_id: String,
        group_id: String,
        group_kind: VcGroupKind,
        member_ids: DiscardingSequence<String>,
        dependencies: DiscardingSequence<String>,
        theorem_type: SkeletonShapeTheoremProbe,
    }

    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct SkeletonShapeTheoremProbe {
        binders: DiscardingSequence<SkeletonShapeBinderProbe>,
        body: SkeletonShapeTermProbe,
    }

    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct SkeletonShapeBinderProbe {
        id: String,
        #[serde(rename = "type")]
        r#type: SkeletonShapeTypeProbe,
    }

    #[derive(Deserialize)]
    #[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
    enum SkeletonShapeTypeProbe {
        Constant {
            name: String,
        },
        Apply {
            function: String,
            args: DiscardingSequence<SkeletonShapeTypeProbe>,
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
    enum SkeletonShapeTermProbe {
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
            args: DiscardingSequence<SkeletonShapeTermProbe>,
        },
        Convert {
            value: Box<SkeletonShapeTermProbe>,
            target: SkeletonShapeTypeProbe,
        },
        Forall {
            binder_type: SkeletonShapeTypeProbe,
            body: Box<SkeletonShapeTermProbe>,
        },
    }

    struct ValidSha256;

    impl<'de> Deserialize<'de> for ValidSha256 {
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
                return Err(serde::de::Error::custom("invalid MPK name"));
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

    struct SkeletonScalarTypeProbe;

    #[derive(Deserialize)]
    #[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
    enum SkeletonScalarTypeValue {
        Constant {
            name: ValidMpkName,
        },
        Apply {
            function: ValidMpkName,
            args: DiscardingSequence<SkeletonScalarTypeProbe>,
        },
        NatLiteral {
            value: NonnegativeU64,
        },
        StringLiteral {
            value: String,
        },
    }

    impl<'de> Deserialize<'de> for SkeletonScalarTypeProbe {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            let value = SkeletonScalarTypeValue::deserialize(deserializer)?;
            if let SkeletonScalarTypeValue::NatLiteral { value } = value {
                if value.0 > MAX_SAFE_JSON_INTEGER as u64 {
                    return Err(serde::de::Error::custom(
                        "type nat_literal is outside the safe JSON range",
                    ));
                }
            }
            Ok(Self)
        }
    }

    struct SkeletonScalarTermProbe(u64);

    struct SkeletonScalarTermSequence(u64);

    impl<'de> Deserialize<'de> for SkeletonScalarTermSequence {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            struct Visitor;

            impl<'de> serde::de::Visitor<'de> for Visitor {
                type Value = SkeletonScalarTermSequence;

                fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                    formatter.write_str("a theorem-term array")
                }

                fn visit_seq<A>(self, mut sequence: A) -> Result<Self::Value, A::Error>
                where
                    A: serde::de::SeqAccess<'de>,
                {
                    let mut required_depth = 0_u64;
                    while let Some(term) = sequence.next_element::<SkeletonScalarTermProbe>()? {
                        required_depth = required_depth.max(term.0);
                    }
                    Ok(SkeletonScalarTermSequence(required_depth))
                }
            }

            deserializer.deserialize_seq(Visitor)
        }
    }

    #[derive(Deserialize)]
    #[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
    enum SkeletonScalarTermValue {
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
            args: SkeletonScalarTermSequence,
        },
        Convert {
            value: Box<SkeletonScalarTermProbe>,
            target: SkeletonScalarTypeProbe,
        },
        Forall {
            binder_type: SkeletonScalarTypeProbe,
            body: Box<SkeletonScalarTermProbe>,
        },
    }

    impl<'de> Deserialize<'de> for SkeletonScalarTermProbe {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            let value = SkeletonScalarTermValue::deserialize(deserializer)?;
            let required_depth = match value {
                SkeletonScalarTermValue::Bound { index } => u64::from(index.0) + 1,
                SkeletonScalarTermValue::Apply { args, .. } => args.0,
                SkeletonScalarTermValue::Convert { value, .. } => value.0,
                SkeletonScalarTermValue::Forall { body, .. } => body.0.saturating_sub(1),
                SkeletonScalarTermValue::BitVecLiteral {
                    value,
                    width,
                    signed,
                } => {
                    crate::vc_canonical::validate_bit_vec_literal(&value, width.0, signed)
                        .map_err(serde::de::Error::custom)?;
                    0
                }
                SkeletonScalarTermValue::Var { .. } | SkeletonScalarTermValue::Constant { .. } => 0,
            };
            Ok(Self(required_depth))
        }
    }

    struct SkeletonScalarBinderSequence;

    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct SkeletonScalarBinderValue {
        id: String,
        #[serde(rename = "type")]
        r#type: SkeletonScalarTypeProbe,
    }

    impl<'de> Deserialize<'de> for SkeletonScalarBinderSequence {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            struct Visitor;

            impl<'de> serde::de::Visitor<'de> for Visitor {
                type Value = SkeletonScalarBinderSequence;

                fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                    formatter.write_str("a theorem binder array")
                }

                fn visit_seq<A>(self, mut sequence: A) -> Result<Self::Value, A::Error>
                where
                    A: serde::de::SeqAccess<'de>,
                {
                    let mut ids = BTreeSet::new();
                    let mut count = 0_usize;
                    while let Some(binder) = sequence.next_element::<SkeletonScalarBinderValue>()? {
                        count = count
                            .checked_add(1)
                            .ok_or_else(|| serde::de::Error::custom("binder count overflow"))?;
                        if count > crate::vir_validate::VIR_PARAMS_MAX {
                            return Err(serde::de::Error::custom(
                                "theorem binder count exceeds the inherited VIR limit",
                            ));
                        }
                        if !ids.insert(binder.id) {
                            return Err(serde::de::Error::custom("duplicate outer binder id"));
                        }
                    }
                    Ok(SkeletonScalarBinderSequence)
                }
            }

            deserializer.deserialize_seq(Visitor)
        }
    }

    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct SkeletonScalarTheoremValue {
        binders: SkeletonScalarBinderSequence,
        body: SkeletonScalarTermProbe,
    }

    struct SkeletonScalarTheoremProbe;

    impl<'de> Deserialize<'de> for SkeletonScalarTheoremProbe {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            let theorem = SkeletonScalarTheoremValue::deserialize(deserializer)?;
            if theorem.body.0 != 0 {
                return Err(serde::de::Error::custom("open de Bruijn index"));
            }
            Ok(Self)
        }
    }

    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct SkeletonScalarDeclarationProbe {
        name: ValidMpkName,
        function_id: IgnoredAny,
        group_id: IgnoredAny,
        group_kind: IgnoredAny,
        member_ids: DiscardingSequence<IgnoredAny>,
        dependencies: DiscardingSequence<ValidMpkName>,
        theorem_type: SkeletonScalarTheoremProbe,
    }

    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct SkeletonScalarRootProbe {
        schema: IgnoredAny,
        source_vc_schema: IgnoredAny,
        source_vc_hash: ValidSha256,
        source_ir_schema: IgnoredAny,
        source_ir_hash: ValidSha256,
        input_set_hash: ValidSha256,
        semantic_profile: crate::SemanticProfile,
        semantic_parameters: crate::SemanticParameters,
        verification_limit_profile: IgnoredAny,
        theorem_declarations: DiscardingSequence<SkeletonScalarDeclarationProbe>,
    }

    pub(super) fn validate_skeleton_pre_stream_phases(
        input: &[u8],
    ) -> Result<(), VcSkeletonValidationError> {
        let mut schema_deserializer = serde_json::Deserializer::from_slice(input);
        schema_deserializer.disable_recursion_limit();
        let schema_probe = SkeletonSchemaLimitProbe::deserialize(&mut schema_deserializer)
            .map_err(|error| {
                VcSkeletonValidationError::new(
                    VcSkeletonValidationPhase::Shape,
                    "VC_SKELETON_SHAPE",
                    error.to_string(),
                )
            })?;
        match schema_probe.0 {
            None => {
                return Err(VcSkeletonValidationError::new(
                    VcSkeletonValidationPhase::Shape,
                    "VC_SKELETON_SHAPE",
                    "skeleton root is missing its schema field",
                ));
            }
            Some(SchemaField::String(value)) if value == VC_CERT_SKELETON_V1_SCHEMA_VERSION => {}
            Some(SchemaField::String(_) | SchemaField::Other) => {
                return Err(VcSkeletonValidationError::new(
                    VcSkeletonValidationPhase::Shape,
                    "VC_SKELETON_SCHEMA",
                    "wrong skeleton schema discriminator",
                ));
            }
        }

        let mut shape_deserializer = serde_json::Deserializer::from_slice(input);
        shape_deserializer.disable_recursion_limit();
        let shape =
            SkeletonShapeRootProbe::deserialize(&mut shape_deserializer).map_err(|error| {
                VcSkeletonValidationError::new(
                    VcSkeletonValidationPhase::Shape,
                    "VC_SKELETON_SHAPE",
                    error.to_string(),
                )
            })?;
        debug_assert_eq!(shape.schema, VC_CERT_SKELETON_V1_SCHEMA_VERSION);

        let mut scalar_deserializer = serde_json::Deserializer::from_slice(input);
        scalar_deserializer.disable_recursion_limit();
        SkeletonScalarRootProbe::deserialize(&mut scalar_deserializer).map_err(|error| {
            VcSkeletonValidationError::new(
                VcSkeletonValidationPhase::Scalar,
                "VC_SKELETON_SHAPE",
                error.to_string(),
            )
        })?;
        Ok(())
    }
}

fn observed_skeleton_max(
    limit: &'static str,
    actual: u64,
    maximum: u64,
) -> Result<(), StrictJsonError> {
    if actual > maximum {
        Err(StrictJsonError::ObservedLimitExceeded {
            limit,
            maximum,
            actual,
        })
    } else {
        Ok(())
    }
}

fn skeleton_declarations_path(path: &[StrictJsonPathSegment]) -> bool {
    matches!(
        path,
        [StrictJsonPathSegment::Key(name)] if name == "theorem_declarations"
    )
}

fn skeleton_member_ids_path(path: &[StrictJsonPathSegment]) -> bool {
    matches!(
        path,
        [
            StrictJsonPathSegment::Key(declarations),
            StrictJsonPathSegment::Index(_),
            StrictJsonPathSegment::Key(member_ids),
        ] if declarations == "theorem_declarations" && member_ids == "member_ids"
    )
}

pub fn canonical_skeleton_json(
    skeleton: &VcCertificateSkeletonV1,
) -> Result<Vec<u8>, VcSkeletonValidationError> {
    let bytes = serialize_json_bounded(skeleton, canonical_skeleton_maximum()).map_err(
        |error| match error {
            BoundedJsonSerializeError::OutputBytesExceeded { .. } => {
                skeleton_canonical_size_error()
            }
            BoundedJsonSerializeError::Serialize(detail) => VcSkeletonValidationError::new(
                VcSkeletonValidationPhase::Shape,
                "VC_SKELETON_SHAPE",
                detail,
            ),
        },
    )?;
    let strict = parse_strict_json(&bytes, SKELETON_JSON_LIMITS).map_err(map_transport_error)?;
    canonical_json_bytes_bounded(&strict, canonical_skeleton_maximum())
        .map_err(map_skeleton_canonical_error)
}

fn canonical_skeleton_maximum() -> usize {
    usize::try_from(VerificationLimitId::CanonicalSkeletonJsonBytes.maximum()).unwrap_or(usize::MAX)
}

fn skeleton_canonical_size_error() -> VcSkeletonValidationError {
    let limit = VerificationLimitId::CanonicalSkeletonJsonBytes;
    VcSkeletonValidationError::new(
        VcSkeletonValidationPhase::CanonicalSize,
        limit.code(),
        format!(
            "{} exceeds inclusive maximum {}",
            limit.as_str(),
            limit.maximum()
        ),
    )
}

fn map_skeleton_canonical_error(error: CanonicalJsonError) -> VcSkeletonValidationError {
    if matches!(error, CanonicalJsonError::OutputBytesExceeded { .. }) {
        skeleton_canonical_size_error()
    } else {
        VcSkeletonValidationError::new(
            VcSkeletonValidationPhase::Shape,
            "VC_SKELETON_SHAPE",
            error.to_string(),
        )
    }
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
    let capacity = document
        .functions
        .len()
        .checked_mul(2)
        .ok_or_else(skeleton_canonical_size_error)?;
    let mut theorem_declarations = Vec::with_capacity(capacity);
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
        if declaration.theorem_type.binders.len() > crate::vir_validate::VIR_PARAMS_MAX {
            return Err(scalar(
                "binders",
                "theorem binder count exceeds the inherited VIR limit",
            ));
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
        validate_skeleton_term_scalars(&declaration.theorem_type.body, 0)?;
    }
    Ok(())
}

fn validate_skeleton_term_scalars(
    term: &VcTerm,
    binder_depth: usize,
) -> Result<(), VcSkeletonValidationError> {
    match term {
        // Binder-to-body equality is a theorem-type reconstruction rule, not
        // a scalar rule. Any variable label is admitted here and compared
        // against the reconstructed theorem later.
        VcTerm::Var { .. } => Ok(()),
        VcTerm::Bound { index } => {
            if usize::try_from(*index)
                .ok()
                .is_some_and(|index| index < binder_depth)
            {
                Ok(())
            } else {
                Err(scalar("theorem_type", "open de Bruijn index"))
            }
        }
        VcTerm::Constant { name } => {
            if is_mpk_name(name) {
                Ok(())
            } else {
                Err(scalar("theorem_type", "invalid constant MPK name"))
            }
        }
        VcTerm::BitVecLiteral {
            value,
            width,
            signed,
        } => crate::vc_canonical::validate_bit_vec_literal(value, *width, *signed)
            .map_err(map_scalar_from_vc),
        VcTerm::Apply { function, args } => {
            if !is_mpk_name(function) {
                return Err(scalar("theorem_type", "invalid application MPK name"));
            }
            for argument in args {
                validate_skeleton_term_scalars(argument, binder_depth)?;
            }
            Ok(())
        }
        VcTerm::Convert { value, target } => {
            validate_skeleton_term_scalars(value, binder_depth)?;
            validate_type_term(target).map_err(map_scalar_from_vc)
        }
        VcTerm::Forall { binder_type, body } => {
            validate_type_term(binder_type).map_err(map_scalar_from_vc)?;
            let nested = binder_depth
                .checked_add(1)
                .ok_or_else(|| scalar("theorem_type", "binder depth overflow"))?;
            validate_skeleton_term_scalars(body, nested)
        }
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
