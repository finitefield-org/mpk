//! Strict, private `mpk.vir.v2` importer for the practical C# candidate.
//!
//! The importer is intentionally detached from Roslyn, the CLR, and every
//! installed route.  Its complete authority is supplied as immutable bytes:
//! the registered foundation, the closed root/instance set, and the W04
//! context-bound binding/operation artifacts.  Nothing accepted here is a
//! compiler or runtime callback result.

use crate::canonical_json::{
    scan_strict_json, StrictJsonError, StrictJsonEvent, StrictJsonLimits, StrictJsonObserver,
    StrictJsonPathSegment, StrictJsonValueKind,
};
use crate::csharp_practical_registry::{FOUNDATION_DESCRIPTOR_ID, FOUNDATION_DESCRIPTOR_SCHEMA};
use crate::csharp_practical_source_artifacts::{
    bind_closed_instances, canonical_practical_json_bytes, parse_canonical_practical_json,
    validate_semantic_bindings_document, ArtifactRef, CapturedInputSet, PracticalArtifactContext,
    PracticalArtifactKind, PracticalJsonValue, OPERATIONS_HASH_DOMAIN, REQUIRED_CHECKS_HASH_DOMAIN,
    SEMANTIC_BINDINGS_SCHEMA, SUCCESSOR_VIR_SCHEMA,
};
use crate::csharp_practical_vir_model::{
    derive_closed_exception_universe, is_known_concrete_type, is_valid_vocabulary_id,
    validate_binding_operation_commutation, validate_closed_instance_set,
    validate_closed_operation_signature, validate_closed_root_set, validate_explicit_control_graph,
    validate_foundation_context_linkage, validate_operation_invocation,
    validate_registered_foundation_bundle, AbruptCompletion, BindingOperationCommutation,
    BindingTypeProjection, ClosedExceptionUniverse, ClosedInstanceSet, ClosedOperationSignature,
    ClosedOperationTag, ConstructionStatus, ControlNode, ControlNodeTag, ExceptionHandlerRegion,
    ExceptionUnwindPlan, ExplicitControlGraph, LoopRegion, OperationInvocation, PatternDecision,
    PracticalVirValidationError, PracticalVirValidationPhase, RequiredCheck, RequiredCheckTag,
    SequenceConstructionAction, SequenceConstructionState, SourceExceptionDefinition,
    TypedValueRef, ValidatedClosedRootSet, ValidatedFoundationBundle, CLOSED_INSTANCE_COUNT_MAX,
    CSHARP_PRACTICAL_OPERATIONS_SCHEMA, CSHARP_PRACTICAL_REQUIRED_CHECKS_SCHEMA,
    EXPANDED_DECLARATIONS_MAX, EXPANDED_OPERATIONS_MAX, FOUNDATION_BINDING_COUNT_MAX,
    PROJECTION_OBLIGATIONS_PER_BINDING_MAX,
};
use crate::hash::{hash_domain_separated_raw, HashDomain};
use crate::vir_validate::{
    VIR_BLOCK_PARAMETERS_MAX, VIR_CALL_ARGS_MAX, VIR_CFG_EDGES_PER_FUNCTION_MAX, VIR_PARAMS_MAX,
};
use serde::{Deserialize, Serialize};
use serde_json::{value::RawValue, Value};
use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use std::fmt;

pub const CSHARP_PRACTICAL_VIR_HASH_DOMAIN: HashDomain = HashDomain::new("MPK-VIR-2.0");

pub const CSHARP_PRACTICAL_VIR_INPUT_BYTES_MAX: u64 = 268_435_456;
pub const CSHARP_PRACTICAL_VIR_CANONICAL_BYTES_MAX: u64 = 201_326_592;
pub const CSHARP_PRACTICAL_VIR_JSON_NODES_MAX: u64 = CSHARP_PRACTICAL_VIR_INPUT_BYTES_MAX;
pub const CSHARP_PRACTICAL_VIR_JSON_NESTING_MAX: u64 = 256;
pub const CSHARP_PRACTICAL_VIR_STRING_BYTES_MAX: u64 = 1_048_576;
pub const CSHARP_PRACTICAL_VIR_FUNCTIONS_MAX: u64 = 128;
pub const CSHARP_PRACTICAL_VIR_BLOCKS_PER_FUNCTION_MAX: u64 = 1_024;
pub const CSHARP_PRACTICAL_VIR_BLOCKS_MAX: u64 = 8_192;
pub const CSHARP_PRACTICAL_VIR_OPERATIONS_PER_FUNCTION_MAX: u64 = 100_000;
pub const CSHARP_PRACTICAL_VIR_OPERATIONS_MAX: u64 = 250_000;
pub const CSHARP_PRACTICAL_VIR_CONSTRUCTIONS_PER_FUNCTION_MAX: u64 = 32;
pub const CSHARP_PRACTICAL_VIR_LIVE_CONSTRUCTIONS_MAX: u64 = 8;
pub const CSHARP_PRACTICAL_VIR_SOURCE_TYPES_MAX: u64 = 128;
pub const CSHARP_PRACTICAL_VIR_SOURCE_EXCEPTIONS_MAX: u64 = 32;
pub const CSHARP_PRACTICAL_VIR_LOOPS_PER_FUNCTION_MAX: u64 = 32;
pub const CSHARP_PRACTICAL_VIR_LOOP_NESTING_MAX: u64 = 8;
pub const CSHARP_PRACTICAL_VIR_PATTERN_ARMS_PER_FUNCTION_MAX: u64 = 256;
pub const CSHARP_PRACTICAL_VIR_EXCEPTION_REGIONS_PER_FUNCTION_MAX: u64 = 32;
pub const CSHARP_PRACTICAL_VIR_RESULTS_PER_FUNCTION_MAX: u64 = 1;
pub const CSHARP_PRACTICAL_VIR_BINDING_PROJECTIONS_MAX: u64 = EXPANDED_DECLARATIONS_MAX;
pub const CSHARP_PRACTICAL_VIR_BINDING_COMMUTATIONS_MAX: u64 =
    FOUNDATION_BINDING_COUNT_MAX * PROJECTION_OBLIGATIONS_PER_BINDING_MAX;

const ZERO_SHA256: &str = "0000000000000000000000000000000000000000000000000000000000000000";
const BOOL_TYPE_ID: &str = "mpk.csharp.value.bool.v1";
const EXCEPTION_VALUE_TYPE_ID: &str = "mpk.csharp.value.exception.v1";
const UNIT_TYPE_ID: &str = "mpk.csharp.value.unit.v1";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PracticalVirImportPhase {
    Transport,
    Resource,
    Schema,
    GenericBarrier,
    Context,
    Foundation,
    ArtifactLinkage,
    Vocabulary,
    Graph,
    Dominance,
    Ownership,
    Exception,
    Binding,
    Hash,
}

impl PracticalVirImportPhase {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Transport => "transport",
            Self::Resource => "resource",
            Self::Schema => "schema",
            Self::GenericBarrier => "generic_barrier",
            Self::Context => "context",
            Self::Foundation => "foundation",
            Self::ArtifactLinkage => "artifact_linkage",
            Self::Vocabulary => "vocabulary",
            Self::Graph => "graph",
            Self::Dominance => "dominance",
            Self::Ownership => "ownership",
            Self::Exception => "exception",
            Self::Binding => "binding",
            Self::Hash => "hash",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PracticalVirImportErrorCode {
    Json,
    Canonical,
    Limit,
    Schema,
    Generic,
    Context,
    Foundation,
    Linkage,
    Identifier,
    Reference,
    Order,
    TypeGraph,
    CallGraph,
    Control,
    Dominance,
    Ownership,
    Exception,
    Binding,
    Operation,
    Hash,
}

impl PracticalVirImportErrorCode {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Json => "CSHARP_PRACTICAL_VIR_JSON",
            Self::Canonical => "CSHARP_PRACTICAL_VIR_CANONICAL",
            Self::Limit => "CSHARP_PRACTICAL_VIR_LIMIT",
            Self::Schema => "CSHARP_PRACTICAL_VIR_SCHEMA",
            Self::Generic => "CSHARP_PRACTICAL_VIR_GENERIC",
            Self::Context => "CSHARP_PRACTICAL_VIR_CONTEXT",
            Self::Foundation => "CSHARP_PRACTICAL_VIR_FOUNDATION",
            Self::Linkage => "CSHARP_PRACTICAL_VIR_LINKAGE",
            Self::Identifier => "CSHARP_PRACTICAL_VIR_IDENTIFIER",
            Self::Reference => "CSHARP_PRACTICAL_VIR_REFERENCE",
            Self::Order => "CSHARP_PRACTICAL_VIR_ORDER",
            Self::TypeGraph => "CSHARP_PRACTICAL_VIR_TYPE_GRAPH",
            Self::CallGraph => "CSHARP_PRACTICAL_VIR_CALL_GRAPH",
            Self::Control => "CSHARP_PRACTICAL_VIR_CONTROL",
            Self::Dominance => "CSHARP_PRACTICAL_VIR_DOMINANCE",
            Self::Ownership => "CSHARP_PRACTICAL_VIR_OWNERSHIP",
            Self::Exception => "CSHARP_PRACTICAL_VIR_EXCEPTION",
            Self::Binding => "CSHARP_PRACTICAL_VIR_BINDING",
            Self::Operation => "CSHARP_PRACTICAL_VIR_OPERATION",
            Self::Hash => "CSHARP_PRACTICAL_VIR_HASH",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PracticalVirImportError {
    phase: PracticalVirImportPhase,
    code: PracticalVirImportErrorCode,
}

impl PracticalVirImportError {
    pub const fn phase(&self) -> PracticalVirImportPhase {
        self.phase
    }

    pub const fn code(&self) -> PracticalVirImportErrorCode {
        self.code
    }
}

impl fmt::Display for PracticalVirImportError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{} at practical VIR phase {}",
            self.code.as_str(),
            self.phase.as_str()
        )
    }
}

impl Error for PracticalVirImportError {}

