//! Captured Roslyn data facts, retained independently from logical identity.
//! This is a private compiler handoff, never a proof or an import allowlist.
use super::*;
use crate::csharp_practical_source_artifacts::{CapturedInputSet, PracticalArtifactContext};

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct DataSourceWire {
    #[serde(default)]
    control_lowering: Option<Value>,
    compilation_id: String,
    input_files: Vec<DataSourceInput>,
    types: Vec<DataSourceType>,
    callables: Vec<DataSourceCallable>,
    source_containers: Vec<DataSourceContainer>,
    source_obligations: Vec<SourceDataObligation>,
    sources: Vec<DataSourceFile>,
    selected_root_ids: Vec<String>,
    reachable_declarations: Vec<String>,
    exact_types: Vec<DataSourceLocal>,
}
/// An owning source validator's pending condition, tied to original byte provenance.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SourceDataObligation {
    pub family: String,
    pub site: String,
    pub kind: String,
    pub type_id: String,
    pub members: Vec<String>,
    pub predicate: String,
    pub discharged: bool,
    pub declaration_id: String,
    pub source_path: String,
    pub source_sha256: String,
    pub start_byte: usize,
    pub end_byte: usize,
}
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct DataSourceInput {
    kind: String,
    path: String,
    raw_sha256: String,
    size_bytes: usize,
}
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct DataSourceFile {
    path: String,
    raw_sha256: String,
    size_bytes: usize,
}
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct DataSourceType {
    id: String,
    kind: String,
    name: String,
    namespace: String,
    source_path: String,
    source_sha256: String,
    start_byte: usize,
    end_byte: usize,
    enum_underlying: String,
    enum_values: Vec<String>,
    members: Vec<DataSourceMember>,
    recursive_default: Option<DataSourceDefaultGraph>,
    public_default: bool,
}
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct DataSourceMember {
    name: String,
    #[serde(rename = "type")]
    ty: Value,
    required: bool,
    storage: String,
    start_byte: usize,
    end_byte: usize,
}
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct DataSourceDefaultGraph {
    root: usize,
    nodes: Vec<DataSourceDefaultNode>,
}
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct DataSourceDefaultNode {
    type_id: String,
    kind: String,
    scalar: String,
    members: Vec<usize>,
}
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DataSourceCallable {
    identity: Value,
    is_static: bool,
    is_property_getter: bool,
    is_synthesized_constructor: bool,
    parameters: Vec<Value>,
    result: Value,
    id: String,
    source_path: String,
    source_sha256: String,
    start_byte: usize,
    end_byte: usize,
    body_sha256: String,
    body_utf8: String,
    data_steps: Vec<DataSourceStep>,
    initialization_plans: Vec<DataSourceInitializationPlan>,
}
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DataSourceInitializationPlan {
    pub node_ordinal: usize,
    pub site: String,
    pub type_id: String,
    pub constructor_id: String,
    pub member_order: Vec<String>,
    pub definitely_assigned: u32,
    pub possibly_assigned: u32,
    pub has_normal_exit: bool,
    pub steps: Vec<DataSourceInitializationStep>,
}
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DataSourceInitializationStep {
    pub kind: String,
    pub target: String,
    pub expression_ordinal: Option<usize>,
    pub exceptional_exit: String,
}
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DataSourceStep {
    pub(crate) node_ordinal: usize,
    pub(crate) family: String,
    pub(crate) operation: String,
    pub(crate) operand_ordinals: Vec<usize>,
    pub(crate) argument_ordinals: Vec<i32>,
    pub(crate) rounding: String,
}
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct DataSourceContainer {
    id: String,
    name: String,
    namespace: String,
    source_path: String,
    source_sha256: String,
    start_byte: usize,
    end_byte: usize,
}
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct DataSourceLocal {
    callable_id: String,
    local_ordinal: usize,
    #[serde(rename = "type")]
    ty: Value,
}
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DataSourceOperation {
    constant: Option<String>,
    child_count: usize,
    implicit: bool,
    kind: String,
    symbol: String,
    traits: String,
    #[serde(rename = "type")]
    ty: Option<String>,
}
#[derive(Clone, Debug)]
pub struct ValidatedDataSource {
    captured_facts: Vec<u8>,
    wire: DataSourceWire,
    bodies: BTreeMap<String, Vec<DataSourceOperation>>,
    constructor_assignments: BTreeMap<String, SourceConstructorAssignment>,
    source_types: Value,
    roots: ValidatedClosedRootSet,
    semantic_context: crate::csharp_practical_source_artifacts::PracticalJsonValue,
    selection_sha256: String,
    snapshot_sha256: String,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SourceConstructorAssignment {
    pub definitely_assigned: u32,
    pub possibly_assigned: u32,
}
impl DataSourceCallable {
    pub fn logical_signature(
        &self,
        bundle: &ValidatedFoundationBundle,
    ) -> Result<ClosedOperationSignature, DataPhaseError> {
        let mut arguments = Vec::new();
        if !self.is_static && self.identity["kind"] != "constructor" {
            arguments.push(
                self.identity["owner"]
                    .as_str()
                    .ok_or(DataPhaseError::Source)?
                    .to_owned(),
            );
        }
        for ty in &self.parameters {
            arguments.push(
                closed_type_id(
                    bundle,
                    &ClosedType::parse(ty).map_err(|_| DataPhaseError::Source)?,
                )
                .map_err(|_| DataPhaseError::Source)?,
            );
        }
        let result = closed_type_id(
            bundle,
            &ClosedType::parse(&self.result).map_err(|_| DataPhaseError::Source)?,
        )
        .map_err(|_| DataPhaseError::Source)?;
        Ok(ClosedOperationSignature {
            id: self.id.clone(),
            tag: ClosedOperationTag::SourceCall,
            argument_type_ids: arguments,
            normal_result_type_id: result,
            ordered_checks: vec![],
        })
    }
    pub fn initialization_plans(&self) -> &[DataSourceInitializationPlan] {
        &self.initialization_plans
    }
    pub fn id(&self) -> &str {
        &self.id
    }
    pub fn identity(&self) -> &Value {
        &self.identity
    }
    pub fn is_synthesized_constructor(&self) -> bool {
        self.is_synthesized_constructor
    }
    pub fn is_property_getter(&self) -> bool {
        self.is_property_getter
    }
    pub fn is_static(&self) -> bool {
        self.is_static
    }
    pub fn parameters(&self) -> &[Value] {
        &self.parameters
    }
    pub fn result(&self) -> &Value {
        &self.result
    }
    pub fn source_sha256(&self) -> &str {
        &self.source_sha256
    }
    pub(crate) fn data_steps(&self) -> &[DataSourceStep] {
        &self.data_steps
    }
    pub fn body_sha256(&self) -> &str {
        &self.body_sha256
    }
}
impl DataSourceOperation {
    pub fn kind(&self) -> &str {
        &self.kind
    }
    pub fn child_count(&self) -> usize {
        self.child_count
    }
    pub fn constant(&self) -> Option<&str> {
        self.constant.as_deref()
    }
    pub fn symbol(&self) -> &str {
        &self.symbol
    }
    pub fn traits(&self) -> &str {
        &self.traits
    }
    pub fn is_implicit(&self) -> bool {
        self.implicit
    }
    pub fn type_key(&self) -> Option<&str> {
        self.ty.as_deref()
    }
}
impl ValidatedDataSource {
    pub fn source_obligations(&self) -> &[SourceDataObligation] {
        &self.wire.source_obligations
    }
    pub fn captured_facts(&self) -> &[u8] {
        &self.captured_facts
    }
    pub(crate) fn has_structural_default(&self, id: &str) -> bool {
        self.wire
            .types
            .iter()
            .any(|t| t.id == id && t.recursive_default.is_some())
    }
    pub fn callables(&self) -> &[DataSourceCallable] {
        &self.wire.callables
    }
    pub fn body(&self, id: &str) -> Option<&[DataSourceOperation]> {
        self.bodies.get(id).map(Vec::as_slice)
    }
    pub fn control_lowering(&self) -> Option<&Value> {
        self.wire.control_lowering.as_ref()
    }
    pub fn source_roots(&self) -> &ValidatedClosedRootSet {
        &self.roots
    }
    pub fn source_types(&self) -> &Value {
        &self.source_types
    }
    pub fn selected_root_ids(&self) -> &[String] {
        &self.wire.selected_root_ids
    }

