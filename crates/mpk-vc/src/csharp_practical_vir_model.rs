//! Private candidate foundation and monomorphic value model for practical C#.
//!
//! This module is deliberately not wired to an installed registry, CLI, or API.
//! Callers must explicitly inject the one frozen descriptor and a validated
//! closed root/provenance document before specialization.

use crate::canonical_json::{
    canonical_json_bytes_bounded, parse_strict_json, StrictJsonError, StrictJsonLimits,
    StrictJsonValue,
};
use crate::csharp_practical_registry::{
    SuccessorSemanticContext, CSHARP_PRACTICAL_PROFILE, FOUNDATION_DESCRIPTOR_CONTENT_SHA256,
    FOUNDATION_DESCRIPTOR_ID, FOUNDATION_DESCRIPTOR_SCHEMA,
};
use crate::hash::{hash_canonical_json, sha256_raw_file_bytes, HashDomain};
use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Number, Value};
use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use std::fmt;

pub const FOUNDATION_DEFINITIONS_SCHEMA: &str = "mpk.csharp.foundation_definitions.v1";
pub const FOUNDATION_EXPANSION_SCHEMA: &str = "mpk.csharp.foundation_expansion.v1";
pub const CLOSED_ROOTS_SCHEMA: &str = "mpk.csharp.closed_roots.v1";
pub const CLOSED_INSTANCES_SCHEMA: &str = "mpk.csharp.closed_instances.v1";

pub const FOUNDATION_DESCRIPTOR_HASH_DOMAIN: HashDomain =
    HashDomain::new("MPK-CSHARP-PRACTICAL-FOUNDATION-1.0");
pub const CLOSED_INSTANCE_HASH_DOMAIN: HashDomain =
    HashDomain::new("MPK-CSHARP-SEMANTIC-INSTANCE-1.0");
pub const CLOSED_INSTANCE_SET_HASH_DOMAIN: HashDomain =
    HashDomain::new("MPK-CSHARP-CLOSED-INSTANCES-1.0");
pub const DECLARATION_HASH_DOMAIN: HashDomain = HashDomain::new("MPK-CSHARP-DECLARATION-1.0");
pub const STORED_MEMBER_HASH_DOMAIN: HashDomain =
    HashDomain::new("MPK-CSHARP-FOUNDATION-MEMBER-1.0");

pub const FOUNDATION_BINDING_COUNT_MAX: u64 = 128;
pub const CLOSED_INSTANCE_COUNT_MAX: u64 = 256;
pub const CLOSED_INSTANCE_DEPTH_MAX: u64 = 16;
pub const EXPANDED_DECLARATIONS_MAX: u64 = 1_024;
pub const EXPANDED_OPERATIONS_MAX: u64 = 4_096;
pub const EXPANDED_RECIPE_NODES_MAX: u64 = 262_144;
pub const PROJECTION_OBLIGATIONS_PER_BINDING_MAX: u64 = 64;

pub const ARRAY_VALUE_LENGTH_MAX: u64 = 4_096;
pub const SEQUENCE_VALUE_LENGTH_MAX: u64 = 4_096;
pub const STRING_VALUE_LENGTH_MAX: u64 = 16_384;
pub const MAP_VALUE_LENGTH_MAX: u64 = 4_096;
pub const SET_VALUE_LENGTH_MAX: u64 = 4_096;
pub const VALIDATION_ERRORS_MAX: u64 = 256;
pub const TRANSITION_EVENTS_MAX: u64 = 4_096;
pub const TOTAL_VALUE_CELLS_MAX: u64 = 65_536;

const FOUNDATION_TRANSPORT_BYTES_MAX: u64 = 1_048_576;
const FOUNDATION_JSON_DEPTH_MAX: u64 = 64;
const FOUNDATION_STRING_BYTES_MAX: u64 = 262_144;
const FOUNDATION_DEFINITIONS_RAW_SHA256: &str =
    "25738447bf793e37dc2125e7a07da55a03fb15f2fa4dfb87b25646a16cc9d1b4";
const FOUNDATION_SEMANTICS_RAW_SHA256: &str =
    "29c5986e3c7ce2ab018e36eea61caaf9d9e53d6b8e47f0229ef4681db8c3fc8b";
const FOUNDATION_DEFINITIONS_SIZE_BYTES: u64 = 18_467;
const FOUNDATION_SEMANTICS_SIZE_BYTES: u64 = 54_806;
const FOUNDATION_DEFINITIONS_PATH: &str =
    "develop/migrations/csharp-03/foundation/foundation-definitions.json";
const FOUNDATION_SEMANTICS_PATH: &str = "develop/specs/CSHARP_PRACTICAL_FOUNDATION_V1.md";

const REGISTERED_DESCRIPTOR_TRANSPORT: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../develop/migrations/csharp-03/foundation/foundation-descriptor.json"
));
const REGISTERED_DEFINITIONS_TRANSPORT: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../develop/migrations/csharp-03/foundation/foundation-definitions.json"
));
const REGISTERED_SEMANTICS_TRANSPORT: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../develop/specs/CSHARP_PRACTICAL_FOUNDATION_V1.md"
));

const PRIMITIVES: &[&str] = &[
    "bool",
    "i8",
    "u8",
    "i16",
    "u16",
    "i32",
    "u32",
    "i64",
    "u64",
    "char",
    "f32",
    "f64",
    "decimal",
    "string",
    "date",
    "time",
    "duration",
    "guid",
    "day_of_week",
    "unit",
    "parse_error",
    "instant",
    "exception",
];

const ROOT_ORIGINS: &[&str] = &[
    "source_array",
    "source_nullable",
    "source_string",
    "source_construction",
    "semantic_binding",
    "contract",
    "boundary",
    "transition",
    "codec_result",
];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FoundationValidationPhase {
    Transport,
    Descriptor,
    ContextLinkage,
    RootSet,
    SourceTypes,
    Type,
    Closure,
    Expansion,
    ClosedSet,
    ConcreteValue,
    Limits,
}

impl FoundationValidationPhase {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Transport => "transport",
            Self::Descriptor => "descriptor",
            Self::ContextLinkage => "context_linkage",
            Self::RootSet => "root_set",
            Self::SourceTypes => "source_types",
            Self::Type => "type",
            Self::Closure => "closure",
            Self::Expansion => "expansion",
            Self::ClosedSet => "closed_set",
            Self::ConcreteValue => "concrete_value",
            Self::Limits => "limits",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FoundationErrorCode {
    Transport,
    DuplicateJsonKey,
    FloatingJson,
    NonfiniteJson,
    CanonicalTransport,
    DescriptorRecomputation,
    ContextLinkage,
    RootSetShape,
    RootShape,
    RootProvenance,
    DuplicateRoot,
    RootDerivationSource,
    SourceTable,
    SourceShape,
    IdentityShape,
    SourceIdentity,
    SourceKind,
    SourceHash,
    SourceFlags,
    StoredMemberShape,
    StoredMemberIdentity,
    StoredMemberOrderOrStorage,
    RequiredStorage,
    NonvalueMember,
    SourceDefaultShape,
    EnumShape,
    SourceCycle,
    TypeShape,
    UnknownType,
    UnknownSourceType,
    UnknownTemplate,
    TemplateArity,
    NonvalueArgument,
    NestedOption,
    NonTotalKey,
    CurrencyType,
    GenericOrUnknownType,
    InstanceDepth,
    FoundationHash,
    BindingCount,
    ClosedInstanceCount,
    ClosedInstanceDepth,
    InstanceCount,
    InstanceCollision,
    ParameterShape,
    ParameterArity,
    ResidualGeneric,
    ExpandedDeclarations,
    ExpandedOperations,
    ExpandedRecipeNodes,
    ProjectionObligationsPerBinding,
    ClosedSetRecomputation,
    ConcreteValueShape,
    ConcreteValueType,
    ConcreteValueInvariant,
    ConcreteValueBound,
    TotalValueCells,
    LimitExceeded,
}

impl FoundationErrorCode {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Transport => "transport",
            Self::DuplicateJsonKey => "duplicate_json_key",
            Self::FloatingJson => "floating_json",
            Self::NonfiniteJson => "nonfinite_json",
            Self::CanonicalTransport => "canonical_transport",
            Self::DescriptorRecomputation => "descriptor_recomputation",
            Self::ContextLinkage => "context_linkage",
            Self::RootSetShape => "root_set_shape",
            Self::RootShape => "root_shape",
            Self::RootProvenance => "root_provenance",
            Self::DuplicateRoot => "duplicate_root",
            Self::RootDerivationSource => "root_derivation_source",
            Self::SourceTable => "source_table",
            Self::SourceShape => "source_shape",
            Self::IdentityShape => "identity_shape",
            Self::SourceIdentity => "source_identity",
            Self::SourceKind => "source_kind",
            Self::SourceHash => "source_hash",
            Self::SourceFlags => "source_flags",
            Self::StoredMemberShape => "stored_member_shape",
            Self::StoredMemberIdentity => "stored_member_identity",
            Self::StoredMemberOrderOrStorage => "stored_member_order_or_storage",
            Self::RequiredStorage => "required_storage",
            Self::NonvalueMember => "nonvalue_member",
            Self::SourceDefaultShape => "source_default_shape",
            Self::EnumShape => "enum_shape",
            Self::SourceCycle => "source_cycle",
            Self::TypeShape => "type_shape",
            Self::UnknownType => "unknown_type",
            Self::UnknownSourceType => "unknown_source_type",
            Self::UnknownTemplate => "unknown_template",
            Self::TemplateArity => "template_arity",
            Self::NonvalueArgument => "nonvalue_argument",
            Self::NestedOption => "nested_option",
            Self::NonTotalKey => "non_total_key",
            Self::CurrencyType => "currency_type",
            Self::GenericOrUnknownType => "generic_or_unknown_type",
            Self::InstanceDepth => "instance_depth",
            Self::FoundationHash => "foundation_hash",
            Self::BindingCount => "binding_count",
            Self::ClosedInstanceCount => "closed_instance_count",
            Self::ClosedInstanceDepth => "closed_instance_depth",
            Self::InstanceCount => "instance_count",
            Self::InstanceCollision => "instance_collision",
            Self::ParameterShape => "parameter_shape",
            Self::ParameterArity => "parameter_arity",
            Self::ResidualGeneric => "residual_generic",
            Self::ExpandedDeclarations => "expanded_declarations",
            Self::ExpandedOperations => "expanded_operations",
            Self::ExpandedRecipeNodes => "expanded_recipe_nodes",
            Self::ProjectionObligationsPerBinding => "projection_obligations_per_binding",
            Self::ClosedSetRecomputation => "closed_set_recomputation",
            Self::ConcreteValueShape => "concrete_value_shape",
            Self::ConcreteValueType => "concrete_value_type",
            Self::ConcreteValueInvariant => "concrete_value_invariant",
            Self::ConcreteValueBound => "concrete_value_bound",
            Self::TotalValueCells => "total_value_cells",
            Self::LimitExceeded => "limit_exceeded",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FoundationValidationError {
    phase: FoundationValidationPhase,
    code: FoundationErrorCode,
}

impl FoundationValidationError {
    pub const fn phase(&self) -> FoundationValidationPhase {
        self.phase
    }

    pub const fn code(&self) -> FoundationErrorCode {
        self.code
    }
}

impl fmt::Display for FoundationValidationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{} at practical-foundation phase {}",
            self.code.as_str(),
            self.phase.as_str()
        )
    }
}

impl Error for FoundationValidationError {}

fn failure(
    phase: FoundationValidationPhase,
    code: FoundationErrorCode,
) -> FoundationValidationError {
    FoundationValidationError { phase, code }
}

#[derive(Clone, Debug, Deserialize)]
struct DefinitionInventory {
    schema: String,
    foundation_id: String,
    version: u64,
    templates: Vec<TemplateDefinition>,
    non_templates: Vec<Value>,
    ordinary_core: Value,
}

#[derive(Clone, Debug, Deserialize)]
struct TemplateDefinition {
    id: String,
    name: String,
    version: u64,
    arity: usize,
    dependencies: Vec<Value>,
    representation: Value,
    operations: Vec<OperationDefinition>,
    default: String,
    derivation_sources: Vec<String>,
    source_callable: bool,
}

#[derive(Clone, Debug, Deserialize)]
struct OperationDefinition {
    name: String,
    arguments: Vec<String>,
    result: String,
    equation: String,
    error_precedence: Vec<String>,
}

#[derive(Clone, Debug)]
pub struct ValidatedFoundationBundle {
    descriptor: Value,
    definitions: Value,
    templates: BTreeMap<String, TemplateDefinition>,
    content_sha256: String,
}

impl ValidatedFoundationBundle {
    pub fn descriptor(&self) -> &Value {
        &self.descriptor
    }

    pub fn definitions(&self) -> &Value {
        &self.definitions
    }

    pub fn content_sha256(&self) -> &str {
        &self.content_sha256
    }

    pub fn template_names(&self) -> impl Iterator<Item = &str> {
        self.templates.keys().map(String::as_str)
    }

    pub fn non_template_definitions(&self) -> &[Value] {
        self.definitions["non_templates"]
            .as_array()
            .expect("validated definitions retain non_templates")
    }
}

pub const fn registered_foundation_descriptor_transport() -> &'static [u8] {
    REGISTERED_DESCRIPTOR_TRANSPORT
}

pub const fn registered_foundation_definitions_transport() -> &'static [u8] {
    REGISTERED_DEFINITIONS_TRANSPORT
}