const fn failure(
    phase: PracticalVirImportPhase,
    code: PracticalVirImportErrorCode,
) -> PracticalVirImportError {
    PracticalVirImportError { phase, code }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PracticalVirArtifactLink {
    schema: String,
    sha256: String,
    canonical_bytes: u64,
}

impl PracticalVirArtifactLink {
    pub fn schema(&self) -> &str {
        &self.schema
    }

    pub fn sha256(&self) -> &str {
        &self.sha256
    }

    pub const fn canonical_bytes(&self) -> u64 {
        self.canonical_bytes
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
struct PracticalFoundationLink {
    schema: String,
    id: String,
    content_sha256: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
struct ExpandedFoundationEntry {
    instance_id: String,
    type_definition: Value,
    operation_definitions: Vec<Value>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PracticalVirPhiIncoming {
    pub predecessor_node_id: String,
    pub value_id: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PracticalVirPhiValue {
    pub value: TypedValueRef,
    pub incoming: Vec<PracticalVirPhiIncoming>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(tag = "tag", rename_all = "snake_case", deny_unknown_fields)]
pub enum PracticalConstructionAction {
    Allocate {
        construction_id: String,
        instance_id: String,
        owner_id: String,
        length: i64,
        default_eligible: bool,
        publication_length_maximum: u32,
    },
    Read {
        construction_id: String,
        actor_id: String,
        index: i32,
        result: TypedValueRef,
    },
    Fill {
        construction_id: String,
        actor_id: String,
        index: i32,
        value: TypedValueRef,
    },
    Rewrite {
        construction_id: String,
        actor_id: String,
        index: i32,
        value: TypedValueRef,
    },
    Borrow {
        construction_id: String,
        actor_id: String,
        borrower_id: String,
    },
    EndBorrow {
        construction_id: String,
        actor_id: String,
        borrower_id: String,
    },
    Transfer {
        construction_id: String,
        actor_id: String,
        new_owner_id: String,
    },
    Freeze {
        construction_id: String,
        actor_id: String,
        result: TypedValueRef,
    },
    Discard {
        construction_id: String,
        actor_id: String,
    },
}

impl PracticalConstructionAction {
    fn construction_id(&self) -> &str {
        match self {
            Self::Allocate {
                construction_id, ..
            }
            | Self::Read {
                construction_id, ..
            }
            | Self::Fill {
                construction_id, ..
            }
            | Self::Rewrite {
                construction_id, ..
            }
            | Self::Borrow {
                construction_id, ..
            }
            | Self::EndBorrow {
                construction_id, ..
            }
            | Self::Transfer {
                construction_id, ..
            }
            | Self::Freeze {
                construction_id, ..
            }
            | Self::Discard {
                construction_id, ..
            } => construction_id,
        }
    }

    fn defined_result(&self) -> Option<&TypedValueRef> {
        match self {
            Self::Read { result, .. } | Self::Freeze { result, .. } => Some(result),
            Self::Allocate { .. }
            | Self::Fill { .. }
            | Self::Rewrite { .. }
            | Self::Borrow { .. }
            | Self::EndBorrow { .. }
            | Self::Transfer { .. }
            | Self::Discard { .. } => None,
        }
    }

    fn used_value(&self) -> Option<&TypedValueRef> {
        match self {
            Self::Fill { value, .. } | Self::Rewrite { value, .. } => Some(value),
            Self::Allocate { .. }
            | Self::Read { .. }
            | Self::Borrow { .. }
            | Self::EndBorrow { .. }
            | Self::Transfer { .. }
            | Self::Freeze { .. }
            | Self::Discard { .. } => None,
        }
    }

    fn as_model_action(&self) -> Option<SequenceConstructionAction> {
        match self {
            Self::Allocate { .. } => None,
            Self::Read {
                actor_id,
                index,
                result,
                ..
            } => Some(SequenceConstructionAction::Read {
                actor_id: actor_id.clone(),
                index: *index,
                result_type_id: result.type_id.clone(),
            }),
            Self::Fill {
                actor_id,
                index,
                value,
                ..
            } => Some(SequenceConstructionAction::Fill {
                actor_id: actor_id.clone(),
                index: *index,
                value_type_id: value.type_id.clone(),
            }),
            Self::Rewrite {
                actor_id,
                index,
                value,
                ..
            } => Some(SequenceConstructionAction::Rewrite {
                actor_id: actor_id.clone(),
                index: *index,
                value_type_id: value.type_id.clone(),
            }),
            Self::Borrow {
                actor_id,
                borrower_id,
                ..
            } => Some(SequenceConstructionAction::Borrow {
                actor_id: actor_id.clone(),
                borrower_id: borrower_id.clone(),
            }),
            Self::EndBorrow {
                actor_id,
                borrower_id,
                ..
            } => Some(SequenceConstructionAction::EndBorrow {
                actor_id: actor_id.clone(),
                borrower_id: borrower_id.clone(),
            }),
            Self::Transfer {
                actor_id,
                new_owner_id,
                ..
            } => Some(SequenceConstructionAction::Transfer {
                actor_id: actor_id.clone(),
                new_owner_id: new_owner_id.clone(),
            }),
            Self::Freeze {
                actor_id, result, ..
            } => Some(SequenceConstructionAction::Freeze {
                actor_id: actor_id.clone(),
                result_type_id: result.type_id.clone(),
            }),
            Self::Discard { actor_id, .. } => Some(SequenceConstructionAction::Discard {
                actor_id: actor_id.clone(),
            }),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PracticalVirBlock {
    pub node: ControlNode,
    pub phi_values: Vec<PracticalVirPhiValue>,
    pub condition_value_id: Option<String>,
    pub return_value_ids: Vec<String>,
    pub abrupt_value_id: Option<String>,
    pub handler_exception_value: Option<TypedValueRef>,
    pub invocation: Option<OperationInvocation>,
    pub ownership_in: Vec<SequenceConstructionState>,
    pub construction_actions: Vec<PracticalConstructionAction>,
    pub ownership_out: Vec<SequenceConstructionState>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PracticalVirFunction {
    pub id: String,
    pub parameter_values: Vec<TypedValueRef>,
    pub result_type_ids: Vec<String>,
    pub blocks: Vec<PracticalVirBlock>,
    pub loops: Vec<LoopRegion>,
    pub patterns: Vec<PatternDecision>,
    pub exception_regions: Vec<ExceptionHandlerRegion>,
    pub unwind_plans: Vec<ExceptionUnwindPlan>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct PracticalVirContents {
    pub source_exceptions: Vec<SourceExceptionDefinition>,
    pub binding_projections: Vec<BindingTypeProjection>,
    pub binding_commutations: Vec<BindingOperationCommutation>,
    pub functions: Vec<PracticalVirFunction>,
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct WirePracticalVirModule {
    schema: String,
    semantic_context: Box<RawValue>,
    compilation_id: String,
    selection_sha256: String,
    source_snapshot_sha256: String,
    foundation_descriptor: PracticalFoundationLink,
    closed_instances: PracticalVirArtifactLink,
    semantic_bindings: PracticalVirArtifactLink,
    required_checks: PracticalVirArtifactLink,
    operations: PracticalVirArtifactLink,
    source_type_ids: Vec<String>,
    expanded_foundation: Vec<ExpandedFoundationEntry>,
    source_exceptions: Vec<SourceExceptionDefinition>,
    binding_projections: Vec<BindingTypeProjection>,
    binding_commutations: Vec<BindingOperationCommutation>,
    functions: Vec<PracticalVirFunction>,
    vir_sha256: String,
}

#[derive(Clone, Copy)]
pub struct PracticalVirImportContext<'a> {
    pub artifact_context: &'a PracticalArtifactContext,
    pub captured_inputs: &'a CapturedInputSet,
    pub foundation_descriptor_transport: &'a [u8],
    pub foundation_definitions_transport: &'a [u8],
    pub closed_roots_transport: &'a [u8],
    pub closed_instances_transport: &'a [u8],
    pub semantic_bindings_transport: &'a [u8],
    pub required_checks_transport: &'a [u8],
    pub operations_transport: &'a [u8],
}

pub struct ValidatedPracticalVir {
    wire: WirePracticalVirModule,
    canonical_bytes: Vec<u8>,
    artifact_ref: ArtifactRef,
    operation_signatures: Vec<ClosedOperationSignature>,
}

impl ValidatedPracticalVir {
    pub fn canonical_bytes(&self) -> &[u8] {
        &self.canonical_bytes
    }

    pub fn hash(&self) -> &str {
        &self.wire.vir_sha256
    }

    pub fn artifact_ref(&self) -> ArtifactRef {
        self.artifact_ref.clone()
    }

    pub fn functions(&self) -> &[PracticalVirFunction] {
        &self.wire.functions
    }

    pub fn source_exceptions(&self) -> &[SourceExceptionDefinition] {
        &self.wire.source_exceptions
    }

    pub fn binding_projections(&self) -> &[BindingTypeProjection] {
        &self.wire.binding_projections
    }

    pub fn binding_commutations(&self) -> &[BindingOperationCommutation] {
        &self.wire.binding_commutations
    }

    pub(crate) fn operation_signatures(&self) -> &[ClosedOperationSignature] {
        &self.operation_signatures
    }
}

struct PreparedInputs {
    foundation: ValidatedFoundationBundle,
    roots: ValidatedClosedRootSet,
    closed: ClosedInstanceSet,
    foundation_link: PracticalFoundationLink,
    closed_link: PracticalVirArtifactLink,
    semantic_bindings_link: PracticalVirArtifactLink,
    required_checks_link: PracticalVirArtifactLink,
    operations_link: PracticalVirArtifactLink,
    source_type_ids: Vec<String>,
    expanded_foundation: Vec<ExpandedFoundationEntry>,
    operations: BTreeMap<String, ClosedOperationSignature>,
    binding_expectations: Vec<BindingExpectation>,
}

#[derive(Clone, Debug)]
struct BindingExpectation {
    binding_id: String,
    projection_id: String,
    source_type_id: String,
    semantic_type_id: String,
    project_operation_id: String,
    reconstruct_operation_id: String,
    operations: Vec<(String, String)>,
}

/// Canonically encodes one candidate module and computes its `MPK-VIR-2.0`
/// hash.  This is an untrusted producer helper: callers must still pass the
/// result through [`import_csharp_practical_vir_json`].
pub fn canonical_csharp_practical_vir_transport(
    context: PracticalVirImportContext<'_>,
    contents: PracticalVirContents,
) -> Result<Vec<u8>, PracticalVirImportError> {
    let prepared = prepare_inputs(context)?;
    let semantic_context_bytes = canonical_practical_json_bytes(
        context.artifact_context.semantic_context(),
    )
    .map_err(|_| {
        failure(
            PracticalVirImportPhase::Context,
            PracticalVirImportErrorCode::Context,
        )
    })?;
    let semantic_context =
        RawValue::from_string(String::from_utf8(semantic_context_bytes).map_err(|_| {
            failure(
                PracticalVirImportPhase::Context,
                PracticalVirImportErrorCode::Context,
            )
        })?)
        .map_err(|_| {
            failure(
                PracticalVirImportPhase::Context,
                PracticalVirImportErrorCode::Context,
            )
        })?;
    let mut wire = WirePracticalVirModule {
        schema: SUCCESSOR_VIR_SCHEMA.to_owned(),
        semantic_context,
        compilation_id: context.artifact_context.compilation_id().to_owned(),
        selection_sha256: context.artifact_context.selection_sha256().to_owned(),
        source_snapshot_sha256: context.captured_inputs.snapshot_sha256().to_owned(),
        foundation_descriptor: prepared.foundation_link,
        closed_instances: prepared.closed_link,
        semantic_bindings: prepared.semantic_bindings_link,
        required_checks: prepared.required_checks_link,
        operations: prepared.operations_link,
        source_type_ids: prepared.source_type_ids,
        expanded_foundation: prepared.expanded_foundation,
        source_exceptions: contents.source_exceptions,
        binding_projections: contents.binding_projections,
        binding_commutations: contents.binding_commutations,
        functions: contents.functions,
        vir_sha256: ZERO_SHA256.to_owned(),
    };
    wire.vir_sha256 = hash_wire(&wire)?;
    encode_wire(&wire)
}

/// Strictly imports and completely validates one practical C# successor VIR.
pub fn import_csharp_practical_vir_json(
    input: &[u8],
    context: PracticalVirImportContext<'_>,
) -> Result<ValidatedPracticalVir, PracticalVirImportError> {
    scan_transport(input)?;
    let mut untyped_deserializer = serde_json::Deserializer::from_slice(input);
    untyped_deserializer.disable_recursion_limit();
    let untyped = Value::deserialize(&mut untyped_deserializer).map_err(|_| {
        failure(
            PracticalVirImportPhase::Transport,
            PracticalVirImportErrorCode::Json,
        )
    })?;
    untyped_deserializer.end().map_err(|_| {
        failure(
            PracticalVirImportPhase::Transport,
            PracticalVirImportErrorCode::Json,
        )
    })?;
    let schema = untyped
        .get("schema")
        .and_then(Value::as_str)
        .ok_or_else(|| {
            failure(
                PracticalVirImportPhase::Schema,
                PracticalVirImportErrorCode::Schema,
            )
        })?;
    if schema != SUCCESSOR_VIR_SCHEMA {
        return Err(failure(
            PracticalVirImportPhase::Schema,
            PracticalVirImportErrorCode::Schema,
        ));
    }
    validate_generic_free_value(&untyped)?;
    let wire = decode_wire(input)?;
    let reencoded = encode_wire(&wire)?;
    if reencoded != input {
        return Err(failure(
            PracticalVirImportPhase::Transport,
            PracticalVirImportErrorCode::Canonical,
        ));
    }
    validate_typed_resource_limits(&wire)?;

    let prepared = prepare_inputs(context)?;
    validate_root_linkage(&wire, context, &prepared)?;
    let universe = derive_closed_exception_universe(
        &prepared.roots,
        &prepared.closed,
        &wire.source_exceptions,
    )
    .map_err(|_| {
        failure(
            PracticalVirImportPhase::Exception,
            PracticalVirImportErrorCode::Exception,
        )
    })?;
    let mut used_operations = prepared
        .operations
        .values()
        .filter(|signature| signature.tag == ClosedOperationTag::Foundation)
        .map(|signature| signature.id.clone())
        .collect::<BTreeSet<_>>();
    validate_bindings(&wire, &prepared, &mut used_operations)?;
    validate_functions(&wire, context, &prepared, &universe, &mut used_operations)?;
    if used_operations != prepared.operations.keys().cloned().collect() {
        return Err(linkage_failure());
    }

    if !valid_sha256(&wire.vir_sha256) || hash_wire(&wire)? != wire.vir_sha256 {
        return Err(failure(
            PracticalVirImportPhase::Hash,
            PracticalVirImportErrorCode::Hash,
        ));
    }
    let artifact_ref = ArtifactRef::validated_successor(
        context.artifact_context,
        context.captured_inputs,
        SUCCESSOR_VIR_SCHEMA,
        &wire.vir_sha256,
        u64::try_from(input.len()).unwrap_or(u64::MAX),
    )
    .map_err(|_| {
        failure(
            PracticalVirImportPhase::ArtifactLinkage,
            PracticalVirImportErrorCode::Linkage,
        )
    })?;
    let operation_signatures = prepared.operations.into_values().collect();
    Ok(ValidatedPracticalVir {
        wire,
        canonical_bytes: input.to_vec(),
        artifact_ref,
        operation_signatures,
    })
}

fn decode_wire(input: &[u8]) -> Result<WirePracticalVirModule, PracticalVirImportError> {
    let mut deserializer = serde_json::Deserializer::from_slice(input);
    deserializer.disable_recursion_limit();
    let wire = WirePracticalVirModule::deserialize(&mut deserializer).map_err(|_| {
        failure(
            PracticalVirImportPhase::Transport,
            PracticalVirImportErrorCode::Json,
        )
    })?;
    deserializer.end().map_err(|_| {
        failure(
            PracticalVirImportPhase::Transport,
            PracticalVirImportErrorCode::Json,
        )
    })?;
    Ok(wire)
}

fn encode_wire(wire: &WirePracticalVirModule) -> Result<Vec<u8>, PracticalVirImportError> {
    let encoded = serde_json::to_vec(wire).map_err(|_| {
        failure(
            PracticalVirImportPhase::Transport,
            PracticalVirImportErrorCode::Canonical,
        )
    })?;
    if u64::try_from(encoded.len()).unwrap_or(u64::MAX) > CSHARP_PRACTICAL_VIR_CANONICAL_BYTES_MAX {
        return Err(failure(
            PracticalVirImportPhase::Resource,
            PracticalVirImportErrorCode::Limit,
        ));
    }
    Ok(encoded)
}

fn hash_wire(wire: &WirePracticalVirModule) -> Result<String, PracticalVirImportError> {
    #[derive(Serialize)]
    struct Preimage<'a> {
        schema: &'a str,
        semantic_context: &'a RawValue,
        compilation_id: &'a str,
        selection_sha256: &'a str,
        source_snapshot_sha256: &'a str,
        foundation_descriptor: &'a PracticalFoundationLink,
        closed_instances: &'a PracticalVirArtifactLink,
        semantic_bindings: &'a PracticalVirArtifactLink,
        required_checks: &'a PracticalVirArtifactLink,
        operations: &'a PracticalVirArtifactLink,
        source_type_ids: &'a [String],
        expanded_foundation: &'a [ExpandedFoundationEntry],
        source_exceptions: &'a [SourceExceptionDefinition],
        binding_projections: &'a [BindingTypeProjection],
        binding_commutations: &'a [BindingOperationCommutation],
        functions: &'a [PracticalVirFunction],
    }

    let preimage = Preimage {
        schema: &wire.schema,
        semantic_context: &wire.semantic_context,
        compilation_id: &wire.compilation_id,
        selection_sha256: &wire.selection_sha256,
        source_snapshot_sha256: &wire.source_snapshot_sha256,
        foundation_descriptor: &wire.foundation_descriptor,
        closed_instances: &wire.closed_instances,
        semantic_bindings: &wire.semantic_bindings,
        required_checks: &wire.required_checks,
        operations: &wire.operations,
        source_type_ids: &wire.source_type_ids,
        expanded_foundation: &wire.expanded_foundation,
        source_exceptions: &wire.source_exceptions,
        binding_projections: &wire.binding_projections,
        binding_commutations: &wire.binding_commutations,
        functions: &wire.functions,
    };
    let bytes = serde_json::to_vec(&preimage).map_err(|_| {
        failure(
            PracticalVirImportPhase::Hash,
            PracticalVirImportErrorCode::Hash,
        )
    })?;
    hash_domain_separated_raw(CSHARP_PRACTICAL_VIR_HASH_DOMAIN, &bytes)
        .map(|hash| hash.to_hex())
        .map_err(|_| {
            failure(
                PracticalVirImportPhase::Hash,
                PracticalVirImportErrorCode::Hash,
            )
        })
}

fn scan_transport(input: &[u8]) -> Result<(), PracticalVirImportError> {
    let limits = StrictJsonLimits::new(
        CSHARP_PRACTICAL_VIR_INPUT_BYTES_MAX,
        CSHARP_PRACTICAL_VIR_JSON_NODES_MAX,
        CSHARP_PRACTICAL_VIR_JSON_NESTING_MAX,
        CSHARP_PRACTICAL_VIR_STRING_BYTES_MAX,
    );
    let mut observer = PracticalVirResourceObserver::default();
    scan_strict_json(input, limits, &mut observer).map_err(|error| {
        let resource = matches!(
            error,
            StrictJsonError::InputBytesExceeded { .. }
                | StrictJsonError::NodeLimitExceeded { .. }
                | StrictJsonError::UnsupportedDepthLimit { .. }
                | StrictJsonError::DepthLimitExceeded { .. }
                | StrictJsonError::StringBytesExceeded { .. }
                | StrictJsonError::ObservedLimitExceeded { .. }
                | StrictJsonError::ObservedCounterOverflow { .. }
        );
        if resource {
            failure(
                PracticalVirImportPhase::Resource,
                PracticalVirImportErrorCode::Limit,
            )
        } else {
            failure(
                PracticalVirImportPhase::Transport,
                PracticalVirImportErrorCode::Json,
            )
        }
    })
}

#[derive(Default)]
struct PracticalVirResourceObserver {
    total_blocks: u64,
    total_operations: u64,
    operations_per_function: BTreeMap<u64, u64>,
    cfg_edges_per_function: BTreeMap<u64, u64>,
    pattern_arms_per_function: BTreeMap<u64, u64>,
    catch_finally_regions_per_function: BTreeMap<u64, u64>,
}

impl StrictJsonObserver for PracticalVirResourceObserver {
    fn observe(&mut self, event: StrictJsonEvent<'_>) -> Result<(), StrictJsonError> {
        self.observe_aggregate_limits(event)?;
        let StrictJsonEvent::ContainerEnd {
            path,
            kind: StrictJsonValueKind::Array,
            count,
        } = event
        else {
            return Ok(());
        };
        let limit = if path_is_root_key(path, "functions") {
            Some((
                "practical_vir_functions",
                CSHARP_PRACTICAL_VIR_FUNCTIONS_MAX,
            ))
        } else if path_is_root_key(path, "source_type_ids") {
            Some((
                "practical_vir_source_types",
                CSHARP_PRACTICAL_VIR_SOURCE_TYPES_MAX,
            ))
        } else if path_is_root_key(path, "expanded_foundation") {
            Some(("practical_vir_closed_instances", CLOSED_INSTANCE_COUNT_MAX))
        } else if path_is_root_key(path, "source_exceptions") {
            Some((
                "practical_vir_source_exceptions",
                CSHARP_PRACTICAL_VIR_SOURCE_EXCEPTIONS_MAX,
            ))
        } else if path_is_root_key(path, "binding_projections") {
            Some((
                "practical_vir_binding_projections",
                CSHARP_PRACTICAL_VIR_BINDING_PROJECTIONS_MAX,
            ))
        } else if path_is_root_key(path, "binding_commutations") {
            Some((
                "practical_vir_binding_commutations",
                CSHARP_PRACTICAL_VIR_BINDING_COMMUTATIONS_MAX,
            ))
        } else if path_is_function_key(path, "blocks") {
            Some((
                "practical_vir_blocks_per_function",
                CSHARP_PRACTICAL_VIR_BLOCKS_PER_FUNCTION_MAX,
            ))
        } else if path_is_function_key(path, "parameter_values") {
            Some(("practical_vir_parameters", VIR_PARAMS_MAX as u64))
        } else if path_is_function_key(path, "result_type_ids") {
            Some((
                "practical_vir_results",
                CSHARP_PRACTICAL_VIR_RESULTS_PER_FUNCTION_MAX,
            ))
        } else if path_is_function_key(path, "loops") {
            Some((
                "practical_vir_loops_per_function",
                CSHARP_PRACTICAL_VIR_LOOPS_PER_FUNCTION_MAX,
            ))
        } else if path_is_function_key(path, "exception_regions") {
            Some((
                "practical_vir_exception_regions_per_function",
                CSHARP_PRACTICAL_VIR_EXCEPTION_REGIONS_PER_FUNCTION_MAX,
            ))
        } else if path_is_pattern_key(path, "arms") {
            Some((
                "practical_vir_pattern_arms_per_decision",
                CSHARP_PRACTICAL_VIR_PATTERN_ARMS_PER_FUNCTION_MAX,
            ))
        } else if path_is_block_key(path, "ownership_in")
            || path_is_block_key(path, "ownership_out")
        {
            Some((
                "practical_vir_live_constructions",
                CSHARP_PRACTICAL_VIR_LIVE_CONSTRUCTIONS_MAX,
            ))
        } else if path_is_block_key(path, "construction_actions") {
            Some((
                "practical_vir_operations_per_function",
                CSHARP_PRACTICAL_VIR_OPERATIONS_PER_FUNCTION_MAX,
            ))
        } else if path_is_block_key(path, "phi_values") {
            Some((
                "practical_vir_phi_values_per_block",
                VIR_BLOCK_PARAMETERS_MAX as u64,
            ))
        } else if path_is_invocation_key(path, "operands") {
            Some(("practical_vir_call_arguments", VIR_CALL_ARGS_MAX as u64))
        } else {
            None
        };
        if let Some((name, maximum)) = limit {
            if count > maximum {
                return Err(StrictJsonError::ObservedLimitExceeded {
                    limit: name,
                    maximum,
                    actual: count,
                });
            }
        }
        Ok(())
    }
}

impl PracticalVirResourceObserver {
    fn observe_aggregate_limits(
        &mut self,
        event: StrictJsonEvent<'_>,
    ) -> Result<(), StrictJsonError> {
        match event {
            StrictJsonEvent::ContainerEnd {
                path,
                kind: StrictJsonValueKind::Array,
                count,
            } => {
                if function_blocks_index(path).is_some() {
                    add_observed_total(
                        &mut self.total_blocks,
                        count,
                        "practical_vir_blocks",
                        CSHARP_PRACTICAL_VIR_BLOCKS_MAX,
                    )?;
                }
                if let Some(function) = function_block_field_index(path, "construction_actions") {
                    add_observed_by_function(
                        &mut self.operations_per_function,
                        function,
                        count,
                        "practical_vir_operations_per_function",
                        CSHARP_PRACTICAL_VIR_OPERATIONS_PER_FUNCTION_MAX,
                    )?;
                    add_observed_total(
                        &mut self.total_operations,
                        count,
                        "practical_vir_operations",
                        CSHARP_PRACTICAL_VIR_OPERATIONS_MAX,
                    )?;
                }
                if let Some(function) = function_node_field_index(path, "normal_successor_ids")
                    .or_else(|| function_node_field_index(path, "exceptional_successors"))
                {
                    add_observed_by_function(
                        &mut self.cfg_edges_per_function,
                        function,
                        count,
                        "practical_vir_cfg_edges_per_function",
                        VIR_CFG_EDGES_PER_FUNCTION_MAX as u64,
                    )?;
                }
                if let Some(function) = function_pattern_field_index(path, "arms") {
                    add_observed_by_function(
                        &mut self.pattern_arms_per_function,
                        function,
                        count,
                        "practical_vir_pattern_arms_per_function",
                        CSHARP_PRACTICAL_VIR_PATTERN_ARMS_PER_FUNCTION_MAX,
                    )?;
                }
                if let Some(function) = function_exception_region_field_index(path, "catches") {
                    add_observed_by_function(
                        &mut self.catch_finally_regions_per_function,
                        function,
                        count,
                        "practical_vir_catch_finally_regions_per_function",
                        CSHARP_PRACTICAL_VIR_EXCEPTION_REGIONS_PER_FUNCTION_MAX,
                    )?;
                }
            }
            StrictJsonEvent::Value { path, kind } if kind != StrictJsonValueKind::Null => {
                if let Some(function) = function_block_value_index(path, "invocation") {
                    add_observed_by_function(
                        &mut self.operations_per_function,
                        function,
                        1,
                        "practical_vir_operations_per_function",
                        CSHARP_PRACTICAL_VIR_OPERATIONS_PER_FUNCTION_MAX,
                    )?;
                    add_observed_total(
                        &mut self.total_operations,
                        1,
                        "practical_vir_operations",
                        CSHARP_PRACTICAL_VIR_OPERATIONS_MAX,
                    )?;
                }
                if let Some(function) =
                    function_exception_region_value_index(path, "finally_entry_node_id")
                {
                    add_observed_by_function(
                        &mut self.catch_finally_regions_per_function,
                        function,
                        1,
                        "practical_vir_catch_finally_regions_per_function",
                        CSHARP_PRACTICAL_VIR_EXCEPTION_REGIONS_PER_FUNCTION_MAX,
                    )?;
                }
                if let Some(function) = function_abrupt_value_index(path, "target_id") {
                    add_observed_by_function(
                        &mut self.cfg_edges_per_function,
                        function,
                        1,
                        "practical_vir_cfg_edges_per_function",
                        VIR_CFG_EDGES_PER_FUNCTION_MAX as u64,
                    )?;
                }
            }
            _ => {}
        }
        Ok(())
    }
}

fn add_observed_total(
    counter: &mut u64,
    add: u64,
    limit: &'static str,
    maximum: u64,
) -> Result<(), StrictJsonError> {
    *counter = counter
        .checked_add(add)
        .ok_or(StrictJsonError::ObservedCounterOverflow { limit })?;
    if *counter > maximum {
        Err(StrictJsonError::ObservedLimitExceeded {
            limit,
            maximum,
            actual: *counter,
        })
    } else {
        Ok(())
    }
}

fn add_observed_by_function(
    counters: &mut BTreeMap<u64, u64>,
    function: u64,
    add: u64,
    limit: &'static str,
    maximum: u64,
) -> Result<(), StrictJsonError> {
    add_observed_total(counters.entry(function).or_default(), add, limit, maximum)
}

fn function_blocks_index(path: &[StrictJsonPathSegment]) -> Option<u64> {
    match path {
        [StrictJsonPathSegment::Key(functions), StrictJsonPathSegment::Index(function), StrictJsonPathSegment::Key(blocks)]
            if functions == "functions" && blocks == "blocks" =>
        {
            Some(*function)
        }
        _ => None,
    }
}

fn function_block_field_index(path: &[StrictJsonPathSegment], field: &str) -> Option<u64> {
    match path {
        [StrictJsonPathSegment::Key(functions), StrictJsonPathSegment::Index(function), StrictJsonPathSegment::Key(blocks), StrictJsonPathSegment::Index(_), StrictJsonPathSegment::Key(found)]
            if functions == "functions" && blocks == "blocks" && found == field =>
        {
            Some(*function)
        }
        _ => None,
    }
}

fn function_block_value_index(path: &[StrictJsonPathSegment], field: &str) -> Option<u64> {
    function_block_field_index(path, field)
}

fn function_node_field_index(path: &[StrictJsonPathSegment], field: &str) -> Option<u64> {
    match path {
        [StrictJsonPathSegment::Key(functions), StrictJsonPathSegment::Index(function), StrictJsonPathSegment::Key(blocks), StrictJsonPathSegment::Index(_), StrictJsonPathSegment::Key(node), StrictJsonPathSegment::Key(found)]
            if functions == "functions"
                && blocks == "blocks"
                && node == "node"
                && found == field =>
        {
            Some(*function)
        }
        _ => None,
    }
}

fn function_abrupt_value_index(path: &[StrictJsonPathSegment], field: &str) -> Option<u64> {
    match path {
        [StrictJsonPathSegment::Key(functions), StrictJsonPathSegment::Index(function), StrictJsonPathSegment::Key(blocks), StrictJsonPathSegment::Index(_), StrictJsonPathSegment::Key(node), StrictJsonPathSegment::Key(abrupt), StrictJsonPathSegment::Key(found)]
            if functions == "functions"
                && blocks == "blocks"
                && node == "node"
                && abrupt == "abrupt"
                && found == field =>
        {
            Some(*function)
        }
        _ => None,
    }
}

fn function_pattern_field_index(path: &[StrictJsonPathSegment], field: &str) -> Option<u64> {
    match path {
        [StrictJsonPathSegment::Key(functions), StrictJsonPathSegment::Index(function), StrictJsonPathSegment::Key(patterns), StrictJsonPathSegment::Index(_), StrictJsonPathSegment::Key(found)]
            if functions == "functions" && patterns == "patterns" && found == field =>
        {
            Some(*function)
        }
        _ => None,
    }
}

fn function_exception_region_field_index(
    path: &[StrictJsonPathSegment],
    field: &str,
) -> Option<u64> {
    match path {
        [StrictJsonPathSegment::Key(functions), StrictJsonPathSegment::Index(function), StrictJsonPathSegment::Key(regions), StrictJsonPathSegment::Index(_), StrictJsonPathSegment::Key(found)]
            if functions == "functions" && regions == "exception_regions" && found == field =>
        {
            Some(*function)
        }
        _ => None,
    }
}

fn function_exception_region_value_index(
    path: &[StrictJsonPathSegment],
    field: &str,
) -> Option<u64> {
    function_exception_region_field_index(path, field)
}

fn path_is_root_key(path: &[StrictJsonPathSegment], key: &str) -> bool {
    matches!(path, [StrictJsonPathSegment::Key(found)] if found == key)
}

fn path_is_function_key(path: &[StrictJsonPathSegment], key: &str) -> bool {
    matches!(
        path,
        [
            StrictJsonPathSegment::Key(functions),
            StrictJsonPathSegment::Index(_),
            StrictJsonPathSegment::Key(found),
        ] if functions == "functions" && found == key
    )
}

fn path_is_block_key(path: &[StrictJsonPathSegment], key: &str) -> bool {
    matches!(
        path,
        [
            StrictJsonPathSegment::Key(functions),
            StrictJsonPathSegment::Index(_),
            StrictJsonPathSegment::Key(blocks),
            StrictJsonPathSegment::Index(_),
            StrictJsonPathSegment::Key(found),
        ] if functions == "functions" && blocks == "blocks" && found == key
    )
}

fn path_is_pattern_key(path: &[StrictJsonPathSegment], key: &str) -> bool {
    matches!(
        path,
        [
            StrictJsonPathSegment::Key(functions),
            StrictJsonPathSegment::Index(_),
            StrictJsonPathSegment::Key(patterns),
            StrictJsonPathSegment::Index(_),
            StrictJsonPathSegment::Key(found),
        ] if functions == "functions" && patterns == "patterns" && found == key
    )
}

fn path_is_invocation_key(path: &[StrictJsonPathSegment], key: &str) -> bool {
    matches!(
        path,
        [
            StrictJsonPathSegment::Key(functions),
            StrictJsonPathSegment::Index(_),
            StrictJsonPathSegment::Key(blocks),
            StrictJsonPathSegment::Index(_),
            StrictJsonPathSegment::Key(invocation),
            StrictJsonPathSegment::Key(found),
        ] if functions == "functions"
            && blocks == "blocks"
            && invocation == "invocation"
            && found == key
    )
}

fn validate_typed_resource_limits(
    wire: &WirePracticalVirModule,
) -> Result<(), PracticalVirImportError> {
    limit_len(wire.functions.len(), CSHARP_PRACTICAL_VIR_FUNCTIONS_MAX)?;
    limit_len(
        wire.source_type_ids.len(),
        CSHARP_PRACTICAL_VIR_SOURCE_TYPES_MAX,
    )?;
    limit_len(
        wire.source_exceptions.len(),
        CSHARP_PRACTICAL_VIR_SOURCE_EXCEPTIONS_MAX,
    )?;
    limit_len(wire.expanded_foundation.len(), CLOSED_INSTANCE_COUNT_MAX)?;
    limit_len(
        wire.binding_projections.len(),
        CSHARP_PRACTICAL_VIR_BINDING_PROJECTIONS_MAX,
    )?;
    limit_len(
        wire.binding_commutations.len(),
        CSHARP_PRACTICAL_VIR_BINDING_COMMUTATIONS_MAX,
    )?;

    let mut total_blocks = 0_u64;
    let mut total_operations = 0_u64;
    for function in &wire.functions {
        limit_len(
            function.blocks.len(),
            CSHARP_PRACTICAL_VIR_BLOCKS_PER_FUNCTION_MAX,
        )?;
        limit_len(function.parameter_values.len(), VIR_PARAMS_MAX as u64)?;
        limit_len(
            function.result_type_ids.len(),
            CSHARP_PRACTICAL_VIR_RESULTS_PER_FUNCTION_MAX,
        )?;
        limit_len(
            function.loops.len(),
            CSHARP_PRACTICAL_VIR_LOOPS_PER_FUNCTION_MAX,
        )?;
        limit_len(
            function.exception_regions.len(),
            CSHARP_PRACTICAL_VIR_EXCEPTION_REGIONS_PER_FUNCTION_MAX,
        )?;
        let catch_finally_regions =
            function
                .exception_regions
                .iter()
                .try_fold(0_u64, |count, region| {
                    count
                        .checked_add(
                            u64::try_from(region.catches.len()).map_err(|_| resource_limit())?,
                        )
                        .and_then(|count| {
                            count.checked_add(u64::from(region.finally_entry_node_id.is_some()))
                        })
                        .ok_or_else(resource_limit)
                })?;
        if catch_finally_regions > CSHARP_PRACTICAL_VIR_EXCEPTION_REGIONS_PER_FUNCTION_MAX {
            return Err(resource_limit());
        }
        validate_loop_nesting_limit(&function.loops)?;
        let pattern_arms = function.patterns.iter().try_fold(0_u64, |count, pattern| {
            count
                .checked_add(u64::try_from(pattern.arms.len()).map_err(|_| resource_limit())?)
                .ok_or_else(resource_limit)
        })?;
        if pattern_arms > CSHARP_PRACTICAL_VIR_PATTERN_ARMS_PER_FUNCTION_MAX {
            return Err(resource_limit());
        }
        total_blocks = add_count(total_blocks, function.blocks.len())?;
        let mut function_operations = 0_u64;
        let mut function_edges = 0_u64;
        let mut construction_ids = BTreeSet::new();
        for block in &function.blocks {
            limit_len(block.phi_values.len(), VIR_BLOCK_PARAMETERS_MAX as u64)?;
            limit_len(
                block.ownership_in.len(),
                CSHARP_PRACTICAL_VIR_LIVE_CONSTRUCTIONS_MAX,
            )?;
            limit_len(
                block.ownership_out.len(),
                CSHARP_PRACTICAL_VIR_LIVE_CONSTRUCTIONS_MAX,
            )?;
            function_operations = add_count(
                function_operations,
                block
                    .construction_actions
                    .len()
                    .checked_add(usize::from(block.invocation.is_some()))
                    .ok_or_else(resource_limit)?,
            )?;
            function_edges = add_count(
                function_edges,
                block
                    .node
                    .normal_successor_ids
                    .len()
                    .checked_add(block.node.exceptional_successors.len())
                    .and_then(|count| {
                        count.checked_add(usize::from(matches!(
                            block.node.abrupt,
                            Some(AbruptCompletion::Break { .. })
                                | Some(AbruptCompletion::Continue { .. })
                        )))
                    })
                    .ok_or_else(resource_limit)?,
            )?;
            if let Some(invocation) = &block.invocation {
                limit_len(invocation.operands.len(), VIR_CALL_ARGS_MAX as u64)?;
            }
            for action in &block.construction_actions {
                construction_ids.insert(action.construction_id());
            }
        }
        if function_operations > CSHARP_PRACTICAL_VIR_OPERATIONS_PER_FUNCTION_MAX {
            return Err(resource_limit());
        }
        if function_edges > VIR_CFG_EDGES_PER_FUNCTION_MAX as u64 {
            return Err(resource_limit());
        }
        limit_len(
            construction_ids.len(),
            CSHARP_PRACTICAL_VIR_CONSTRUCTIONS_PER_FUNCTION_MAX,
        )?;
        total_operations = total_operations
            .checked_add(function_operations)
            .ok_or_else(resource_limit)?;
    }
    if total_blocks > CSHARP_PRACTICAL_VIR_BLOCKS_MAX
        || total_operations > CSHARP_PRACTICAL_VIR_OPERATIONS_MAX
    {
        return Err(resource_limit());
    }
    Ok(())
}

fn validate_loop_nesting_limit(loops: &[LoopRegion]) -> Result<(), PracticalVirImportError> {
    let parents = loops
        .iter()
        .map(|region| (region.id.as_str(), region.parent_loop_id.as_deref()))
        .collect::<BTreeMap<_, _>>();
    for region in loops {
        let mut seen = BTreeSet::new();
        let mut current = Some(region.id.as_str());
        let mut depth = 0_u64;
        while let Some(id) = current {
            if !seen.insert(id) {
                // Cycles are malformed graph structure and are classified later.
                break;
            }
            let Some(parent) = parents.get(id) else {
                // Unknown parents are likewise graph errors rather than limits.
                break;
            };
            depth = depth.checked_add(1).ok_or_else(resource_limit)?;
            if depth > CSHARP_PRACTICAL_VIR_LOOP_NESTING_MAX {
                return Err(resource_limit());
            }
            current = *parent;
        }
    }
    Ok(())
}

fn add_count(current: u64, add: usize) -> Result<u64, PracticalVirImportError> {
    current
        .checked_add(u64::try_from(add).map_err(|_| resource_limit())?)
        .ok_or_else(resource_limit)
}

fn limit_len(length: usize, maximum: u64) -> Result<(), PracticalVirImportError> {
    if u64::try_from(length).unwrap_or(u64::MAX) <= maximum {
        Ok(())
    } else {
        Err(resource_limit())
    }
}

const fn resource_limit() -> PracticalVirImportError {
    failure(
        PracticalVirImportPhase::Resource,
        PracticalVirImportErrorCode::Limit,
    )
}

fn validate_generic_free_value(value: &Value) -> Result<(), PracticalVirImportError> {
    match value {
        Value::String(text) => {
            let lower = text.to_ascii_lowercase();
            if text.contains(['<', '>', '`'])
                || text.contains("[[")
                || lower.contains("system.collections.generic")
                || lower.contains("generic_definition")
                || lower.contains("constructed_generic")
                || lower.contains("generic_call")
                || lower.contains("semantic_template")
                || text.starts_with("mpk.csharp.semantic.")
            {
                return Err(generic_failure());
            }
        }
        Value::Array(values) => {
            for value in values {
                validate_generic_free_value(value)?;
            }
        }
        Value::Object(entries) => {
            for (name, value) in entries {
                if matches!(
                    name.as_str(),
                    "template"
                        | "template_id"
                        | "type_parameter"
                        | "type_parameters"
                        | "type_argument"
                        | "type_arguments"
                        | "generic_definition"
                        | "constructed_generic"
                        | "generic_call"
                        | "semantic_template"
                ) || name.starts_with("generic_")
                {
                    return Err(generic_failure());
                }
                validate_generic_free_value(value)?;
            }
        }
        Value::Null | Value::Bool(_) | Value::Number(_) => {}
    }
    Ok(())
}

const fn generic_failure() -> PracticalVirImportError {
    failure(
        PracticalVirImportPhase::GenericBarrier,
        PracticalVirImportErrorCode::Generic,
    )
}

fn prepare_inputs(
    context: PracticalVirImportContext<'_>,
) -> Result<PreparedInputs, PracticalVirImportError> {
    if !context
        .captured_inputs
        .matches_context(context.artifact_context)
    {
        return Err(linkage_failure());
    }
    let foundation = validate_registered_foundation_bundle(
        context.foundation_descriptor_transport,
        context.foundation_definitions_transport,
    )
    .map_err(|_| foundation_failure())?;
    validate_foundation_context_linkage(&foundation, context.artifact_context.typed_context())
        .map_err(|_| foundation_failure())?;
    let roots = validate_closed_root_set(&foundation, context.closed_roots_transport)
        .map_err(|_| foundation_failure())?;
    let closed =
        validate_closed_instance_set(&foundation, &roots, context.closed_instances_transport)
            .map_err(|_| foundation_failure())?;
    let validated_closed_ref = bind_closed_instances(
        context.artifact_context,
        &foundation,
        context.captured_inputs,
        &roots,
        &closed,
    )
    .map_err(|_| foundation_failure())?;

    let semantic_bindings = validate_semantic_bindings_document(
        Some(context.artifact_context),
        Some(context.captured_inputs),
        context.semantic_bindings_transport,
    )
    .map_err(|_| linkage_failure())?;
    let closed_link = PracticalVirArtifactLink {
        schema: validated_closed_ref.schema().to_owned(),
        sha256: validated_closed_ref.sha256().to_owned(),
        canonical_bytes: validated_closed_ref.canonical_bytes(),
    };
    let semantic_bindings_link = PracticalVirArtifactLink {
        schema: SEMANTIC_BINDINGS_SCHEMA.to_owned(),
        sha256: semantic_bindings.hash().to_owned(),
        canonical_bytes: u64::try_from(context.semantic_bindings_transport.len())
            .map_err(|_| resource_limit())?,
    };
    let (required_checks_link, checks) =
        parse_required_checks(context, &closed_link, context.required_checks_transport)?;
    let (operations_link, operations) = parse_operations(
        context,
        &roots,
        &closed,
        &closed_link,
        &required_checks_link,
        &checks,
        context.operations_transport,
    )?;

    let roots_value: Value =
        serde_json::from_slice(roots.canonical_json()).map_err(|_| foundation_failure())?;
    let source_types = roots_value
        .get("source_types")
        .and_then(Value::as_object)
        .ok_or_else(foundation_failure)?;
    let source_type_ids = source_types.keys().cloned().collect::<Vec<_>>();
    limit_len(source_type_ids.len(), CSHARP_PRACTICAL_VIR_SOURCE_TYPES_MAX)?;
    validate_source_type_graph(source_types)?;

    let expanded_foundation = closed
        .entries()
        .iter()
        .map(|entry| {
            let instance_id = entry
                .get("instance_id")
                .and_then(Value::as_str)
                .ok_or_else(foundation_failure)?;
            let type_definition = entry
                .get("type_definition")
                .cloned()
                .ok_or_else(foundation_failure)?;
            let operation_definitions = entry
                .get("operation_definitions")
                .and_then(Value::as_array)
                .cloned()
                .ok_or_else(foundation_failure)?;
            Ok(ExpandedFoundationEntry {
                instance_id: instance_id.to_owned(),
                type_definition,
                operation_definitions,
            })
        })
        .collect::<Result<Vec<_>, PracticalVirImportError>>()?;

    let binding_expectations = parse_binding_expectations(
        semantic_bindings.value(),
        closed.entries(),
        source_types,
        roots_value.get("roots").ok_or_else(foundation_failure)?,
    )?;
    Ok(PreparedInputs {
        foundation_link: PracticalFoundationLink {
            schema: FOUNDATION_DESCRIPTOR_SCHEMA.to_owned(),
            id: FOUNDATION_DESCRIPTOR_ID.to_owned(),
            content_sha256: foundation.content_sha256().to_owned(),
        },
        foundation,
        roots,
        closed,
        closed_link,
        semantic_bindings_link,
        required_checks_link,
        operations_link,
        source_type_ids,
        expanded_foundation,
        operations,
        binding_expectations,
    })
}

fn validate_source_type_graph(
    source_types: &serde_json::Map<String, Value>,
) -> Result<(), PracticalVirImportError> {
    let mut edges = BTreeMap::<&str, BTreeSet<&str>>::new();
    for (id, source) in source_types {
        let members = source
            .get("members")
            .and_then(Value::as_array)
            .ok_or_else(type_graph_failure)?;
        let targets = edges.entry(id).or_default();
        for member in members {
            collect_source_type_edges(
                member.get("type").ok_or_else(type_graph_failure)?,
                source_types,
                targets,
            )?;
        }
    }
    let mut visiting = BTreeSet::new();
    let mut visited = BTreeSet::new();
    for id in source_types.keys() {
        visit_acyclic_type(id, &edges, &mut visiting, &mut visited)?;
    }
    Ok(())
}

fn collect_source_type_edges<'a>(
    ty: &'a Value,
    source_types: &'a serde_json::Map<String, Value>,
    targets: &mut BTreeSet<&'a str>,
) -> Result<(), PracticalVirImportError> {
    let kind = ty
        .get("kind")
        .and_then(Value::as_str)
        .ok_or_else(type_graph_failure)?;
    match kind {
        "primitive" => Ok(()),
        "source" => {
            let id = ty
                .get("id")
                .and_then(Value::as_str)
                .filter(|id| source_types.contains_key(*id))
                .ok_or_else(type_graph_failure)?;
            targets.insert(id);
            Ok(())
        }
        "instance" => {
            for argument in ty
                .get("arguments")
                .and_then(Value::as_array)
                .ok_or_else(type_graph_failure)?
            {
                collect_source_type_edges(argument, source_types, targets)?;
            }
            Ok(())
        }
        _ => Err(type_graph_failure()),
    }
}

fn visit_acyclic_type<'a>(
    id: &'a str,
    edges: &BTreeMap<&'a str, BTreeSet<&'a str>>,
    visiting: &mut BTreeSet<&'a str>,
    visited: &mut BTreeSet<&'a str>,
) -> Result<(), PracticalVirImportError> {
    if visited.contains(id) {
        return Ok(());
    }
    if !visiting.insert(id) {
        return Err(type_graph_failure());
    }
    if let Some(targets) = edges.get(id) {
        for target in targets {
            visit_acyclic_type(target, edges, visiting, visited)?;
        }
    }
    visiting.remove(id);
    visited.insert(id);
    Ok(())
}

const fn type_graph_failure() -> PracticalVirImportError {
    failure(
        PracticalVirImportPhase::Foundation,
        PracticalVirImportErrorCode::TypeGraph,
    )
}

fn parse_required_checks(
    context: PracticalVirImportContext<'_>,
    closed_link: &PracticalVirArtifactLink,
    transport: &[u8],
) -> Result<(PracticalVirArtifactLink, BTreeMap<String, RequiredCheck>), PracticalVirImportError> {
    const ROOT_FIELDS: &[&str] = &[
        "schema",
        "semantic_context",
        "compilation_id",
        "closed_instances",
        "checks",
        "required_checks_sha256",
    ];
    const CHECK_FIELDS: &[&str] = &["id", "tag", "failure_type_id"];
    let value = parse_canonical_practical_json(PracticalArtifactKind::RequiredChecks, transport)
        .map_err(|_| linkage_failure())?;
    require_fields(&value, ROOT_FIELDS)?;
    require_context_fields(&value, context)?;
    if string_field(&value, "schema")? != CSHARP_PRACTICAL_REQUIRED_CHECKS_SCHEMA
        || parse_practical_link(value.get("closed_instances"))? != *closed_link
    {
        return Err(linkage_failure());
    }
    let hash = validate_artifact_hash(
        &value,
        "required_checks_sha256",
        REQUIRED_CHECKS_HASH_DOMAIN,
    )?;
    let rows = value
        .get("checks")
        .and_then(PracticalJsonValue::as_array)
        .ok_or_else(linkage_failure)?;
    limit_len(rows.len(), EXPANDED_OPERATIONS_MAX)?;
    let mut checks = BTreeMap::new();
    let mut previous: Option<&str> = None;
    for row in rows {
        require_fields(row, CHECK_FIELDS)?;
        let id = string_field(row, "id")?;
        if !is_valid_vocabulary_id(id) || previous.is_some_and(|prior| prior >= id) {
            return Err(order_failure());
        }
        previous = Some(id);
        let tag = RequiredCheckTag::from_id(string_field(row, "tag")?).ok_or_else(|| {
            failure(
                PracticalVirImportPhase::Vocabulary,
                PracticalVirImportErrorCode::Operation,
            )
        })?;
        let failure_type_id = match row.get("failure_type_id") {
            Some(PracticalJsonValue::Null) => None,
            Some(value) => Some(
                value
                    .as_str()
                    .filter(|id| is_valid_vocabulary_id(id))
                    .ok_or_else(linkage_failure)?
                    .to_owned(),
            ),
            None => return Err(linkage_failure()),
        };
        checks.insert(
            id.to_owned(),
            RequiredCheck {
                id: id.to_owned(),
                tag,
                failure_type_id,
            },
        );
    }
    Ok((
        PracticalVirArtifactLink {
            schema: CSHARP_PRACTICAL_REQUIRED_CHECKS_SCHEMA.to_owned(),
            sha256: hash,
            canonical_bytes: u64::try_from(transport.len()).map_err(|_| resource_limit())?,
        },
        checks,
    ))
}

fn parse_operations(
    context: PracticalVirImportContext<'_>,
    roots: &ValidatedClosedRootSet,
    closed: &ClosedInstanceSet,
    closed_link: &PracticalVirArtifactLink,
    required_checks_link: &PracticalVirArtifactLink,
    checks: &BTreeMap<String, RequiredCheck>,
    transport: &[u8],
) -> Result<
    (
        PracticalVirArtifactLink,
        BTreeMap<String, ClosedOperationSignature>,
    ),
    PracticalVirImportError,
> {
    const ROOT_FIELDS: &[&str] = &[
        "schema",
        "semantic_context",
        "compilation_id",
        "closed_instances",
        "required_checks",
        "operations",
        "operations_sha256",
    ];
    const OPERATION_FIELDS: &[&str] = &[
        "id",
        "tag",
        "argument_type_ids",
        "normal_result_type_id",
        "ordered_check_ids",
    ];
    let value = parse_canonical_practical_json(PracticalArtifactKind::Operations, transport)
        .map_err(|_| linkage_failure())?;
    require_fields(&value, ROOT_FIELDS)?;
    require_context_fields(&value, context)?;
    if string_field(&value, "schema")? != CSHARP_PRACTICAL_OPERATIONS_SCHEMA
        || parse_practical_link(value.get("closed_instances"))? != *closed_link
        || parse_practical_link(value.get("required_checks"))? != *required_checks_link
    {
        return Err(linkage_failure());
    }
    let hash = validate_artifact_hash(&value, "operations_sha256", OPERATIONS_HASH_DOMAIN)?;
    let rows = value
        .get("operations")
        .and_then(PracticalJsonValue::as_array)
        .ok_or_else(linkage_failure)?;
    limit_len(rows.len(), EXPANDED_OPERATIONS_MAX)?;
    let mut operations = BTreeMap::new();
    let mut previous: Option<&str> = None;
    let mut used_checks = BTreeSet::new();
    for row in rows {
        require_fields(row, OPERATION_FIELDS)?;
        let id = string_field(row, "id")?;
        if !is_valid_vocabulary_id(id) || previous.is_some_and(|prior| prior >= id) {
            return Err(order_failure());
        }
        previous = Some(id);
        let tag = ClosedOperationTag::from_id(string_field(row, "tag")?).ok_or_else(|| {
            failure(
                PracticalVirImportPhase::Vocabulary,
                PracticalVirImportErrorCode::Operation,
            )
        })?;
        let argument_type_ids = practical_string_array(row.get("argument_type_ids"))?;
        limit_len(argument_type_ids.len(), VIR_CALL_ARGS_MAX as u64)?;
        let normal_result_type_id = string_field(row, "normal_result_type_id")?.to_owned();
        let ordered_check_ids = practical_string_array(row.get("ordered_check_ids"))?;
        let mut ordered_checks = Vec::with_capacity(ordered_check_ids.len());
        for check_id in ordered_check_ids {
            if !used_checks.insert(check_id.clone())
                && ordered_checks
                    .iter()
                    .any(|check: &RequiredCheck| check.id == check_id)
            {
                return Err(order_failure());
            }
            ordered_checks.push(checks.get(&check_id).cloned().ok_or_else(linkage_failure)?);
        }
        let signature = ClosedOperationSignature {
            id: id.to_owned(),
            tag,
            argument_type_ids,
            normal_result_type_id,
            ordered_checks,
        };
        validate_closed_operation_signature(roots, closed, &signature).map_err(|_| {
            failure(
                PracticalVirImportPhase::Vocabulary,
                PracticalVirImportErrorCode::Operation,
            )
        })?;
        operations.insert(id.to_owned(), signature);
    }
    if used_checks != checks.keys().cloned().collect::<BTreeSet<_>>() {
        return Err(linkage_failure());
    }
    let expected_foundation = closed
        .entries()
        .iter()
        .flat_map(|entry| {
            entry
                .get("operation_definitions")
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
        })
        .filter_map(|operation| operation.get("id").and_then(Value::as_str))
        .collect::<BTreeSet<_>>();
    let actual_foundation = operations
        .values()
        .filter(|operation| operation.tag == ClosedOperationTag::Foundation)
        .map(|operation| operation.id.as_str())
        .collect::<BTreeSet<_>>();
    if actual_foundation != expected_foundation {
        return Err(linkage_failure());
    }
    Ok((
        PracticalVirArtifactLink {
            schema: CSHARP_PRACTICAL_OPERATIONS_SCHEMA.to_owned(),
            sha256: hash,
            canonical_bytes: u64::try_from(transport.len()).map_err(|_| resource_limit())?,
        },
        operations,
    ))
}

fn require_context_fields(
    value: &PracticalJsonValue,
    context: PracticalVirImportContext<'_>,
) -> Result<(), PracticalVirImportError> {
    if value.get("semantic_context") != Some(context.artifact_context.semantic_context())
        || string_field(value, "compilation_id")? != context.artifact_context.compilation_id()
    {
        return Err(linkage_failure());
    }
    Ok(())
}

fn require_fields(
    value: &PracticalJsonValue,
    expected: &[&str],
) -> Result<(), PracticalVirImportError> {
    let entries = value.as_object().ok_or_else(linkage_failure)?;
    if entries.len() != expected.len()
        || entries
            .iter()
            .zip(expected)
            .any(|((actual, _), expected)| actual != expected)
    {
        return Err(linkage_failure());
    }
    Ok(())
}

fn string_field<'a>(
    value: &'a PracticalJsonValue,
    name: &str,
) -> Result<&'a str, PracticalVirImportError> {
    value
        .get(name)
        .and_then(PracticalJsonValue::as_str)
        .ok_or_else(linkage_failure)
}

fn practical_string_array(
    value: Option<&PracticalJsonValue>,
) -> Result<Vec<String>, PracticalVirImportError> {
    value
        .and_then(PracticalJsonValue::as_array)
        .ok_or_else(linkage_failure)?
        .iter()
        .map(|value| {
            value
                .as_str()
                .filter(|id| is_valid_vocabulary_id(id))
                .map(str::to_owned)
                .ok_or_else(linkage_failure)
        })
        .collect()
}

fn parse_practical_link(
    value: Option<&PracticalJsonValue>,
) -> Result<PracticalVirArtifactLink, PracticalVirImportError> {
    let value = value.ok_or_else(linkage_failure)?;
    require_fields(value, &["schema", "sha256", "canonical_bytes"])?;
    let schema = string_field(value, "schema")?;
    let sha256 = string_field(value, "sha256")?;
    let canonical_bytes = value
        .get("canonical_bytes")
        .and_then(PracticalJsonValue::as_u64)
        .filter(|size| *size > 0)
        .ok_or_else(linkage_failure)?;
    if !is_valid_vocabulary_id(schema) || !valid_sha256(sha256) {
        return Err(linkage_failure());
    }
    Ok(PracticalVirArtifactLink {
        schema: schema.to_owned(),
        sha256: sha256.to_owned(),
        canonical_bytes,
    })
}

fn validate_artifact_hash(
    value: &PracticalJsonValue,
    hash_field: &str,
    domain: HashDomain,
) -> Result<String, PracticalVirImportError> {
    let entries = value.as_object().ok_or_else(linkage_failure)?;
    let (name, actual) = entries.last().ok_or_else(linkage_failure)?;
    let actual = actual
        .as_str()
        .filter(|hash| valid_sha256(hash))
        .ok_or_else(linkage_failure)?;
    if name != hash_field {
        return Err(linkage_failure());
    }
    let preimage = PracticalJsonValue::Object(entries[..entries.len() - 1].to_vec());
    let bytes = canonical_practical_json_bytes(&preimage).map_err(|_| linkage_failure())?;
    let computed = hash_domain_separated_raw(domain, &bytes)
        .map_err(|_| linkage_failure())?
        .to_hex();
    if computed != actual {
        return Err(linkage_failure());
    }
    Ok(actual.to_owned())
}

fn parse_binding_expectations(
    bindings: &PracticalJsonValue,
    closed_entries: &[Value],
    source_types: &serde_json::Map<String, Value>,
    roots: &Value,
) -> Result<Vec<BindingExpectation>, PracticalVirImportError> {
    let rows = bindings
        .get("bindings")
        .and_then(PracticalJsonValue::as_array)
        .ok_or_else(linkage_failure)?;
    let reachable_source_types = reachable_source_type_ids(roots, source_types)?;
    let mut expected = Vec::with_capacity(rows.len());
    for row in rows {
        let hash = string_field(row, "binding_sha256")?;
        let source_type_id = string_field(row, "source_type_id")?;
        let role = string_field(row, "role")?;
        let arguments = practical_string_array(row.get("inferred_argument_ids"))?;
        validate_binding_source_shape(
            row,
            source_type_id,
            role,
            &arguments,
            closed_entries,
            source_types,
            &reachable_source_types,
        )?;
        let semantic_type_id = if role == "instant" {
            "mpk.csharp.value.instant.v1".to_owned()
        } else {
            let template_id = format!("mpk.csharp.semantic.{role}.v1");
            closed_entries
                .iter()
                .find(|entry| {
                    entry.get("template_id").and_then(Value::as_str) == Some(template_id.as_str())
                        && entry
                            .get("argument_ids")
                            .and_then(Value::as_array)
                            .is_some_and(|ids| {
                                ids.iter()
                                    .filter_map(Value::as_str)
                                    .eq(arguments.iter().map(String::as_str))
                            })
                })
                .and_then(|entry| entry.get("instance_id"))
                .and_then(Value::as_str)
                .ok_or_else(linkage_failure)?
                .to_owned()
        };
        let operation_map = row
            .get("operation_map")
            .and_then(PracticalJsonValue::as_object)
            .ok_or_else(linkage_failure)?;
        let operations = operation_map
            .iter()
            .map(|(name, source)| {
                source
                    .as_str()
                    .map(|source| (name.clone(), source.to_owned()))
                    .ok_or_else(linkage_failure)
            })
            .collect::<Result<Vec<_>, _>>()?;
        expected.push(BindingExpectation {
            binding_id: format!("binding.{hash}"),
            projection_id: format!("projection.{hash}"),
            source_type_id: source_type_id.to_owned(),
            semantic_type_id,
            project_operation_id: format!("binding.project.{hash}"),
            reconstruct_operation_id: format!("binding.reconstruct.{hash}"),
            operations,
        });
    }
    Ok(expected)
}

fn reachable_source_type_ids(
    roots: &Value,
    source_types: &serde_json::Map<String, Value>,
) -> Result<BTreeSet<String>, PracticalVirImportError> {
    let roots = roots.as_array().ok_or_else(binding_failure)?;
    let mut reachable = BTreeSet::new();
    for root in roots {
        collect_reachable_source_types(
            root.get("type").ok_or_else(binding_failure)?,
            source_types,
            &mut reachable,
        )?;
    }
    Ok(reachable)
}

fn collect_reachable_source_types(
    ty: &Value,
    source_types: &serde_json::Map<String, Value>,
    reachable: &mut BTreeSet<String>,
) -> Result<(), PracticalVirImportError> {
    match ty
        .get("kind")
        .and_then(Value::as_str)
        .ok_or_else(binding_failure)?
    {
        "primitive" => Ok(()),
        "instance" => {
            for argument in ty
                .get("arguments")
                .and_then(Value::as_array)
                .ok_or_else(binding_failure)?
            {
                collect_reachable_source_types(argument, source_types, reachable)?;
            }
            Ok(())
        }
        "source" => {
            let source_id = ty
                .get("id")
                .and_then(Value::as_str)
                .ok_or_else(binding_failure)?;
            let source = source_types.get(source_id).ok_or_else(binding_failure)?;
            if !reachable.insert(source_id.to_owned()) {
                return Ok(());
            }
            for member in source
                .get("members")
                .and_then(Value::as_array)
                .ok_or_else(binding_failure)?
            {
                collect_reachable_source_types(
                    member.get("type").ok_or_else(binding_failure)?,
                    source_types,
                    reachable,
                )?;
            }
            Ok(())
        }
        _ => Err(binding_failure()),
    }
}

fn validate_binding_source_shape(
    row: &PracticalJsonValue,
    source_type_id: &str,
    role: &str,
    supplied_arguments: &[String],
    closed_entries: &[Value],
    source_types: &serde_json::Map<String, Value>,
    reachable_source_types: &BTreeSet<String>,
) -> Result<(), PracticalVirImportError> {
    let source = source_types
        .get(source_type_id)
        .filter(|_| reachable_source_types.contains(source_type_id))
        .ok_or_else(binding_failure)?;
    if source.get("source_sha256").and_then(Value::as_str)
        != Some(string_field(row, "source_content_sha256")?)
        || source.get("identity_sensitive").and_then(Value::as_bool) != Some(false)
        || source.get("kind").and_then(Value::as_str) == Some("enum")
    {
        return Err(binding_failure());
    }
    let member_map = row
        .get("member_map")
        .and_then(PracticalJsonValue::as_object)
        .ok_or_else(binding_failure)?;
    let source_members = source
        .get("members")
        .and_then(Value::as_array)
        .ok_or_else(binding_failure)?;
    if member_map.iter().any(|(_, member_id)| {
        member_id.as_str().is_none_or(|member_id| {
            !source_members
                .iter()
                .any(|member| member.get("id").and_then(Value::as_str) == Some(member_id))
        })
    }) {
        return Err(binding_failure());
    }

    validate_binding_tag_carriers(row, source, source_types)?;
    let expected_arguments =
        derive_binding_arguments(row, source, role, closed_entries, source_types)?;
    if expected_arguments != supplied_arguments {
        return Err(binding_failure());
    }
    Ok(())
}

fn validate_binding_tag_carriers(
    row: &PracticalJsonValue,
    source: &Value,
    source_types: &serde_json::Map<String, Value>,
) -> Result<(), PracticalVirImportError> {
    let tag_arms = row
        .get("tag_arms")
        .and_then(PracticalJsonValue::as_object)
        .ok_or_else(binding_failure)?;
    if tag_arms.is_empty() {
        return Ok(());
    }
    let tag_member_id = binding_member_id(row, "tag")?;
    let tag_type = source_member_type(source, tag_member_id)?;
    let tag_source_id = tag_type
        .get("kind")
        .and_then(Value::as_str)
        .filter(|kind| *kind == "source")
        .and_then(|_| tag_type.get("id"))
        .and_then(Value::as_str)
        .ok_or_else(binding_failure)?;
    let enum_type = source_types
        .get(tag_source_id)
        .filter(|source| source.get("kind").and_then(Value::as_str) == Some("enum"))
        .ok_or_else(binding_failure)?;
    let enum_values = enum_type
        .get("enum_values")
        .and_then(Value::as_array)
        .ok_or_else(binding_failure)?
        .iter()
        .map(|value| value.as_str().ok_or_else(binding_failure))
        .collect::<Result<BTreeSet<_>, _>>()?;
    let arm_values = tag_arms
        .iter()
        .map(|(_, value)| value.as_str().ok_or_else(binding_failure))
        .collect::<Result<BTreeSet<_>, _>>()?;
    if enum_values != arm_values {
        return Err(binding_failure());
    }
    let default_arm = string_field(row, "default_arm")?;
    if default_arm != "ineligible" {
        let expected_default = tag_arms
            .iter()
            .find_map(|(arm, value)| (arm == default_arm).then_some(value))
            .and_then(PracticalJsonValue::as_str)
            .ok_or_else(binding_failure)?;
        if source
            .get("actual_default")
            .and_then(Value::as_object)
            .and_then(|defaults| defaults.get(tag_member_id))
            .and_then(Value::as_str)
            != Some(expected_default)
        {
            return Err(binding_failure());
        }
    }
    Ok(())
}

fn derive_binding_arguments(
    row: &PracticalJsonValue,
    source: &Value,
    role: &str,
    closed_entries: &[Value],
    source_types: &serde_json::Map<String, Value>,
) -> Result<Vec<String>, PracticalVirImportError> {
    let direct = |member_role: &str| {
        let member_id = binding_member_id(row, member_role)?;
        concrete_source_type_id(
            source_member_type(source, member_id)?,
            closed_entries,
            source_types,
        )
    };
    let sequence_argument = |member_role: &str| {
        let member_id = binding_member_id(row, member_role)?;
        let arguments = source_template_arguments(
            source_member_type(source, member_id)?,
            "bounded_sequence",
            1,
        )?;
        concrete_source_type_id(&arguments[0], closed_entries, source_types)
    };
    match role {
        "option" | "lookup" | "boundary_field" => Ok(vec![direct("value")?]),
        "result" => Ok(vec![direct("value")?, direct("error")?]),
        "validation" => Ok(vec![direct("value")?, sequence_argument("errors")?]),
        "transition" => Ok(vec![
            direct("state")?,
            sequence_argument("events")?,
            direct("response")?,
        ]),
        "instant" => {
            if direct("milliseconds")? != "mpk.csharp.value.i64.v1" {
                return Err(binding_failure());
            }
            Ok(Vec::new())
        }
        "money" => {
            if source.get("kind").and_then(Value::as_str) != Some("readonly_struct")
                || direct("amount")? != "mpk.csharp.value.decimal.v1"
            {
                return Err(binding_failure());
            }
            Ok(vec![direct("currency")?])
        }
        "bounded_sequence" | "ordered_set" => Ok(vec![sequence_argument("elements")?]),
        "ordered_entry" => Ok(vec![direct("key")?, direct("value")?]),
        "ordered_map" => {
            let entries_id = binding_member_id(row, "entries")?;
            let sequence = source_template_arguments(
                source_member_type(source, entries_id)?,
                "bounded_sequence",
                1,
            )?;
            let entry = source_template_arguments(&sequence[0], "ordered_entry", 2)?;
            Ok(entry
                .iter()
                .map(|argument| concrete_source_type_id(argument, closed_entries, source_types))
                .collect::<Result<Vec<_>, _>>()?)
        }
        _ => Err(binding_failure()),
    }
}

fn binding_member_id<'a>(
    row: &'a PracticalJsonValue,
    role: &str,
) -> Result<&'a str, PracticalVirImportError> {
    row.get("member_map")
        .and_then(PracticalJsonValue::as_object)
        .and_then(|members| {
            members
                .iter()
                .find_map(|(name, value)| (name == role).then_some(value))
        })
        .and_then(PracticalJsonValue::as_str)
        .ok_or_else(binding_failure)
}

fn source_member_type<'a>(
    source: &'a Value,
    member_id: &str,
) -> Result<&'a Value, PracticalVirImportError> {
    source
        .get("members")
        .and_then(Value::as_array)
        .and_then(|members| {
            members
                .iter()
                .find(|member| member.get("id").and_then(Value::as_str) == Some(member_id))
        })
        .and_then(|member| member.get("type"))
        .ok_or_else(binding_failure)
}

fn source_template_arguments<'a>(
    ty: &'a Value,
    template: &str,
    arity: usize,
) -> Result<&'a [Value], PracticalVirImportError> {
    if ty.get("kind").and_then(Value::as_str) != Some("instance")
        || ty.get("template").and_then(Value::as_str) != Some(template)
    {
        return Err(binding_failure());
    }
    ty.get("arguments")
        .and_then(Value::as_array)
        .filter(|arguments| arguments.len() == arity)
        .map(Vec::as_slice)
        .ok_or_else(binding_failure)
}