    pub fn import_captured_facts(
        b: &ValidatedFoundationBundle,
        context: &PracticalArtifactContext,
        captures: &CapturedInputSet,
        bytes: &[u8],
    ) -> Result<Self, DataPhaseError> {
        let fail = DataPhaseError::Source;
        // System.Text.Json's source-fact escaping is not canonical artifact JSON.
        // Scan duplicates and resources before serde constructs any maps.
        parse_strict_json(
            bytes,
            StrictJsonLimits::new(268_435_456, 4_000_000, 128, 16_777_216),
        )
        .map_err(|_| fail.clone())?;
        let wire: DataSourceWire = serde_json::from_slice(bytes).map_err(|_| fail.clone())?;
        let mut inputs = captures.entries().iter().collect::<Vec<_>>();
        inputs.sort_by_key(|e| e.path());
        if wire.compilation_id != context.compilation_id()
            || wire.input_files.len() != inputs.len()
            || wire.input_files.iter().zip(inputs).any(|(wire, actual)| {
                wire.path != actual.path()
                    || wire.raw_sha256 != actual.raw_sha256()
                    || wire.size_bytes != actual.bytes().len()
                    || wire.kind != match actual.kind() {
                        crate::csharp_practical_source_artifacts::OriginalInputKind::Source => {
                            "source"
                        }
                        crate::csharp_practical_source_artifacts::OriginalInputKind::Sidecar => {
                            "sidecar"
                        }
                    }
            })
        {
            return Err(fail);
        }
        if wire.types.len() > 128
            || wire.callables.len() > 128
            || wire.callables.is_empty()
            || wire.selected_root_ids != context.selected_root_ids()
            || wire
                .sources
                .iter()
                .map(|s| s.path.as_str())
                .ne(context.source_paths().iter().map(String::as_str))
            || wire.reachable_declarations.windows(2).any(|p| p[0] >= p[1])
        {
            return Err(fail);
        }
        for source in &wire.sources {
            let capture = captures.entry(&source.path).ok_or(fail.clone())?;
            if capture.raw_sha256() != source.raw_sha256
                || capture.bytes().len() != source.size_bytes
            {
                return Err(fail);
            }
        }
        let location =
            |path: &str, sha: &str, start: usize, end: usize| -> Result<(), DataPhaseError> {
                let capture = captures.entry(path).ok_or(DataPhaseError::Source)?;
                if !context.source_paths().iter().any(|p| p == path)
                    || capture.raw_sha256() != sha
                    || start >= end
                    || end > capture.bytes().len()
                {
                    return Err(DataPhaseError::Source);
                }
                let source =
                    std::str::from_utf8(capture.bytes()).map_err(|_| DataPhaseError::Source)?;
                if !source.is_char_boundary(start) || !source.is_char_boundary(end) {
                    return Err(DataPhaseError::Source);
                }
                Ok(())
            };
        let mut containers = BTreeMap::new();
        for container in &wire.source_containers {
            location(
                &container.source_path,
                &container.source_sha256,
                container.start_byte,
                container.end_byte,
            )?;
            let identity = json!({"kind":"type","name":container.name,"namespace":container.namespace,"owner":"","parameter_type_ids":[],"result_type_id":""});
            if csharp_practical_declaration_id(&identity).map_err(|_| fail.clone())? != container.id
                || !wire.reachable_declarations.contains(&container.id)
                || containers.insert(container.id.clone(), container).is_some()
                || wire.types.iter().any(|t| t.id == container.id)
            {
                return Err(fail);
            }
        }
        let owner_ids = wire
            .callables
            .iter()
            .filter_map(|c| c.identity["owner"].as_str())
            .filter(|id| !wire.types.iter().any(|t| t.id == *id))
            .collect::<BTreeSet<_>>();
        if containers
            .keys()
            .map(String::as_str)
            .collect::<BTreeSet<_>>()
            != owner_ids
            || wire.source_containers.len() > 128
            || wire
                .source_containers
                .windows(2)
                .any(|p| p[0].id >= p[1].id)
        {
            return Err(fail);
        }
        for callable in &wire.callables {
            if let Some(owner) = callable.identity["owner"]
                .as_str()
                .and_then(|id| containers.get(id))
            {
                if !callable.is_static
                    || callable.source_path != owner.source_path
                    || callable.start_byte < owner.start_byte
                    || callable.end_byte > owner.end_byte
                    || callable.identity["namespace"] != owner.namespace
                {
                    return Err(fail);
                }
            }
        }
        let mut source_types = Map::new();
        let mut root_values = BTreeSet::<String>::new();
        fn add_root(
            value: &Value,
            provenance: &str,
            roots: &mut BTreeSet<String>,
        ) -> Result<(), DataPhaseError> {
            let ty = ClosedType::parse(value).map_err(|_| DataPhaseError::Source)?;
            if let ClosedType::Instance { template, .. } = &ty {
                let origin = match template.as_str() {
                    "option" => "source_nullable",
                    "bounded_sequence" => "source_array",
                    _ => return Err(DataPhaseError::Source),
                };
                roots.insert(
                    serde_json::to_string(
                        &json!({"origin":origin,"provenance_id":provenance,"type":value}),
                    )
                    .map_err(|_| DataPhaseError::Source)?,
                );
            }
            Ok(())
        }
        for ty in &wire.types {
            location(
                &ty.source_path,
                &ty.source_sha256,
                ty.start_byte,
                ty.end_byte,
            )?;
            let identity = json!({"kind":"type","name":ty.name,"namespace":ty.namespace,"owner":"","parameter_type_ids":[],"result_type_id":""});
            if csharp_practical_declaration_id(&identity).map_err(|_| fail.clone())? != ty.id
                || !wire.reachable_declarations.contains(&ty.id)
                || source_types.contains_key(&ty.id)
            {
                return Err(fail);
            }
            let mut members = vec![];
            let mut defaults = Map::new();
            let mut default_cells = 0;
            if let Some(graph) = &ty.recursive_default {
                validate_default_graph(graph)?;
            }
            if ty.public_default && ty.recursive_default.is_none() {
                return Err(DataPhaseError::Default);
            }
            for (ordinal, member) in ty.members.iter().enumerate() {
                location(
                    &ty.source_path,
                    &ty.source_sha256,
                    member.start_byte,
                    member.end_byte,
                )?;
                if member.start_byte < ty.start_byte || member.end_byte > ty.end_byte {
                    return Err(fail);
                }
                let id = csharp_practical_stored_member_id(
                    &ty.id,
                    &member.name,
                    &member.ty,
                    &member.storage,
                )
                .map_err(|_| fail.clone())?;
                add_root(&member.ty, &id, &mut root_values)?;
                members.push(json!({"id":id,"name":member.name,"type":member.ty,"storage":member.storage,"ordinal":ordinal,"required":member.required}));
                // CLR zero exists even when public/default construction is
                // ineligible. Derive its complete metadata from actual member
                // types; never publish a placeholder or a validity assertion.
                let default = raw_clr_default(
                    &member.ty,
                    &wire.types,
                    &mut BTreeSet::new(),
                    &mut default_cells,
                )?;
                defaults.insert(id, default);
            }
            let carrier = if ty.enum_underlying.is_empty() {
                Value::Null
            } else {
                json!(ty
                    .enum_underlying
                    .strip_prefix("mpk.csharp.value.")
                    .and_then(|s| s.strip_suffix(".v1"))
                    .ok_or(fail.clone())?)
            };
            source_types.insert(ty.id.clone(), json!({"id":ty.id,"identity":identity,"kind":ty.kind,"members":members,
                "enum_values":ty.enum_values,"enum_underlying":carrier,"actual_default":defaults,
                "public_default":ty.public_default,"identity_sensitive":false,"source_sha256":ty.source_sha256}));
            if ty.members.iter().any(|m| m.storage == "init_auto") {
                let member_types = ty
                    .members
                    .iter()
                    .map(|m| ClosedType::parse(&m.ty).map_err(|_| fail.clone()))
                    .collect::<Result<Vec<_>, _>>()?;
                let carrier = object_construction::object_construction_type(&member_types);
                root_values.insert(serde_json::to_string(&json!({"origin":"source_construction",
                    "provenance_id":format!("{}.object_construction",ty.id),"type":carrier.to_value()})).map_err(|_|fail.clone())?);
            }
        }
        let mut bodies = BTreeMap::new();
        let mut total_nodes = 0usize;
        for callable in &wire.callables {
            location(
                &callable.source_path,
                &callable.source_sha256,
                callable.start_byte,
                callable.end_byte,
            )?;
            if csharp_practical_declaration_id(&callable.identity).map_err(|_| fail.clone())?
                != callable.id
                || !wire.reachable_declarations.contains(&callable.id)
                || bodies.contains_key(&callable.id)
                || sha256_raw_file_bytes(callable.body_utf8.as_bytes()).to_hex()
                    != callable.body_sha256
            {
                return Err(fail);
            }
            let argument_ids = callable
                .parameters
                .iter()
                .map(|p| {
                    let ty = ClosedType::parse(p).map_err(|_| DataPhaseError::Source)?;
                    closed_type_id(b, &clr_type(&ty, &source_types)?)
                        .map_err(|_| DataPhaseError::Source)
                })
                .collect::<Result<Vec<_>, _>>()?;
            let result = ClosedType::parse(&callable.result).map_err(|_| fail.clone())?;
            if callable.identity["kind"] == "constructor" && callable.is_static {
                return Err(fail);
            }
            if callable.is_synthesized_constructor {
                let owner = wire
                    .types
                    .iter()
                    .find(|t| callable.identity["owner"] == t.id)
                    .ok_or(fail.clone())?;
                if callable.identity["kind"] != "constructor"
                    || callable.is_static
                    || callable.is_property_getter
                    || !callable.parameters.is_empty()
                    || callable.body_utf8 != "[]"
                    || !callable.data_steps.is_empty()
                    || !callable.initialization_plans.is_empty()
                    || callable.source_path != owner.source_path
                    || callable.start_byte != owner.start_byte
                    || callable.end_byte != owner.end_byte
                    || owner.kind == "sealed_class"
                        && wire.callables.iter().any(|c| {
                            c.id != callable.id
                                && c.identity["kind"] == "constructor"
                                && c.identity["owner"] == owner.id
                        })
                {
                    return Err(fail);
                }
            } else if callable.identity["kind"] == "constructor"
                && wire.types.iter().any(|t| {
                    callable.identity["owner"] == t.id
                        && callable.source_path == t.source_path
                        && callable.start_byte == t.start_byte
                        && callable.end_byte == t.end_byte
                })
            {
                return Err(fail);
            }
            if callable.is_property_getter
                && (callable.is_static
                    || callable.identity["kind"] != "method"
                    || !callable.identity["name"]
                        .as_str()
                        .is_some_and(|n| n.starts_with("get_"))
                    || !callable.parameters.is_empty())
            {
                return Err(fail);
            }
            if callable.identity["parameter_type_ids"] != json!(argument_ids)
                || callable.identity["result_type_id"]
                    != closed_type_id(b, &clr_type(&result, &source_types)?)
                        .map_err(|_| fail.clone())?
            {
                return Err(fail);
            }
            for (ordinal, ty) in callable
                .parameters
                .iter()
                .chain(std::iter::once(&callable.result))
                .enumerate()
            {
                add_root(
                    ty,
                    &format!("{}.signature.{ordinal}", callable.id),
                    &mut root_values,
                )?;
            }
            parse_strict_json(
                callable.body_utf8.as_bytes(),
                StrictJsonLimits::new(16_777_216, 1_000_000, 16, 1_048_576),
            )
            .map_err(|_| fail.clone())?;
            let body: Vec<DataSourceOperation> =
                serde_json::from_str(&callable.body_utf8).map_err(|_| fail.clone())?;
            total_nodes = total_nodes.checked_add(body.len()).ok_or(fail.clone())?;
            if body.len() > 100_000 || total_nodes > 250_000 {
                return Err(fail);
            }
            let mut open = vec![];
            for (node_ordinal, node) in body.iter().enumerate() {
                if let Some(key) = &node.ty {
                    if key.starts_with("intrinsic_argument:") {
                        if !matches!(
                            key.as_str(),
                            "intrinsic_argument:System.StringComparison"
                                | "intrinsic_argument:System.MidpointRounding"
                        ) || !matches!(node.kind.as_str(), "Argument" | "FieldReference")
                        {
                            return Err(fail);
                        }
                    } else if wire.control_lowering.is_some()
                        && key.starts_with("source_exception:")
                    {
                        if !matches!(key.as_str(),
                            "source_exception:System.Exception" |
                            "source_exception:System.DivideByZeroException" |
                            "source_exception:System.OverflowException" |
                            "source_exception:System.IndexOutOfRangeException" |
                            "source_exception:System.ArgumentException" |
                            "source_exception:System.ArgumentOutOfRangeException" |
                            "source_exception:System.ArgumentNullException" |
                            "source_exception:System.InvalidOperationException" |
                            "source_exception:System.NullReferenceException" |
                            "source_exception:System.Runtime.CompilerServices.SwitchExpressionException") {
                            return Err(fail);
                        }
                    } else {
                        let ty = parse_data_type_key(b, key)?;
                        add_root(
                            &ty,
                            &format!("{}.body.{node_ordinal}", callable.id),
                            &mut root_values,
                        )?;
                        if node.kind == "ArrayCreation" {
                            let ClosedType::Instance {
                                template,
                                arguments,
                            } = ClosedType::parse(&ty).map_err(|_| fail.clone())?
                            else {
                                return Err(fail);
                            };
                            if template != "bounded_sequence" || arguments.len() != 1 {
                                return Err(fail);
                            }
                            let construction = ClosedType::Instance {
                                template: "sequence_construction".into(),
                                arguments,
                            };
                            root_values.insert(serde_json::to_string(&json!({"origin":"source_construction","provenance_id":format!("{}.construction.{node_ordinal}",callable.id),"type":construction.to_value()})).map_err(|_|fail.clone())?);
                        }
                    }
                }
                while open.last() == Some(&0) {
                    open.pop();
                }
                if let Some(parent) = open.last_mut() {
                    *parent -= 1;
                }
                if node.child_count > 100_000 || open.len() > 512 {
                    return Err(fail);
                }
                open.push(node.child_count);
                if wire.control_lowering.is_none()
                    && matches!(
                        node.kind.as_str(),
                        "Loop" | "ForEachLoop" | "WhileLoop" | "ForLoop"
                    )
                {
                    return Err(DataPhaseError::LaterOwner("CSHARP-03-T04-W01"));
                }
                if wire.control_lowering.is_none()
                    && matches!(node.kind.as_str(), "Throw" | "Try" | "CatchClause")
                {
                    return Err(DataPhaseError::LaterOwner("CSHARP-03-T04-W05"));
                }
            }
            if open.iter().any(|n| *n != 0) {
                return Err(fail);
            }
            let mut previous = None;
            let mut step_ends = BTreeMap::new();
            for step in &callable.data_steps {
                if previous.is_some_and(|p| p >= step.node_ordinal)
                    || step.node_ordinal >= body.len()
                    || !matches!(
                        step.family.as_str(),
                        "string" | "numeric" | "business" | "structural" | "domain" | "array"
                    )
                    || step.argument_ordinals.len() != step.operand_ordinals.len()
                    || step
                        .argument_ordinals
                        .iter()
                        .copied()
                        .collect::<BTreeSet<_>>()
                        .len()
                        != step.argument_ordinals.len()
                    || step.operand_ordinals.windows(2).any(|p| p[0] >= p[1])
                    || !matches!(
                        step.rounding.as_str(),
                        "" | "ToEven"
                            | "AwayFromZero"
                            | "ToZero"
                            | "ToNegativeInfinity"
                            | "ToPositiveInfinity"
                    )
                    || (!step.rounding.is_empty() && step.operation != "decimal.round")
                {
                    return Err(fail);
                }
                if step.family == "array"
                    && (body[step.node_ordinal].kind != "SimpleAssignment"
                        || !matches!(
                            step.operation.as_str(),
                            "fill" | "rewrite" | "fill_or_rewrite"
                        )
                        || !step.operand_ordinals.is_empty()
                        || !step.rounding.is_empty())
                {
                    return Err(fail);
                }
                let mut end = step.node_ordinal + 1;
                let mut pending = body[step.node_ordinal].child_count;
                while pending != 0 {
                    let node = body.get(end).ok_or(fail.clone())?;
                    pending = pending - 1 + node.child_count;
                    end += 1;
                }
                if step
                    .operand_ordinals
                    .iter()
                    .any(|&i| i < step.node_ordinal || i >= end)
                    || (step.operand_ordinals.contains(&step.node_ordinal)
                        && (!step.operation.ends_with("literal")
                            && step.operation != "string.literal.decode"
                            || body[step.node_ordinal].constant.is_none()))
                {
                    return Err(fail);
                }
                step_ends.insert(step.node_ordinal, end);
                previous = Some(step.node_ordinal);
            }
            // Intrinsic argument carriers may only occur under a corresponding
            // recipe and cannot become values, roots or recipe operands.
            for (i, node) in body.iter().enumerate().filter(|(_, n)| {
                n.ty.as_deref()
                    .is_some_and(|t| t.starts_with("intrinsic_argument:"))
            }) {
                let step = callable
                    .data_steps
                    .iter()
                    .rev()
                    .find(|s| s.node_ordinal < i && i < step_ends[&s.node_ordinal])
                    .ok_or(fail.clone())?;
                let (name, carrier) =
                    if node.ty.as_deref() == Some("intrinsic_argument:System.StringComparison") {
                        if step.family != "string"
                            || !matches!(
                                step.operation.as_str(),
                                "string.contains.ordinal"
                                    | "string.starts_with.ordinal"
                                    | "string.ends_with.ordinal"
                                    | "string.equals.ordinal"
                                    | "string.equals.instance.ordinal"
                                    | "string.compare.ordinal"
                            )
                        {
                            return Err(fail);
                        }
                        ("System.StringComparison.Ordinal".to_owned(), 4)
                    } else {
                        if step.family != "numeric" || step.operation != "decimal.round" {
                            return Err(fail);
                        }
                        let carrier = [
                            "ToEven",
                            "AwayFromZero",
                            "ToZero",
                            "ToNegativeInfinity",
                            "ToPositiveInfinity",
                        ]
                        .iter()
                        .position(|m| *m == step.rounding)
                        .ok_or(fail.clone())?;
                        (
                            format!("System.MidpointRounding.{}", step.rounding),
                            carrier,
                        )
                    };
                if node.kind != "FieldReference"
                    || node.child_count != 0
                    || node.symbol != format!("System.Runtime|{name}")
                    || node.constant.as_deref() != Some(format!("System.Int32:{carrier}").as_str())
                    || step.operand_ordinals.contains(&i)
                {
                    return Err(fail);
                }
            }
            bodies.insert(callable.id.clone(), body);
        }
        let mut locals = BTreeSet::new();
        for local in &wire.exact_types {
            if !bodies.contains_key(&local.callable_id)
                || !locals.insert((&local.callable_id, local.local_ordinal))
            {
                return Err(fail);
            }
            add_root(
                &local.ty,
                &format!("{}.local.{}", local.callable_id, local.local_ordinal),
                &mut root_values,
            )?;
        }
        let getters = wire
            .callables
            .iter()
            .filter(|c| c.is_property_getter)
            .filter_map(|c| {
                let name = c.identity["name"].as_str()?.strip_prefix("get_")?;
                Some((
                    format!("{}.{}", c.identity["owner"].as_str()?, name),
                    c.id.clone(),
                ))
            })
            .collect::<BTreeMap<_, _>>();
        let mut reachable = BTreeSet::new();
        let mut active = BTreeSet::new();
        fn visit<'a>(
            id: &'a str,
            bodies: &'a BTreeMap<String, Vec<DataSourceOperation>>,
            getters: &'a BTreeMap<String, String>,
            active: &mut BTreeSet<&'a str>,
            done: &mut BTreeSet<&'a str>,
        ) -> Result<(), DataPhaseError> {
            if done.contains(id) {
                return Ok(());
            }
            if !active.insert(id) {
                return Err(DataPhaseError::Cycle);
            }
            let body = bodies.get(id).ok_or(DataPhaseError::Source)?;
            for (ordinal, op) in body.iter().enumerate() {
                let write_only = ordinal > 0 && body[ordinal - 1].kind == "SimpleAssignment";
                if op.kind == "PropertyReference" && !write_only {
                    if let Some(getter) = getters.get(&op.symbol) {
                        visit(getter, bodies, getters, active, done)?;
                    }
                }
                if bodies.contains_key(&op.symbol) {
                    visit(&op.symbol, bodies, getters, active, done)?;
                }
            }
            active.remove(id);
            done.insert(id);
            Ok(())
        }
        for root in &wire.selected_root_ids {
            visit(root, &bodies, &getters, &mut active, &mut reachable)?;
        }
        // W03 expands every reached data type through its property getters,
        // including getters whose value is never read by the selected method.
        // These are declaration roots, not fabricated calls from assignments.
        for getter in getters.values() {
            visit(getter, &bodies, &getters, &mut active, &mut reachable)?;
        }
        // W04's synthesized constructor plans are type declaration proof
        // roots, just as W03's getters are. They create no caller invocation.
        for constructor in wire
            .callables
            .iter()
            .filter(|c| c.is_synthesized_constructor)
        {
            visit(
                &constructor.id,
                &bodies,
                &getters,
                &mut active,
                &mut reachable,
            )?;
        }
        if reachable.len() != bodies.len() {
            return Err(DataPhaseError::Unreachable);
        }
        // ValueId in W12 makes the nullable carrier explicit even for a
        // non-null source position. Reuse the source type parser/ID algorithm
        // and the sole root/closure engine for those proof subjects as well.
        let mut obligation_types = BTreeMap::<String, Value>::new();
        fn catalog(
            b: &ValidatedFoundationBundle,
            value: &Value,
            out: &mut BTreeMap<String, Value>,
        ) -> Result<(), DataPhaseError> {
            let ty = ClosedType::parse(value).map_err(|_| DataPhaseError::Source)?;
            let id = closed_type_id(b, &ty).map_err(|_| DataPhaseError::Source)?;
            out.insert(id, value.clone());
            if let Some(args) = value["arguments"].as_array() {
                for arg in args {
                    catalog(b, arg, out)?;
                }
            }
            Ok(())
        }
        for ty in &wire.types {
            catalog(
                b,
                &json!({"kind":"source","id":ty.id}),
                &mut obligation_types,
            )?;
            for member in &ty.members {
                catalog(b, &member.ty, &mut obligation_types)?;
            }
        }
        for callable in &wire.callables {
            for ty in callable
                .parameters
                .iter()
                .chain(std::iter::once(&callable.result))
            {
                catalog(b, ty, &mut obligation_types)?;
            }
        }
        for local in &wire.exact_types {
            catalog(b, &local.ty, &mut obligation_types)?;
        }
        for value in obligation_types.values().cloned().collect::<Vec<_>>() {
            catalog(
                b,
                &json!({"kind":"instance","template":"option","arguments":[value]}),
                &mut obligation_types,
            )?;
        }
        if wire.source_obligations.len() > 16384 {
            return Err(fail);
        }
        let mut obligation_keys = BTreeSet::new();
        for (ordinal, o) in wire.source_obligations.iter().enumerate() {
            location(&o.source_path, &o.source_sha256, o.start_byte, o.end_byte)?;
            let declaration = wire
                .callables
                .iter()
                .find(|c| c.id == o.declaration_id)
                .map(|c| (&c.source_path, c.start_byte, c.end_byte))
                .or_else(|| {
                    wire.types
                        .iter()
                        .find(|t| t.id == o.declaration_id)
                        .map(|t| (&t.source_path, t.start_byte, t.end_byte))
                })
                .or_else(|| {
                    wire.source_containers
                        .iter()
                        .find(|t| t.id == o.declaration_id)
                        .map(|t| (&t.source_path, t.start_byte, t.end_byte))
                })
                .ok_or_else(|| fail.clone())?;
            if o.discharged
                || !matches!(
                    o.family.as_str(),
                    "construction" | "string" | "domain" | "business" | "array"
                )
                || !(valid_obligation_kind(&o.family, &o.kind)
                    || wire.control_lowering.is_some()
                        && o.family == "array"
                        && o.type_id.is_empty()
                        && matches!(
                            (o.kind.as_str(), o.predicate.as_str()),
                            ("loop_header", "ownership_and_initialized_prefix_invariant")
                                | (
                                    "loop_backedge",
                                    "ownership_phi_and_initialized_prefix_preservation"
                                )
                                | ("loop_exit", "structured_exit_ownership_and_publication")
                        ))
                || (o.family == "array") != !o.predicate.is_empty()
                || matches!(o.family.as_str(), "construction" | "domain" | "business")
                    && o.type_id.is_empty()
                || o.family == "string" && !o.type_id.is_empty()
                || o.family == "string" && !o.members.is_empty()
                || o.site.is_empty()
                || o.members.len() > 256
                || *declaration.0 != o.source_path
                || o.start_byte < declaration.1
                || o.end_byte > declaration.2
                || !obligation_keys.insert(serde_json::to_string(o).map_err(|_| fail.clone())?)
            {
                return Err(fail);
            }
            if let Some(ty) = obligation_types.get(&o.type_id) {
                add_root(
                    ty,
                    &format!("{}.obligation.{ordinal:06}", o.declaration_id),
                    &mut root_values,
                )?;
            }
        }
        if wire.control_lowering.is_some() {
            for callable in &wire.callables {
                if wire
                    .control_lowering
                    .as_ref()
                    .and_then(|c| c["handlers"].as_array())
                    .is_some_and(|handlers| {
                        handlers.iter().any(|h| {
                            h["callable_id"] == callable.id
                                && h["regions"].as_array().is_some_and(|rs| {
                                    rs.iter().any(|r| r["finally_entry"].is_string())
                                })
                        })
                    })
                    && callable.result["template"] != "option"
                {
                    add_root(
                        &json!({"kind":"instance","template":"option","arguments":[callable.result]}),
                        &format!("{}.control.pending_return", callable.id),
                        &mut root_values,
                    )?;
                }
            }
        }
        let constructor_assignments = derive_constructor_assignments(&wire, &bodies)?;
        validate_initialization_plans(&wire, &bodies, &constructor_assignments)?;
        let root_values = root_values
            .iter()
            .map(|s| serde_json::from_str::<Value>(s).map_err(|_| fail.clone()))
            .collect::<Result<Vec<_>, _>>()?;
        let source_types = Value::Object(source_types);
        let transport = canonical_closed_root_set_transport(b, &json!(root_values), &source_types)
            .map_err(|_| fail.clone())?;
        let roots = validate_closed_root_set(b, &transport).map_err(|_| fail.clone())?;
        let closed = derive_closed_instances(b, &roots).map_err(|_| fail.clone())?;
        for ty in &wire.types {
            if let Some(graph) = &ty.recursive_default {
                check_default_graph_types(
                    b,
                    &roots,
                    &closed,
                    graph,
                    graph.root,
                    &ty.id,
                    &mut BTreeSet::new(),
                )?;
            }
        }
        let source = Self {
            captured_facts: bytes.to_vec(),
            wire,
            bodies,
            constructor_assignments,
            source_types,
            roots,
            semantic_context: context.semantic_context().clone(),
            selection_sha256: context.selection_sha256().into(),
            snapshot_sha256: captures.snapshot_sha256().into(),
        };
        if source.control_lowering().is_some() {
            // The marker widens the admitted source vocabulary, so validation
            // belongs at this boundary, before any caller can consume roots.
            validate_control_source(b, &source)?;
        }
        Ok(source)
    }
}
/// Reconstruct the W05 ordering from the captured operation tree. The private
/// plan is evidence from the owning validator, never an instruction to reorder
/// the body or to publish a partially initialized source value.
fn validate_initialization_plans(
    wire: &DataSourceWire,
    bodies: &BTreeMap<String, Vec<DataSourceOperation>>,
    constructor_assignments: &BTreeMap<String, SourceConstructorAssignment>,
) -> Result<(), DataPhaseError> {
    let fail = DataPhaseError::Source;
    fn children(
        body: &[DataSourceOperation],
        ordinal: usize,
    ) -> Result<Vec<usize>, DataPhaseError> {
        let node = body.get(ordinal).ok_or(DataPhaseError::Source)?;
        let mut next = ordinal + 1;
        let mut result = Vec::with_capacity(node.child_count);
        for _ in 0..node.child_count {
            result.push(next);
            let mut pending = 1usize;
            while pending != 0 {
                pending = pending - 1 + body.get(next).ok_or(DataPhaseError::Source)?.child_count;
                next += 1;
            }
        }
        Ok(result)
    }
    for callable in &wire.callables {
        let body = &bodies[&callable.id];
        let expected_creations = body
            .iter()
            .enumerate()
            .filter_map(|(ordinal, node)| {
                (node.kind == "ObjectCreation"
                    && wire
                        .callables
                        .iter()
                        .any(|c| c.id == node.symbol && c.identity["kind"] == "constructor"))
                .then_some(ordinal)
            })
            .collect::<Vec<_>>();
        if callable
            .initialization_plans
            .iter()
            .map(|p| p.node_ordinal)
            .collect::<Vec<_>>()
            != expected_creations
        {
            return Err(fail);
        }
        for plan in &callable.initialization_plans {
            let creation = &body[plan.node_ordinal];
            let constructor = wire
                .callables
                .iter()
                .find(|c| c.id == creation.symbol)
                .ok_or(fail.clone())?;
            let owner = wire
                .types
                .iter()
                .find(|t| t.id == plan.type_id)
                .ok_or(fail.clone())?;
            if constructor.identity["owner"] != plan.type_id
                || plan.constructor_id != constructor.id
                || !plan.site.starts_with(&format!("{}:", callable.source_path))
                || plan.member_order
                    != owner
                        .members
                        .iter()
                        .map(|m| m.name.clone())
                        .collect::<Vec<_>>()
                || owner.members.len() > 32
                || !plan.has_normal_exit
                || plan.definitely_assigned & !plan.possibly_assigned != 0
                || owner.members.len() < 32 && plan.possibly_assigned >> owner.members.len() != 0
            {
                return Err(fail);
            }
            let mut expected = Vec::new();
            let mut add = |kind: &str, target: &str, expression_ordinal, exceptional_exit: &str| {
                expected.push(DataSourceInitializationStep {
                    kind: kind.into(),
                    target: target.into(),
                    expression_ordinal,
                    exceptional_exit: exceptional_exit.into(),
                });
            };
            let creation_children = children(body, plan.node_ordinal)?;
            for ordinal in &creation_children {
                let node = &body[*ordinal];
                if node.kind == "Argument" {
                    let arguments = children(body, *ordinal)?;
                    if arguments.len() != 1 {
                        return Err(fail);
                    }
                    add(
                        "EvaluateArgument",
                        &node.symbol,
                        Some(arguments[0]),
                        "no_value",
                    );
                } else if node.kind != "ObjectOrCollectionInitializer" {
                    return Err(fail);
                }
            }
            add("Begin", &plan.site, None, "no_value");
            add("InvokeConstructor", &plan.constructor_id, None, "discard");
            if owner.members.iter().any(|m| m.storage == "init_auto") {
                add(
                    "ConstructionInvariant",
                    &plan.constructor_id,
                    None,
                    "discard",
                );
            }
            let mut assigned = BTreeSet::new();
            let mut final_assignment = constructor_assignments[&constructor.id];
            for initializer in creation_children
                .iter()
                .filter(|i| body[**i].kind == "ObjectOrCollectionInitializer")
            {
                for ordinal in children(body, *initializer)? {
                    let assignment = children(body, ordinal)?;
                    if body[ordinal].kind != "SimpleAssignment" || assignment.len() != 2 {
                        return Err(fail);
                    }
                    let target = &body[assignment[0]];
                    let receiver = children(body, assignment[0])?;
                    let member = owner
                        .members
                        .iter()
                        .enumerate()
                        .find(|(_, member)| {
                            target.symbol == format!("{}.{}", owner.id, member.name)
                        })
                        .ok_or(fail.clone())?;
                    if target.kind != "PropertyReference"
                        || member.1.storage != "init_auto"
                        || receiver.len() != 1
                        || body[receiver[0]].kind != "InstanceReference"
                        || !assigned.insert(member.1.name.as_str())
                        || plan.definitely_assigned & (1u32 << member.0) == 0
                    {
                        return Err(fail);
                    }
                    let bit = 1u32 << member.0;
                    if final_assignment.possibly_assigned & bit != 0 {
                        return Err(fail);
                    }
                    final_assignment.definitely_assigned |= bit;
                    final_assignment.possibly_assigned |= bit;
                    add(
                        "EvaluateInitializer",
                        &member.1.name,
                        Some(assignment[1]),
                        "discard",
                    );
                    add("Assign", &member.1.name, None, "discard");
                }
            }
            if owner
                .members
                .iter()
                .any(|m| m.required && !assigned.contains(m.name.as_str()))
            {
                return Err(fail);
            }
            add("PublicInvariant", &plan.site, None, "discard");
            add("Finalize", &plan.site, None, "discard");
            if plan.steps != expected
                || plan.definitely_assigned != final_assignment.definitely_assigned
                || plan.possibly_assigned != final_assignment.possibly_assigned
            {
                return Err(fail);
            }
        }
    }
    Ok(())
}
impl ValidatedDataSource {
    pub fn constructor_assignment(&self, id: &str) -> Option<SourceConstructorAssignment> {
        self.constructor_assignments.get(id).copied()
    }
    pub fn validate_source_call_signatures(
        &self,
        bundle: &ValidatedFoundationBundle,
        operations: &BTreeMap<String, ClosedOperationSignature>,
    ) -> Result<(), DataPhaseError> {
        let actual = operations
            .values()
            .filter(|s| {
                matches!(
                    s.tag,
                    ClosedOperationTag::SourceCall | ClosedOperationTag::ConstructorExecute
                )
            })
            .map(|s| s.id.as_str())
            .collect::<BTreeSet<_>>();
        let expected = self
            .callables()
            .iter()
            .map(|c| c.id())
            .collect::<BTreeSet<_>>();
        if actual != expected {
            return Err(DataPhaseError::Source);
        }
        for callable in self.callables() {
            let signature = &operations[callable.id()];
            let logical = if signature.tag == ClosedOperationTag::ConstructorExecute {
                let closed = derive_closed_instances(bundle, &self.roots)
                    .map_err(|_| DataPhaseError::Source)?;
                object_constructor_execution_signature(bundle, &self.roots, &closed, callable)
                    .map_err(|_| DataPhaseError::Source)?
            } else {
                callable.logical_signature(bundle)?
            };
            if signature.argument_type_ids != logical.argument_type_ids
                || signature.normal_result_type_id != logical.normal_result_type_id
            {
                return Err(DataPhaseError::Source);
            }
        }
        Ok(())
    }
}