pub fn validate_registered_foundation_bundle(
    descriptor_transport: &[u8],
    definitions_transport: &[u8],
) -> Result<ValidatedFoundationBundle, FoundationValidationError> {
    let descriptor_strict = parse_canonical_transport(
        descriptor_transport,
        FOUNDATION_TRANSPORT_BYTES_MAX,
        FoundationValidationPhase::Descriptor,
    )?;
    let definitions_strict = parse_canonical_transport(
        definitions_transport,
        FOUNDATION_TRANSPORT_BYTES_MAX,
        FoundationValidationPhase::Descriptor,
    )?;
    let expected_descriptor = parse_canonical_transport(
        REGISTERED_DESCRIPTOR_TRANSPORT,
        FOUNDATION_TRANSPORT_BYTES_MAX,
        FoundationValidationPhase::Descriptor,
    )?;
    let expected_definitions = parse_canonical_transport(
        REGISTERED_DEFINITIONS_TRANSPORT,
        FOUNDATION_TRANSPORT_BYTES_MAX,
        FoundationValidationPhase::Descriptor,
    )?;

    if definitions_strict != expected_definitions
        || sha256_raw_file_bytes(definitions_transport).to_hex()
            != FOUNDATION_DEFINITIONS_RAW_SHA256
        || u64::try_from(definitions_transport.len()).ok()
            != Some(FOUNDATION_DEFINITIONS_SIZE_BYTES)
    {
        return Err(failure(
            FoundationValidationPhase::Descriptor,
            FoundationErrorCode::DescriptorRecomputation,
        ));
    }

    let descriptor_without_hash = descriptor_strict
        .clone_without_fields(&["content_sha256"])
        .map_err(|_| {
            failure(
                FoundationValidationPhase::Descriptor,
                FoundationErrorCode::DescriptorRecomputation,
            )
        })?;
    let recomputed =
        hash_canonical_json(FOUNDATION_DESCRIPTOR_HASH_DOMAIN, &descriptor_without_hash)
            .map_err(|_| {
                failure(
                    FoundationValidationPhase::Descriptor,
                    FoundationErrorCode::DescriptorRecomputation,
                )
            })?
            .to_hex();
    let declared = descriptor_strict
        .get("content_sha256")
        .and_then(StrictJsonValue::as_str);
    if descriptor_strict != expected_descriptor
        || declared != Some(recomputed.as_str())
        || recomputed != FOUNDATION_DESCRIPTOR_CONTENT_SHA256
    {
        return Err(failure(
            FoundationValidationPhase::Descriptor,
            FoundationErrorCode::DescriptorRecomputation,
        ));
    }

    let descriptor = strict_to_serde(&descriptor_strict);
    let definitions = strict_to_serde(&definitions_strict);
    validate_descriptor_members(&descriptor, definitions_transport)?;
    let inventory: DefinitionInventory =
        serde_json::from_value(definitions.clone()).map_err(|_| {
            failure(
                FoundationValidationPhase::Descriptor,
                FoundationErrorCode::DescriptorRecomputation,
            )
        })?;
    let templates = validate_definition_inventory(&descriptor, &inventory)?;

    Ok(ValidatedFoundationBundle {
        descriptor,
        definitions,
        templates,
        content_sha256: recomputed,
    })
}

fn validate_descriptor_members(
    descriptor: &Value,
    definitions_transport: &[u8],
) -> Result<(), FoundationValidationError> {
    let members = descriptor["members"].as_array().ok_or_else(|| {
        failure(
            FoundationValidationPhase::Descriptor,
            FoundationErrorCode::DescriptorRecomputation,
        )
    })?;
    let expected = [
        (
            FOUNDATION_DEFINITIONS_PATH,
            FOUNDATION_DEFINITIONS_SCHEMA,
            FOUNDATION_DEFINITIONS_RAW_SHA256,
            FOUNDATION_DEFINITIONS_SIZE_BYTES,
        ),
        (
            FOUNDATION_SEMANTICS_PATH,
            "mpk.csharp.foundation_semantics.v1",
            FOUNDATION_SEMANTICS_RAW_SHA256,
            FOUNDATION_SEMANTICS_SIZE_BYTES,
        ),
    ];
    if members.len() != expected.len() {
        return Err(failure(
            FoundationValidationPhase::Descriptor,
            FoundationErrorCode::DescriptorRecomputation,
        ));
    }
    for (member, (path, schema, sha256, size)) in members.iter().zip(expected) {
        if !has_exact_fields(member, &["path", "schema", "sha256", "size_bytes"])
            || member["path"] != path
            || member["schema"] != schema
            || member["sha256"] != sha256
            || member["size_bytes"] != size
        {
            return Err(failure(
                FoundationValidationPhase::Descriptor,
                FoundationErrorCode::DescriptorRecomputation,
            ));
        }
    }
    if sha256_raw_file_bytes(definitions_transport).to_hex() != FOUNDATION_DEFINITIONS_RAW_SHA256 {
        return Err(failure(
            FoundationValidationPhase::Descriptor,
            FoundationErrorCode::DescriptorRecomputation,
        ));
    }
    if sha256_raw_file_bytes(REGISTERED_SEMANTICS_TRANSPORT).to_hex()
        != FOUNDATION_SEMANTICS_RAW_SHA256
        || u64::try_from(REGISTERED_SEMANTICS_TRANSPORT.len()).ok()
            != Some(FOUNDATION_SEMANTICS_SIZE_BYTES)
    {
        return Err(failure(
            FoundationValidationPhase::Descriptor,
            FoundationErrorCode::DescriptorRecomputation,
        ));
    }
    Ok(())
}

fn validate_definition_inventory(
    descriptor: &Value,
    inventory: &DefinitionInventory,
) -> Result<BTreeMap<String, TemplateDefinition>, FoundationValidationError> {
    let invalid = || {
        failure(
            FoundationValidationPhase::Descriptor,
            FoundationErrorCode::DescriptorRecomputation,
        )
    };
    if inventory.schema != FOUNDATION_DEFINITIONS_SCHEMA
        || inventory.foundation_id != FOUNDATION_DESCRIPTOR_ID
        || inventory.version != 1
        || inventory.templates.len() != 12
        || inventory.non_templates.len() != 4
        || inventory.ordinary_core["schema"] != FOUNDATION_EXPANSION_SCHEMA
    {
        return Err(invalid());
    }
    let mut templates = BTreeMap::new();
    for template in &inventory.templates {
        if template.id != format!("mpk.csharp.semantic.{}.v1", template.name)
            || template.version != 1
            || template.source_callable
            || template.arity == 0
            || template.operations.is_empty()
            || template.default.is_empty()
            || template.derivation_sources.is_empty()
            || templates
                .insert(template.name.clone(), template.clone())
                .is_some()
        {
            return Err(invalid());
        }
    }
    let template_ids = templates
        .values()
        .map(|template| Value::String(template.id.clone()))
        .collect::<Vec<_>>();
    if descriptor["template_ids"] != Value::Array(template_ids) {
        return Err(invalid());
    }
    let mut non_template_ids = inventory
        .non_templates
        .iter()
        .map(|definition| definition["id"].clone())
        .collect::<Vec<_>>();
    non_template_ids.sort_by(|left, right| left.as_str().cmp(&right.as_str()));
    if descriptor["non_template_ids"] != Value::Array(non_template_ids) {
        return Err(invalid());
    }
    Ok(templates)
}