fn concrete_source_type_id(
    ty: &Value,
    closed_entries: &[Value],
    source_types: &serde_json::Map<String, Value>,
) -> Result<String, PracticalVirImportError> {
    match ty
        .get("kind")
        .and_then(Value::as_str)
        .ok_or_else(binding_failure)?
    {
        "primitive" => ty
            .get("id")
            .and_then(Value::as_str)
            .map(|id| format!("mpk.csharp.value.{id}.v1"))
            .ok_or_else(binding_failure),
        "source" => ty
            .get("id")
            .and_then(Value::as_str)
            .filter(|id| source_types.contains_key(*id))
            .map(str::to_owned)
            .ok_or_else(binding_failure),
        "instance" => {
            let template = ty
                .get("template")
                .and_then(Value::as_str)
                .ok_or_else(binding_failure)?;
            let argument_ids = ty
                .get("arguments")
                .and_then(Value::as_array)
                .ok_or_else(binding_failure)?
                .iter()
                .map(|argument| concrete_source_type_id(argument, closed_entries, source_types))
                .collect::<Result<Vec<_>, _>>()?;
            let template_id = format!("mpk.csharp.semantic.{template}.v1");
            closed_entries
                .iter()
                .find(|entry| {
                    entry.get("template_id").and_then(Value::as_str) == Some(&template_id)
                        && entry
                            .get("argument_ids")
                            .and_then(Value::as_array)
                            .is_some_and(|arguments| {
                                arguments
                                    .iter()
                                    .filter_map(Value::as_str)
                                    .eq(argument_ids.iter().map(String::as_str))
                            })
                })
                .and_then(|entry| entry.get("instance_id"))
                .and_then(Value::as_str)
                .map(str::to_owned)
                .ok_or_else(binding_failure)
        }
        _ => Err(binding_failure()),
    }
}