/// Independently compute the constructor's explicit writes from its body. CLR
/// zeroing is not an assignment and defaults are applied only at finalization.
fn derive_constructor_assignments(
    wire: &DataSourceWire,
    bodies: &BTreeMap<String, Vec<DataSourceOperation>>,
) -> Result<BTreeMap<String, SourceConstructorAssignment>, DataPhaseError> {
    type State = SourceConstructorAssignment;
    fn merge(a: State, b: State) -> State {
        State {
            definitely_assigned: a.definitely_assigned & b.definitely_assigned,
            possibly_assigned: a.possibly_assigned | b.possibly_assigned,
        }
    }
    struct Analysis<'a> {
        wire: &'a DataSourceWire,
        bodies: &'a BTreeMap<String, Vec<DataSourceOperation>>,
        summaries: BTreeMap<String, State>,
        active: BTreeSet<String>,
    }
    impl Analysis<'_> {
        fn constructor(&mut self, id: &str) -> Result<State, DataPhaseError> {
            if let Some(state) = self.summaries.get(id) {
                return Ok(*state);
            }
            if !self.active.insert(id.into()) {
                return Err(DataPhaseError::Cycle);
            }
            let callable = self
                .wire
                .callables
                .iter()
                .find(|c| c.id == id && c.identity["kind"] == "constructor")
                .ok_or(DataPhaseError::Source)?;
            let owner = self
                .wire
                .types
                .iter()
                .find(|t| callable.identity["owner"] == t.id)
                .ok_or(DataPhaseError::Source)?;
            let body = &self.bodies[id];
            let mut children = vec![Vec::new(); body.len()];
            let mut roots = Vec::new();
            let mut stack: Vec<(usize, usize)> = Vec::new();
            for (i, node) in body.iter().enumerate() {
                while stack.last().is_some_and(|(_, left)| *left == 0) {
                    stack.pop();
                }
                if let Some((parent, left)) = stack.last_mut() {
                    children[*parent].push(i);
                    *left -= 1;
                } else {
                    roots.push(i);
                }
                stack.push((i, node.child_count));
            }
            let mut normal = Some(State {
                definitely_assigned: 0,
                possibly_assigned: 0,
            });
            let mut exits = None;
            for ordinal in roots {
                if let Some(state) = normal {
                    normal = self.node(owner, body, &children, ordinal, state, &mut exits)?;
                }
            }
            let result = match (normal, exits) {
                (Some(a), Some(b)) => merge(a, b),
                (Some(a), None) | (None, Some(a)) => a,
                _ => return Err(DataPhaseError::Source),
            };
            self.active.remove(id);
            self.summaries.insert(id.into(), result);
            Ok(result)
        }
        fn node(
            &mut self,
            owner: &DataSourceType,
            body: &[DataSourceOperation],
            children: &[Vec<usize>],
            ordinal: usize,
            mut state: State,
            exits: &mut Option<State>,
        ) -> Result<Option<State>, DataPhaseError> {
            let node = &body[ordinal];
            let nested = &children[ordinal];
            let member = |target: &DataSourceOperation| {
                owner
                    .members
                    .iter()
                    .enumerate()
                    .find(|(_, m)| target.symbol == format!("{}.{}", owner.id, m.name))
            };
            if node.kind == "Return" {
                if !nested.is_empty() {
                    return Err(DataPhaseError::Source);
                }
                *exits = Some(exits.map_or(state, |prior| merge(prior, state)));
                return Ok(None);
            }
            if node.kind == "Conditional" {
                if !(2..=3).contains(&nested.len()) {
                    return Err(DataPhaseError::Source);
                }
                let Some(condition) = self.node(owner, body, children, nested[0], state, exits)?
                else {
                    return Err(DataPhaseError::Source);
                };
                let constant = body[nested[0]].constant.as_deref();
                let yes = if constant == Some("bool:false") {
                    None
                } else {
                    self.node(owner, body, children, nested[1], condition, exits)?
                };
                let no = if constant == Some("bool:true") {
                    None
                } else if nested.len() == 3 {
                    self.node(owner, body, children, nested[2], condition, exits)?
                } else {
                    Some(condition)
                };
                return Ok(match (yes, no) {
                    (Some(a), Some(b)) => Some(merge(a, b)),
                    (Some(a), None) | (None, Some(a)) => Some(a),
                    _ => None,
                });
            }
            if node.kind == "ObjectCreation" {
                // A nested initializer's implicit receiver is its own fresh
                // object. Analyze arguments and RHSs for effects on our receiver,
                // without interpreting the nested target as our member write.
                for &child in nested {
                    if body[child].kind == "ObjectOrCollectionInitializer" {
                        for assignment in &children[child] {
                            if body[*assignment].kind != "SimpleAssignment"
                                || children[*assignment].len() != 2
                            {
                                return Err(DataPhaseError::Source);
                            }
                            state = self
                                .node(
                                    owner,
                                    body,
                                    children,
                                    children[*assignment][1],
                                    state,
                                    exits,
                                )?
                                .ok_or(DataPhaseError::Source)?;
                        }
                    } else {
                        state = self
                            .node(owner, body, children, child, state, exits)?
                            .ok_or(DataPhaseError::Source)?;
                    }
                }
                return Ok(Some(state));
            }
            if node.kind == "ConstructorInitializer" {
                if nested.len() != 1 {
                    return Err(DataPhaseError::Source);
                }
                let call = nested[0];
                if body[call].kind != "Invocation" {
                    return Err(DataPhaseError::Source);
                }
                if body[call].symbol == "System.Runtime|System.Exception.Exception()"
                    && children[call].len() == 1
                    && body[children[call][0]].kind == "InstanceReference"
                    && body[children[call][0]].ty.as_deref()
                        == Some("source_exception:System.Exception")
                    && self
                        .wire
                        .control_lowering
                        .as_ref()
                        .and_then(|c| c["exception_definitions"].as_array())
                        .is_some_and(|defs| {
                            defs.iter().any(|d| {
                                d["type_id"] == owner.id
                                    && d["sealed_type"] == true
                                    && d["direct_base_type_id"] == "System.Exception"
                            })
                        })
                {
                    return Ok(Some(state));
                }
                for argument in children[call]
                    .iter()
                    .filter(|i| body[**i].kind == "Argument")
                {
                    state = self
                        .node(owner, body, children, *argument, state, exits)?
                        .ok_or(DataPhaseError::Source)?;
                }
                let target = self
                    .wire
                    .callables
                    .iter()
                    .find(|c| c.id == body[call].symbol)
                    .ok_or(DataPhaseError::Source)?;
                if target.identity["owner"] != owner.id {
                    return Err(DataPhaseError::Source);
                }
                let delegated = self.constructor(&target.id)?;
                if state.possibly_assigned & delegated.possibly_assigned != 0 {
                    return Err(DataPhaseError::Source);
                }
                state.definitely_assigned |= delegated.definitely_assigned;
                state.possibly_assigned |= delegated.possibly_assigned;
                return Ok(Some(state));
            }
            if node.kind == "SimpleAssignment" && nested.len() == 2 {
                if let Some((index, stored)) = member(&body[nested[0]]) {
                    let receiver = &children[nested[0]];
                    if receiver.len() != 1 || body[receiver[0]].kind != "InstanceReference" {
                        return Err(DataPhaseError::Source);
                    }
                    state = self
                        .node(owner, body, children, nested[1], state, exits)?
                        .ok_or(DataPhaseError::Source)?;
                    if !node.implicit {
                        let bit = 1u32 << index;
                        if stored.required || state.possibly_assigned & bit != 0 {
                            return Err(DataPhaseError::Source);
                        }
                        state.definitely_assigned |= bit;
                        state.possibly_assigned |= bit;
                    }
                    return Ok(Some(state));
                }
            }
            if matches!(node.kind.as_str(), "FieldReference" | "PropertyReference") {
                if let Some((index, _)) = member(node) {
                    if nested.len() == 1 && body[nested[0]].kind == "InstanceReference" {
                        if state.definitely_assigned & (1u32 << index) == 0 {
                            return Err(DataPhaseError::Source);
                        }
                        return Ok(Some(state));
                    }
                }
            }
            if node.kind == "InstanceReference" {
                return Err(DataPhaseError::Source);
            }
            for child in nested {
                let Some(next) = self.node(owner, body, children, *child, state, exits)? else {
                    return Ok(None);
                };
                state = next;
            }
            Ok(Some(state))
        }
    }
    let mut analysis = Analysis {
        wire,
        bodies,
        summaries: BTreeMap::new(),
        active: BTreeSet::new(),
    };
    for callable in wire
        .callables
        .iter()
        .filter(|c| c.identity["kind"] == "constructor")
    {
        analysis.constructor(&callable.id)?;
    }
    Ok(analysis.summaries)
}
fn raw_clr_default(
    ty: &Value,
    sources: &[DataSourceType],
    active: &mut BTreeSet<String>,
    cells: &mut usize,
) -> Result<Value, DataPhaseError> {
    *cells += 1;
    if *cells > 8192 {
        return Err(DataPhaseError::Default);
    }
    match ClosedType::parse(ty).map_err(|_| DataPhaseError::Default)? {
        ClosedType::Primitive(p) => Ok(match p.as_str() {
            "string" => Value::Null,
            "bool" => Value::Bool(false),
            "f32" => json!("00000000"),
            "f64" => json!("0000000000000000"),
            "guid" => json!("00000000-0000-0000-0000-000000000000"),
            "date" => json!("0001-01-01"),
            "time" => json!("00:00:00.0000000"),
            _ => json!("0"),
        }),
        ClosedType::Instance { template, .. }
            if template == "option" || template == "bounded_sequence" =>
        {
            Ok(Value::Null)
        }
        ClosedType::Source(id) => {
            let source = sources
                .iter()
                .find(|s| s.id == id)
                .ok_or(DataPhaseError::Default)?;
            if source.kind == "enum" {
                return Ok(json!("0"));
            }
            if source.kind == "sealed_class" {
                return Ok(Value::Null);
            }
            if !active.insert(id.clone()) {
                return Err(DataPhaseError::Cycle);
            }
            let mut value = Map::new();
            for member in &source.members {
                let member_id = csharp_practical_stored_member_id(
                    &id,
                    &member.name,
                    &member.ty,
                    &member.storage,
                )
                .map_err(|_| DataPhaseError::Default)?;
                value.insert(
                    member_id,
                    raw_clr_default(&member.ty, sources, active, cells)?,
                );
            }
            active.remove(&id);
            Ok(Value::Object(value))
        }
        _ => Err(DataPhaseError::Default),
    }
}