pub fn validate_foundation_context_linkage(
    bundle: &ValidatedFoundationBundle,
    context: &SuccessorSemanticContext,
) -> Result<(), FoundationValidationError> {
    let descriptor = context.foundation_descriptor();
    if context.source_language() == "csharp"
        && context.semantic_profile() == CSHARP_PRACTICAL_PROFILE
        && descriptor.schema() == FOUNDATION_DESCRIPTOR_SCHEMA
        && descriptor.id() == FOUNDATION_DESCRIPTOR_ID
        && descriptor.content_sha256() == bundle.content_sha256()
    {
        Ok(())
    } else {
        Err(failure(
            FoundationValidationPhase::ContextLinkage,
            FoundationErrorCode::ContextLinkage,
        ))
    }
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
enum ClosedType {
    Primitive(String),
    Source(String),
    Instance {
        template: String,
        arguments: Vec<ClosedType>,
    },
}

impl ClosedType {
    fn parse(value: &Value) -> Result<Self, FoundationValidationError> {
        let object = value.as_object().ok_or_else(|| {
            failure(
                FoundationValidationPhase::Type,
                FoundationErrorCode::TypeShape,
            )
        })?;
        let kind = object.get("kind").and_then(Value::as_str).ok_or_else(|| {
            failure(
                FoundationValidationPhase::Type,
                FoundationErrorCode::GenericOrUnknownType,
            )
        })?;
        match kind {
            "primitive" => {
                require_exact_map_fields(object, &["kind", "id"], FoundationErrorCode::TypeShape)?;
                let id = object.get("id").and_then(Value::as_str).ok_or_else(|| {
                    failure(
                        FoundationValidationPhase::Type,
                        FoundationErrorCode::TypeShape,
                    )
                })?;
                Ok(Self::Primitive(id.to_owned()))
            }
            "source" => {
                require_exact_map_fields(object, &["kind", "id"], FoundationErrorCode::TypeShape)?;
                let id = object.get("id").and_then(Value::as_str).ok_or_else(|| {
                    failure(
                        FoundationValidationPhase::Type,
                        FoundationErrorCode::TypeShape,
                    )
                })?;
                Ok(Self::Source(id.to_owned()))
            }
            "instance" => {
                require_exact_map_fields(
                    object,
                    &["kind", "template", "arguments"],
                    FoundationErrorCode::TypeShape,
                )?;
                let template = object
                    .get("template")
                    .and_then(Value::as_str)
                    .ok_or_else(|| {
                        failure(
                            FoundationValidationPhase::Type,
                            FoundationErrorCode::TypeShape,
                        )
                    })?;
                let arguments = object
                    .get("arguments")
                    .and_then(Value::as_array)
                    .ok_or_else(|| {
                        failure(
                            FoundationValidationPhase::Type,
                            FoundationErrorCode::TypeShape,
                        )
                    })?
                    .iter()
                    .map(Self::parse)
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(Self::Instance {
                    template: template.to_owned(),
                    arguments,
                })
            }
            _ => Err(failure(
                FoundationValidationPhase::Type,
                FoundationErrorCode::GenericOrUnknownType,
            )),
        }
    }

    fn to_value(&self) -> Value {
        match self {
            Self::Primitive(id) => json!({"kind": "primitive", "id": id}),
            Self::Source(id) => json!({"kind": "source", "id": id}),
            Self::Instance {
                template,
                arguments,
            } => json!({
                "kind": "instance",
                "template": template,
                "arguments": arguments.iter().map(Self::to_value).collect::<Vec<_>>()
            }),
        }
    }
}

pub fn csharp_practical_declaration_id(
    identity: &Value,
) -> Result<String, FoundationValidationError> {
    if !has_exact_fields(
        identity,
        &[
            "kind",
            "namespace",
            "owner",
            "name",
            "parameter_type_ids",
            "result_type_id",
        ],
    ) {
        return Err(source_failure(FoundationErrorCode::IdentityShape));
    }
    Ok(format!(
        "mpk.csharp.source.{}",
        hash_value(DECLARATION_HASH_DOMAIN, identity)?
    ))
}

pub fn csharp_practical_stored_member_id(
    owner: &str,
    name: &str,
    ty: &Value,
    storage: &str,
) -> Result<String, FoundationValidationError> {
    let ty = ClosedType::parse(ty)?;
    if !matches!(storage, "readonly_field" | "get_auto" | "init_auto") {
        return Err(source_failure(
            FoundationErrorCode::StoredMemberOrderOrStorage,
        ));
    }
    let preimage = json!({
        "owner": owner,
        "name": name,
        "type": ty.to_value(),
        "storage": storage,
    });
    Ok(format!(
        "mpk.csharp.member.{}",
        hash_value(STORED_MEMBER_HASH_DOMAIN, &preimage)?
    ))
}

pub fn csharp_practical_closed_instance_id(
    bundle: &ValidatedFoundationBundle,
    ty: &Value,
) -> Result<String, FoundationValidationError> {
    let ty = ClosedType::parse(ty)?;
    let ClosedType::Instance { .. } = ty else {
        return Err(failure(
            FoundationValidationPhase::Type,
            FoundationErrorCode::TypeShape,
        ));
    };
    closed_type_id(bundle, &ty)
}

#[derive(Clone, Debug)]
struct DeclarationIdentity {
    kind: String,
    namespace: String,
    owner: String,
    name: String,
    parameter_type_ids: Vec<String>,
    result_type_id: String,
}

impl DeclarationIdentity {
    fn to_value(&self) -> Value {
        json!({
            "kind": self.kind,
            "namespace": self.namespace,
            "owner": self.owner,
            "name": self.name,
            "parameter_type_ids": self.parameter_type_ids,
            "result_type_id": self.result_type_id,
        })
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum SourceKind {
    ReadonlyStruct,
    SealedClass,
    Enum,
}

#[derive(Clone, Debug)]
struct StoredMember {
    id: String,
    name: String,
    ty: ClosedType,
    storage: String,
    ordinal: usize,
    required: bool,
}

#[derive(Clone, Debug)]
struct SourceType {
    id: String,
    identity: DeclarationIdentity,
    kind: SourceKind,
    members: Vec<StoredMember>,
    enum_values: Vec<String>,
    enum_underlying: Option<String>,
    actual_default: Map<String, Value>,
    public_default: bool,
    identity_sensitive: bool,
    source_sha256: String,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct ClosedRoot {
    origin: String,
    provenance_id: String,
    ty: ClosedType,
}

#[derive(Clone, Debug)]
pub struct ValidatedClosedRootSet {
    roots: Vec<ClosedRoot>,
    source_types: BTreeMap<String, SourceType>,
    canonical_json: Vec<u8>,
}

impl ValidatedClosedRootSet {
    pub fn root_count(&self) -> usize {
        self.roots.len()
    }

    pub fn source_type_count(&self) -> usize {
        self.source_types.len()
    }

    pub fn canonical_json(&self) -> &[u8] {
        &self.canonical_json
    }
}

pub fn canonical_closed_root_set_transport(
    bundle: &ValidatedFoundationBundle,
    roots: &Value,
    source_types: &Value,
) -> Result<Vec<u8>, FoundationValidationError> {
    let value = json!({
        "schema": CLOSED_ROOTS_SCHEMA,
        "semantic_profile": CSHARP_PRACTICAL_PROFILE,
        "foundation_id": FOUNDATION_DESCRIPTOR_ID,
        "foundation_sha256": bundle.content_sha256(),
        "roots": roots,
        "source_types": source_types,
    });
    canonical_transport(&value, FOUNDATION_TRANSPORT_BYTES_MAX)
}

pub fn validate_closed_root_set(
    bundle: &ValidatedFoundationBundle,
    transport: &[u8],
) -> Result<ValidatedClosedRootSet, FoundationValidationError> {
    let strict = parse_canonical_transport(
        transport,
        FOUNDATION_TRANSPORT_BYTES_MAX,
        FoundationValidationPhase::RootSet,
    )?;
    let value = strict_to_serde(&strict);
    if !has_exact_fields(
        &value,
        &[
            "schema",
            "semantic_profile",
            "foundation_id",
            "foundation_sha256",
            "roots",
            "source_types",
        ],
    ) || value["schema"] != CLOSED_ROOTS_SCHEMA
        || value["semantic_profile"] != CSHARP_PRACTICAL_PROFILE
        || value["foundation_id"] != FOUNDATION_DESCRIPTOR_ID
        || value["foundation_sha256"] != bundle.content_sha256()
    {
        return Err(failure(
            FoundationValidationPhase::RootSet,
            FoundationErrorCode::RootSetShape,
        ));
    }
    let source_object = value["source_types"].as_object().ok_or_else(|| {
        failure(
            FoundationValidationPhase::SourceTypes,
            FoundationErrorCode::SourceTable,
        )
    })?;
    let mut source_types = BTreeMap::new();
    for (key, source) in source_object {
        let parsed = parse_source_type(source)?;
        if key != &parsed.id || source_types.insert(key.clone(), parsed).is_some() {
            return Err(failure(
                FoundationValidationPhase::SourceTypes,
                FoundationErrorCode::SourceIdentity,
            ));
        }
    }
    validate_source_types(bundle, &source_types)?;

    let root_values = value["roots"].as_array().ok_or_else(|| {
        failure(
            FoundationValidationPhase::RootSet,
            FoundationErrorCode::RootSetShape,
        )
    })?;
    let mut roots = Vec::with_capacity(root_values.len());
    let mut seen = BTreeSet::new();
    for root in root_values {
        if !has_exact_fields(root, &["origin", "provenance_id", "type"]) {
            return Err(failure(
                FoundationValidationPhase::RootSet,
                FoundationErrorCode::RootShape,
            ));
        }
        let origin = root["origin"].as_str().ok_or_else(|| {
            failure(
                FoundationValidationPhase::RootSet,
                FoundationErrorCode::RootShape,
            )
        })?;
        let provenance_id = root["provenance_id"].as_str().ok_or_else(|| {
            failure(
                FoundationValidationPhase::RootSet,
                FoundationErrorCode::RootShape,
            )
        })?;
        if !ROOT_ORIGINS.contains(&origin) || !valid_provenance_id(provenance_id) {
            return Err(failure(
                FoundationValidationPhase::RootSet,
                FoundationErrorCode::RootProvenance,
            ));
        }
        let ty = ClosedType::parse(&root["type"])?;
        validate_closed_type(bundle, &source_types, &ty, 0)?;
        validate_specialization_depth(bundle, &source_types, &ty, 0, &mut BTreeSet::new())?;
        if let ClosedType::Instance { template, .. } = &ty {
            let definition = bundle
                .templates
                .get(template)
                .expect("type validation resolved template");
            if !definition
                .derivation_sources
                .iter()
                .any(|allowed| allowed == origin)
            {
                return Err(failure(
                    FoundationValidationPhase::RootSet,
                    FoundationErrorCode::RootDerivationSource,
                ));
            }
        }
        let parsed = ClosedRoot {
            origin: origin.to_owned(),
            provenance_id: provenance_id.to_owned(),
            ty,
        };
        if !seen.insert(parsed.clone()) {
            return Err(failure(
                FoundationValidationPhase::RootSet,
                FoundationErrorCode::DuplicateRoot,
            ));
        }
        roots.push(parsed);
    }
    Ok(ValidatedClosedRootSet {
        roots,
        source_types,
        canonical_json: transport.to_vec(),
    })
}

fn parse_source_type(value: &Value) -> Result<SourceType, FoundationValidationError> {
    let object = value
        .as_object()
        .ok_or_else(|| source_failure(FoundationErrorCode::SourceShape))?;
    require_exact_map_fields(
        object,
        &[
            "id",
            "identity",
            "kind",
            "members",
            "enum_values",
            "enum_underlying",
            "actual_default",
            "public_default",
            "identity_sensitive",
            "source_sha256",
        ],
        FoundationErrorCode::SourceShape,
    )?;
    let id = required_string(object, "id", FoundationErrorCode::SourceShape)?;
    let identity_object = object
        .get("identity")
        .and_then(Value::as_object)
        .ok_or_else(|| source_failure(FoundationErrorCode::IdentityShape))?;
    require_exact_map_fields(
        identity_object,
        &[
            "kind",
            "namespace",
            "owner",
            "name",
            "parameter_type_ids",
            "result_type_id",
        ],
        FoundationErrorCode::IdentityShape,
    )?;
    let identity = DeclarationIdentity {
        kind: required_string(identity_object, "kind", FoundationErrorCode::IdentityShape)?,
        namespace: required_string(
            identity_object,
            "namespace",
            FoundationErrorCode::IdentityShape,
        )?,
        owner: required_string(identity_object, "owner", FoundationErrorCode::IdentityShape)?,
        name: required_string(identity_object, "name", FoundationErrorCode::IdentityShape)?,
        parameter_type_ids: required_string_array(
            identity_object,
            "parameter_type_ids",
            FoundationErrorCode::IdentityShape,
        )?,
        result_type_id: required_string(
            identity_object,
            "result_type_id",
            FoundationErrorCode::IdentityShape,
        )?,
    };
    let kind = match required_string(object, "kind", FoundationErrorCode::SourceShape)?.as_str() {
        "readonly_struct" => SourceKind::ReadonlyStruct,
        "sealed_class" => SourceKind::SealedClass,
        "enum" => SourceKind::Enum,
        _ => return Err(source_failure(FoundationErrorCode::SourceKind)),
    };
    let member_values = object
        .get("members")
        .and_then(Value::as_array)
        .ok_or_else(|| source_failure(FoundationErrorCode::SourceShape))?;
    let mut members = Vec::with_capacity(member_values.len());
    for member in member_values {
        let member_object = member
            .as_object()
            .ok_or_else(|| source_failure(FoundationErrorCode::StoredMemberShape))?;
        require_exact_map_fields(
            member_object,
            &["id", "name", "type", "storage", "ordinal", "required"],
            FoundationErrorCode::StoredMemberShape,
        )?;
        let ordinal = member_object
            .get("ordinal")
            .and_then(Value::as_u64)
            .and_then(|value| usize::try_from(value).ok())
            .ok_or_else(|| source_failure(FoundationErrorCode::StoredMemberShape))?;
        let required = member_object
            .get("required")
            .and_then(Value::as_bool)
            .ok_or_else(|| source_failure(FoundationErrorCode::StoredMemberShape))?;
        members.push(StoredMember {
            id: required_string(member_object, "id", FoundationErrorCode::StoredMemberShape)?,
            name: required_string(
                member_object,
                "name",
                FoundationErrorCode::StoredMemberShape,
            )?,
            ty: ClosedType::parse(member_object.get("type").expect("exact field checked"))?,
            storage: required_string(
                member_object,
                "storage",
                FoundationErrorCode::StoredMemberShape,
            )?,
            ordinal,
            required,
        });
    }
    let enum_values = object
        .get("enum_values")
        .and_then(Value::as_array)
        .ok_or_else(|| source_failure(FoundationErrorCode::SourceShape))?
        .iter()
        .map(|value| {
            value
                .as_str()
                .map(str::to_owned)
                .ok_or_else(|| source_failure(FoundationErrorCode::EnumShape))
        })
        .collect::<Result<Vec<_>, _>>()?;
    let enum_underlying = match object.get("enum_underlying") {
        Some(Value::Null) => None,
        Some(Value::String(value)) => Some(value.clone()),
        _ => return Err(source_failure(FoundationErrorCode::EnumShape)),
    };
    let actual_default = object
        .get("actual_default")
        .and_then(Value::as_object)
        .cloned()
        .ok_or_else(|| source_failure(FoundationErrorCode::SourceDefaultShape))?;
    let public_default = object
        .get("public_default")
        .and_then(Value::as_bool)
        .ok_or_else(|| source_failure(FoundationErrorCode::SourceFlags))?;
    let identity_sensitive = object
        .get("identity_sensitive")
        .and_then(Value::as_bool)
        .ok_or_else(|| source_failure(FoundationErrorCode::SourceFlags))?;
    let source_sha256 = required_string(object, "source_sha256", FoundationErrorCode::SourceHash)?;
    Ok(SourceType {
        id,
        identity,
        kind,
        members,
        enum_values,
        enum_underlying,
        actual_default,
        public_default,
        identity_sensitive,
        source_sha256,
    })
}

fn validate_source_types(
    bundle: &ValidatedFoundationBundle,
    source_types: &BTreeMap<String, SourceType>,
) -> Result<(), FoundationValidationError> {
    for (key, source) in source_types {
        if source.identity.kind != "type"
            || !source.identity.owner.is_empty()
            || source.identity.name.is_empty()
            || !source.identity.parameter_type_ids.is_empty()
            || !source.identity.result_type_id.is_empty()
        {
            return Err(source_failure(FoundationErrorCode::SourceIdentity));
        }
        let expected_id = format!(
            "mpk.csharp.source.{}",
            hash_value(DECLARATION_HASH_DOMAIN, &source.identity.to_value())?
        );
        if key != &source.id || source.id != expected_id {
            return Err(source_failure(FoundationErrorCode::SourceIdentity));
        }
        if !valid_sha256(&source.source_sha256) {
            return Err(source_failure(FoundationErrorCode::SourceHash));
        }
        let _ = (source.public_default, source.identity_sensitive);
        let mut member_ids = BTreeSet::new();
        for (expected_ordinal, member) in source.members.iter().enumerate() {
            validate_closed_type(bundle, source_types, &member.ty, 0)?;
            if matches!(
                &member.ty,
                ClosedType::Primitive(id) if id == "exception"
            ) || matches!(
                &member.ty,
                ClosedType::Instance { template, .. } if template == "sequence_construction"
            ) {
                return Err(source_failure(FoundationErrorCode::NonvalueMember));
            }
            let member_preimage = json!({
                "owner": source.id,
                "name": member.name,
                "type": member.ty.to_value(),
                "storage": member.storage,
            });
            let expected_member_id = format!(
                "mpk.csharp.member.{}",
                hash_value(STORED_MEMBER_HASH_DOMAIN, &member_preimage)?
            );
            if member.id != expected_member_id || !member_ids.insert(member.id.clone()) {
                return Err(source_failure(FoundationErrorCode::StoredMemberIdentity));
            }
            if member.ordinal != expected_ordinal
                || !matches!(
                    member.storage.as_str(),
                    "readonly_field" | "get_auto" | "init_auto"
                )
            {
                return Err(source_failure(
                    FoundationErrorCode::StoredMemberOrderOrStorage,
                ));
            }
            if member.required && member.storage != "init_auto" {
                return Err(source_failure(FoundationErrorCode::RequiredStorage));
            }
        }
        if source
            .actual_default
            .keys()
            .cloned()
            .collect::<BTreeSet<_>>()
            != member_ids
        {
            return Err(source_failure(FoundationErrorCode::SourceDefaultShape));
        }
        validate_enum_shape(source)?;
    }
    for source_id in source_types.keys() {
        visit_source_cycle(source_id, source_types, &mut BTreeSet::new())?;
    }
    Ok(())
}

fn validate_enum_shape(source: &SourceType) -> Result<(), FoundationValidationError> {
    if source.kind != SourceKind::Enum {
        if source.enum_underlying.is_some() || !source.enum_values.is_empty() {
            return Err(source_failure(FoundationErrorCode::EnumShape));
        }
        return Ok(());
    }
    if !source.members.is_empty() || source.enum_values.is_empty() {
        return Err(source_failure(FoundationErrorCode::EnumShape));
    }
    let underlying = source
        .enum_underlying
        .as_deref()
        .ok_or_else(|| source_failure(FoundationErrorCode::EnumShape))?;
    let (minimum, maximum) = match underlying {
        "i8" => (i8::MIN as i128, i8::MAX as i128),
        "u8" => (0, u8::MAX as i128),
        "i16" => (i16::MIN as i128, i16::MAX as i128),
        "u16" => (0, u16::MAX as i128),
        "i32" => (i32::MIN as i128, i32::MAX as i128),
        "u32" => (0, u32::MAX as i128),
        "i64" => (i64::MIN as i128, i64::MAX as i128),
        "u64" => (0, u64::MAX as i128),
        _ => return Err(source_failure(FoundationErrorCode::EnumShape)),
    };
    for carrier in &source.enum_values {
        let parsed = parse_canonical_integer(carrier)
            .ok_or_else(|| source_failure(FoundationErrorCode::EnumShape))?;
        if parsed < minimum || parsed > maximum {
            return Err(source_failure(FoundationErrorCode::EnumShape));
        }
    }
    Ok(())
}

fn visit_source_cycle(
    source_id: &str,
    source_types: &BTreeMap<String, SourceType>,
    active: &mut BTreeSet<String>,
) -> Result<(), FoundationValidationError> {
    if !active.insert(source_id.to_owned()) {
        return Err(source_failure(FoundationErrorCode::SourceCycle));
    }
    let source = source_types
        .get(source_id)
        .ok_or_else(|| source_failure(FoundationErrorCode::UnknownSourceType))?;
    for member in &source.members {
        visit_type_sources(&member.ty, source_types, active)?;
    }
    active.remove(source_id);
    Ok(())
}

fn visit_type_sources(
    ty: &ClosedType,
    source_types: &BTreeMap<String, SourceType>,
    active: &mut BTreeSet<String>,
) -> Result<(), FoundationValidationError> {
    match ty {
        ClosedType::Primitive(_) => Ok(()),
        ClosedType::Source(source_id) => visit_source_cycle(source_id, source_types, active),
        ClosedType::Instance { arguments, .. } => {
            for argument in arguments {
                visit_type_sources(argument, source_types, active)?;
            }
            Ok(())
        }
    }
}

fn validate_closed_type(
    bundle: &ValidatedFoundationBundle,
    source_types: &BTreeMap<String, SourceType>,
    ty: &ClosedType,
    depth: u64,
) -> Result<(), FoundationValidationError> {
    enforce_internal_limit(
        FoundationLimit::ClosedInstanceDepth,
        depth,
        FoundationValidationPhase::Type,
    )?;
    match ty {
        ClosedType::Primitive(id) => {
            if PRIMITIVES.contains(&id.as_str()) {
                Ok(())
            } else {
                Err(failure(
                    FoundationValidationPhase::Type,
                    FoundationErrorCode::UnknownType,
                ))
            }
        }
        ClosedType::Source(id) => {
            if source_types.contains_key(id) {
                Ok(())
            } else {
                Err(failure(
                    FoundationValidationPhase::Type,
                    FoundationErrorCode::UnknownSourceType,
                ))
            }
        }
        ClosedType::Instance {
            template,
            arguments,
        } => {
            let definition = bundle.templates.get(template).ok_or_else(|| {
                failure(
                    FoundationValidationPhase::Type,
                    FoundationErrorCode::UnknownTemplate,
                )
            })?;
            if arguments.len() != definition.arity {
                return Err(failure(
                    FoundationValidationPhase::Type,
                    FoundationErrorCode::TemplateArity,
                ));
            }
            for argument in arguments {
                validate_closed_type(
                    bundle,
                    source_types,
                    argument,
                    depth.checked_add(1).ok_or_else(|| {
                        failure(
                            FoundationValidationPhase::Type,
                            FoundationErrorCode::InstanceDepth,
                        )
                    })?,
                )?;
                if matches!(argument, ClosedType::Primitive(id) if id == "exception")
                    || matches!(argument, ClosedType::Instance { template, .. } if template == "sequence_construction")
                {
                    return Err(failure(
                        FoundationValidationPhase::Type,
                        FoundationErrorCode::NonvalueArgument,
                    ));
                }
            }
            if template == "option"
                && matches!(&arguments[0], ClosedType::Instance { template, .. } if template == "option")
            {
                return Err(failure(
                    FoundationValidationPhase::Type,
                    FoundationErrorCode::NestedOption,
                ));
            }
            if matches!(template.as_str(), "ordered_map" | "ordered_set")
                && !is_total_order(bundle, source_types, &arguments[0], &mut BTreeSet::new())?
            {
                return Err(failure(
                    FoundationValidationPhase::Type,
                    FoundationErrorCode::NonTotalKey,
                ));
            }
            if template == "money" {
                let valid_currency = matches!(&arguments[0], ClosedType::Primitive(id) if id == "string")
                    || matches!(&arguments[0], ClosedType::Source(id) if source_types.get(id).is_some_and(|source| source.kind == SourceKind::Enum));
                if !valid_currency {
                    return Err(failure(
                        FoundationValidationPhase::Type,
                        FoundationErrorCode::CurrencyType,
                    ));
                }
            }
            Ok(())
        }
    }
}

fn validate_specialization_depth(
    bundle: &ValidatedFoundationBundle,
    source_types: &BTreeMap<String, SourceType>,
    ty: &ClosedType,
    depth: u64,
    active_instances: &mut BTreeSet<String>,
) -> Result<(), FoundationValidationError> {
    enforce_internal_limit(
        FoundationLimit::ClosedInstanceDepth,
        depth,
        FoundationValidationPhase::Closure,
    )?;
    match ty {
        ClosedType::Primitive(_) => Ok(()),
        ClosedType::Source(id) => {
            for member in &source_types[id].members {
                validate_specialization_depth(
                    bundle,
                    source_types,
                    &member.ty,
                    depth,
                    active_instances,
                )?;
            }
            Ok(())
        }
        ClosedType::Instance {
            template,
            arguments,
        } => {
            let identity = closed_type_id(bundle, ty)?;
            if !active_instances.insert(identity.clone()) {
                return Err(failure(
                    FoundationValidationPhase::Closure,
                    FoundationErrorCode::InstanceDepth,
                ));
            }
            let next = depth.checked_add(1).ok_or_else(|| {
                failure(
                    FoundationValidationPhase::Closure,
                    FoundationErrorCode::InstanceDepth,
                )
            })?;
            for argument in arguments {
                validate_specialization_depth(
                    bundle,
                    source_types,
                    argument,
                    next,
                    active_instances,
                )?;
            }
            let definition = &bundle.templates[template];
            for dependency in &definition.dependencies {
                let dependency = substitute_parameters(dependency, arguments)?;
                let dependency = ClosedType::parse(&dependency)?;
                validate_closed_type(bundle, source_types, &dependency, 0)?;
                validate_specialization_depth(
                    bundle,
                    source_types,
                    &dependency,
                    next,
                    active_instances,
                )?;
            }
            active_instances.remove(&identity);
            Ok(())
        }
    }
}

fn is_total_order(
    bundle: &ValidatedFoundationBundle,
    source_types: &BTreeMap<String, SourceType>,
    ty: &ClosedType,
    active_sources: &mut BTreeSet<String>,
) -> Result<bool, FoundationValidationError> {
    match ty {
        ClosedType::Primitive(id) => Ok(!matches!(id.as_str(), "f32" | "f64" | "exception")),
        ClosedType::Instance {
            template,
            arguments,
        } => {
            if template == "sequence_construction" || !bundle.templates.contains_key(template) {
                return Ok(false);
            }
            for argument in arguments {
                if !is_total_order(bundle, source_types, argument, active_sources)? {
                    return Ok(false);
                }
            }
            Ok(true)
        }
        ClosedType::Source(id) => {
            if !active_sources.insert(id.clone()) {
                return Err(source_failure(FoundationErrorCode::SourceCycle));
            }
            let source = source_types
                .get(id)
                .ok_or_else(|| source_failure(FoundationErrorCode::UnknownSourceType))?;
            let result = if source.kind == SourceKind::Enum {
                true
            } else {
                let mut orderable = true;
                for member in &source.members {
                    if !is_total_order(bundle, source_types, &member.ty, active_sources)? {
                        orderable = false;
                        break;
                    }
                }
                orderable
            };
            active_sources.remove(id);
            Ok(result)
        }
    }
}

#[derive(Clone, Debug)]
struct InstanceMetadata {
    template_id: String,
    argument_ids: Vec<String>,
    dependency_ids: Vec<String>,
}

#[derive(Clone, Debug)]
pub struct ClosedInstanceSet {
    value: Value,
    canonical_json: Vec<u8>,
    metadata: BTreeMap<String, InstanceMetadata>,
}

impl ClosedInstanceSet {
    pub fn value(&self) -> &Value {
        &self.value
    }

    pub fn entries(&self) -> &[Value] {
        self.value["entries"]
            .as_array()
            .expect("validated closed set retains entries")
    }

    pub fn counters(&self) -> &Value {
        &self.value["counters"]
    }

    pub fn closed_set_sha256(&self) -> &str {
        self.value["closed_set_sha256"]
            .as_str()
            .expect("validated closed set retains hash")
    }

    pub fn canonical_json(&self) -> &[u8] {
        &self.canonical_json
    }
}

pub fn derive_closed_instances(
    bundle: &ValidatedFoundationBundle,
    roots: &ValidatedClosedRootSet,
) -> Result<ClosedInstanceSet, FoundationValidationError> {
    if !valid_sha256(bundle.content_sha256()) {
        return Err(failure(
            FoundationValidationPhase::Closure,
            FoundationErrorCode::FoundationHash,
        ));
    }
    let mut pending = Vec::<(ClosedType, String)>::new();
    for root in &roots.roots {
        collect_instances(
            &root.ty,
            &roots.source_types,
            &root.provenance_id,
            &mut pending,
        )?;
    }
    let mut instances = BTreeMap::<String, ClosedType>::new();
    let mut provenance = BTreeMap::<String, BTreeSet<String>>::new();
    while let Some((ty, provenance_id)) = pending.pop() {
        validate_closed_type(bundle, &roots.source_types, &ty, 0)?;
        let identity = closed_type_id(bundle, &ty)?;
        if let Some(previous) = instances.get(&identity) {
            if previous != &ty {
                return Err(failure(
                    FoundationValidationPhase::Closure,
                    FoundationErrorCode::InstanceCollision,
                ));
            }
        } else {
            let candidate = u64::try_from(instances.len())
                .ok()
                .and_then(|count| count.checked_add(1))
                .ok_or_else(|| {
                    failure(
                        FoundationValidationPhase::Closure,
                        FoundationErrorCode::InstanceCount,
                    )
                })?;
            enforce_internal_limit(
                FoundationLimit::ClosedInstanceCount,
                candidate,
                FoundationValidationPhase::Closure,
            )?;
            instances.insert(identity.clone(), ty.clone());
            provenance.insert(identity.clone(), BTreeSet::new());
        }
        let origins = provenance
            .get_mut(&identity)
            .expect("new instances receive provenance storage");
        if !origins.insert(provenance_id.clone()) {
            continue;
        }
        let ClosedType::Instance {
            template,
            arguments,
        } = &ty
        else {
            continue;
        };
        for dependency in &bundle.templates[template].dependencies {
            let dependency = ClosedType::parse(&substitute_parameters(dependency, arguments)?)?;
            collect_instances(
                &dependency,
                &roots.source_types,
                &provenance_id,
                &mut pending,
            )?;
        }
    }

    let mut entries = Vec::with_capacity(instances.len());
    let mut declarations = 0_u64;
    let mut operations = 0_u64;
    let mut recipe_nodes = 0_u64;
    let mut metadata = BTreeMap::new();
    for (identity, ty) in &instances {
        let ClosedType::Instance {
            template,
            arguments,
        } = ty
        else {
            return Err(failure(
                FoundationValidationPhase::Closure,
                FoundationErrorCode::InstanceCollision,
            ));
        };
        let definition = &bundle.templates[template];
        let dependency_types = definition
            .dependencies
            .iter()
            .map(|dependency| {
                substitute_parameters(dependency, arguments)
                    .and_then(|value| ClosedType::parse(&value))
            })
            .collect::<Result<Vec<_>, _>>()?;
        let dependency_recipe_ids = dependency_types
            .iter()
            .map(|dependency| closed_type_id(bundle, dependency))
            .collect::<Result<Vec<_>, _>>()?;
        let argument_ids = arguments
            .iter()
            .map(|argument| closed_type_id(bundle, argument))
            .collect::<Result<Vec<_>, _>>()?;
        let substituted_representation =
            substitute_parameters(&definition.representation, arguments)?;
        let representation = concrete_representation(bundle, &substituted_representation)?;
        let mut operation_definitions = Vec::new();
        for operation in &definition.operations {
            if operation.name == "compare"
                && !is_total_order(bundle, &roots.source_types, ty, &mut BTreeSet::new())?
            {
                continue;
            }
            let argument_type_ids = operation
                .arguments
                .iter()
                .map(|reference| {
                    resolve_type_reference(
                        bundle,
                        identity,
                        arguments,
                        &dependency_recipe_ids,
                        reference,
                    )
                })
                .collect::<Result<Vec<_>, _>>()?;
            let normal_result_type_id = resolve_type_reference(
                bundle,
                identity,
                arguments,
                &dependency_recipe_ids,
                &operation.result,
            )?;
            operation_definitions.push(json!({
                "id": format!("{}.{}", identity, operation.name),
                "argument_type_ids": argument_type_ids,
                "normal_result_type_id": normal_result_type_id,
                "equation": operation.equation,
                "error_precedence": operation.error_precedence,
            }));
        }
        let type_definition = json!({
            "id": identity,
            "representation": representation,
        });
        let entry_declarations = 1_u64;
        let entry_operations = u64::try_from(operation_definitions.len()).map_err(|_| {
            failure(
                FoundationValidationPhase::Expansion,
                FoundationErrorCode::ExpandedOperations,
            )
        })?;
        let entry_recipe_nodes = node_count(&type_definition)?
            .checked_add(node_count(&Value::Array(operation_definitions.clone()))?)
            .ok_or_else(|| {
                failure(
                    FoundationValidationPhase::Expansion,
                    FoundationErrorCode::ExpandedRecipeNodes,
                )
            })?;
        declarations = declarations
            .checked_add(entry_declarations)
            .ok_or_else(|| {
                failure(
                    FoundationValidationPhase::Expansion,
                    FoundationErrorCode::ExpandedDeclarations,
                )
            })?;
        operations = operations.checked_add(entry_operations).ok_or_else(|| {
            failure(
                FoundationValidationPhase::Expansion,
                FoundationErrorCode::ExpandedOperations,
            )
        })?;
        recipe_nodes = recipe_nodes
            .checked_add(entry_recipe_nodes)
            .ok_or_else(|| {
                failure(
                    FoundationValidationPhase::Expansion,
                    FoundationErrorCode::ExpandedRecipeNodes,
                )
            })?;
        enforce_internal_limit(
            FoundationLimit::ExpandedDeclarations,
            declarations,
            FoundationValidationPhase::Expansion,
        )?;
        enforce_internal_limit(
            FoundationLimit::ExpandedOperations,
            operations,
            FoundationValidationPhase::Expansion,
        )?;
        enforce_internal_limit(
            FoundationLimit::ExpandedRecipeNodes,
            recipe_nodes,
            FoundationValidationPhase::Expansion,
        )?;
        let dependency_ids = dependency_recipe_ids
            .iter()
            .cloned()
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect::<Vec<_>>();
        let provenance_ids = provenance[identity].iter().cloned().collect::<Vec<_>>();
        entries.push(json!({
            "instance_id": identity,
            "template_id": definition.id,
            "version": 1,
            "semantic_profile": CSHARP_PRACTICAL_PROFILE,
            "arity": definition.arity,
            "argument_ids": argument_ids,
            "dependency_ids": dependency_ids,
            "provenance_ids": provenance_ids,
            "type_definition": type_definition,
            "operation_definitions": operation_definitions,
            "counters": {
                "declarations": entry_declarations,
                "operations": entry_operations,
                "recipe_nodes": entry_recipe_nodes,
            },
        }));
        metadata.insert(
            identity.clone(),
            InstanceMetadata {
                template_id: definition.id.clone(),
                argument_ids,
                dependency_ids,
            },
        );
    }
    let mut value = json!({
        "schema": CLOSED_INSTANCES_SCHEMA,
        "semantic_profile": CSHARP_PRACTICAL_PROFILE,
        "foundation_id": FOUNDATION_DESCRIPTOR_ID,
        "foundation_sha256": bundle.content_sha256(),
        "entries": entries,
        "counters": {
            "declarations": declarations,
            "operations": operations,
            "recipe_nodes": recipe_nodes,
        },
    });
    let closed_set_sha256 = hash_value(CLOSED_INSTANCE_SET_HASH_DOMAIN, &value)?;
    value
        .as_object_mut()
        .expect("closed set root is an object")
        .insert(
            "closed_set_sha256".to_owned(),
            Value::String(closed_set_sha256),
        );
    let canonical_json = canonical_transport(&value, FOUNDATION_TRANSPORT_BYTES_MAX)?;
    Ok(ClosedInstanceSet {
        value,
        canonical_json,
        metadata,
    })
}

pub fn validate_closed_instance_set(
    bundle: &ValidatedFoundationBundle,
    roots: &ValidatedClosedRootSet,
    transport: &[u8],
) -> Result<ClosedInstanceSet, FoundationValidationError> {
    let submitted = parse_canonical_transport(
        transport,
        FOUNDATION_TRANSPORT_BYTES_MAX,
        FoundationValidationPhase::ClosedSet,
    )?;
    let expected = derive_closed_instances(bundle, roots)?;
    if strict_to_serde(&submitted) != expected.value {
        return Err(failure(
            FoundationValidationPhase::ClosedSet,
            FoundationErrorCode::ClosedSetRecomputation,
        ));
    }
    Ok(expected)
}

fn collect_instances(
    ty: &ClosedType,
    source_types: &BTreeMap<String, SourceType>,
    provenance_id: &str,
    pending: &mut Vec<(ClosedType, String)>,
) -> Result<(), FoundationValidationError> {
    match ty {
        ClosedType::Primitive(_) => Ok(()),
        ClosedType::Source(id) => {
            let source = source_types
                .get(id)
                .ok_or_else(|| source_failure(FoundationErrorCode::UnknownSourceType))?;
            for member in &source.members {
                collect_instances(&member.ty, source_types, provenance_id, pending)?;
            }
            Ok(())
        }
        ClosedType::Instance { arguments, .. } => {
            pending.push((ty.clone(), provenance_id.to_owned()));
            for argument in arguments {
                collect_instances(argument, source_types, provenance_id, pending)?;
            }
            Ok(())
        }
    }
}

fn closed_type_id(
    bundle: &ValidatedFoundationBundle,
    ty: &ClosedType,
) -> Result<String, FoundationValidationError> {
    match ty {
        ClosedType::Primitive(id) => Ok(format!("mpk.csharp.value.{id}.v1")),
        ClosedType::Source(id) => Ok(id.clone()),
        ClosedType::Instance {
            template,
            arguments,
        } => {
            let definition = bundle.templates.get(template).ok_or_else(|| {
                failure(
                    FoundationValidationPhase::Type,
                    FoundationErrorCode::UnknownTemplate,
                )
            })?;
            let argument_ids = arguments
                .iter()
                .map(|argument| closed_type_id(bundle, argument))
                .collect::<Result<Vec<_>, _>>()?;
            let preimage = json!({
                "template": definition.id,
                "version": 1,
                "arguments": argument_ids,
            });
            Ok(format!(
                "mpk.csharp.instance.{}",
                hash_value(CLOSED_INSTANCE_HASH_DOMAIN, &preimage)?
            ))
        }
    }
}

fn substitute_parameters(
    value: &Value,
    arguments: &[ClosedType],
) -> Result<Value, FoundationValidationError> {
    match value {
        Value::Object(object) => {
            if object.get("kind").and_then(Value::as_str) == Some("parameter") {
                require_exact_map_fields(
                    object,
                    &["kind", "index"],
                    FoundationErrorCode::ParameterShape,
                )?;
                let index = object
                    .get("index")
                    .and_then(Value::as_u64)
                    .and_then(|value| usize::try_from(value).ok())
                    .ok_or_else(|| {
                        failure(
                            FoundationValidationPhase::Expansion,
                            FoundationErrorCode::ParameterArity,
                        )
                    })?;
                return arguments
                    .get(index)
                    .map(ClosedType::to_value)
                    .ok_or_else(|| {
                        failure(
                            FoundationValidationPhase::Expansion,
                            FoundationErrorCode::ParameterArity,
                        )
                    });
            }
            Ok(Value::Object(
                object
                    .iter()
                    .map(|(key, item)| {
                        substitute_parameters(item, arguments).map(|item| (key.clone(), item))
                    })
                    .collect::<Result<Map<_, _>, _>>()?,
            ))
        }
        Value::Array(values) => Ok(Value::Array(
            values
                .iter()
                .map(|item| substitute_parameters(item, arguments))
                .collect::<Result<Vec<_>, _>>()?,
        )),
        _ => Ok(value.clone()),
    }
}

fn concrete_representation(
    bundle: &ValidatedFoundationBundle,
    value: &Value,
) -> Result<Value, FoundationValidationError> {
    match value {
        Value::Object(object) => match object.get("kind").and_then(Value::as_str) {
            Some("primitive" | "source" | "instance") => {
                let ty = ClosedType::parse(value)?;
                Ok(json!({"kind": "concrete", "type_id": closed_type_id(bundle, &ty)?}))
            }
            Some("parameter") => Err(failure(
                FoundationValidationPhase::Expansion,
                FoundationErrorCode::ResidualGeneric,
            )),
            _ => Ok(Value::Object(
                object
                    .iter()
                    .map(|(key, item)| {
                        concrete_representation(bundle, item).map(|item| (key.clone(), item))
                    })
                    .collect::<Result<Map<_, _>, _>>()?,
            )),
        },
        Value::Array(values) => Ok(Value::Array(
            values
                .iter()
                .map(|item| concrete_representation(bundle, item))
                .collect::<Result<Vec<_>, _>>()?,
        )),
        _ => Ok(value.clone()),
    }
}

fn resolve_type_reference(
    bundle: &ValidatedFoundationBundle,
    self_id: &str,
    arguments: &[ClosedType],
    dependency_ids: &[String],
    reference: &str,
) -> Result<String, FoundationValidationError> {
    if reference == "self" {
        return Ok(self_id.to_owned());
    }
    if let Some(index) = reference.strip_prefix("arg") {
        let index = index.parse::<usize>().map_err(|_| {
            failure(
                FoundationValidationPhase::Expansion,
                FoundationErrorCode::ResidualGeneric,
            )
        })?;
        return arguments
            .get(index)
            .ok_or_else(|| {
                failure(
                    FoundationValidationPhase::Expansion,
                    FoundationErrorCode::ResidualGeneric,
                )
            })
            .and_then(|argument| closed_type_id(bundle, argument));
    }
    if let Some(index) = reference.strip_prefix("dependency") {
        let index = index.parse::<usize>().map_err(|_| {
            failure(
                FoundationValidationPhase::Expansion,
                FoundationErrorCode::ResidualGeneric,
            )
        })?;
        return dependency_ids.get(index).cloned().ok_or_else(|| {
            failure(
                FoundationValidationPhase::Expansion,
                FoundationErrorCode::ResidualGeneric,
            )
        });
    }
    if PRIMITIVES.contains(&reference) {
        Ok(format!("mpk.csharp.value.{reference}.v1"))
    } else {
        Err(failure(
            FoundationValidationPhase::Expansion,
            FoundationErrorCode::ResidualGeneric,
        ))
    }
}

fn node_count(value: &Value) -> Result<u64, FoundationValidationError> {
    let children = match value {
        Value::Array(values) => values
            .iter()
            .map(node_count)
            .collect::<Result<Vec<_>, _>>()?,
        Value::Object(object) => object
            .values()
            .map(node_count)
            .collect::<Result<Vec<_>, _>>()?,
        _ => Vec::new(),
    };
    children.into_iter().try_fold(1_u64, |total, child| {
        total.checked_add(child).ok_or_else(|| {
            failure(
                FoundationValidationPhase::Expansion,
                FoundationErrorCode::ExpandedRecipeNodes,
            )
        })
    })
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FoundationLimit {
    BindingCount,
    ClosedInstanceCount,
    ClosedInstanceDepth,
    ExpandedDeclarations,
    ExpandedOperations,
    ExpandedRecipeNodes,
    ProjectionObligationsPerBinding,
}

impl FoundationLimit {
    pub fn from_id(id: &str) -> Option<Self> {
        match id {
            "binding_count" => Some(Self::BindingCount),
            "closed_instance_count" | "closed_semantic_instances" => {
                Some(Self::ClosedInstanceCount)
            }
            "closed_instance_depth" | "closed_instance_nesting" => Some(Self::ClosedInstanceDepth),
            "expanded_declarations" | "specialized_declarations" => {
                Some(Self::ExpandedDeclarations)
            }
            "expanded_operations" | "specialized_operations" => Some(Self::ExpandedOperations),
            "expanded_recipe_nodes" => Some(Self::ExpandedRecipeNodes),
            "projection_obligations_per_binding" => Some(Self::ProjectionObligationsPerBinding),
            _ => None,
        }
    }

    pub const fn inclusive_maximum(self) -> u64 {
        match self {
            Self::BindingCount => FOUNDATION_BINDING_COUNT_MAX,
            Self::ClosedInstanceCount => CLOSED_INSTANCE_COUNT_MAX,
            Self::ClosedInstanceDepth => CLOSED_INSTANCE_DEPTH_MAX,
            Self::ExpandedDeclarations => EXPANDED_DECLARATIONS_MAX,
            Self::ExpandedOperations => EXPANDED_OPERATIONS_MAX,
            Self::ExpandedRecipeNodes => EXPANDED_RECIPE_NODES_MAX,
            Self::ProjectionObligationsPerBinding => PROJECTION_OBLIGATIONS_PER_BINDING_MAX,
        }
    }

    const fn error_code(self) -> FoundationErrorCode {
        match self {
            Self::BindingCount => FoundationErrorCode::BindingCount,
            Self::ClosedInstanceCount => FoundationErrorCode::InstanceCount,
            Self::ClosedInstanceDepth => FoundationErrorCode::InstanceDepth,
            Self::ExpandedDeclarations => FoundationErrorCode::ExpandedDeclarations,
            Self::ExpandedOperations => FoundationErrorCode::ExpandedOperations,
            Self::ExpandedRecipeNodes => FoundationErrorCode::ExpandedRecipeNodes,
            Self::ProjectionObligationsPerBinding => {
                FoundationErrorCode::ProjectionObligationsPerBinding
            }
        }
    }
}

pub fn validate_foundation_structural_limit(
    limit: FoundationLimit,
    value: u64,
) -> Result<(), FoundationValidationError> {
    if value <= limit.inclusive_maximum() {
        return Ok(());
    }
    let code = match limit {
        FoundationLimit::ClosedInstanceCount => FoundationErrorCode::ClosedInstanceCount,
        FoundationLimit::ClosedInstanceDepth => FoundationErrorCode::ClosedInstanceDepth,
        _ => limit.error_code(),
    };
    Err(failure(FoundationValidationPhase::Limits, code))
}

pub fn validate_practical_foundation_limit(
    limit_id: &str,
    value: u64,
) -> Result<(), FoundationValidationError> {
    let limit = FoundationLimit::from_id(limit_id).ok_or_else(|| {
        failure(
            FoundationValidationPhase::Limits,
            FoundationErrorCode::LimitExceeded,
        )
    })?;
    if value <= limit.inclusive_maximum() {
        Ok(())
    } else {
        Err(failure(
            FoundationValidationPhase::Limits,
            FoundationErrorCode::LimitExceeded,
        ))
    }
}

fn enforce_internal_limit(
    limit: FoundationLimit,
    value: u64,
    phase: FoundationValidationPhase,
) -> Result<(), FoundationValidationError> {
    if value <= limit.inclusive_maximum() {
        Ok(())
    } else {
        Err(failure(phase, limit.error_code()))
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NamedMonomorphicValue {
    pub name: String,
    pub value: Box<MonomorphicValue>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MonomorphicMapEntry {
    pub key: Box<MonomorphicValue>,
    pub value: Box<MonomorphicValue>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OptionArm {
    None,
    Some,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BoundaryArm {
    Missing,
    Null,
    Value,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ParseErrorArm {
    InputBound,
    Syntax,
    Noncanonical,
    ScalePrecision,
    Range,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum MonomorphicValue {
    Unit {
        type_id: String,
    },
    Bool {
        type_id: String,
        value: bool,
    },
    Signed {
        type_id: String,
        value: String,
    },
    Unsigned {
        type_id: String,
        value: String,
    },
    Char {
        type_id: String,
        utf16: u16,
    },
    String {
        type_id: String,
        utf16: Vec<u16>,
    },
    F32Bits {
        type_id: String,
        bits: String,
    },
    F64Bits {
        type_id: String,
        bits: String,
    },
    DecimalBits {
        type_id: String,
        negative: bool,
        scale: u8,
        coefficient: String,
    },
    Enum {
        type_id: String,
        underlying: String,
        carrier: String,
    },
    Product {
        type_id: String,
        fields: Vec<NamedMonomorphicValue>,
    },
    Array {
        type_id: String,
        elements: Vec<MonomorphicValue>,
    },
    Sequence {
        type_id: String,
        elements: Vec<MonomorphicValue>,
    },
    OrderedEntry {
        type_id: String,
        key: Box<MonomorphicValue>,
        value: Box<MonomorphicValue>,
    },
    OrderedMap {
        type_id: String,
        entries: Vec<MonomorphicMapEntry>,
    },
    OrderedSet {
        type_id: String,
        elements: Vec<MonomorphicValue>,
    },
    Option {
        type_id: String,
        arm: OptionArm,
        value: Option<Box<MonomorphicValue>>,
    },
    TaggedSum {
        type_id: String,
        arm: String,
        payload: Vec<MonomorphicValue>,
    },
    BoundaryPresence {
        type_id: String,
        arm: BoundaryArm,
        value: Option<Box<MonomorphicValue>>,
    },
    Date {
        type_id: String,
        day_number: u32,
    },
    Time {
        type_id: String,
        ticks: String,
    },
    Duration {
        type_id: String,
        ticks: String,
    },
    Instant {
        type_id: String,
        milliseconds: String,
    },
    Guid {
        type_id: String,
        n: String,
    },
    Money {
        type_id: String,
        amount: Box<MonomorphicValue>,
        currency: Box<MonomorphicValue>,
    },
    Transition {
        type_id: String,
        state: Box<MonomorphicValue>,
        events: Vec<MonomorphicValue>,
        response: Box<MonomorphicValue>,
    },
    ParseError {
        type_id: String,
        arm: ParseErrorArm,
    },
    ClosedException {
        type_id: String,
        tag: u32,
        source_type_id: Option<String>,
        payload: Option<Box<MonomorphicValue>>,
    },
}

impl MonomorphicValue {
    pub fn type_id(&self) -> &str {
        match self {
            Self::Unit { type_id }
            | Self::Bool { type_id, .. }
            | Self::Signed { type_id, .. }
            | Self::Unsigned { type_id, .. }
            | Self::Char { type_id, .. }
            | Self::String { type_id, .. }
            | Self::F32Bits { type_id, .. }
            | Self::F64Bits { type_id, .. }
            | Self::DecimalBits { type_id, .. }
            | Self::Enum { type_id, .. }
            | Self::Product { type_id, .. }
            | Self::Array { type_id, .. }
            | Self::Sequence { type_id, .. }
            | Self::OrderedEntry { type_id, .. }
            | Self::OrderedMap { type_id, .. }
            | Self::OrderedSet { type_id, .. }
            | Self::Option { type_id, .. }
            | Self::TaggedSum { type_id, .. }
            | Self::BoundaryPresence { type_id, .. }
            | Self::Date { type_id, .. }
            | Self::Time { type_id, .. }
            | Self::Duration { type_id, .. }
            | Self::Instant { type_id, .. }
            | Self::Guid { type_id, .. }
            | Self::Money { type_id, .. }
            | Self::Transition { type_id, .. }
            | Self::ParseError { type_id, .. }
            | Self::ClosedException { type_id, .. } => type_id,
        }
    }
}

pub fn canonical_monomorphic_value_transport(
    bundle: &ValidatedFoundationBundle,
    roots: &ValidatedClosedRootSet,
    closed_set: &ClosedInstanceSet,
    value: &MonomorphicValue,
) -> Result<Vec<u8>, FoundationValidationError> {
    validate_monomorphic_value(bundle, roots, closed_set, value)?;
    let encoded = serde_json::to_value(value).map_err(|_| {
        failure(
            FoundationValidationPhase::ConcreteValue,
            FoundationErrorCode::ConcreteValueShape,
        )
    })?;
    canonical_transport(&encoded, FOUNDATION_TRANSPORT_BYTES_MAX)
}

pub fn import_monomorphic_value(
    bundle: &ValidatedFoundationBundle,
    roots: &ValidatedClosedRootSet,
    closed_set: &ClosedInstanceSet,
    transport: &[u8],
) -> Result<MonomorphicValue, FoundationValidationError> {
    let strict = parse_canonical_transport(
        transport,
        FOUNDATION_TRANSPORT_BYTES_MAX,
        FoundationValidationPhase::ConcreteValue,
    )?;
    let value: MonomorphicValue =
        serde_json::from_value(strict_to_serde(&strict)).map_err(|_| {
            failure(
                FoundationValidationPhase::ConcreteValue,
                FoundationErrorCode::ConcreteValueShape,
            )
        })?;
    validate_monomorphic_value(bundle, roots, closed_set, &value)?;
    Ok(value)
}

pub fn validate_monomorphic_value(
    bundle: &ValidatedFoundationBundle,
    roots: &ValidatedClosedRootSet,
    closed_set: &ClosedInstanceSet,
    value: &MonomorphicValue,
) -> Result<(), FoundationValidationError> {
    let cells = validate_value_inner(bundle, roots, closed_set, value, true)?;
    if cells > TOTAL_VALUE_CELLS_MAX {
        Err(failure(
            FoundationValidationPhase::ConcreteValue,
            FoundationErrorCode::TotalValueCells,
        ))
    } else {
        Ok(())
    }
}

fn validate_value_inner(
    bundle: &ValidatedFoundationBundle,
    roots: &ValidatedClosedRootSet,
    closed_set: &ClosedInstanceSet,
    value: &MonomorphicValue,
    allow_exception: bool,
) -> Result<u64, FoundationValidationError> {
    let primitive = |name: &str, actual: &str| {
        if actual == format!("mpk.csharp.value.{name}.v1") {
            Ok(())
        } else {
            Err(value_failure(FoundationErrorCode::ConcreteValueType))
        }
    };
    let nested_cells = |values: &[MonomorphicValue]| -> Result<u64, FoundationValidationError> {
        values.iter().try_fold(0_u64, |total, item| {
            let cells = validate_value_inner(bundle, roots, closed_set, item, false)?;
            total
                .checked_add(cells)
                .ok_or_else(|| value_failure(FoundationErrorCode::TotalValueCells))
        })
    };
    let child_cells =
        |item: &MonomorphicValue| validate_value_inner(bundle, roots, closed_set, item, false);
    let cells = match value {
        MonomorphicValue::Unit { type_id } => {
            primitive("unit", type_id)?;
            1
        }
        MonomorphicValue::Bool { type_id, .. } => {
            primitive("bool", type_id)?;
            1
        }
        MonomorphicValue::Signed { type_id, value } => {
            let primitive_name = type_id
                .strip_prefix("mpk.csharp.value.")
                .and_then(|value| value.strip_suffix(".v1"))
                .filter(|name| matches!(*name, "i8" | "i16" | "i32" | "i64"))
                .ok_or_else(|| value_failure(FoundationErrorCode::ConcreteValueType))?;
            let parsed = parse_canonical_integer(value)
                .ok_or_else(|| value_failure(FoundationErrorCode::ConcreteValueInvariant))?;
            let (minimum, maximum) = match primitive_name {
                "i8" => (i8::MIN as i128, i8::MAX as i128),
                "i16" => (i16::MIN as i128, i16::MAX as i128),
                "i32" => (i32::MIN as i128, i32::MAX as i128),
                "i64" => (i64::MIN as i128, i64::MAX as i128),
                _ => unreachable!(),
            };
            if parsed < minimum || parsed > maximum {
                return Err(value_failure(FoundationErrorCode::ConcreteValueInvariant));
            }
            1
        }
        MonomorphicValue::Unsigned { type_id, value } => {
            let primitive_name = type_id
                .strip_prefix("mpk.csharp.value.")
                .and_then(|value| value.strip_suffix(".v1"))
                .filter(|name| matches!(*name, "u8" | "u16" | "u32" | "u64"))
                .ok_or_else(|| value_failure(FoundationErrorCode::ConcreteValueType))?;
            let parsed = parse_canonical_integer(value)
                .filter(|value| *value >= 0)
                .ok_or_else(|| value_failure(FoundationErrorCode::ConcreteValueInvariant))?;
            let maximum = match primitive_name {
                "u8" => u8::MAX as i128,
                "u16" => u16::MAX as i128,
                "u32" => u32::MAX as i128,
                "u64" => u64::MAX as i128,
                _ => unreachable!(),
            };
            if parsed > maximum {
                return Err(value_failure(FoundationErrorCode::ConcreteValueInvariant));
            }
            1
        }
        MonomorphicValue::Char { type_id, .. } => {
            primitive("char", type_id)?;
            1
        }
        MonomorphicValue::String { type_id, utf16 } => {
            primitive("string", type_id)?;
            enforce_value_bound(utf16.len(), STRING_VALUE_LENGTH_MAX)?;
            1_u64
                .checked_add(u64::try_from(utf16.len()).unwrap_or(u64::MAX))
                .ok_or_else(|| value_failure(FoundationErrorCode::TotalValueCells))?
        }
        MonomorphicValue::F32Bits { type_id, bits } => {
            primitive("f32", type_id)?;
            if !valid_fixed_hex(bits, 8) {
                return Err(value_failure(FoundationErrorCode::ConcreteValueInvariant));
            }
            1
        }
        MonomorphicValue::F64Bits { type_id, bits } => {
            primitive("f64", type_id)?;
            if !valid_fixed_hex(bits, 16) {
                return Err(value_failure(FoundationErrorCode::ConcreteValueInvariant));
            }
            1
        }
        MonomorphicValue::DecimalBits {
            type_id,
            scale,
            coefficient,
            ..
        } => {
            primitive("decimal", type_id)?;
            let parsed = parse_canonical_integer(coefficient)
                .filter(|value| *value >= 0)
                .ok_or_else(|| value_failure(FoundationErrorCode::ConcreteValueInvariant))?;
            const DECIMAL_MAX_COEFFICIENT: i128 = (1_i128 << 96) - 1;
            if *scale > 28 || parsed > DECIMAL_MAX_COEFFICIENT {
                return Err(value_failure(FoundationErrorCode::ConcreteValueInvariant));
            }
            1
        }
        MonomorphicValue::Enum {
            type_id,
            underlying,
            carrier,
        } => {
            if type_id == "mpk.csharp.value.day_of_week.v1" {
                if underlying != "i32"
                    || !parse_canonical_integer(carrier)
                        .is_some_and(|value| (0..=6).contains(&value))
                {
                    return Err(value_failure(FoundationErrorCode::ConcreteValueInvariant));
                }
            } else {
                let source = roots
                    .source_types
                    .get(type_id)
                    .filter(|source| source.kind == SourceKind::Enum)
                    .ok_or_else(|| value_failure(FoundationErrorCode::ConcreteValueType))?;
                if source.enum_underlying.as_deref() != Some(underlying)
                    || !source.enum_values.iter().any(|value| value == carrier)
                {
                    return Err(value_failure(FoundationErrorCode::ConcreteValueInvariant));
                }
            }
            1
        }
        MonomorphicValue::Product { type_id, fields } => {
            validate_product_shape(bundle, roots, closed_set, type_id, fields)?;
            fields.iter().try_fold(1_u64, |total, field| {
                let cells = child_cells(&field.value)?;
                total
                    .checked_add(cells)
                    .ok_or_else(|| value_failure(FoundationErrorCode::TotalValueCells))
            })?
        }
        MonomorphicValue::Array { type_id, elements } => {
            let arguments = require_instance(closed_set, type_id, "bounded_sequence")?;
            enforce_value_bound(elements.len(), ARRAY_VALUE_LENGTH_MAX)?;
            validate_homogeneous_values(elements, &arguments[0])?;
            1_u64
                .checked_add(nested_cells(elements)?)
                .ok_or_else(|| value_failure(FoundationErrorCode::TotalValueCells))?
        }
        MonomorphicValue::Sequence { type_id, elements } => {
            let arguments = require_instance(closed_set, type_id, "bounded_sequence")?;
            enforce_value_bound(elements.len(), SEQUENCE_VALUE_LENGTH_MAX)?;
            validate_homogeneous_values(elements, &arguments[0])?;
            1_u64
                .checked_add(nested_cells(elements)?)
                .ok_or_else(|| value_failure(FoundationErrorCode::TotalValueCells))?
        }
        MonomorphicValue::OrderedEntry {
            type_id,
            key,
            value,
        } => {
            let arguments = require_instance(closed_set, type_id, "ordered_entry")?;
            require_value_type(key, &arguments[0])?;
            require_value_type(value, &arguments[1])?;
            let key_cells = child_cells(key)?;
            let value_cells = child_cells(value)?;
            1_u64
                .checked_add(key_cells)
                .and_then(|count| count.checked_add(value_cells))
                .ok_or_else(|| value_failure(FoundationErrorCode::TotalValueCells))?
        }
        MonomorphicValue::OrderedMap { type_id, entries } => {
            let arguments = require_instance(closed_set, type_id, "ordered_map")?;
            enforce_value_bound(entries.len(), MAP_VALUE_LENGTH_MAX)?;
            let mut total = 1_u64;
            let mut seen_keys = BTreeSet::new();
            for entry in entries {
                require_value_type(&entry.key, &arguments[0])?;
                require_value_type(&entry.value, &arguments[1])?;
                let encoded_key = serde_json::to_vec(&entry.key)
                    .map_err(|_| value_failure(FoundationErrorCode::ConcreteValueShape))?;
                if !seen_keys.insert(encoded_key) {
                    return Err(value_failure(FoundationErrorCode::ConcreteValueInvariant));
                }
                let key_cells = child_cells(&entry.key)?;
                let value_cells = child_cells(&entry.value)?;
                total = total
                    .checked_add(key_cells)
                    .and_then(|count| count.checked_add(value_cells))
                    .ok_or_else(|| value_failure(FoundationErrorCode::TotalValueCells))?;
            }
            ensure_strictly_increasing(
                bundle,
                roots,
                closed_set,
                entries.iter().map(|entry| entry.key.as_ref()),
            )?;
            total
        }
        MonomorphicValue::OrderedSet { type_id, elements } => {
            let arguments = require_instance(closed_set, type_id, "ordered_set")?;
            enforce_value_bound(elements.len(), SET_VALUE_LENGTH_MAX)?;
            validate_homogeneous_values(elements, &arguments[0])?;
            let mut seen = BTreeSet::new();
            for element in elements {
                let encoded = serde_json::to_vec(element)
                    .map_err(|_| value_failure(FoundationErrorCode::ConcreteValueShape))?;
                if !seen.insert(encoded) {
                    return Err(value_failure(FoundationErrorCode::ConcreteValueInvariant));
                }
            }
            let element_cells = nested_cells(elements)?;
            ensure_strictly_increasing(bundle, roots, closed_set, elements.iter())?;
            1_u64
                .checked_add(element_cells)
                .ok_or_else(|| value_failure(FoundationErrorCode::TotalValueCells))?
        }
        MonomorphicValue::Option {
            type_id,
            arm,
            value,
        } => {
            let arguments = require_instance(closed_set, type_id, "option")?;
            match (arm, value) {
                (OptionArm::None, None) => 1,
                (OptionArm::Some, Some(value)) => {
                    require_value_type(value, &arguments[0])?;
                    1_u64
                        .checked_add(child_cells(value)?)
                        .ok_or_else(|| value_failure(FoundationErrorCode::TotalValueCells))?
                }
                _ => return Err(value_failure(FoundationErrorCode::ConcreteValueInvariant)),
            }
        }
        MonomorphicValue::TaggedSum {
            type_id,
            arm,
            payload,
        } => validate_tagged_sum(bundle, roots, closed_set, type_id, arm, payload)?,
        MonomorphicValue::BoundaryPresence {
            type_id,
            arm,
            value,
        } => {
            let arguments = require_instance(closed_set, type_id, "boundary_field")?;
            match (arm, value) {
                (BoundaryArm::Missing | BoundaryArm::Null, None) => 1,
                (BoundaryArm::Value, Some(value)) => {
                    require_value_type(value, &arguments[0])?;
                    1_u64
                        .checked_add(child_cells(value)?)
                        .ok_or_else(|| value_failure(FoundationErrorCode::TotalValueCells))?
                }
                _ => return Err(value_failure(FoundationErrorCode::ConcreteValueInvariant)),
            }
        }
        MonomorphicValue::Date {
            type_id,
            day_number,
        } => {
            primitive("date", type_id)?;
            if *day_number > 3_652_058 {
                return Err(value_failure(FoundationErrorCode::ConcreteValueInvariant));
            }
            1
        }
        MonomorphicValue::Time { type_id, ticks } => {
            primitive("time", type_id)?;
            let ticks = parse_canonical_integer(ticks)
                .filter(|value| (0..=863_999_999_999).contains(value))
                .ok_or_else(|| value_failure(FoundationErrorCode::ConcreteValueInvariant))?;
            let _ = ticks;
            1
        }
        MonomorphicValue::Duration { type_id, ticks } => {
            primitive("duration", type_id)?;
            require_i64_string(ticks)?;
            1
        }
        MonomorphicValue::Instant {
            type_id,
            milliseconds,
        } => {
            primitive("instant", type_id)?;
            require_i64_string(milliseconds)?;
            1
        }
        MonomorphicValue::Guid { type_id, n } => {
            primitive("guid", type_id)?;
            if !valid_fixed_hex(n, 32) {
                return Err(value_failure(FoundationErrorCode::ConcreteValueInvariant));
            }
            1
        }
        MonomorphicValue::Money {
            type_id,
            amount,
            currency,
        } => {
            let arguments = require_instance(closed_set, type_id, "money")?;
            require_value_type(amount, "mpk.csharp.value.decimal.v1")?;
            require_value_type(currency, &arguments[0])?;
            let amount_cells = child_cells(amount)?;
            let currency_cells = child_cells(currency)?;
            1_u64
                .checked_add(amount_cells)
                .and_then(|count| count.checked_add(currency_cells))
                .ok_or_else(|| value_failure(FoundationErrorCode::TotalValueCells))?
        }
        MonomorphicValue::Transition {
            type_id,
            state,
            events,
            response,
        } => {
            let arguments = require_instance(closed_set, type_id, "transition")?;
            require_value_type(state, &arguments[0])?;
            require_value_type(response, &arguments[2])?;
            enforce_value_bound(events.len(), TRANSITION_EVENTS_MAX)?;
            validate_homogeneous_values(events, &arguments[1])?;
            let state_cells = child_cells(state)?;
            let event_cells = nested_cells(events)?;
            let response_cells = child_cells(response)?;
            1_u64
                .checked_add(state_cells)
                .and_then(|count| count.checked_add(event_cells))
                .and_then(|count| count.checked_add(response_cells))
                .ok_or_else(|| value_failure(FoundationErrorCode::TotalValueCells))?
        }
        MonomorphicValue::ParseError { type_id, .. } => {
            primitive("parse_error", type_id)?;
            1
        }
        MonomorphicValue::ClosedException {
            type_id,
            tag,
            source_type_id,
            payload,
        } => {
            primitive("exception", type_id)?;
            if !allow_exception {
                return Err(value_failure(FoundationErrorCode::ConcreteValueInvariant));
            }
            if *tag < 9 {
                if source_type_id.is_some() || payload.is_some() {
                    return Err(value_failure(FoundationErrorCode::ConcreteValueInvariant));
                }
                1
            } else {
                let source_id = source_type_id
                    .as_deref()
                    .ok_or_else(|| value_failure(FoundationErrorCode::ConcreteValueInvariant))?;
                if !roots
                    .source_types
                    .get(source_id)
                    .is_some_and(|source| source.kind == SourceKind::SealedClass)
                {
                    return Err(value_failure(FoundationErrorCode::ConcreteValueType));
                }
                let payload = payload
                    .as_deref()
                    .ok_or_else(|| value_failure(FoundationErrorCode::ConcreteValueInvariant))?;
                require_value_type(payload, source_id)?;
                1_u64
                    .checked_add(child_cells(payload)?)
                    .ok_or_else(|| value_failure(FoundationErrorCode::TotalValueCells))?
            }
        }
    };
    if cells > TOTAL_VALUE_CELLS_MAX {
        Err(value_failure(FoundationErrorCode::TotalValueCells))
    } else {
        Ok(cells)
    }
}

fn validate_product_shape(
    bundle: &ValidatedFoundationBundle,
    roots: &ValidatedClosedRootSet,
    _closed_set: &ClosedInstanceSet,
    type_id: &str,
    fields: &[NamedMonomorphicValue],
) -> Result<(), FoundationValidationError> {
    if let Some(source) = roots.source_types.get(type_id) {
        if source.kind == SourceKind::Enum || fields.len() != source.members.len() {
            return Err(value_failure(FoundationErrorCode::ConcreteValueType));
        }
        for (field, member) in fields.iter().zip(&source.members) {
            if field.name != member.name
                || field.value.type_id() != closed_type_id(bundle, &member.ty)?
            {
                return Err(value_failure(FoundationErrorCode::ConcreteValueInvariant));
            }
        }
        return Ok(());
    }
    Err(value_failure(FoundationErrorCode::ConcreteValueType))
}

fn validate_tagged_sum(
    bundle: &ValidatedFoundationBundle,
    roots: &ValidatedClosedRootSet,
    closed_set: &ClosedInstanceSet,
    type_id: &str,
    arm: &str,
    payload: &[MonomorphicValue],
) -> Result<u64, FoundationValidationError> {
    let metadata = closed_set
        .metadata
        .get(type_id)
        .ok_or_else(|| value_failure(FoundationErrorCode::ConcreteValueType))?;
    let template = template_name(&metadata.template_id)
        .ok_or_else(|| value_failure(FoundationErrorCode::ConcreteValueType))?;
    let expected_types: Vec<&str> = match (template, arm) {
        ("lookup", "missing_key") => Vec::new(),
        ("lookup", "found") => vec![&metadata.argument_ids[0]],
        ("result", "ok") => vec![&metadata.argument_ids[0]],
        ("result", "error") => vec![&metadata.argument_ids[1]],
        ("validation", "valid") => vec![&metadata.argument_ids[0]],
        ("validation", "invalid") => {
            let dependency = metadata
                .dependency_ids
                .first()
                .ok_or_else(|| value_failure(FoundationErrorCode::ConcreteValueInvariant))?;
            vec![dependency]
        }
        _ => return Err(value_failure(FoundationErrorCode::ConcreteValueInvariant)),
    };
    if payload.len() != expected_types.len() {
        return Err(value_failure(FoundationErrorCode::ConcreteValueInvariant));
    }
    for (value, expected) in payload.iter().zip(expected_types) {
        require_value_type(value, expected)?;
    }
    if template == "validation" && arm == "invalid" {
        let MonomorphicValue::Sequence { elements, .. } = &payload[0] else {
            return Err(value_failure(FoundationErrorCode::ConcreteValueInvariant));
        };
        if elements.is_empty()
            || u64::try_from(elements.len()).unwrap_or(u64::MAX) > VALIDATION_ERRORS_MAX
        {
            return Err(value_failure(FoundationErrorCode::ConcreteValueBound));
        }
    }
    payload.iter().try_fold(1_u64, |total, item| {
        let cells = validate_value_inner(bundle, roots, closed_set, item, false)?;
        total
            .checked_add(cells)
            .ok_or_else(|| value_failure(FoundationErrorCode::TotalValueCells))
    })
}

fn ensure_strictly_increasing<'a>(
    bundle: &ValidatedFoundationBundle,
    roots: &ValidatedClosedRootSet,
    closed_set: &ClosedInstanceSet,
    values: impl IntoIterator<Item = &'a MonomorphicValue>,
) -> Result<(), FoundationValidationError> {
    let mut previous = None;
    for value in values {
        if let Some(previous) = previous {
            if compare_monomorphic_values(bundle, roots, closed_set, previous, value)?
                != Ordering::Less
            {
                return Err(value_failure(FoundationErrorCode::ConcreteValueInvariant));
            }
        }
        previous = Some(value);
    }
    Ok(())
}

fn compare_monomorphic_values(
    bundle: &ValidatedFoundationBundle,
    roots: &ValidatedClosedRootSet,
    closed_set: &ClosedInstanceSet,
    left: &MonomorphicValue,
    right: &MonomorphicValue,
) -> Result<Ordering, FoundationValidationError> {
    if left.type_id() != right.type_id() {
        return Err(value_failure(FoundationErrorCode::ConcreteValueType));
    }
    let ordering = match (left, right) {
        (MonomorphicValue::Unit { .. }, MonomorphicValue::Unit { .. }) => Ordering::Equal,
        (
            MonomorphicValue::Bool { value: left, .. },
            MonomorphicValue::Bool { value: right, .. },
        ) => left.cmp(right),
        (
            MonomorphicValue::Signed { value: left, .. },
            MonomorphicValue::Signed { value: right, .. },
        )
        | (
            MonomorphicValue::Unsigned { value: left, .. },
            MonomorphicValue::Unsigned { value: right, .. },
        ) => canonical_integer_for_order(left)?.cmp(&canonical_integer_for_order(right)?),
        (
            MonomorphicValue::Char { utf16: left, .. },
            MonomorphicValue::Char { utf16: right, .. },
        ) => left.cmp(right),
        (
            MonomorphicValue::String { utf16: left, .. },
            MonomorphicValue::String { utf16: right, .. },
        ) => left.cmp(right),
        (
            MonomorphicValue::DecimalBits {
                negative: left_negative,
                scale: left_scale,
                coefficient: left_coefficient,
                ..
            },
            MonomorphicValue::DecimalBits {
                negative: right_negative,
                scale: right_scale,
                coefficient: right_coefficient,
                ..
            },
        ) => compare_decimal_values(
            *left_negative,
            *left_scale,
            left_coefficient,
            *right_negative,
            *right_scale,
            right_coefficient,
        )?,
        (
            MonomorphicValue::Enum { carrier: left, .. },
            MonomorphicValue::Enum { carrier: right, .. },
        ) => canonical_integer_for_order(left)?.cmp(&canonical_integer_for_order(right)?),
        (
            MonomorphicValue::Product { fields: left, .. },
            MonomorphicValue::Product { fields: right, .. },
        ) => compare_named_values(bundle, roots, closed_set, left, right)?,
        (
            MonomorphicValue::Array { elements: left, .. },
            MonomorphicValue::Array {
                elements: right, ..
            },
        )
        | (
            MonomorphicValue::Array { elements: left, .. },
            MonomorphicValue::Sequence {
                elements: right, ..
            },
        )
        | (
            MonomorphicValue::Sequence { elements: left, .. },
            MonomorphicValue::Array {
                elements: right, ..
            },
        )
        | (
            MonomorphicValue::Sequence { elements: left, .. },
            MonomorphicValue::Sequence {
                elements: right, ..
            },
        )
        | (
            MonomorphicValue::OrderedSet { elements: left, .. },
            MonomorphicValue::OrderedSet {
                elements: right, ..
            },
        ) => compare_value_slices(bundle, roots, closed_set, left, right)?,
        (
            MonomorphicValue::OrderedEntry {
                key: left_key,
                value: left_value,
                ..
            },
            MonomorphicValue::OrderedEntry {
                key: right_key,
                value: right_value,
                ..
            },
        ) => compare_pair(
            bundle,
            roots,
            closed_set,
            left_key,
            right_key,
            left_value,
            right_value,
        )?,
        (
            MonomorphicValue::OrderedMap { entries: left, .. },
            MonomorphicValue::OrderedMap { entries: right, .. },
        ) => compare_map_entries(bundle, roots, closed_set, left, right)?,
        (
            MonomorphicValue::Option {
                arm: left_arm,
                value: left_value,
                ..
            },
            MonomorphicValue::Option {
                arm: right_arm,
                value: right_value,
                ..
            },
        ) => compare_optional_arm(
            bundle,
            roots,
            closed_set,
            option_rank(*left_arm),
            left_value.as_deref(),
            option_rank(*right_arm),
            right_value.as_deref(),
        )?,
        (
            MonomorphicValue::TaggedSum {
                type_id,
                arm: left_arm,
                payload: left_payload,
            },
            MonomorphicValue::TaggedSum {
                arm: right_arm,
                payload: right_payload,
                ..
            },
        ) => {
            let left_rank = tagged_sum_rank(closed_set, type_id, left_arm)?;
            let right_rank = tagged_sum_rank(closed_set, type_id, right_arm)?;
            let rank = left_rank.cmp(&right_rank);
            if rank != Ordering::Equal {
                rank
            } else {
                compare_value_slices(bundle, roots, closed_set, left_payload, right_payload)?
            }
        }
        (
            MonomorphicValue::BoundaryPresence {
                arm: left_arm,
                value: left_value,
                ..
            },
            MonomorphicValue::BoundaryPresence {
                arm: right_arm,
                value: right_value,
                ..
            },
        ) => compare_optional_arm(
            bundle,
            roots,
            closed_set,
            boundary_rank(*left_arm),
            left_value.as_deref(),
            boundary_rank(*right_arm),
            right_value.as_deref(),
        )?,
        (
            MonomorphicValue::Date {
                day_number: left, ..
            },
            MonomorphicValue::Date {
                day_number: right, ..
            },
        ) => left.cmp(right),
        (
            MonomorphicValue::Time { ticks: left, .. },
            MonomorphicValue::Time { ticks: right, .. },
        )
        | (
            MonomorphicValue::Duration { ticks: left, .. },
            MonomorphicValue::Duration { ticks: right, .. },
        ) => canonical_integer_for_order(left)?.cmp(&canonical_integer_for_order(right)?),
        (
            MonomorphicValue::Instant {
                milliseconds: left, ..
            },
            MonomorphicValue::Instant {
                milliseconds: right,
                ..
            },
        ) => canonical_integer_for_order(left)?.cmp(&canonical_integer_for_order(right)?),
        (MonomorphicValue::Guid { n: left, .. }, MonomorphicValue::Guid { n: right, .. }) => {
            left.as_bytes().cmp(right.as_bytes())
        }
        (
            MonomorphicValue::Money {
                amount: left_amount,
                currency: left_currency,
                ..
            },
            MonomorphicValue::Money {
                amount: right_amount,
                currency: right_currency,
                ..
            },
        ) => compare_pair(
            bundle,
            roots,
            closed_set,
            left_currency,
            right_currency,
            left_amount,
            right_amount,
        )?,
        (
            MonomorphicValue::Transition {
                state: left_state,
                events: left_events,
                response: left_response,
                ..
            },
            MonomorphicValue::Transition {
                state: right_state,
                events: right_events,
                response: right_response,
                ..
            },
        ) => {
            let state =
                compare_monomorphic_values(bundle, roots, closed_set, left_state, right_state)?;
            if state != Ordering::Equal {
                state
            } else {
                let events =
                    compare_value_slices(bundle, roots, closed_set, left_events, right_events)?;
                if events != Ordering::Equal {
                    events
                } else {
                    compare_monomorphic_values(
                        bundle,
                        roots,
                        closed_set,
                        left_response,
                        right_response,
                    )?
                }
            }
        }
        (
            MonomorphicValue::ParseError { arm: left, .. },
            MonomorphicValue::ParseError { arm: right, .. },
        ) => parse_error_rank(*left).cmp(&parse_error_rank(*right)),
        (
            MonomorphicValue::F32Bits { .. }
            | MonomorphicValue::F64Bits { .. }
            | MonomorphicValue::ClosedException { .. },
            _,
        ) => return Err(value_failure(FoundationErrorCode::ConcreteValueInvariant)),
        _ => return Err(value_failure(FoundationErrorCode::ConcreteValueType)),
    };
    Ok(ordering)
}

fn compare_named_values(
    bundle: &ValidatedFoundationBundle,
    roots: &ValidatedClosedRootSet,
    closed_set: &ClosedInstanceSet,
    left: &[NamedMonomorphicValue],
    right: &[NamedMonomorphicValue],
) -> Result<Ordering, FoundationValidationError> {
    if left.len() != right.len()
        || left
            .iter()
            .zip(right)
            .any(|(left, right)| left.name != right.name)
    {
        return Err(value_failure(FoundationErrorCode::ConcreteValueInvariant));
    }
    for (left, right) in left.iter().zip(right) {
        let ordering =
            compare_monomorphic_values(bundle, roots, closed_set, &left.value, &right.value)?;
        if ordering != Ordering::Equal {
            return Ok(ordering);
        }
    }
    Ok(Ordering::Equal)
}

fn compare_value_slices(
    bundle: &ValidatedFoundationBundle,
    roots: &ValidatedClosedRootSet,
    closed_set: &ClosedInstanceSet,
    left: &[MonomorphicValue],
    right: &[MonomorphicValue],
) -> Result<Ordering, FoundationValidationError> {
    for (left, right) in left.iter().zip(right) {
        let ordering = compare_monomorphic_values(bundle, roots, closed_set, left, right)?;
        if ordering != Ordering::Equal {
            return Ok(ordering);
        }
    }
    Ok(left.len().cmp(&right.len()))
}

fn compare_pair(
    bundle: &ValidatedFoundationBundle,
    roots: &ValidatedClosedRootSet,
    closed_set: &ClosedInstanceSet,
    left_first: &MonomorphicValue,
    right_first: &MonomorphicValue,
    left_second: &MonomorphicValue,
    right_second: &MonomorphicValue,
) -> Result<Ordering, FoundationValidationError> {
    let first = compare_monomorphic_values(bundle, roots, closed_set, left_first, right_first)?;
    if first != Ordering::Equal {
        Ok(first)
    } else {
        compare_monomorphic_values(bundle, roots, closed_set, left_second, right_second)
    }
}

fn compare_map_entries(
    bundle: &ValidatedFoundationBundle,
    roots: &ValidatedClosedRootSet,
    closed_set: &ClosedInstanceSet,
    left: &[MonomorphicMapEntry],
    right: &[MonomorphicMapEntry],
) -> Result<Ordering, FoundationValidationError> {
    for (left, right) in left.iter().zip(right) {
        let ordering = compare_pair(
            bundle,
            roots,
            closed_set,
            &left.key,
            &right.key,
            &left.value,
            &right.value,
        )?;
        if ordering != Ordering::Equal {
            return Ok(ordering);
        }
    }
    Ok(left.len().cmp(&right.len()))
}

fn compare_optional_arm(
    bundle: &ValidatedFoundationBundle,
    roots: &ValidatedClosedRootSet,
    closed_set: &ClosedInstanceSet,
    left_rank: u8,
    left_value: Option<&MonomorphicValue>,
    right_rank: u8,
    right_value: Option<&MonomorphicValue>,
) -> Result<Ordering, FoundationValidationError> {
    let rank = left_rank.cmp(&right_rank);
    if rank != Ordering::Equal {
        return Ok(rank);
    }
    match (left_value, right_value) {
        (None, None) => Ok(Ordering::Equal),
        (Some(left), Some(right)) => {
            compare_monomorphic_values(bundle, roots, closed_set, left, right)
        }
        _ => Err(value_failure(FoundationErrorCode::ConcreteValueInvariant)),
    }
}

const fn option_rank(arm: OptionArm) -> u8 {
    match arm {
        OptionArm::None => 0,
        OptionArm::Some => 1,
    }
}

const fn boundary_rank(arm: BoundaryArm) -> u8 {
    match arm {
        BoundaryArm::Missing => 0,
        BoundaryArm::Null => 1,
        BoundaryArm::Value => 2,
    }
}

const fn parse_error_rank(arm: ParseErrorArm) -> u8 {
    match arm {
        ParseErrorArm::InputBound => 0,
        ParseErrorArm::Syntax => 1,
        ParseErrorArm::Noncanonical => 2,
        ParseErrorArm::ScalePrecision => 3,
        ParseErrorArm::Range => 4,
    }
}

fn tagged_sum_rank(
    closed_set: &ClosedInstanceSet,
    type_id: &str,
    arm: &str,
) -> Result<u8, FoundationValidationError> {
    let template = closed_set
        .metadata
        .get(type_id)
        .and_then(|metadata| template_name(&metadata.template_id));
    match (template, arm) {
        (Some("lookup"), "missing_key")
        | (Some("result"), "ok")
        | (Some("validation"), "valid") => Ok(0),
        (Some("lookup"), "found") | (Some("result"), "error") | (Some("validation"), "invalid") => {
            Ok(1)
        }
        _ => Err(value_failure(FoundationErrorCode::ConcreteValueInvariant)),
    }
}

fn canonical_integer_for_order(value: &str) -> Result<i128, FoundationValidationError> {
    parse_canonical_integer(value)
        .ok_or_else(|| value_failure(FoundationErrorCode::ConcreteValueInvariant))
}

fn compare_decimal_values(
    left_negative: bool,
    left_scale: u8,
    left_coefficient: &str,
    right_negative: bool,
    right_scale: u8,
    right_coefficient: &str,
) -> Result<Ordering, FoundationValidationError> {
    let (left_digits, left_scale) = normalized_decimal(left_coefficient, left_scale)?;
    let (right_digits, right_scale) = normalized_decimal(right_coefficient, right_scale)?;
    let left_zero = left_digits == "0";
    let right_zero = right_digits == "0";
    if left_zero && right_zero {
        return Ok(Ordering::Equal);
    }
    let left_negative = left_negative && !left_zero;
    let right_negative = right_negative && !right_zero;
    if left_negative != right_negative {
        return Ok(if left_negative {
            Ordering::Less
        } else {
            Ordering::Greater
        });
    }
    if left_zero {
        return Ok(Ordering::Less);
    }
    if right_zero {
        return Ok(Ordering::Greater);
    }
    let magnitude = compare_decimal_magnitude(&left_digits, left_scale, &right_digits, right_scale);
    Ok(if left_negative {
        magnitude.reverse()
    } else {
        magnitude
    })
}

fn normalized_decimal(
    coefficient: &str,
    mut scale: u8,
) -> Result<(String, u8), FoundationValidationError> {
    let parsed = canonical_integer_for_order(coefficient)?;
    if parsed < 0 || scale > 28 {
        return Err(value_failure(FoundationErrorCode::ConcreteValueInvariant));
    }
    let mut digits = coefficient.to_owned();
    while scale > 0 && digits.ends_with('0') && digits.len() > 1 {
        digits.pop();
        scale -= 1;
    }
    Ok((digits, scale))
}

fn compare_decimal_magnitude(
    left_digits: &str,
    left_scale: u8,
    right_digits: &str,
    right_scale: u8,
) -> Ordering {
    let left_integer_digits =
        i32::try_from(left_digits.len()).unwrap_or(i32::MAX) - i32::from(left_scale);
    let right_integer_digits =
        i32::try_from(right_digits.len()).unwrap_or(i32::MAX) - i32::from(right_scale);
    let exponent = left_integer_digits.cmp(&right_integer_digits);
    if exponent != Ordering::Equal {
        return exponent;
    }
    let scale = left_scale.max(right_scale);
    let mut left = left_digits.to_owned();
    let mut right = right_digits.to_owned();
    left.extend(std::iter::repeat_n('0', usize::from(scale - left_scale)));
    right.extend(std::iter::repeat_n('0', usize::from(scale - right_scale)));
    left.as_bytes().cmp(right.as_bytes())
}

fn require_instance<'a>(
    closed_set: &'a ClosedInstanceSet,
    type_id: &str,
    expected_template: &str,
) -> Result<&'a [String], FoundationValidationError> {
    let metadata = closed_set
        .metadata
        .get(type_id)
        .filter(|metadata| template_name(&metadata.template_id) == Some(expected_template))
        .ok_or_else(|| value_failure(FoundationErrorCode::ConcreteValueType))?;
    Ok(&metadata.argument_ids)
}

fn template_name(template_id: &str) -> Option<&str> {
    template_id
        .strip_prefix("mpk.csharp.semantic.")?
        .strip_suffix(".v1")
}

fn require_value_type(
    value: &MonomorphicValue,
    expected: &str,
) -> Result<(), FoundationValidationError> {
    if value.type_id() == expected {
        Ok(())
    } else {
        Err(value_failure(FoundationErrorCode::ConcreteValueType))
    }
}

fn validate_homogeneous_values(
    values: &[MonomorphicValue],
    expected: &str,
) -> Result<(), FoundationValidationError> {
    for value in values {
        require_value_type(value, expected)?;
    }
    Ok(())
}

fn enforce_value_bound(length: usize, maximum: u64) -> Result<(), FoundationValidationError> {
    if u64::try_from(length).unwrap_or(u64::MAX) <= maximum {
        Ok(())
    } else {
        Err(value_failure(FoundationErrorCode::ConcreteValueBound))
    }
}

fn require_i64_string(value: &str) -> Result<i64, FoundationValidationError> {
    if parse_canonical_integer(value).is_none() {
        return Err(value_failure(FoundationErrorCode::ConcreteValueInvariant));
    }
    value
        .parse::<i64>()
        .map_err(|_| value_failure(FoundationErrorCode::ConcreteValueInvariant))
}

fn parse_canonical_integer(value: &str) -> Option<i128> {
    if value == "0"
        || value.strip_prefix('-').is_some_and(|digits| {
            !digits.is_empty()
                && !digits.starts_with('0')
                && digits.bytes().all(|b| b.is_ascii_digit())
        })
        || (!value.starts_with('0') && value.bytes().all(|byte| byte.is_ascii_digit()))
    {
        value.parse::<i128>().ok()
    } else {
        None
    }
}

fn valid_fixed_hex(value: &str, length: usize) -> bool {
    value.len() == length
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn value_failure(code: FoundationErrorCode) -> FoundationValidationError {
    failure(FoundationValidationPhase::ConcreteValue, code)
}

fn source_failure(code: FoundationErrorCode) -> FoundationValidationError {
    failure(FoundationValidationPhase::SourceTypes, code)
}

fn required_string(
    object: &Map<String, Value>,
    name: &str,
    code: FoundationErrorCode,
) -> Result<String, FoundationValidationError> {
    object
        .get(name)
        .and_then(Value::as_str)
        .map(str::to_owned)
        .ok_or_else(|| source_failure(code))
}

fn required_string_array(
    object: &Map<String, Value>,
    name: &str,
    code: FoundationErrorCode,
) -> Result<Vec<String>, FoundationValidationError> {
    object
        .get(name)
        .and_then(Value::as_array)
        .ok_or_else(|| source_failure(code))?
        .iter()
        .map(|value| {
            value
                .as_str()
                .map(str::to_owned)
                .ok_or_else(|| source_failure(code))
        })
        .collect()
}

fn require_exact_map_fields(
    object: &Map<String, Value>,
    expected: &[&str],
    code: FoundationErrorCode,
) -> Result<(), FoundationValidationError> {
    if object.len() == expected.len() && expected.iter().all(|name| object.contains_key(*name)) {
        Ok(())
    } else {
        let phase = if matches!(
            code,
            FoundationErrorCode::TypeShape
                | FoundationErrorCode::ParameterShape
                | FoundationErrorCode::ParameterArity
        ) {
            FoundationValidationPhase::Type
        } else {
            FoundationValidationPhase::SourceTypes
        };
        Err(failure(phase, code))
    }
}

fn has_exact_fields(value: &Value, expected: &[&str]) -> bool {
    value.as_object().is_some_and(|object| {
        object.len() == expected.len() && expected.iter().all(|name| object.contains_key(*name))
    })
}

fn valid_provenance_id(value: &str) -> bool {
    let bytes = value.as_bytes();
    (1..=256).contains(&bytes.len())
        && bytes[0].is_ascii_lowercase()
        && bytes[1..].iter().all(|byte| {
            byte.is_ascii_lowercase()
                || byte.is_ascii_digit()
                || matches!(byte, b'_' | b'.' | b':' | b'-')
        })
}

fn valid_sha256(value: &str) -> bool {
    valid_fixed_hex(value, 64)
}

fn parse_canonical_transport(
    transport: &[u8],
    maximum: u64,
    phase: FoundationValidationPhase,
) -> Result<StrictJsonValue, FoundationValidationError> {
    let limits = StrictJsonLimits::new(
        maximum,
        maximum,
        FOUNDATION_JSON_DEPTH_MAX,
        FOUNDATION_STRING_BYTES_MAX,
    );
    let value = parse_strict_json(transport, limits).map_err(|error| {
        let code = match error {
            StrictJsonError::DuplicateObjectName { .. } => FoundationErrorCode::DuplicateJsonKey,
            StrictJsonError::FloatingPointNumber { .. } => FoundationErrorCode::FloatingJson,
            _ if contains_nonfinite_token(transport) => FoundationErrorCode::NonfiniteJson,
            _ => FoundationErrorCode::Transport,
        };
        failure(phase, code)
    })?;
    let mut canonical = canonical_json_bytes_bounded(
        &value,
        usize::try_from(maximum).map_err(|_| failure(phase, FoundationErrorCode::Transport))?,
    )
    .map_err(|_| failure(phase, FoundationErrorCode::Transport))?;
    canonical.push(b'\n');
    if canonical != transport {
        return Err(failure(phase, FoundationErrorCode::CanonicalTransport));
    }
    Ok(value)
}

fn contains_nonfinite_token(transport: &[u8]) -> bool {
    [
        b"NaN".as_slice(),
        b"Infinity".as_slice(),
        b"-Infinity".as_slice(),
    ]
    .iter()
    .any(|needle| {
        transport
            .windows(needle.len())
            .any(|window| window == *needle)
    })
}

fn canonical_transport(value: &Value, maximum: u64) -> Result<Vec<u8>, FoundationValidationError> {
    let strict = serde_to_strict(value).map_err(|_| {
        failure(
            FoundationValidationPhase::Transport,
            FoundationErrorCode::Transport,
        )
    })?;
    let mut bytes = canonical_json_bytes_bounded(
        &strict,
        usize::try_from(maximum).map_err(|_| {
            failure(
                FoundationValidationPhase::Transport,
                FoundationErrorCode::Transport,
            )
        })?,
    )
    .map_err(|_| {
        failure(
            FoundationValidationPhase::Transport,
            FoundationErrorCode::Transport,
        )
    })?;
    bytes.push(b'\n');
    Ok(bytes)
}

fn hash_value(domain: HashDomain, value: &Value) -> Result<String, FoundationValidationError> {
    let strict = serde_to_strict(value).map_err(|_| {
        failure(
            FoundationValidationPhase::Expansion,
            FoundationErrorCode::ResidualGeneric,
        )
    })?;
    hash_canonical_json(domain, &strict)
        .map(|hash| hash.to_hex())
        .map_err(|_| {
            failure(
                FoundationValidationPhase::Expansion,
                FoundationErrorCode::ResidualGeneric,
            )
        })
}

fn strict_to_serde(value: &StrictJsonValue) -> Value {
    match value {
        StrictJsonValue::Null => Value::Null,
        StrictJsonValue::Bool(value) => Value::Bool(*value),
        StrictJsonValue::Integer(value) => Value::Number(Number::from(*value)),
        StrictJsonValue::String(value) => Value::String(value.clone()),
        StrictJsonValue::Array(values) => {
            Value::Array(values.iter().map(strict_to_serde).collect())
        }
        StrictJsonValue::Object(entries) => Value::Object(
            entries
                .iter()
                .map(|(name, value)| (name.clone(), strict_to_serde(value)))
                .collect(),
        ),
    }
}

fn serde_to_strict(value: &Value) -> Result<StrictJsonValue, ()> {
    match value {
        Value::Null => Ok(StrictJsonValue::Null),
        Value::Bool(value) => Ok(StrictJsonValue::Bool(*value)),
        Value::Number(value) => value.as_i64().map(StrictJsonValue::Integer).ok_or(()),
        Value::String(value) => Ok(StrictJsonValue::String(value.clone())),
        Value::Array(values) => values
            .iter()
            .map(serde_to_strict)
            .collect::<Result<Vec<_>, _>>()
            .map(StrictJsonValue::Array),
        Value::Object(object) => object
            .iter()
            .map(|(name, value)| serde_to_strict(value).map(|value| (name.clone(), value)))
            .collect::<Result<Vec<_>, _>>()
            .map(StrictJsonValue::Object),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registered_bundle_self_check() {
        let bundle = validate_registered_foundation_bundle(
            registered_foundation_descriptor_transport(),
            registered_foundation_definitions_transport(),
        )
        .expect("embedded registered foundation");
        assert_eq!(bundle.templates.len(), 12);
        assert_eq!(bundle.non_template_definitions().len(), 4);
        assert_eq!(
            bundle.content_sha256(),
            FOUNDATION_DESCRIPTOR_CONTENT_SHA256
        );
    }

    #[test]
    fn primitive_ids_are_closed() {
        let bundle = validate_registered_foundation_bundle(
            registered_foundation_descriptor_transport(),
            registered_foundation_definitions_transport(),
        )
        .expect("embedded registered foundation");
        for primitive in PRIMITIVES {
            assert_eq!(
                closed_type_id(&bundle, &ClosedType::Primitive((*primitive).to_owned()))
                    .expect("primitive ID"),
                format!("mpk.csharp.value.{primitive}.v1")
            );
        }
    }
}