fn valid_sha256(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

const fn foundation_failure() -> PracticalVirImportError {
    failure(
        PracticalVirImportPhase::Foundation,
        PracticalVirImportErrorCode::Foundation,
    )
}

const fn linkage_failure() -> PracticalVirImportError {
    failure(
        PracticalVirImportPhase::ArtifactLinkage,
        PracticalVirImportErrorCode::Linkage,
    )
}

const fn order_failure() -> PracticalVirImportError {
    failure(
        PracticalVirImportPhase::Vocabulary,
        PracticalVirImportErrorCode::Order,
    )
}

fn validate_root_linkage(
    wire: &WirePracticalVirModule,
    context: PracticalVirImportContext<'_>,
    prepared: &PreparedInputs,
) -> Result<(), PracticalVirImportError> {
    if wire.schema != SUCCESSOR_VIR_SCHEMA
        || wire.compilation_id != context.artifact_context.compilation_id()
        || wire.selection_sha256 != context.artifact_context.selection_sha256()
        || wire.source_snapshot_sha256 != context.captured_inputs.snapshot_sha256()
        || wire.foundation_descriptor != prepared.foundation_link
        || wire.closed_instances != prepared.closed_link
        || wire.semantic_bindings != prepared.semantic_bindings_link
        || wire.required_checks != prepared.required_checks_link
        || wire.operations != prepared.operations_link
    {
        return Err(linkage_failure());
    }
    let expected_context_bytes =
        canonical_practical_json_bytes(context.artifact_context.semantic_context())
            .map_err(|_| linkage_failure())?;
    if wire.semantic_context.get().as_bytes() != expected_context_bytes {
        return Err(failure(
            PracticalVirImportPhase::Context,
            PracticalVirImportErrorCode::Context,
        ));
    }
    if wire.source_type_ids != prepared.source_type_ids
        || wire.expanded_foundation != prepared.expanded_foundation
    {
        return Err(foundation_failure());
    }
    if wire
        .source_type_ids
        .windows(2)
        .any(|pair| pair[0] >= pair[1])
        || wire
            .source_exceptions
            .windows(2)
            .any(|pair| pair[0].type_id >= pair[1].type_id)
    {
        return Err(order_failure());
    }
    if prepared.foundation.content_sha256() != wire.foundation_descriptor.content_sha256 {
        return Err(foundation_failure());
    }
    Ok(())
}

fn validate_bindings(
    wire: &WirePracticalVirModule,
    prepared: &PreparedInputs,
    used_operations: &mut BTreeSet<String>,
) -> Result<(), PracticalVirImportError> {
    if wire
        .binding_projections
        .windows(2)
        .any(|pair| pair[0].id >= pair[1].id)
        || wire.binding_commutations.windows(2).any(|pair| {
            (&pair[0].binding_id, &pair[0].source_operation.id)
                >= (&pair[1].binding_id, &pair[1].source_operation.id)
        })
    {
        return Err(failure(
            PracticalVirImportPhase::Binding,
            PracticalVirImportErrorCode::Order,
        ));
    }
    let by_projection_id = wire
        .binding_projections
        .iter()
        .map(|projection| (projection.id.as_str(), projection))
        .collect::<BTreeMap<_, _>>();
    if by_projection_id.len() != wire.binding_projections.len() {
        return Err(binding_failure());
    }
    for projection in &wire.binding_projections {
        require_registered_signature(&projection.project, &prepared.operations)?;
        require_registered_signature(&projection.reconstruct, &prepared.operations)?;
        used_operations.insert(projection.project.id.clone());
        used_operations.insert(projection.reconstruct.id.clone());
        validate_binding_projection_shape(projection, &prepared.roots, &prepared.closed)?;
    }

    let expected_by_binding = prepared
        .binding_expectations
        .iter()
        .map(|expected| (expected.binding_id.as_str(), expected))
        .collect::<BTreeMap<_, _>>();
    for expected in &prepared.binding_expectations {
        let projection = by_projection_id
            .get(expected.projection_id.as_str())
            .ok_or_else(binding_failure)?;
        if projection.binding_id != expected.binding_id
            || projection.source_type_id != expected.source_type_id
            || projection.semantic_type_id != expected.semantic_type_id
            || projection.project.id != expected.project_operation_id
            || projection.reconstruct.id != expected.reconstruct_operation_id
        {
            return Err(binding_failure());
        }
    }

    let expected_mappings = prepared
        .binding_expectations
        .iter()
        .flat_map(|binding| {
            binding.operations.iter().map(move |(name, source)| {
                (
                    binding.binding_id.clone(),
                    source.clone(),
                    format!("{}.{}", binding.semantic_type_id, name),
                )
            })
        })
        .collect::<BTreeSet<_>>();
    let mut actual_mappings = BTreeSet::new();
    let mut used_projection_ids = BTreeSet::new();
    for commutation in &wire.binding_commutations {
        require_registered_signature(&commutation.source_operation, &prepared.operations)?;
        require_registered_signature(&commutation.semantic_operation, &prepared.operations)?;
        used_operations.insert(commutation.source_operation.id.clone());
        used_operations.insert(commutation.semantic_operation.id.clone());
        let expectation = expected_by_binding
            .get(commutation.binding_id.as_str())
            .ok_or_else(binding_failure)?;
        let mapping = (
            commutation.binding_id.clone(),
            commutation.source_operation.id.clone(),
            commutation.semantic_operation.id.clone(),
        );
        if !expected_mappings.contains(&mapping) || !actual_mappings.insert(mapping) {
            return Err(binding_failure());
        }
        if commutation.binding_id != expectation.binding_id {
            return Err(binding_failure());
        }
        for projection_id in commutation
            .operand_projection_ids
            .iter()
            .chain(std::iter::once(&commutation.result_projection_id))
            .chain(
                commutation
                    .ordered_outcomes
                    .iter()
                    .filter_map(|outcome| outcome.failure_projection_id.as_ref()),
            )
        {
            used_projection_ids.insert(projection_id.as_str());
        }
        validate_binding_operation_commutation(
            &prepared.roots,
            &prepared.closed,
            &wire.binding_projections,
            commutation,
        )
        .map_err(|_| binding_failure())?;
    }
    if actual_mappings != expected_mappings {
        return Err(binding_failure());
    }
    for projection in &wire.binding_projections {
        let required = prepared
            .binding_expectations
            .iter()
            .any(|expected| expected.projection_id == projection.id);
        if !required {
            let identity_id = format!("projection.identity.{}", projection.source_type_id);
            let identity_project =
                format!("binding.project.identity.{}", projection.source_type_id);
            let identity_reconstruct =
                format!("binding.reconstruct.identity.{}", projection.source_type_id);
            if projection.id != identity_id
                || projection.binding_id != "binding.identity"
                || projection.source_type_id != projection.semantic_type_id
                || projection.project.id != identity_project
                || projection.reconstruct.id != identity_reconstruct
                || !used_projection_ids.contains(projection.id.as_str())
            {
                return Err(binding_failure());
            }
        }
    }
    Ok(())
}

fn validate_binding_projection_shape(
    projection: &BindingTypeProjection,
    roots: &ValidatedClosedRootSet,
    closed: &ClosedInstanceSet,
) -> Result<(), PracticalVirImportError> {
    if !is_valid_vocabulary_id(&projection.id)
        || !is_valid_vocabulary_id(&projection.binding_id)
        || !is_known_concrete_type(roots, closed, &projection.source_type_id)
        || !is_known_concrete_type(roots, closed, &projection.semantic_type_id)
        || projection.project.tag != ClosedOperationTag::BindingProject
        || projection.project.argument_type_ids != [projection.source_type_id.as_str()]
        || projection.project.normal_result_type_id != projection.semantic_type_id
        || projection.reconstruct.tag != ClosedOperationTag::BindingReconstruct
        || projection.reconstruct.argument_type_ids != [projection.semantic_type_id.as_str()]
        || projection.reconstruct.normal_result_type_id != projection.source_type_id
    {
        return Err(binding_failure());
    }
    validate_closed_operation_signature(roots, closed, &projection.project)
        .map_err(|_| binding_failure())?;
    validate_closed_operation_signature(roots, closed, &projection.reconstruct)
        .map_err(|_| binding_failure())
}

fn require_registered_signature(
    signature: &ClosedOperationSignature,
    operations: &BTreeMap<String, ClosedOperationSignature>,
) -> Result<(), PracticalVirImportError> {
    if operations.get(&signature.id) == Some(signature) {
        Ok(())
    } else {
        Err(failure(
            PracticalVirImportPhase::Vocabulary,
            PracticalVirImportErrorCode::Operation,
        ))
    }
}

const fn binding_failure() -> PracticalVirImportError {
    failure(
        PracticalVirImportPhase::Binding,
        PracticalVirImportErrorCode::Binding,
    )
}

fn validate_functions(
    wire: &WirePracticalVirModule,
    context: PracticalVirImportContext<'_>,
    prepared: &PreparedInputs,
    universe: &ClosedExceptionUniverse,
    used_operations: &mut BTreeSet<String>,
) -> Result<(), PracticalVirImportError> {
    if wire.functions.is_empty()
        || wire
            .functions
            .windows(2)
            .any(|pair| pair[0].id >= pair[1].id)
    {
        return Err(failure(
            PracticalVirImportPhase::Vocabulary,
            PracticalVirImportErrorCode::Order,
        ));
    }
    let function_ids = wire
        .functions
        .iter()
        .map(|function| function.id.as_str())
        .collect::<BTreeSet<_>>();
    if function_ids.len() != wire.functions.len()
        || function_ids
            .iter()
            .any(|id| !valid_source_declaration_id(id))
    {
        return Err(identifier_failure());
    }

    let mut global_node_ids = BTreeSet::new();
    let mut global_value_ids = BTreeSet::new();
    let mut calls = BTreeMap::<&str, BTreeSet<&str>>::new();
    for function in &wire.functions {
        let function_calls = validate_function(
            function,
            prepared,
            universe,
            &mut global_node_ids,
            &mut global_value_ids,
            used_operations,
        )?;
        if function_calls
            .iter()
            .any(|callee| !function_ids.contains(callee))
        {
            return Err(call_graph_failure());
        }
        calls.insert(function.id.as_str(), function_calls);
    }
    validate_acyclic_calls(&calls)?;

    let selected = context
        .artifact_context
        .selected_root_ids()
        .iter()
        .map(String::as_str)
        .collect::<BTreeSet<_>>();
    if selected.is_empty() || !selected.is_subset(&function_ids) {
        return Err(call_graph_failure());
    }
    let mut reachable = BTreeSet::new();
    let mut pending = selected.iter().copied().collect::<Vec<_>>();
    while let Some(function_id) = pending.pop() {
        if !reachable.insert(function_id) {
            continue;
        }
        pending.extend(
            calls
                .get(function_id)
                .into_iter()
                .flat_map(|callees| callees.iter().copied()),
        );
    }
    if reachable != function_ids {
        return Err(call_graph_failure());
    }
    Ok(())
}

#[derive(Clone, Copy)]
struct ValueDefinition<'a> {
    type_id: &'a str,
    node_id: Option<&'a str>,
    phi: bool,
    normal_successor_id: Option<&'a str>,
    construction_action_ordinal: Option<usize>,
}