fn validate_default_graph(graph: &DataSourceDefaultGraph) -> Result<(), DataPhaseError> {
    if graph.nodes.is_empty() || graph.nodes.len() > 8192 || graph.root != graph.nodes.len() - 1 {
        return Err(DataPhaseError::Default);
    }
    let mut seen = BTreeSet::new();
    let mut pending = vec![graph.root];
    while let Some(i) = pending.pop() {
        if !seen.insert(i) {
            continue;
        }
        let node = &graph.nodes[i];
        if node.members.iter().any(|m| *m >= i)
            || !matches!(node.kind.as_str(), "product" | "scalar" | "enum" | "none")
            || node.kind != "product" && !node.members.is_empty()
        {
            return Err(DataPhaseError::Default);
        }
        pending.extend(&node.members);
    }
    if seen.len() != graph.nodes.len() {
        return Err(DataPhaseError::Default);
    }
    Ok(())
}

fn check_default_graph_types(
    b: &ValidatedFoundationBundle,
    r: &ValidatedClosedRootSet,
    c: &ClosedInstanceSet,
    graph: &DataSourceDefaultGraph,
    ordinal: usize,
    type_id: &str,
    checked: &mut BTreeSet<(usize, String)>,
) -> Result<(), DataPhaseError> {
    if !checked.insert((ordinal, type_id.into())) {
        return Ok(());
    }
    let node = &graph.nodes[ordinal];
    let fail = DataPhaseError::Default;
    if r.source_types
        .get(type_id)
        .is_some_and(|s| s.kind == SourceKind::Enum)
    {
        if node.type_id != type_id
            || node.kind != "enum"
            || node.scalar != "0"
            || !node.members.is_empty()
            || !r.source_types[type_id].enum_values.iter().any(|v| v == "0")
        {
            return Err(fail);
        }
        return Ok(());
    }
    if let Some(source) = r
        .source_types
        .get(type_id)
        .filter(|s| s.kind != SourceKind::Enum)
    {
        if source.kind != SourceKind::ReadonlyStruct
            || node.type_id != type_id
            || node.kind != "product"
            || !node.scalar.is_empty()
            || node.members.len() != source.members.len()
            || source.members.iter().any(|m| m.required)
        {
            return Err(fail);
        }
        for (member, child) in source.members.iter().zip(&node.members) {
            let id = closed_type_id(b, &member.ty).map_err(|_| fail.clone())?;
            check_default_graph_types(b, r, c, graph, *child, &id, checked)?;
        }
        return Ok(());
    }
    let actual = domain_default(b, r, c, type_id).map_err(|_| fail.clone())?;
    let (kind, scalar) = match actual {
        MonomorphicValue::Enum { carrier, .. } => ("enum", carrier),
        MonomorphicValue::Option {
            arm: OptionArm::None,
            ..
        } => ("none", String::new()),
        MonomorphicValue::Bool { value, .. } => ("scalar", value.to_string()),
        MonomorphicValue::Char { utf16, .. } => ("scalar", utf16.to_string()),
        MonomorphicValue::F32Bits { bits, .. } | MonomorphicValue::F64Bits { bits, .. } => {
            ("scalar", bits)
        }
        MonomorphicValue::Signed { value, .. } | MonomorphicValue::Unsigned { value, .. } => {
            ("scalar", value)
        }
        value => {
            let codec = match value {
                MonomorphicValue::DecimalBits { .. } => "decimal.normalized",
                MonomorphicValue::Date { .. } => "date",
                MonomorphicValue::Time { .. } => "time",
                MonomorphicValue::Duration { .. } => "duration_ticks",
                MonomorphicValue::Instant { .. } => "unix_milliseconds",
                MonomorphicValue::Guid { .. } => "guid.d",
                _ => return Err(fail),
            };
            let text = BoundaryCodec::new(codec, type_id, None, None)
                .map_err(|_| fail.clone())?
                .format(b, r, c, &value)
                .map_err(|_| fail.clone())?;
            (
                "scalar",
                String::from_utf16(&text).map_err(|_| fail.clone())?,
            )
        }
    };
    // W01 keeps the CLR reference's ID for annotated reference defaults.
    // Its semantic presence type is independently established by the member.
    let reference_id = c
        .metadata
        .get(type_id)
        .filter(|m| template_name(&m.template_id) == Some("option"))
        .and_then(|m| m.argument_ids.first())
        .map(String::as_str);
    if node.kind != kind
        || node.scalar != scalar
        || !node.members.is_empty()
        || node.type_id != type_id
            && !(kind == "none" && reference_id == Some(node.type_id.as_str()))
    {
        return Err(fail);
    }
    Ok(())
}

// C# declaration identities describe CLR signatures. Reference annotations are
// represented by option in VIR but do not alter the original CLR declaration.
fn clr_type(ty: &ClosedType, sources: &Map<String, Value>) -> Result<ClosedType, DataPhaseError> {
    Ok(match ty {
        ClosedType::Instance {
            template,
            arguments,
        } => {
            if template == "option"
                && arguments.len() == 1
                && match &arguments[0] {
                    ClosedType::Primitive(t) => t == "string",
                    ClosedType::Source(id) => {
                        sources.get(id).is_some_and(|s| s["kind"] == "sealed_class")
                    }
                    ClosedType::Instance { template, .. } => template == "bounded_sequence",
                }
            {
                return clr_type(&arguments[0], sources);
            }
            ClosedType::Instance {
                template: template.clone(),
                arguments: arguments
                    .iter()
                    .map(|a| clr_type(a, sources))
                    .collect::<Result<_, _>>()?,
            }
        }
        _ => ty.clone(),
    })
}

pub fn parse_data_type_key(
    b: &ValidatedFoundationBundle,
    key: &str,
) -> Result<Value, DataPhaseError> {
    fn number(input: &mut &str) -> Result<usize, DataPhaseError> {
        let (n, rest) = input.split_once(':').ok_or(DataPhaseError::Source)?;
        let value = n.parse::<usize>().map_err(|_| DataPhaseError::Source)?;
        if value.to_string() != n {
            return Err(DataPhaseError::Source);
        }
        *input = rest;
        Ok(value)
    }
    fn part<'a>(input: &mut &'a str) -> Result<&'a str, DataPhaseError> {
        let len = number(input)?;
        let value = input.get(..len).ok_or(DataPhaseError::Source)?;
        *input = &input[len..];
        Ok(value)
    }
    fn read(
        b: &ValidatedFoundationBundle,
        input: &mut &str,
        depth: usize,
    ) -> Result<(ClosedType, ClosedType), DataPhaseError> {
        if depth > 16 {
            return Err(DataPhaseError::Source);
        }
        let id = part(input)?.to_owned();
        let annotation = part(input)?.to_owned();
        let count = number(input)?;
        if !matches!(annotation.as_str(), "value" | "not_annotated" | "annotated") || count > 1 {
            return Err(DataPhaseError::Source);
        }
        let (raw, mut semantic) = if count == 0 {
            let ty = if id.starts_with("mpk.csharp.source.") {
                ClosedType::Source(id.clone())
            } else {
                ClosedType::Primitive(
                    id.strip_prefix("mpk.csharp.value.")
                        .and_then(|s| s.strip_suffix(".v1"))
                        .filter(|s| PRIMITIVES.contains(s))
                        .ok_or(DataPhaseError::Source)?
                        .into(),
                )
            };
            (ty.clone(), ty)
        } else {
            let (raw, semantic) = read(b, input, depth + 1)?;
            let mut result = None;
            for name in ["option", "bounded_sequence"] {
                let ty = ClosedType::Instance {
                    template: name.into(),
                    arguments: vec![raw.clone()],
                };
                if closed_type_id(b, &ty).map_err(|_| DataPhaseError::Source)? == id {
                    result = Some((
                        ty,
                        ClosedType::Instance {
                            template: name.into(),
                            arguments: vec![semantic.clone()],
                        },
                    ));
                }
            }
            result.ok_or(DataPhaseError::Source)?
        };
        if annotation == "annotated" {
            semantic = ClosedType::Instance {
                template: "option".into(),
                arguments: vec![semantic],
            };
        }
        Ok((raw, semantic))
    }
    let mut remaining = key;
    let (_, ty) = read(b, &mut remaining, 0)?;
    if !remaining.is_empty() {
        return Err(DataPhaseError::Source);
    }
    Ok(ty.to_value())
}