fn validate_function<'a>(
    function: &'a PracticalVirFunction,
    prepared: &'a PreparedInputs,
    universe: &ClosedExceptionUniverse,
    global_node_ids: &mut BTreeSet<&'a str>,
    global_value_ids: &mut BTreeSet<&'a str>,
    used_operations: &mut BTreeSet<String>,
) -> Result<BTreeSet<&'a str>, PracticalVirImportError> {
    if !valid_source_declaration_id(&function.id)
        || function.blocks.is_empty()
        || function
            .blocks
            .iter()
            .enumerate()
            .any(|(ordinal, block)| usize::try_from(block.node.ordinal).ok() != Some(ordinal))
    {
        return Err(identifier_failure());
    }
    let function_signature = prepared
        .operations
        .get(&function.id)
        .filter(|signature| signature.tag == ClosedOperationTag::SourceCall)
        .ok_or_else(operation_failure)?;
    if function
        .parameter_values
        .iter()
        .map(|parameter| parameter.type_id.as_str())
        .ne(function_signature
            .argument_type_ids
            .iter()
            .map(String::as_str))
        || if function_signature.normal_result_type_id == UNIT_TYPE_ID {
            !function.result_type_ids.is_empty()
        } else {
            function.result_type_ids != [function_signature.normal_result_type_id.as_str()]
        }
    {
        return Err(operation_failure());
    }
    used_operations.insert(function_signature.id.clone());
    let graph = ExplicitControlGraph {
        nodes: function
            .blocks
            .iter()
            .map(|block| block.node.clone())
            .collect(),
        loops: function.loops.clone(),
        patterns: function.patterns.clone(),
        exception_regions: function.exception_regions.clone(),
        unwind_plans: function.unwind_plans.clone(),
    };
    validate_explicit_control_graph(&prepared.roots, &prepared.closed, universe, &graph)
        .map_err(map_graph_validation_error)?;

    let nodes = function
        .blocks
        .iter()
        .map(|block| (block.node.id.as_str(), block))
        .collect::<BTreeMap<_, _>>();
    if nodes.len() != function.blocks.len() || nodes.keys().any(|id| !global_node_ids.insert(*id)) {
        return Err(identifier_failure());
    }
    let predecessors = control_predecessors(function, &nodes)?;
    let reachable = validate_control_reachability(function, &nodes)?;
    validate_control_region_edges(function, &nodes)?;
    let dominators = compute_dominators(function, &predecessors, &reachable)?;

    let mut definitions = BTreeMap::<&str, ValueDefinition<'_>>::new();
    for parameter in &function.parameter_values {
        validate_typed_value(parameter, &prepared.roots, &prepared.closed)?;
        insert_value_definition(
            &mut definitions,
            global_value_ids,
            &parameter.id,
            ValueDefinition {
                type_id: &parameter.type_id,
                node_id: None,
                phi: false,
                normal_successor_id: None,
                construction_action_ordinal: None,
            },
        )?;
    }
    for result_type_id in &function.result_type_ids {
        require_concrete_type(&prepared.roots, &prepared.closed, result_type_id)?;
    }
    for block in &function.blocks {
        if block
            .phi_values
            .windows(2)
            .any(|pair| pair[0].value.id >= pair[1].value.id)
        {
            return Err(failure(
                PracticalVirImportPhase::Dominance,
                PracticalVirImportErrorCode::Order,
            ));
        }
        for phi in &block.phi_values {
            validate_typed_value(&phi.value, &prepared.roots, &prepared.closed)?;
            insert_value_definition(
                &mut definitions,
                global_value_ids,
                &phi.value.id,
                ValueDefinition {
                    type_id: &phi.value.type_id,
                    node_id: Some(&block.node.id),
                    phi: true,
                    normal_successor_id: None,
                    construction_action_ordinal: None,
                },
            )?;
        }
        if let Some(invocation) = &block.invocation {
            if predecessors
                .get(invocation.normal_successor_id.as_str())
                .is_none_or(|incoming| incoming.as_slice() != [block.node.id.as_str()])
            {
                return Err(dominance_failure());
            }
            validate_typed_value(&invocation.result, &prepared.roots, &prepared.closed)?;
            insert_value_definition(
                &mut definitions,
                global_value_ids,
                &invocation.result.id,
                ValueDefinition {
                    type_id: &invocation.result.type_id,
                    node_id: Some(&block.node.id),
                    phi: false,
                    normal_successor_id: Some(&invocation.normal_successor_id),
                    construction_action_ordinal: None,
                },
            )?;
        }
        if let Some(exception_value) = &block.handler_exception_value {
            validate_typed_value(exception_value, &prepared.roots, &prepared.closed)?;
            if exception_value.type_id != EXCEPTION_VALUE_TYPE_ID {
                return Err(failure(
                    PracticalVirImportPhase::Exception,
                    PracticalVirImportErrorCode::Exception,
                ));
            }
            insert_value_definition(
                &mut definitions,
                global_value_ids,
                &exception_value.id,
                ValueDefinition {
                    type_id: &exception_value.type_id,
                    node_id: Some(&block.node.id),
                    phi: true,
                    normal_successor_id: None,
                    construction_action_ordinal: None,
                },
            )?;
        }
        for (action_ordinal, action) in block.construction_actions.iter().enumerate() {
            let Some(result) = action.defined_result() else {
                continue;
            };
            let normal_successor_id = block
                .node
                .normal_successor_ids
                .first()
                .ok_or_else(control_failure)?;
            if predecessors
                .get(normal_successor_id.as_str())
                .is_none_or(|incoming| incoming.as_slice() != [block.node.id.as_str()])
            {
                return Err(dominance_failure());
            }
            validate_typed_value(result, &prepared.roots, &prepared.closed)?;
            insert_value_definition(
                &mut definitions,
                global_value_ids,
                &result.id,
                ValueDefinition {
                    type_id: &result.type_id,
                    node_id: Some(&block.node.id),
                    phi: false,
                    normal_successor_id: Some(normal_successor_id),
                    construction_action_ordinal: Some(action_ordinal),
                },
            )?;
        }
    }

    let mut calls = BTreeSet::new();
    for block in &function.blocks {
        validate_block_shape(block)?;
        validate_phi_values(block, &predecessors, &definitions, &dominators)?;
        validate_block_values(
            block,
            function,
            &definitions,
            &dominators,
            prepared,
            &mut calls,
            used_operations,
        )?;
    }
    validate_pattern_values(function, &definitions, &dominators, prepared)?;
    validate_ownership(function, &nodes, &predecessors, &prepared.closed)?;
    Ok(calls)
}