impl ValidatedDataSource {
    pub(super) fn require_lineage(
        &self,
        context: &PracticalArtifactContext,
        captures: &CapturedInputSet,
    ) -> Result<(), DataPhaseError> {
        if self.semantic_context != *context.semantic_context()
            || self.selection_sha256 != context.selection_sha256()
            || self.snapshot_sha256 != captures.snapshot_sha256()
        {
            return Err(DataPhaseError::Source);
        }
        Ok(())
    }
    pub(super) fn source_map_declarations(
        &self,
        vir: &crate::csharp_practical_vir_validation::ValidatedPracticalVir,
    ) -> Result<Vec<crate::csharp_practical_source_artifacts::SourceMapDeclaration>, DataPhaseError>
    {
        use crate::csharp_practical_source_artifacts as a;
        let mut entries = vec![];
        for callable in &self.wire.callables {
            let fields = &callable.identity;
            let identity = a::SourceDeclarationIdentity {
                namespace: fields["namespace"]
                    .as_str()
                    .ok_or(DataPhaseError::Source)?
                    .into(),
                source_name: fields["name"]
                    .as_str()
                    .ok_or(DataPhaseError::Source)?
                    .into(),
                kind: if fields["kind"] == "constructor" {
                    a::SourceDeclarationKind::Constructor
                } else {
                    a::SourceDeclarationKind::Method
                },
                containing_source_type_id: Some(
                    fields["owner"]
                        .as_str()
                        .ok_or(DataPhaseError::Source)?
                        .into(),
                ),
                parameter_type_ids: fields["parameter_type_ids"]
                    .as_array()
                    .ok_or(DataPhaseError::Source)?
                    .iter()
                    .map(|v| v.as_str().map(str::to_owned).ok_or(DataPhaseError::Source))
                    .collect::<Result<_, _>>()?,
                result_type_id: Some(
                    fields["result_type_id"]
                        .as_str()
                        .ok_or(DataPhaseError::Source)?
                        .into(),
                ),
            };
            let function = vir
                .functions()
                .iter()
                .find(|f| f.id == callable.id)
                .ok_or(DataPhaseError::Source)?;
            entries.push(a::SourceMapDeclaration {
                declaration_id: callable.id.clone(),
                identity: a::SourceMapIdentity::Declaration(identity),
                provenance_id: format!(
                    "{}.source.{}.{}",
                    callable.id, callable.start_byte, callable.end_byte
                ),
                source_path: callable.source_path.clone(),
                start_byte: callable.start_byte as u32,
                end_byte: callable.end_byte as u32,
                artifact_node_ids: function.blocks.iter().map(|b| b.node.id.clone()).collect(),
            });
        }
        for owner in &self.wire.source_containers {
            entries.push(a::SourceMapDeclaration {
                declaration_id: owner.id.clone(),
                identity: a::SourceMapIdentity::Declaration(a::SourceDeclarationIdentity {
                    namespace: owner.namespace.clone(),
                    source_name: owner.name.clone(),
                    kind: a::SourceDeclarationKind::Type,
                    containing_source_type_id: None,
                    parameter_type_ids: vec![],
                    result_type_id: None,
                }),
                provenance_id: format!(
                    "{}.source.{}.{}",
                    owner.id, owner.start_byte, owner.end_byte
                ),
                source_path: owner.source_path.clone(),
                start_byte: owner.start_byte as u32,
                end_byte: owner.end_byte as u32,
                artifact_node_ids: vec![owner.id.clone()],
            });
        }
        for ty in &self.wire.types {
            entries.push(a::SourceMapDeclaration {
                declaration_id: ty.id.clone(),
                identity: a::SourceMapIdentity::Declaration(a::SourceDeclarationIdentity {
                    namespace: ty.namespace.clone(),
                    source_name: ty.name.clone(),
                    kind: a::SourceDeclarationKind::Type,
                    containing_source_type_id: None,
                    parameter_type_ids: vec![],
                    result_type_id: None,
                }),
                provenance_id: format!("{}.source.{}.{}", ty.id, ty.start_byte, ty.end_byte),
                source_path: ty.source_path.clone(),
                start_byte: ty.start_byte as u32,
                end_byte: ty.end_byte as u32,
                artifact_node_ids: vec![ty.id.clone()],
            });
            for member in &ty.members {
                let identity = a::SourceStoredMemberIdentity {
                    owner_source_type_id: ty.id.clone(),
                    source_name: member.name.clone(),
                    closed_type: member.ty.clone(),
                    storage: match member.storage.as_str() {
                        "readonly_field" => a::SourceStoredMemberStorage::ReadonlyField,
                        "get_auto" => a::SourceStoredMemberStorage::GetAuto,
                        "init_auto" => a::SourceStoredMemberStorage::InitAuto,
                        _ => return Err(DataPhaseError::Source),
                    },
                };
                let id = a::canonical_source_stored_member_id(&identity)
                    .map_err(|_| DataPhaseError::Source)?;
                entries.push(a::SourceMapDeclaration {
                    declaration_id: id.clone(),
                    identity: a::SourceMapIdentity::StoredMember(identity),
                    provenance_id: format!("{id}.source.{}.{}", member.start_byte, member.end_byte),
                    source_path: ty.source_path.clone(),
                    start_byte: member.start_byte as u32,
                    end_byte: member.end_byte as u32,
                    artifact_node_ids: vec![id],
                });
            }
        }
        Ok(entries)
    }
}