fn insert_value_definition<'a>(
    definitions: &mut BTreeMap<&'a str, ValueDefinition<'a>>,
    global_value_ids: &mut BTreeSet<&'a str>,
    id: &'a str,
    definition: ValueDefinition<'a>,
) -> Result<(), PracticalVirImportError> {
    if !is_valid_vocabulary_id(id)
        || definitions.insert(id, definition).is_some()
        || !global_value_ids.insert(id)
    {
        return Err(identifier_failure());
    }
    Ok(())
}

fn validate_typed_value(
    value: &TypedValueRef,
    roots: &ValidatedClosedRootSet,
    closed: &ClosedInstanceSet,
) -> Result<(), PracticalVirImportError> {
    if !is_valid_vocabulary_id(&value.id) {
        return Err(identifier_failure());
    }
    require_concrete_type(roots, closed, &value.type_id)
}

fn require_concrete_type(
    roots: &ValidatedClosedRootSet,
    closed: &ClosedInstanceSet,
    type_id: &str,
) -> Result<(), PracticalVirImportError> {
    if is_known_concrete_type(roots, closed, type_id) {
        Ok(())
    } else {
        Err(failure(
            PracticalVirImportPhase::Vocabulary,
            PracticalVirImportErrorCode::Reference,
        ))
    }
}

fn validate_block_shape(block: &PracticalVirBlock) -> Result<(), PracticalVirImportError> {
    let has_invocation = block.invocation.is_some();
    let has_construction = !block.construction_actions.is_empty();
    if has_invocation && has_construction {
        return Err(control_failure());
    }
    match block.node.tag {
        ControlNodeTag::Operation if has_invocation != has_construction => {}
        ControlNodeTag::Operation => return Err(control_failure()),
        _ if has_invocation || has_construction => return Err(control_failure()),
        _ => {}
    }
    let condition_required = matches!(
        block.node.tag,
        ControlNodeTag::Branch | ControlNodeTag::LoopHeader
    );
    if block.condition_value_id.is_some() != condition_required {
        return Err(control_failure());
    }
    if block.node.tag != ControlNodeTag::Return && !block.return_value_ids.is_empty() {
        return Err(control_failure());
    }
    if block.handler_exception_value.is_some() != (block.node.tag == ControlNodeTag::HandlerEntry) {
        return Err(control_failure());
    }
    let abrupt_value_required = matches!(
        block.node.tag,
        ControlNodeTag::Throw | ControlNodeTag::Rethrow
    );
    if block.abrupt_value_id.is_some() != abrupt_value_required {
        return Err(control_failure());
    }
    Ok(())
}

fn control_predecessors<'a>(
    function: &'a PracticalVirFunction,
    nodes: &BTreeMap<&'a str, &'a PracticalVirBlock>,
) -> Result<BTreeMap<&'a str, Vec<&'a str>>, PracticalVirImportError> {
    let mut predecessors = nodes
        .keys()
        .map(|id| (*id, Vec::new()))
        .collect::<BTreeMap<_, _>>();
    for block in &function.blocks {
        for target in control_targets(&block.node) {
            predecessors
                .get_mut(target)
                .ok_or_else(control_failure)?
                .push(block.node.id.as_str());
        }
    }
    for values in predecessors.values_mut() {
        values.sort_by_key(|id| nodes[id].node.ordinal);
        values.dedup();
    }
    Ok(predecessors)
}

fn control_targets(node: &ControlNode) -> Vec<&str> {
    let mut targets = node
        .normal_successor_ids
        .iter()
        .map(String::as_str)
        .chain(
            node.exceptional_successors
                .iter()
                .map(|edge| edge.target_id.as_str()),
        )
        .collect::<Vec<_>>();
    match node.abrupt.as_ref() {
        Some(AbruptCompletion::Break { target_id, .. })
        | Some(AbruptCompletion::Continue { target_id, .. }) => targets.push(target_id),
        _ => {}
    }
    targets
}

fn validate_control_reachability<'a>(
    function: &'a PracticalVirFunction,
    nodes: &BTreeMap<&'a str, &'a PracticalVirBlock>,
) -> Result<BTreeSet<&'a str>, PracticalVirImportError> {
    let entry = function
        .blocks
        .first()
        .filter(|block| block.node.tag == ControlNodeTag::Entry)
        .ok_or_else(control_failure)?;
    let mut reachable = BTreeSet::new();
    let mut pending = vec![entry.node.id.as_str()];
    while let Some(id) = pending.pop() {
        if !reachable.insert(id) {
            continue;
        }
        pending.extend(control_targets(&nodes[id].node));
    }
    if function.blocks.iter().any(|block| {
        block.node.tag != ControlNodeTag::Exit && !reachable.contains(block.node.id.as_str())
    }) {
        return Err(control_failure());
    }
    Ok(reachable)
}

fn validate_control_region_edges(
    function: &PracticalVirFunction,
    nodes: &BTreeMap<&str, &PracticalVirBlock>,
) -> Result<(), PracticalVirImportError> {
    let regions = function
        .exception_regions
        .iter()
        .map(|region| (region.id.as_str(), region))
        .collect::<BTreeMap<_, _>>();
    for block in &function.blocks {
        for target_id in &block.node.normal_successor_ids {
            let target = nodes.get(target_id.as_str()).ok_or_else(control_failure)?;
            let source_stack = &block.node.region_stack;
            let target_stack = &target.node.region_stack;
            if source_stack == target_stack {
                continue;
            }
            if target_stack.len() == source_stack.len() + 1
                && target_stack.starts_with(source_stack)
            {
                let entered = target_stack.last().expect("one region was added");
                if regions
                    .get(entered.as_str())
                    .is_some_and(|region| region.try_entry_node_id == target.node.id)
                {
                    continue;
                }
            }
            if source_stack.len() == target_stack.len() + 1
                && source_stack.starts_with(target_stack)
            {
                let exited = source_stack.last().expect("one region was removed");
                let Some(region) = regions.get(exited.as_str()) else {
                    return Err(control_failure());
                };
                if region.finally_entry_node_id.is_none()
                    || block.node.tag == ControlNodeTag::FinallyExit
                    || region.finally_entry_node_id.as_deref() == Some(target.node.id.as_str())
                {
                    continue;
                }
            }
            return Err(control_failure());
        }
    }
    Ok(())
}

fn compute_dominators<'a>(
    function: &'a PracticalVirFunction,
    predecessors: &BTreeMap<&'a str, Vec<&'a str>>,
    reachable: &BTreeSet<&'a str>,
) -> Result<BTreeMap<&'a str, BTreeSet<&'a str>>, PracticalVirImportError> {
    let entry = function
        .blocks
        .first()
        .map(|block| block.node.id.as_str())
        .ok_or_else(control_failure)?;
    let mut dominators = BTreeMap::new();
    for id in reachable {
        if *id == entry {
            dominators.insert(*id, [*id].into_iter().collect());
        } else {
            dominators.insert(*id, reachable.clone());
        }
    }
    loop {
        let mut changed = false;
        for id in reachable.iter().copied().filter(|id| *id != entry) {
            let incoming = predecessors
                .get(id)
                .ok_or_else(control_failure)?
                .iter()
                .copied()
                .filter(|predecessor| reachable.contains(predecessor))
                .collect::<Vec<_>>();
            if incoming.is_empty() {
                return Err(control_failure());
            }
            let mut next = dominators[&incoming[0]].clone();
            for predecessor in &incoming[1..] {
                next = next
                    .intersection(&dominators[predecessor])
                    .copied()
                    .collect();
            }
            next.insert(id);
            if next != dominators[id] {
                dominators.insert(id, next);
                changed = true;
            }
        }
        if !changed {
            break;
        }
    }
    Ok(dominators)
}