fn valid_obligation_kind(family: &str, kind: &str) -> bool {
    match family {
        "construction" => matches!(
            kind,
            "construction_invariant"
                | "public_invariant"
                | "default_public_invariant"
                | "recursive_default"
        ),
        "string" => matches!(
            kind,
            "input_utf16_length_le_16384" | "output_utf16_length_le_16384"
        ),
        "array" => matches!(
            kind,
            "allocate_unique"
                | "complete_publication_vc"
                | "csharp_length_check"
                | "first_write_vc"
                | "functional_update"
                | "index_lower_bound"
                | "index_upper_bound"
                | "initialize_element"
                | "initialized_read_vc"
                | "parameter_profile_bound"
                | "profile_bound"
        ),
        "domain" | "business" => {
            matches!(
                kind,
                "source_invariant_implies_projection"
                    | "semantic_invariant_implies_reconstruction"
                    | "source_round_trip"
                    | "semantic_round_trip"
                    | "distinct_arms"
                    | "public_invariant"
                    | "identity_unobservable"
                    | "field_complete_reconstruction"
                    | "operation_normal_commutation"
                    | "operation_error_commutation"
                    | "operation_exception_commutation"
            ) || family == "domain"
                && matches!(
                    kind,
                    "explicit_not_null_precondition"
                        | "non_null_normal_result"
                        | "payload_public_invariant"
                        | "fallback_public_invariant"
                        | "non_null_stored_field"
                        | "invalid_errors_1_through_256"
                        | "errors_left_before_right_preserve_duplicates"
                        | "actual_default_public_invariant"
                        | "inactive_payload_exact_source_exception_or_active_precondition"
                )
                || family == "business"
                    && matches!(
                        kind,
                        "application_currency_predicate"
                            | "default_ineligible"
                            | "separate_closed_result_projection"
                            | "exhaustive_error_projection"
                            | "exhaustive_rounding_projection"
                    )
        }
        _ => false,
    }
}