fn validate_phi_values<'a>(
    block: &'a PracticalVirBlock,
    predecessors: &BTreeMap<&'a str, Vec<&'a str>>,
    definitions: &BTreeMap<&'a str, ValueDefinition<'a>>,
    dominators: &BTreeMap<&'a str, BTreeSet<&'a str>>,
) -> Result<(), PracticalVirImportError> {
    let expected_predecessors = predecessors
        .get(block.node.id.as_str())
        .ok_or_else(dominance_failure)?;
    if block.node.tag == ControlNodeTag::Entry && !block.phi_values.is_empty() {
        return Err(dominance_failure());
    }
    for phi in &block.phi_values {
        if phi.incoming.len() != expected_predecessors.len()
            || phi
                .incoming
                .iter()
                .map(|incoming| incoming.predecessor_node_id.as_str())
                .ne(expected_predecessors.iter().copied())
        {
            return Err(dominance_failure());
        }
        for incoming in &phi.incoming {
            let definition = definitions
                .get(incoming.value_id.as_str())
                .ok_or_else(dominance_failure)?;
            if definition.type_id != phi.value.type_id
                || !value_available_on_edge(
                    *definition,
                    &incoming.predecessor_node_id,
                    &block.node.id,
                    dominators,
                )
            {
                return Err(dominance_failure());
            }
        }
    }
    Ok(())
}

fn validate_block_values<'a>(
    block: &'a PracticalVirBlock,
    function: &'a PracticalVirFunction,
    definitions: &BTreeMap<&'a str, ValueDefinition<'a>>,
    dominators: &BTreeMap<&'a str, BTreeSet<&'a str>>,
    prepared: &'a PreparedInputs,
    calls: &mut BTreeSet<&'a str>,
    used_operations: &mut BTreeSet<String>,
) -> Result<(), PracticalVirImportError> {
    if let Some(condition_id) = block.condition_value_id.as_deref() {
        require_value_use(
            condition_id,
            BOOL_TYPE_ID,
            &block.node.id,
            definitions,
            dominators,
        )?;
    }
    if block.node.tag == ControlNodeTag::Return {
        let abrupt_type = match block.node.abrupt.as_ref() {
            Some(AbruptCompletion::Return { value_type_id }) => value_type_id.as_deref(),
            _ => return Err(control_failure()),
        };
        if function.result_type_ids.len() > 1
            || abrupt_type != function.result_type_ids.first().map(String::as_str)
        {
            return Err(control_failure());
        }
        if block.return_value_ids.len() != function.result_type_ids.len() {
            return Err(dominance_failure());
        }
        for (value_id, type_id) in block.return_value_ids.iter().zip(&function.result_type_ids) {
            require_value_use(value_id, type_id, &block.node.id, definitions, dominators)?;
        }
    }
    if let Some(value_id) = block.abrupt_value_id.as_deref() {
        require_value_use(
            value_id,
            EXCEPTION_VALUE_TYPE_ID,
            &block.node.id,
            definitions,
            dominators,
        )?;
    }
    for (action_ordinal, action) in block.construction_actions.iter().enumerate() {
        if let Some(value) = action.used_value() {
            require_value_use_at_action(
                &value.id,
                &value.type_id,
                &block.node.id,
                action_ordinal,
                definitions,
                dominators,
            )?;
        }
    }
    if let Some(invocation) = &block.invocation {
        let signature = prepared
            .operations
            .get(&invocation.operation_id)
            .ok_or_else(operation_failure)?;
        if signature.tag == ClosedOperationTag::Foundation
            && is_construction_operation(&prepared.closed, &signature.id)
        {
            return Err(ownership_failure());
        }
        used_operations.insert(signature.id.clone());
        validate_operation_invocation(&prepared.roots, &prepared.closed, signature, invocation)
            .map_err(|_| operation_failure())?;
        if invocation.normal_successor_id
            != block
                .node
                .normal_successor_ids
                .first()
                .map(String::as_str)
                .unwrap_or_default()
            || invocation.exceptional_successors != block.node.exceptional_successors
        {
            return Err(operation_failure());
        }
        for operand in &invocation.operands {
            require_value_use(
                &operand.id,
                &operand.type_id,
                &block.node.id,
                definitions,
                dominators,
            )?;
        }
        if signature.tag == ClosedOperationTag::SourceCall {
            calls.insert(signature.id.as_str());
        }
    }
    Ok(())
}

fn is_construction_operation(closed: &ClosedInstanceSet, operation_id: &str) -> bool {
    closed.entries().iter().any(|entry| {
        entry.get("template_id").and_then(Value::as_str)
            == Some("mpk.csharp.semantic.sequence_construction.v1")
            && entry
                .get("operation_definitions")
                .and_then(Value::as_array)
                .is_some_and(|operations| {
                    operations.iter().any(|operation| {
                        operation.get("id").and_then(Value::as_str) == Some(operation_id)
                    })
                })
    })
}

fn validate_pattern_values<'a>(
    function: &'a PracticalVirFunction,
    definitions: &BTreeMap<&'a str, ValueDefinition<'a>>,
    dominators: &BTreeMap<&'a str, BTreeSet<&'a str>>,
    prepared: &PreparedInputs,
) -> Result<(), PracticalVirImportError> {
    for pattern in &function.patterns {
        require_concrete_type(
            &prepared.roots,
            &prepared.closed,
            &pattern.governing_type_id,
        )?;
        require_value_use(
            &pattern.governing_value_id,
            &pattern.governing_type_id,
            &pattern.node_id,
            definitions,
            dominators,
        )?;
        for arm in &pattern.arms {
            for type_id in &arm.bound_parameter_type_ids {
                require_concrete_type(&prepared.roots, &prepared.closed, type_id)?;
            }
        }
    }
    Ok(())
}

fn require_value_use<'a>(
    value_id: &str,
    expected_type_id: &str,
    use_node_id: &str,
    definitions: &BTreeMap<&'a str, ValueDefinition<'a>>,
    dominators: &BTreeMap<&'a str, BTreeSet<&'a str>>,
) -> Result<(), PracticalVirImportError> {
    let definition = definitions.get(value_id).ok_or_else(dominance_failure)?;
    if definition.type_id != expected_type_id
        || !value_available_at_start(*definition, use_node_id, dominators)
    {
        return Err(dominance_failure());
    }
    Ok(())
}

fn require_value_use_at_action<'a>(
    value_id: &str,
    expected_type_id: &str,
    use_node_id: &str,
    action_ordinal: usize,
    definitions: &BTreeMap<&'a str, ValueDefinition<'a>>,
    dominators: &BTreeMap<&'a str, BTreeSet<&'a str>>,
) -> Result<(), PracticalVirImportError> {
    let definition = definitions.get(value_id).ok_or_else(dominance_failure)?;
    if definition.type_id != expected_type_id {
        return Err(dominance_failure());
    }
    if definition.node_id == Some(use_node_id) {
        if definition
            .construction_action_ordinal
            .is_some_and(|definition_ordinal| definition_ordinal < action_ordinal)
        {
            return Ok(());
        }
        return Err(dominance_failure());
    }
    if value_available_at_start(*definition, use_node_id, dominators) {
        Ok(())
    } else {
        Err(dominance_failure())
    }
}

fn value_available_at_start(
    definition: ValueDefinition<'_>,
    use_node_id: &str,
    dominators: &BTreeMap<&str, BTreeSet<&str>>,
) -> bool {
    if let Some(normal_successor_id) = definition.normal_successor_id {
        return dominators
            .get(use_node_id)
            .is_some_and(|set| set.contains(normal_successor_id));
    }
    match definition.node_id {
        None => true,
        Some(definition_node) if definition_node == use_node_id => definition.phi,
        Some(definition_node) => dominators
            .get(use_node_id)
            .is_some_and(|set| set.contains(definition_node)),
    }
}

fn value_available_on_edge(
    definition: ValueDefinition<'_>,
    predecessor_node_id: &str,
    successor_node_id: &str,
    dominators: &BTreeMap<&str, BTreeSet<&str>>,
) -> bool {
    if let (Some(producer_node_id), Some(normal_successor_id)) =
        (definition.node_id, definition.normal_successor_id)
    {
        if producer_node_id == predecessor_node_id {
            return normal_successor_id == successor_node_id;
        }
        return dominators
            .get(predecessor_node_id)
            .is_some_and(|set| set.contains(normal_successor_id));
    }
    match definition.node_id {
        None => true,
        Some(definition_node) if definition_node == predecessor_node_id => true,
        Some(definition_node) => dominators
            .get(predecessor_node_id)
            .is_some_and(|set| set.contains(definition_node)),
    }
}

fn validate_ownership(
    function: &PracticalVirFunction,
    nodes: &BTreeMap<&str, &PracticalVirBlock>,
    predecessors: &BTreeMap<&str, Vec<&str>>,
    closed: &ClosedInstanceSet,
) -> Result<(), PracticalVirImportError> {
    for block in &function.blocks {
        validate_state_sequence(&block.ownership_in, closed)?;
        validate_state_sequence(&block.ownership_out, closed)?;
        let mut state = block
            .ownership_in
            .iter()
            .cloned()
            .map(|state| (state.construction_id.clone(), state))
            .collect::<BTreeMap<_, _>>();
        for action in &block.construction_actions {
            let construction_id = action.construction_id();
            match action {
                PracticalConstructionAction::Allocate {
                    instance_id,
                    owner_id,
                    length,
                    default_eligible,
                    publication_length_maximum,
                    ..
                } => {
                    if state.contains_key(construction_id) {
                        return Err(ownership_failure());
                    }
                    let allocated = SequenceConstructionState::allocate(
                        closed,
                        construction_id,
                        instance_id,
                        owner_id,
                        *length,
                        *default_eligible,
                        *publication_length_maximum,
                    )
                    .map_err(|_| ownership_failure())?;
                    state.insert(construction_id.to_owned(), allocated);
                }
                _ => {
                    let previous = state
                        .get(construction_id)
                        .cloned()
                        .ok_or_else(ownership_failure)?;
                    let model_action = action
                        .as_model_action()
                        .expect("non-allocation action has a model action");
                    let next = previous
                        .apply(closed, &model_action)
                        .map_err(|_| ownership_failure())?
                        .state;
                    state.insert(construction_id.to_owned(), next);
                }
            }
            limit_len(state.len(), CSHARP_PRACTICAL_VIR_LIVE_CONSTRUCTIONS_MAX)?;
        }
        let expected = state.into_values().collect::<Vec<_>>();
        if expected != block.ownership_out {
            return Err(ownership_failure());
        }
        if matches!(
            block.node.tag,
            ControlNodeTag::Return
                | ControlNodeTag::Throw
                | ControlNodeTag::Rethrow
                | ControlNodeTag::Exit
        ) && block
            .ownership_out
            .iter()
            .any(|state| state.status == ConstructionStatus::Active)
        {
            return Err(ownership_failure());
        }
    }
    let entry = function.blocks.first().ok_or_else(ownership_failure)?;
    if !entry.ownership_in.is_empty() {
        return Err(ownership_failure());
    }
    for block in &function.blocks {
        let incoming = predecessors
            .get(block.node.id.as_str())
            .ok_or_else(ownership_failure)?;
        let mut expected = match incoming.first() {
            Some(predecessor_id) => nodes
                .get(predecessor_id)
                .ok_or_else(ownership_failure)?
                .ownership_out
                .clone(),
            None => Vec::new(),
        };
        for predecessor_id in incoming.iter().skip(1) {
            let predecessor = nodes.get(predecessor_id).ok_or_else(ownership_failure)?;
            expected = merge_ownership_states(closed, &expected, &predecessor.ownership_out)?;
        }
        if expected != block.ownership_in {
            return Err(ownership_failure());
        }
    }
    Ok(())
}

fn merge_ownership_states(
    closed: &ClosedInstanceSet,
    left: &[SequenceConstructionState],
    right: &[SequenceConstructionState],
) -> Result<Vec<SequenceConstructionState>, PracticalVirImportError> {
    if left.len() != right.len()
        || left
            .iter()
            .zip(right)
            .any(|(left, right)| left.construction_id != right.construction_id)
    {
        return Err(ownership_failure());
    }
    left.iter()
        .zip(right)
        .map(|(left, right)| {
            SequenceConstructionState::merge(closed, left, right).map_err(|_| ownership_failure())
        })
        .collect()
}

fn validate_state_sequence(
    states: &[SequenceConstructionState],
    closed: &ClosedInstanceSet,
) -> Result<(), PracticalVirImportError> {
    if states
        .windows(2)
        .any(|pair| pair[0].construction_id >= pair[1].construction_id)
    {
        return Err(ownership_failure());
    }
    for state in states {
        if state.version > CSHARP_PRACTICAL_VIR_OPERATIONS_PER_FUNCTION_MAX
            || !is_valid_vocabulary_id(&state.construction_id)
        {
            return Err(ownership_failure());
        }
        state.validate(closed).map_err(|_| ownership_failure())?;
    }
    Ok(())
}

fn validate_acyclic_calls(
    calls: &BTreeMap<&str, BTreeSet<&str>>,
) -> Result<(), PracticalVirImportError> {
    fn visit<'a>(
        id: &'a str,
        calls: &BTreeMap<&'a str, BTreeSet<&'a str>>,
        visiting: &mut BTreeSet<&'a str>,
        visited: &mut BTreeSet<&'a str>,
    ) -> Result<(), PracticalVirImportError> {
        if visited.contains(id) {
            return Ok(());
        }
        if !visiting.insert(id) {
            return Err(call_graph_failure());
        }
        for callee in calls.get(id).into_iter().flat_map(|items| items.iter()) {
            visit(callee, calls, visiting, visited)?;
        }
        visiting.remove(id);
        visited.insert(id);
        Ok(())
    }

    let mut visiting = BTreeSet::new();
    let mut visited = BTreeSet::new();
    for id in calls.keys() {
        visit(id, calls, &mut visiting, &mut visited)?;
    }
    Ok(())
}

fn valid_source_declaration_id(value: &str) -> bool {
    value
        .strip_prefix("mpk.csharp.source.")
        .is_some_and(valid_sha256)
}

const fn identifier_failure() -> PracticalVirImportError {
    failure(
        PracticalVirImportPhase::Vocabulary,
        PracticalVirImportErrorCode::Identifier,
    )
}

const fn call_graph_failure() -> PracticalVirImportError {
    failure(
        PracticalVirImportPhase::Graph,
        PracticalVirImportErrorCode::CallGraph,
    )
}

const fn control_failure() -> PracticalVirImportError {
    failure(
        PracticalVirImportPhase::Graph,
        PracticalVirImportErrorCode::Control,
    )
}

fn map_graph_validation_error(error: PracticalVirValidationError) -> PracticalVirImportError {
    match error.phase() {
        PracticalVirValidationPhase::Exception => failure(
            PracticalVirImportPhase::Exception,
            PracticalVirImportErrorCode::Exception,
        ),
        PracticalVirValidationPhase::Construction => ownership_failure(),
        PracticalVirValidationPhase::Binding => binding_failure(),
        PracticalVirValidationPhase::Operation => operation_failure(),
        PracticalVirValidationPhase::Control | PracticalVirValidationPhase::Pattern => {
            control_failure()
        }
    }
}

const fn dominance_failure() -> PracticalVirImportError {
    failure(
        PracticalVirImportPhase::Dominance,
        PracticalVirImportErrorCode::Dominance,
    )
}

const fn ownership_failure() -> PracticalVirImportError {
    failure(
        PracticalVirImportPhase::Ownership,
        PracticalVirImportErrorCode::Ownership,
    )
}

const fn operation_failure() -> PracticalVirImportError {
    failure(
        PracticalVirImportPhase::Vocabulary,
        PracticalVirImportErrorCode::Operation,
    )
}
