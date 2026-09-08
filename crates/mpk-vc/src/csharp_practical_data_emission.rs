//! Source-body emission into the ordinary private VIR and its independent importer.
use super::*;
use crate::csharp_practical_source_artifacts::{
    self as a, CapturedInputSet, PracticalArtifactContext,
};
use crate::csharp_practical_vir_validation as v;
#[path = "csharp_practical_control_emission.rs"]
mod control_emission;
#[path = "csharp_practical_control_exceptions.rs"]
mod control_exceptions;

pub struct EmittedDataPhase {
    closure: DataBindingClosure,
    operations: a::ConcreteOperationTables,
    vir: v::ValidatedPracticalVir,
    routes: Vec<DataTypeRoute>,
    boundaries: Vec<ValidatedBoundaryContract>,
    transitions: Vec<ValidatedTransitionContract>,
    source_map: a::ValidatedPracticalArtifact,
    manifest: a::ValidatedPracticalArtifact,
    artifacts: a::ValidatedPracticalArtifact,
}
impl EmittedDataPhase {
    pub(super) fn boundary_input_artifacts(
        &self,
        b: &ValidatedFoundationBundle,
        context: &PracticalArtifactContext,
        captures: &CapturedInputSet,
        capture: &a::ValidatedPracticalArtifact,
    ) -> Result<(a::ValidatedPracticalArtifact, a::ValidatedPracticalArtifact), BoundaryInputError>
    {
        self.boundary_run_artifacts(b, context, captures, capture, None)
    }
    pub(super) fn boundary_run_artifacts(
        &self,
        b: &ValidatedFoundationBundle,
        context: &PracticalArtifactContext,
        captures: &CapturedInputSet,
        capture: &a::ValidatedPracticalArtifact,
        output: Option<&a::ValidatedPracticalArtifact>,
    ) -> Result<(a::ValidatedPracticalArtifact, a::ValidatedPracticalArtifact), BoundaryInputError>
    {
        let fail = |_| BoundaryInputError::Linkage;
        let sidecars = DataSidecars::capture(context, captures).map_err(fail)?;
        let closed = a::bind_closed_instances(
            context,
            b,
            captures,
            self.closure.roots(),
            self.closure.closed(),
        )
        .map_err(|_| BoundaryInputError::Linkage)?;
        let boundary_contracts = self
            .boundaries
            .iter()
            .map(|b| b.artifact().artifact_ref())
            .collect::<Vec<_>>();
        let manifest = a::build_frontend_source_manifest(
            context,
            b,
            captures,
            a::FrontendManifestArtifacts {
                type_contracts: sidecars
                    .contracts()
                    .iter()
                    .filter(|c| c.schema() == a::TYPE_CONTRACT_SCHEMA)
                    .map(|c| c.artifact_ref())
                    .collect(),
                method_contracts: sidecars
                    .contracts()
                    .iter()
                    .filter(|c| c.schema() == a::METHOD_CONTRACT_SCHEMA)
                    .map(|c| c.artifact_ref())
                    .collect(),
                semantic_bindings: self.closure.bindings().artifact_ref(),
                boundary_contracts: boundary_contracts.clone(),
                boundary_inputs: vec![capture.artifact_ref()],
                boundary_outputs: output.map(|a| vec![a.artifact_ref()]).unwrap_or_default(),
                transition_contracts: self
                    .transitions
                    .iter()
                    .map(|t| t.artifact().artifact_ref())
                    .collect(),
                closed_instances: closed.clone(),
                operations: self.operations.operations().artifact_ref(),
                required_checks: self.operations.required_checks().artifact_ref(),
                vir: self.vir.artifact_ref(),
                source_map: self.source_map.artifact_ref(),
            },
        )
        .map_err(|_| BoundaryInputError::Linkage)?;
        let artifacts = a::build_frontend_source_artifacts(
            context,
            b,
            a::FrontendSourceArtifactLinks {
                vir: &self.vir.artifact_ref(),
                source_map: &self.source_map.artifact_ref(),
                source_manifest: &manifest,
                semantic_bindings: &self.closure.bindings().artifact_ref(),
                closed_instances: &closed,
                boundary_contracts,
                transition_contracts: self
                    .transitions
                    .iter()
                    .map(|t| t.artifact().artifact_ref())
                    .collect(),
            },
        )
        .map_err(|_| BoundaryInputError::Linkage)?;
        Ok((manifest, artifacts))
    }
    pub fn transitions(&self) -> &[ValidatedTransitionContract] {
        &self.transitions
    }
    pub fn boundaries(&self) -> &[ValidatedBoundaryContract] {
        &self.boundaries
    }
    pub fn vir(&self) -> &v::ValidatedPracticalVir {
        &self.vir
    }
    pub fn closure(&self) -> &DataBindingClosure {
        &self.closure
    }
    pub fn operations(&self) -> &a::ConcreteOperationTables {
        &self.operations
    }
    pub fn source_map(&self) -> &a::ValidatedPracticalArtifact {
        &self.source_map
    }
    pub fn manifest(&self) -> &a::ValidatedPracticalArtifact {
        &self.manifest
    }
    pub fn artifacts(&self) -> &a::ValidatedPracticalArtifact {
        &self.artifacts
    }
    pub fn routes(&self) -> &[DataTypeRoute] {
        &self.routes
    }
}
#[derive(Clone)]
struct Expr<'a> {
    operation: &'a DataSourceOperation,
    ordinal: usize,
    children: Vec<Expr<'a>>,
}
fn fresh_array_expression(node: &Expr<'_>) -> bool {
    match node.operation.kind() {
        "ArrayCreation" => true,
        "Argument" | "VariableInitializer" | "Parenthesized" => {
            node.children.first().is_some_and(fresh_array_expression)
        }
        "Conversion" if node.operation.traits().split('|').nth(3) == Some("True") => {
            node.children.first().is_some_and(fresh_array_expression)
        }
        "Conditional" => {
            node.children.len() == 3 && node.children[1..].iter().all(fresh_array_expression)
        }
        _ => false,
    }
}
fn trees(body: &[DataSourceOperation]) -> Result<Vec<Expr<'_>>, DataPhaseError> {
    fn read<'a>(
        body: &'a [DataSourceOperation],
        offset: &mut usize,
        depth: usize,
    ) -> Result<Expr<'a>, DataPhaseError> {
        if depth > 512 {
            return Err(DataPhaseError::Emission);
        }
        let ordinal = *offset;
        let operation = body.get(*offset).ok_or(DataPhaseError::Emission)?;
        *offset += 1;
        let children = (0..operation.child_count())
            .map(|_| read(body, offset, depth + 1))
            .collect::<Result<_, _>>()?;
        Ok(Expr {
            ordinal,
            operation,
            children,
        })
    }
    let mut offset = 0;
    let mut result = vec![];
    while offset < body.len() {
        result.push(read(body, &mut offset, 0)?);
    }
    Ok(result)
}
struct Emitter<'a> {
    b: &'a ValidatedFoundationBundle,
    r: &'a ValidatedClosedRootSet,
    c: &'a ClosedInstanceSet,
    source: &'a ValidatedDataSource,
    control: Option<std::sync::Arc<ValidatedControlSource>>,
    signatures: BTreeMap<String, ClosedOperationSignature>,
    functions: BTreeMap<String, v::PracticalVirFunction>,
    active: BTreeSet<String>,
}
struct Body {
    expression_values: BTreeMap<usize, TypedValueRef>,
    function: v::PracticalVirFunction,
    current: usize,
    next_value: usize,
    next_block: usize,
    ended: bool,
    variables: BTreeMap<String, TypedValueRef>,
    fields: BTreeMap<String, TypedValueRef>,
    constructor: bool,
    object_constructor: bool,
    live_objects: BTreeMap<String, TypedValueRef>,
    exception_object_origins: BTreeMap<String, BTreeSet<String>>,
    owner: String,
    live_constructions: BTreeMap<String, TypedValueRef>,
    construction_lengths: BTreeMap<String, TypedValueRef>,
    published_constructions: BTreeMap<String, TypedValueRef>,
    exception_construction_origins: BTreeMap<String, BTreeSet<String>>,
    normal_construction_origins: BTreeMap<String, BTreeSet<String>>,
}
impl Body {
    fn block(&mut self, tag: ControlNodeTag) -> usize {
        let ordinal = self.function.blocks.len();
        let block_identity = self.next_block;
        self.next_block += 1;
        self.function.blocks.push(v::PracticalVirBlock {
            node: ControlNode {
                id: format!("{}.node.{block_identity:06}", self.function.id),
                ordinal: ordinal as u32,
                tag,
                condition_type_id: None,
                normal_successor_ids: vec![],
                exceptional_successors: vec![],
                abrupt: None,
                loop_id: None,
                region_stack: vec![],
            },
            phi_values: vec![],
            literal_values: vec![],
            exception_values: vec![],
            condition_value_id: None,
            return_value_ids: vec![],
            abrupt_value_id: None,
            handler_exception_source_id: None,
            handler_exception_value: None,
            invocation: None,
            ownership_in: vec![],
            construction_actions: vec![],
            ownership_out: vec![],
        });
        ordinal
    }
    fn value(&mut self, type_id: &str) -> TypedValueRef {
        let id = format!("{}.value.{:06}", self.function.id, self.next_value);
        self.next_value += 1;
        TypedValueRef {
            id,
            type_id: type_id.into(),
        }
    }
    fn literal(&mut self, value: MonomorphicValue) -> TypedValueRef {
        let result = self.value(value.type_id());
        self.function.blocks[self.current]
            .literal_values
            .push(v::PracticalVirLiteral {
                result: result.clone(),
                value,
            });
        result
    }
    fn object_protocol(&mut self) -> &mut v::PracticalObjectProtocol {
        self.function
            .object_protocol
            .get_or_insert_with(|| v::PracticalObjectProtocol {
                constructor_owner: None,
                initializations: vec![],
                exceptional_discards: vec![],
            })
    }
    fn finish(&mut self, value: Option<TypedValueRef>) -> Result<(), DataPhaseError> {
        if !self.object_constructor && !self.live_objects.is_empty() {
            return Err(DataPhaseError::Emission);
        }
        let expected = self.function.result_type_ids.first();
        if expected.map(String::as_str) != value.as_ref().map(|v| v.type_id.as_str()) {
            return Err(DataPhaseError::Emission);
        }
        if !self.live_constructions.is_empty() {
            let current = self.current;
            let next = self.block(ControlNodeTag::Jump);
            let target = self.function.blocks[next].node.id.clone();
            let actions = self
                .live_constructions
                .values()
                .map(|v| v::PracticalConstructionAction::Discard {
                    construction_id: v.id.clone(),
                    actor_id: self.function.id.clone(),
                })
                .collect();
            self.function.blocks[current].node.tag = ControlNodeTag::Operation;
            self.function.blocks[current].node.normal_successor_ids = vec![target];
            self.function.blocks[current].construction_actions = actions;
            self.live_constructions.clear();
            self.current = next;
        }
        let block = &mut self.function.blocks[self.current];
        block.node.tag = ControlNodeTag::Return;
        block.node.abrupt = Some(AbruptCompletion::Return {
            value_type_id: value.as_ref().map(|v| v.type_id.clone()),
        });
        block.return_value_ids = value.into_iter().map(|v| v.id).collect();
        self.ended = true;
        Ok(())
    }
}
impl Emitter<'_> {
    fn compile_all(&mut self) -> Result<(), DataPhaseError> {
        let source = self.source;
        // The frozen closure admits long call chains. Schedule the already
        // validated DAG explicitly so Rust expression frames do not accumulate
        // across source calls; compile's recursive lookup then hits a finished body.
        let ids = source
            .callables()
            .iter()
            .map(|c| c.id())
            .collect::<BTreeSet<_>>();
        let getters = source
            .callables()
            .iter()
            .filter(|c| c.is_property_getter())
            .map(|c| {
                (
                    format!(
                        "{}.{}",
                        c.identity()["owner"].as_str().unwrap(),
                        c.identity()["name"]
                            .as_str()
                            .unwrap()
                            .strip_prefix("get_")
                            .unwrap()
                    ),
                    c.id(),
                )
            })
            .collect::<BTreeMap<_, _>>();
        let mut active = BTreeSet::new();
        for root in source.callables() {
            let mut pending = vec![(root.id(), false)];
            while let Some((id, finish)) = pending.pop() {
                if self.functions.contains_key(id) {
                    continue;
                }
                if finish {
                    self.compile(id)?;
                    active.remove(id);
                    continue;
                }
                if !active.insert(id) {
                    return Err(DataPhaseError::Cycle);
                }
                pending.push((id, true));
                for node in source.body(id).ok_or(DataPhaseError::Source)?.iter().rev() {
                    let target = if ids.contains(node.symbol()) {
                        Some(node.symbol())
                    } else if node.kind() == "PropertyReference" {
                        getters.get(node.symbol()).copied()
                    } else {
                        None
                    };
                    if let Some(target) = target {
                        pending.push((target, false));
                    }
                }
            }
        }
        Ok(())
    }

    fn invoke(
        &mut self,
        body: &mut Body,
        signature: ClosedOperationSignature,
        operands: Vec<TypedValueRef>,
    ) -> Result<TypedValueRef, DataPhaseError> {
        if operands
            .iter()
            .map(|v| &v.type_id)
            .ne(signature.argument_type_ids.iter())
        {
            return Err(DataPhaseError::Emission);
        }
        validate_closed_operation_signature(self.r, self.c, &signature)
            .map_err(|_| DataPhaseError::Emission)?;
        if let Some(previous) = self
            .signatures
            .insert(signature.id.clone(), signature.clone())
        {
            if previous != signature {
                return Err(DataPhaseError::Emission);
            }
        }
        let result = body.value(&signature.normal_result_type_id);
        let current = body.current;
        let next = body.block(ControlNodeTag::Jump);
        let normal = body.function.blocks[next].node.id.clone();
        let mut exceptional = vec![];
        for check in &signature.ordered_checks {
            if check.tag != RequiredCheckTag::Exception {
                continue;
            }
            let exception = check
                .failure_type_id
                .as_ref()
                .ok_or(DataPhaseError::Emission)?;
            if let Some(arm) = builtin_exception_arms()
                .into_iter()
                .find(|arm| arm.type_id == *exception)
            {
                body.function.blocks[current].exception_values.push(
                    v::PracticalVirExceptionValue {
                        check_id: check.id.clone(),
                        value: MonomorphicValue::ClosedException {
                            type_id: EXCEPTION_TYPE_ID.into(),
                            tag: arm.tag,
                            source_type_id: None,
                            payload: None,
                        },
                    },
                );
            } else if !matches!(
                signature.tag,
                ClosedOperationTag::SourceCall | ClosedOperationTag::ConstructorExecute
            ) || !self
                .control
                .as_ref()
                .is_some_and(|c| c.universe().arm(exception).is_some())
            {
                return Err(DataPhaseError::Emission);
            }
            body.exception_construction_origins
                .entry(body.function.blocks[current].node.id.clone())
                .or_default()
                .extend(body.live_constructions.keys().cloned());
            let discarded = body
                .live_objects
                .iter()
                .filter(|(_, v)| {
                    signature.tag != ClosedOperationTag::ConstructorExecute
                        || operands.first().is_none_or(|a| a.id != v.id)
                })
                .map(|(origin, _)| origin.clone())
                .collect::<BTreeSet<_>>();
            body.exception_object_origins
                .entry(body.function.blocks[current].node.id.clone())
                .or_default()
                .extend(discarded);
            exceptional.push(ExceptionalSuccessor {
                check_id: check.id.clone(),
                exception_type_id: exception.clone(),
                target_id: "pending.exit".into(),
            });
        }
        if self
            .c
            .metadata
            .get(&result.type_id)
            .is_some_and(|m| template_name(&m.template_id) == Some("sequence_construction"))
            && signature.id.ends_with(".allocate")
        {
            body.live_constructions
                .insert(result.id.clone(), result.clone());
            body.construction_lengths
                .insert(result.id.clone(), operands[0].clone());
        } else if let Some(receiver) = operands.first().filter(|r| {
            self.c
                .metadata
                .get(&r.type_id)
                .is_some_and(|m| template_name(&m.template_id) == Some("sequence_construction"))
        }) {
            let origin = body
                .live_constructions
                .iter()
                .find(|(_, v)| v.id == receiver.id)
                .map(|(k, _)| k.clone())
                .ok_or(DataPhaseError::Emission)?;
            if signature.id.ends_with(".fill")
                || signature.id.ends_with(".rewrite")
                || signature.id.ends_with(".freeze")
            {
                for value in body.variables.values_mut() {
                    if value.id == receiver.id {
                        *value = result.clone();
                    }
                }
                if signature.id.ends_with(".freeze") {
                    body.live_constructions.remove(&origin);
                    body.published_constructions
                        .insert(receiver.id.clone(), result.clone());
                } else {
                    body.live_constructions.insert(origin, result.clone());
                }
            }
        }
        if signature.id.starts_with("object.begin.") {
            body.live_objects.insert(result.id.clone(), result.clone());
        } else if signature.id.starts_with("object.write.")
            || signature.id.starts_with("object.finalize.")
            || signature.tag == ClosedOperationTag::ConstructorExecute
        {
            let receiver = operands.first().ok_or(DataPhaseError::Emission)?;
            let origin = body
                .live_objects
                .iter()
                .find(|(_, v)| v.id == receiver.id)
                .map(|(id, _)| id.clone())
                .ok_or(DataPhaseError::Emission)?;
            if signature.id.starts_with("object.finalize.") {
                body.live_objects.remove(&origin);
            } else {
                body.live_objects.insert(origin, result.clone());
                for variable in body.variables.values_mut() {
                    if variable.id == receiver.id {
                        *variable = result.clone();
                    }
                }
            }
        }
        let block = &mut body.function.blocks[current];
        block.node.tag = ControlNodeTag::Operation;
        block.node.normal_successor_ids = vec![normal.clone()];
        block.node.exceptional_successors = exceptional.clone();
        block.invocation = Some(OperationInvocation {
            operation_id: signature.id,
            operands,
            result: result.clone(),
            ordered_check_ids: signature
                .ordered_checks
                .iter()
                .map(|c| c.id.clone())
                .collect(),
            normal_successor_id: normal,
            exceptional_successors: exceptional,
        });
        body.current = next;
        Ok(result)
    }
    fn publish(
        &mut self,
        body: &mut Body,
        value: TypedValueRef,
    ) -> Result<TypedValueRef, DataPhaseError> {
        if !self
            .c
            .metadata
            .get(&value.type_id)
            .is_some_and(|m| template_name(&m.template_id) == Some("sequence_construction"))
        {
            return Ok(value);
        }
        if let Some(published) = body.published_constructions.get(&value.id) {
            return Ok(published.clone());
        }
        let signature = self
            .signatures
            .get(&format!("{}.freeze", value.type_id))
            .cloned()
            .ok_or(DataPhaseError::Emission)?;
        self.invoke(body, signature, vec![value])
    }
    fn coerce(
        &mut self,
        body: &mut Body,
        value: TypedValueRef,
        expected: &str,
    ) -> Result<TypedValueRef, DataPhaseError> {
        if value.type_id == expected {
            return Ok(value);
        }
        if self.c.metadata.get(expected).is_some_and(|m| {
            template_name(&m.template_id) == Some("option")
                && m.argument_ids == [value.type_id.clone()]
        }) {
            let signature = self
                .signatures
                .get(&format!("{expected}.some"))
                .cloned()
                .ok_or(DataPhaseError::Emission)?;
            self.invoke(body, signature, vec![value])
        } else {
            Err(DataPhaseError::Emission)
        }
    }
    fn call_arguments(
        &mut self,
        body: &mut Body,
        signature: &ClosedOperationSignature,
        args: Vec<TypedValueRef>,
    ) -> Result<Vec<TypedValueRef>, DataPhaseError> {
        if args.len() != signature.argument_type_ids.len() {
            return Err(DataPhaseError::Emission);
        }
        args.into_iter()
            .zip(&signature.argument_type_ids)
            .map(|(value, expected)| self.coerce(body, value, expected))
            .collect()
    }
    fn string_invoke(
        &mut self,
        body: &mut Body,
        id: &str,
        operands: Vec<TypedValueRef>,
    ) -> Result<TypedValueRef, DataPhaseError> {
        let signature =
            strings::operation_signature(self.c, id).map_err(|_| DataPhaseError::Emission)?;
        if signature.argument_type_ids.len() != operands.len() {
            return Err(DataPhaseError::Emission);
        }
        let mut widened = vec![];
        for (value, expected) in operands.into_iter().zip(&signature.argument_type_ids) {
            let value = if value.type_id == *expected {
                value
            } else if value.type_id == STRING_TYPE_ID
                && self.c.metadata.get(expected).is_some_and(|m| {
                    template_name(&m.template_id) == Some("option")
                        && m.argument_ids == [STRING_TYPE_ID]
                })
            {
                self.invoke(
                    body,
                    self.signatures[&format!("{expected}.some")].clone(),
                    vec![value],
                )?
            } else {
                return Err(DataPhaseError::Emission);
            };
            widened.push(value);
        }
        self.invoke(body, signature, widened)
    }
    fn compile(&mut self, id: &str) -> Result<(), DataPhaseError> {
        if self.functions.contains_key(id) {
            return Ok(());
        }
        if !self.active.insert(id.into()) {
            return Err(DataPhaseError::Cycle);
        }
        let callable = self
            .source
            .callables()
            .iter()
            .find(|c| c.id() == id)
            .ok_or(DataPhaseError::Source)?;
        let constructor = callable.identity()["kind"] == "constructor";
        let owner = callable.identity()["owner"]
            .as_str()
            .ok_or(DataPhaseError::Source)?
            .to_owned();
        let args = callable
            .parameters()
            .iter()
            .map(|ty| {
                ClosedType::parse(ty)
                    .and_then(|ty| closed_type_id(self.b, &ty))
                    .map_err(|_| DataPhaseError::Emission)
            })
            .collect::<Result<Vec<_>, _>>()?;
        let result = ClosedType::parse(callable.result())
            .and_then(|ty| closed_type_id(self.b, &ty))
            .map_err(|_| DataPhaseError::Emission)?;
        let object_constructor = constructor && object_construction_root(self.r, &owner).is_ok();
        let result = if object_constructor {
            object_construction_signature(self.r, self.c, &format!("object.begin.{owner}"))
                .map_err(|_| DataPhaseError::Emission)?
                .normal_result_type_id
        } else {
            result
        };
        let mut body = Body {
            expression_values: BTreeMap::new(),
            function: v::PracticalVirFunction {
                control_protocol: None,
                object_protocol: object_constructor.then(|| v::PracticalObjectProtocol {
                    constructor_owner: Some(owner.clone()),
                    initializations: vec![],
                    exceptional_discards: vec![],
                }),
                id: id.into(),
                parameter_values: vec![],
                result_type_ids: if result == "mpk.csharp.value.unit.v1" {
                    vec![]
                } else {
                    vec![result.clone()]
                },
                blocks: vec![],
                loops: vec![],
                patterns: vec![],
                exception_regions: vec![],
                unwind_plans: vec![],
            },
            current: 0,
            next_value: 0,
            next_block: 0,
            ended: false,
            variables: BTreeMap::new(),
            fields: BTreeMap::new(),
            constructor,
            object_constructor,
            live_objects: BTreeMap::new(),
            exception_object_origins: BTreeMap::new(),
            owner,
            live_constructions: BTreeMap::new(),
            construction_lengths: BTreeMap::new(),
            published_constructions: BTreeMap::new(),
            exception_construction_origins: BTreeMap::new(),
            normal_construction_origins: BTreeMap::new(),
        };
        if object_constructor {
            let value = body.value(&result);
            body.live_objects.insert(value.id.clone(), value.clone());
            body.variables
                .insert("construction:this".into(), value.clone());
            body.function.parameter_values.push(value);
        }
        if !callable.is_static() && !constructor {
            let receiver = body.value(&body.owner.clone());
            body.variables.insert("this".into(), receiver.clone());
            body.function.parameter_values.push(receiver);
        }
        for (i, ty) in args.iter().enumerate() {
            let value = body.value(ty);
            body.variables
                .insert(format!("parameter:{i}"), value.clone());
            body.function.parameter_values.push(value);
        }
        let entry = body.block(ControlNodeTag::Entry);
        let first = body.block(ControlNodeTag::Jump);
        body.function.blocks[entry].node.normal_successor_ids =
            vec![body.function.blocks[first].node.id.clone()];
        body.current = first;
        if self.source.body(id).is_some_and(|b| b.is_empty()) {
            if let Some(name) = callable.identity()["name"]
                .as_str()
                .and_then(|n| n.strip_prefix("get_"))
            {
                let member = self
                    .r
                    .source_types
                    .get(&body.owner)
                    .and_then(|s| {
                        s.members.iter().find(|m| {
                            m.name == name && matches!(m.storage.as_str(), "get_auto" | "init_auto")
                        })
                    })
                    .ok_or(DataPhaseError::Source)?;
                let signature = source_field_operation(self.r, self.c, &member.id)
                    .map_err(|_| DataPhaseError::Emission)?;
                let receiver = body
                    .variables
                    .get("this")
                    .cloned()
                    .ok_or(DataPhaseError::Emission)?;
                let value = self.invoke(&mut body, signature, vec![receiver])?;
                body.finish(Some(value))?;
            }
        }
        let control = self.control.clone();
        let graph = control
            .as_ref()
            .and_then(|c| c.functions().iter().find(|f| f.callable_id == id));
        if let Some(graph) = graph.filter(|g| {
            !g.loops.is_empty()
                || g.nodes.iter().any(|n| {
                    matches!(
                        n.kind.as_str(),
                        "pattern_decision"
                            | "explicit_throw"
                            | "handler_entry"
                            | "handler_finally_entry"
                    )
                })
        }) {
            let control = control.as_ref().ok_or(DataPhaseError::Source)?;
            let handler = control
                .handlers()
                .iter()
                .find(|h| h.graph().callable_id == id)
                .ok_or(DataPhaseError::Source)?;
            self.control_body(&mut body, graph, handler, control.universe())?;
        } else {
            for node in trees(self.source.body(id).ok_or(DataPhaseError::Source)?)? {
                self.statement(&mut body, &node)?;
            }
        }
        if constructor && !body.ended {
            self.finish_constructor(&mut body)?;
        }
        if !body.ended {
            body.finish(None)?;
        }
        let exit = body.block(ControlNodeTag::Exit);
        let exit_id = body.function.blocks[exit].node.id.clone();
        body.function.blocks[exit].node.abrupt = Some(AbruptCompletion::Normal);
        if let Some(protocol) = &mut body.function.object_protocol {
            protocol
                .initializations
                .sort_by_key(|i| i.source_node_ordinal);
        }
        for plan in &mut body.function.unwind_plans {
            if plan.destination_node_id == "pending.exit" {
                plan.destination_node_id = exit_id.clone();
            }
        }
        let mut checks = BTreeMap::new();
        for block in &mut body.function.blocks {
            for edge in &mut block.node.exceptional_successors {
                if edge.target_id == "pending.exit" {
                    edge.target_id = exit_id.clone();
                    checks.insert(
                        edge.check_id.clone(),
                        RequiredCheck {
                            id: edge.check_id.clone(),
                            tag: RequiredCheckTag::Exception,
                            failure_type_id: Some(edge.exception_type_id.clone()),
                        },
                    );
                }
            }
            if let Some(invocation) = &mut block.invocation {
                invocation.exceptional_successors = block.node.exceptional_successors.clone();
            }
            for edge in &block.node.exceptional_successors {
                if body
                    .function
                    .unwind_plans
                    .iter()
                    .any(|p| p.source_node_id == block.node.id && p.check_id == edge.check_id)
                {
                    continue;
                }
                body.function.unwind_plans.push(ExceptionUnwindPlan {
                    search_entry_node_id: None,
                    source_node_id: block.node.id.clone(),
                    check_id: edge.check_id.clone(),
                    from_region_id: block.node.region_stack.last().cloned(),
                    selected_handler_region_id: None,
                    finally_region_ids: vec![],
                    destination_node_id: edge.target_id.clone(),
                });
            }
            block
                .literal_values
                .sort_by(|a, b| a.result.id.cmp(&b.result.id));
        }
        let mut sequence_cleanup = BTreeMap::<String, BTreeSet<String>>::new();
        for block in &body.function.blocks {
            if let Some(origins) = body.exception_construction_origins.get(&block.node.id) {
                for edge in &block.node.exceptional_successors {
                    sequence_cleanup
                        .entry(edge.target_id.clone())
                        .or_default()
                        .extend(origins.iter().cloned());
                }
            }
        }
        let finally_entries = body
            .function
            .blocks
            .iter()
            .filter(|b| b.node.tag == ControlNodeTag::FinallyEntry)
            .map(|b| b.node.id.clone())
            .collect::<BTreeSet<_>>();
        for block in &body.function.blocks {
            if let Some(origins) = body.normal_construction_origins.get(&block.node.id) {
                for target in block
                    .node
                    .normal_successor_ids
                    .iter()
                    .filter(|id| finally_entries.contains(*id))
                {
                    sequence_cleanup
                        .entry(target.clone())
                        .or_default()
                        .extend(origins.iter().cloned());
                }
            }
        }
        for block in &mut body.function.blocks {
            if let Some(origins) = sequence_cleanup.remove(&block.node.id) {
                for construction_id in origins {
                    let action = v::PracticalConstructionAction::Discard {
                        construction_id,
                        actor_id: body.function.id.clone(),
                    };
                    if !block.construction_actions.contains(&action) {
                        block.construction_actions.push(action);
                    }
                }
            }
        }
        let mut cleanup = BTreeMap::<String, BTreeSet<String>>::new();
        for block in &body.function.blocks {
            if let Some(origins) = body
                .exception_object_origins
                .get(&block.node.id)
                .filter(|o| !o.is_empty())
            {
                for edge in &block.node.exceptional_successors {
                    cleanup
                        .entry(edge.target_id.clone())
                        .or_default()
                        .extend(origins.iter().cloned());
                }
            }
        }
        if !cleanup.is_empty() {
            body.object_protocol().exceptional_discards = cleanup
                .into_iter()
                .map(|(exit_node_id, origins)| v::PracticalObjectDiscard {
                    exit_node_id,
                    origin_value_ids: origins.into_iter().collect(),
                })
                .collect();
        }
        self.signatures.insert(
            id.into(),
            ClosedOperationSignature {
                id: id.into(),
                tag: if object_constructor {
                    ClosedOperationTag::ConstructorExecute
                } else {
                    ClosedOperationTag::SourceCall
                },
                argument_type_ids: body
                    .function
                    .parameter_values
                    .iter()
                    .map(|v| v.type_id.clone())
                    .collect(),
                normal_result_type_id: result,
                ordered_checks: checks.into_values().collect(),
            },
        );
        self.functions.insert(id.into(), body.function);
        self.active.remove(id);
        Ok(())
    }
    #[allow(clippy::too_many_arguments)]
    fn branch(
        &mut self,
        body: &mut Body,
        condition: TypedValueRef,
        yes: Option<&Expr<'_>>,
        no: Option<&Expr<'_>>,
        yes_constant: Option<bool>,
        no_constant: Option<bool>,
        statement: bool,
    ) -> Result<Option<TypedValueRef>, DataPhaseError> {
        self.branch_with_value(
            body,
            condition,
            yes,
            no,
            yes_constant,
            no_constant,
            statement,
            None,
            None,
            None,
        )
    }
    #[allow(clippy::too_many_arguments)]
    fn branch_with_value(
        &mut self,
        body: &mut Body,
        condition: TypedValueRef,
        yes: Option<&Expr<'_>>,
        no: Option<&Expr<'_>>,
        yes_constant: Option<bool>,
        no_constant: Option<bool>,
        statement: bool,
        yes_invocation: Option<(ClosedOperationSignature, Vec<TypedValueRef>)>,
        conditional_result: Option<&str>,
        no_invocation: Option<(ClosedOperationSignature, Vec<TypedValueRef>)>,
    ) -> Result<Option<TypedValueRef>, DataPhaseError> {
        if condition.type_id != BOOL_TYPE_ID {
            return Err(DataPhaseError::Emission);
        }
        let start = body.current;
        let yes_start = body.block(ControlNodeTag::Jump);
        let no_start = body.block(ControlNodeTag::Jump);
        let targets = vec![
            body.function.blocks[yes_start].node.id.clone(),
            body.function.blocks[no_start].node.id.clone(),
        ];
        let node = &mut body.function.blocks[start];
        node.node.tag = ControlNodeTag::Branch;
        node.node.normal_successor_ids = targets;
        node.node.condition_type_id = Some(BOOL_TYPE_ID.into());
        node.condition_value_id = Some(condition.id);
        let variables = body.variables.clone();
        let fields = body.fields.clone();
        let objects = body.live_objects.clone();
        let live = body.live_constructions.clone();
        let lengths = body.construction_lengths.clone();
        let published = body.published_constructions.clone();
        let mut states = vec![];
        let mut ends = vec![];
        for (start, expression, constant) in
            [(yes_start, yes, yes_constant), (no_start, no, no_constant)]
        {
            body.current = start;
            body.ended = false;
            body.variables = variables.clone();
            body.fields = fields.clone();
            body.live_objects = objects.clone();
            body.live_constructions = live.clone();
            body.construction_lengths = lengths.clone();
            body.published_constructions = published.clone();
            let value = if let Some((signature, args)) = if start == yes_start {
                yes_invocation.as_ref()
            } else {
                no_invocation.as_ref()
            } {
                let mut value = self.invoke(body, signature.clone(), args.clone())?;
                if let Some(expression) = expression {
                    body.variables.insert("conditional_receiver".into(), value);
                    value = self.expression(body, expression)?;
                }
                if let Some(target) = conditional_result {
                    let some = self
                        .signatures
                        .get(&format!("{target}.some"))
                        .cloned()
                        .ok_or(DataPhaseError::Emission)?;
                    value = self.invoke(body, some, vec![value])?;
                }
                Some(value)
            } else if let Some(target) = conditional_result.filter(|_| start == no_start) {
                Some(body.literal(MonomorphicValue::Option {
                    type_id: target.into(),
                    arm: OptionArm::None,
                    value: None,
                }))
            } else if let Some(value) = constant {
                Some(body.literal(MonomorphicValue::Bool {
                    type_id: BOOL_TYPE_ID.into(),
                    value,
                }))
            } else if let Some(expression) = expression {
                if statement {
                    self.statement(body, expression)?;
                    None
                } else {
                    Some(self.expression(body, expression)?)
                }
            } else {
                None
            };
            if !body.ended {
                if body.constructor && !body.object_constructor {
                    for member in &self.r.source_types[&body.owner].members {
                        if !body.fields.contains_key(&member.name) {
                            let id = closed_type_id(self.b, &member.ty)
                                .map_err(|_| DataPhaseError::Emission)?;
                            let value = body.literal(
                                domain_default(self.b, self.r, self.c, &id)
                                    .map_err(|_| DataPhaseError::Default)?,
                            );
                            body.fields.insert(member.name.clone(), value);
                        }
                    }
                }
                states.push((
                    body.live_constructions.clone(),
                    body.construction_lengths.clone(),
                    body.published_constructions.clone(),
                    body.live_objects.clone(),
                ));
                ends.push((
                    body.current,
                    value,
                    body.variables.clone(),
                    body.fields.clone(),
                ));
            }
        }
        if ends.is_empty() {
            body.ended = true;
            return Ok(None);
        }
        let join = body.block(ControlNodeTag::Jump);
        let join_id = body.function.blocks[join].node.id.clone();
        for (end, _, _, _) in &ends {
            body.function.blocks[*end].node.normal_successor_ids = vec![join_id.clone()];
        }
        body.current = join;
        body.ended = false;
        body.variables.clear();
        body.fields.clear();
        fn merge(
            body: &mut Body,
            ends: &[(usize, TypedValueRef)],
        ) -> Result<TypedValueRef, DataPhaseError> {
            let first = &ends[0].1;
            if ends.iter().any(|(_, value)| value.type_id != first.type_id) {
                return Err(DataPhaseError::Emission);
            }
            if ends.iter().all(|(_, value)| value == first) {
                return Ok(first.clone());
            }
            let mut incoming = ends
                .iter()
                .map(|(index, value)| v::PracticalVirPhiIncoming {
                    predecessor_node_id: body.function.blocks[*index].node.id.clone(),
                    value_id: value.id.clone(),
                })
                .collect::<Vec<_>>();
            incoming.sort_by(|a, b| a.predecessor_node_id.cmp(&b.predecessor_node_id));
            if let Some(phi) = body.function.blocks[body.current]
                .phi_values
                .iter()
                .find(|phi| phi.incoming == incoming && phi.value.type_id == first.type_id)
            {
                return Ok(phi.value.clone());
            }
            let result = body.value(&first.type_id);
            body.function.blocks[body.current]
                .phi_values
                .push(v::PracticalVirPhiValue {
                    value: result.clone(),
                    incoming,
                });
            Ok(result)
        }
        body.live_constructions.clear();
        body.construction_lengths.clear();
        body.published_constructions.clear();
        let origins = states[0].0.keys().collect::<BTreeSet<_>>();
        if states
            .iter()
            .any(|state| state.0.keys().collect::<BTreeSet<_>>() != origins)
        {
            return Err(DataPhaseError::Emission);
        }
        for origin in origins {
            let incoming = ends
                .iter()
                .zip(&states)
                .map(|(end, state)| (end.0, state.0[origin].clone()))
                .collect::<Vec<_>>();
            let value = merge(body, &incoming)?;
            body.live_constructions.insert(origin.clone(), value);
            let length = states[0].1.get(origin).ok_or(DataPhaseError::Emission)?;
            if states
                .iter()
                .any(|state| state.1.get(origin) != Some(length))
            {
                return Err(DataPhaseError::Emission);
            }
            body.construction_lengths
                .insert(origin.clone(), length.clone());
        }
        for key in states[0].2.keys() {
            if states.iter().all(|state| state.2.contains_key(key)) {
                let incoming = ends
                    .iter()
                    .zip(&states)
                    .map(|(end, state)| (end.0, state.2[key].clone()))
                    .collect::<Vec<_>>();
                let value = merge(body, &incoming)?;
                body.published_constructions.insert(key.clone(), value);
            }
        }
        body.live_objects.clear();
        let object_origins = states[0].3.keys().cloned().collect::<BTreeSet<_>>();
        if states
            .iter()
            .any(|state| state.3.keys().cloned().collect::<BTreeSet<_>>() != object_origins)
        {
            return Err(DataPhaseError::Emission);
        }
        for origin in object_origins {
            let incoming = ends
                .iter()
                .zip(&states)
                .map(|(end, state)| (end.0, state.3[&origin].clone()))
                .collect::<Vec<_>>();
            let value = merge(body, &incoming)?;
            body.live_objects.insert(origin, value);
        }
        for key in ends[0].2.keys() {
            if ends.iter().all(|(_, _, vars, _)| vars.contains_key(key)) {
                let incoming = ends
                    .iter()
                    .map(|(end, _, vars, _)| (*end, vars[key].clone()))
                    .collect::<Vec<_>>();
                let value = merge(body, &incoming)?;
                body.variables.insert(key.clone(), value);
            }
        }
        for key in ends[0].3.keys() {
            if ends
                .iter()
                .all(|(_, _, _, fields)| fields.contains_key(key))
            {
                let incoming = ends
                    .iter()
                    .map(|(end, _, _, fields)| (*end, fields[key].clone()))
                    .collect::<Vec<_>>();
                let value = merge(body, &incoming)?;
                body.fields.insert(key.clone(), value);
            }
        }
        let result = if statement {
            None
        } else {
            let incoming = ends
                .iter()
                .map(|(end, value, _, _)| {
                    Ok((*end, value.clone().ok_or(DataPhaseError::Emission)?))
                })
                .collect::<Result<Vec<_>, DataPhaseError>>()?;
            Some(merge(body, &incoming)?)
        };
        body.function.blocks[join]
            .phi_values
            .sort_by(|a, b| a.value.id.cmp(&b.value.id));
        Ok(result)
    }
    fn default_value(&self, type_id: &str) -> Result<MonomorphicValue, DataPhaseError> {
        domain::default_with_obligations(
            self.b,
            self.r,
            self.c,
            type_id,
            !self.source.has_structural_default(type_id),
        )
        .map_err(|_| DataPhaseError::Default)
    }
    fn finish_constructor(&mut self, body: &mut Body) -> Result<(), DataPhaseError> {
        if body.object_constructor {
            let value = body
                .variables
                .get("construction:this")
                .cloned()
                .ok_or(DataPhaseError::Emission)?;
            return body.finish(Some(value));
        }
        let source = self
            .r
            .source_types
            .get(&body.owner)
            .ok_or(DataPhaseError::Source)?;
        let mut operands = vec![];
        for member in &source.members {
            let value = if let Some(value) = body.fields.get(&member.name) {
                value.clone()
            } else {
                let ty =
                    closed_type_id(self.b, &member.ty).map_err(|_| DataPhaseError::Emission)?;
                body.literal(
                    domain_default(self.b, self.r, self.c, &ty)
                        .map_err(|_| DataPhaseError::Default)?,
                )
            };
            operands.push(value);
        }
        let signature = source_value_constructor_operation(self.r, self.c, &body.owner)
            .map_err(|_| DataPhaseError::Emission)?;
        let value = self.invoke(body, signature, operands)?;
        body.finish(Some(value))?;
        Ok(())
    }
    fn statement(&mut self, body: &mut Body, node: &Expr<'_>) -> Result<(), DataPhaseError> {
        if body.ended {
            return Err(DataPhaseError::Emission);
        }
        match node.operation.kind() {
            "ConstructorInitializer"
                if self.source.control_lowering().is_some()
                    && node.children.len() == 1
                    && node.children[0].operation.symbol()
                        == "System.Runtime|System.Exception.Exception()"
                    && validate_control_source(self.b, self.source)?
                        .definitions()
                        .iter()
                        .any(|d| d.type_id == body.owner) => {}
            "Block" | "VariableDeclarationGroup" | "VariableDeclaration" => {
                for child in &node.children {
                    self.statement(body, child)?;
                }
            }
            "ExpressionStatement" => {
                self.expression(body, node.children.first().ok_or(DataPhaseError::Emission)?)?;
            }
            "Throw" if self.source.control_lowering().is_some() => self.source_throw(body, node)?,
            "Return" => {
                if body.constructor && node.children.is_empty() {
                    return self.finish_constructor(body);
                }
                let value = node
                    .children
                    .first()
                    .map(|c| self.expression(body, c))
                    .transpose()?;
                let expected = body.function.result_type_ids.first().cloned();
                let value = value.map(|v| self.publish(body, v)).transpose()?;
                let value = value
                    .map(|v| {
                        self.coerce(
                            body,
                            v,
                            expected.as_deref().ok_or(DataPhaseError::Emission)?,
                        )
                    })
                    .transpose()?;
                body.finish(value)?;
            }
            "VariableDeclarator" => {
                if node.children.is_empty() {
                    return Ok(());
                }
                let initializer = node.children.first().ok_or(DataPhaseError::Emission)?;
                let value = self.expression(body, initializer)?;
                let value = if fresh_array_expression(initializer) {
                    value
                } else {
                    self.publish(body, value)?
                };
                if body
                    .variables
                    .insert(node.operation.symbol().into(), value)
                    .is_some()
                {
                    return Err(DataPhaseError::Emission);
                }
            }
            "Conditional" => {
                let condition =
                    self.expression(body, node.children.first().ok_or(DataPhaseError::Emission)?)?;
                self.branch(
                    body,
                    condition,
                    node.children.get(1),
                    node.children.get(2),
                    None,
                    None,
                    true,
                )?;
            }
            "ConstructorInitializer" if body.constructor => {
                let call = node.children.first().ok_or(DataPhaseError::Emission)?;
                self.compile(call.operation.symbol())?;
                let signature = self
                    .signatures
                    .get(call.operation.symbol())
                    .cloned()
                    .ok_or(DataPhaseError::Emission)?;
                let mut positional = BTreeMap::new();
                for argument in call
                    .children
                    .iter()
                    .filter(|c| c.operation.kind() == "Argument")
                {
                    let value = self.expression(body, argument)?;
                    let value = self.publish(body, value)?;
                    let ordinal = argument
                        .operation
                        .symbol()
                        .strip_prefix("argument:")
                        .and_then(|n| n.parse::<usize>().ok())
                        .ok_or(DataPhaseError::Emission)?;
                    if positional.insert(ordinal, value).is_some() {
                        return Err(DataPhaseError::Emission);
                    }
                }
                let mut args = vec![];
                if body.object_constructor {
                    if signature.tag != ClosedOperationTag::ConstructorExecute {
                        return Err(DataPhaseError::Emission);
                    }
                    args.push(
                        body.variables
                            .get("construction:this")
                            .cloned()
                            .ok_or(DataPhaseError::Emission)?,
                    );
                } else if signature.normal_result_type_id != body.owner {
                    return Err(DataPhaseError::Emission);
                }
                for (expected, (ordinal, value)) in positional.into_iter().enumerate() {
                    if expected != ordinal {
                        return Err(DataPhaseError::Emission);
                    }
                    args.push(value);
                }
                let args = self.call_arguments(body, &signature, args)?;
                let value = self.invoke(body, signature, args)?;
                if !body.object_constructor {
                    let members = self.r.source_types[&body.owner].members.clone();
                    for member in members {
                        let signature = source_field_operation(self.r, self.c, &member.id)
                            .map_err(|_| DataPhaseError::Emission)?;
                        let field = self.invoke(body, signature, vec![value.clone()])?;
                        body.fields.insert(member.name, field);
                    }
                }
            }
            "Empty" => {}
            _ => {
                self.expression(body, node)?;
            }
        }
        Ok(())
    }
    fn receiver_expression(
        &mut self,
        body: &mut Body,
        node: &Expr<'_>,
    ) -> Result<TypedValueRef, DataPhaseError> {
        // Roslyn's flow-narrowed reference type does not change the stored
        // optional SSA type. Delay its null check until the operation's call
        // point, after any arguments or array index have been evaluated.
        if matches!(
            node.operation.kind(),
            "ParameterReference" | "LocalReference"
        ) {
            if let Some(value) = body.variables.get(node.operation.symbol()).filter(|v| {
                domain::reference_value_signature(
                    self.r,
                    self.c,
                    &format!("reference.value.{}", v.type_id),
                )
                .is_ok()
            }) {
                return Ok(value.clone());
            }
        }
        self.expression(body, node)
    }
    fn expression(
        &mut self,
        body: &mut Body,
        node: &Expr<'_>,
    ) -> Result<TypedValueRef, DataPhaseError> {
        if let Some(value) = body.expression_values.get(&node.ordinal) {
            return Ok(value.clone());
        }
        let fail = DataPhaseError::Emission;
        let op = node.operation;
        let get = |i| node.children.get(i).ok_or(DataPhaseError::Emission);
        let step = self
            .source
            .callables()
            .iter()
            .find(|c| c.id() == body.function.id)
            .and_then(|c| {
                c.data_steps()
                    .iter()
                    .find(|s| s.node_ordinal == node.ordinal)
            })
            .cloned();
        let array_write_mode = step
            .as_ref()
            .filter(|s| s.family == "array")
            .map(|s| s.operation.clone());
        if let Some(step) = step.filter(|s| s.family != "array") {
            if step.operation == "string.literal.decode" || step.operation.ends_with(".literal") {
                let ty = parse_data_type_key(self.b, op.type_key().ok_or(fail.clone())?)?;
                let id = closed_type_id(self.b, &ClosedType::parse(&ty).map_err(|_| fail.clone())?)
                    .map_err(|_| fail.clone())?;
                return Ok(body.literal(source_literal(&id, op.constant().ok_or(fail)?)?));
            }
            fn find<'a, 'b>(node: &'a Expr<'b>, ordinal: usize) -> Option<&'a Expr<'b>> {
                if node.ordinal == ordinal {
                    return Some(node);
                }
                node.children.iter().find_map(|child| find(child, ordinal))
            }
            if step.family == "domain" && step.operation == "nullable.coalesce" {
                if step.operand_ordinals.len() != 2 {
                    return Err(fail);
                }
                let left = self.expression(
                    body,
                    find(node, step.operand_ordinals[0]).ok_or(fail.clone())?,
                )?;
                let has = self
                    .signatures
                    .get(&format!("{}.has_value", left.type_id))
                    .cloned()
                    .ok_or(fail.clone())?;
                let get = self
                    .signatures
                    .get(&format!("{}.value", left.type_id))
                    .cloned()
                    .ok_or(fail.clone())?;
                let condition = self.invoke(body, has, vec![left.clone()])?;
                return self
                    .branch_with_value(
                        body,
                        condition,
                        None,
                        Some(find(node, step.operand_ordinals[1]).ok_or(fail.clone())?),
                        None,
                        None,
                        false,
                        Some((get, vec![left])),
                        None,
                        None,
                    )?
                    .ok_or(fail);
            }
            if step.family == "domain" && step.operation == "reference.conditional_access" {
                if step.operand_ordinals.len() != 2 {
                    return Err(fail);
                }
                let left = self.expression(
                    body,
                    find(node, step.operand_ordinals[0]).ok_or(fail.clone())?,
                )?;
                let has = self
                    .signatures
                    .get(&format!("{}.has_value", left.type_id))
                    .cloned()
                    .ok_or(fail.clone())?;
                let get = domain::reference_value_signature(
                    self.r,
                    self.c,
                    &format!("reference.value.{}", left.type_id),
                )
                .map_err(|_| fail.clone())?;
                let condition = self.invoke(body, has, vec![left.clone()])?;
                let ty = parse_data_type_key(self.b, op.type_key().ok_or(fail.clone())?)?;
                let result =
                    closed_type_id(self.b, &ClosedType::parse(&ty).map_err(|_| fail.clone())?)
                        .map_err(|_| fail.clone())?;
                return self
                    .branch_with_value(
                        body,
                        condition,
                        Some(find(node, step.operand_ordinals[1]).ok_or(fail.clone())?),
                        None,
                        None,
                        None,
                        false,
                        Some((get, vec![left])),
                        Some(&result),
                        None,
                    )?
                    .ok_or(fail);
            }
            let mut values = BTreeMap::new();
            let structural_null_type = if step.family == "structural" {
                step.operand_ordinals
                    .iter()
                    .filter_map(|ordinal| find(node, *ordinal))
                    .find(|operand| operand.operation.constant() != Some("null"))
                    .and_then(|operand| operand.operation.type_key())
                    .map(|key| {
                        parse_data_type_key(self.b, key).and_then(|ty| {
                            ClosedType::parse(&ty)
                                .and_then(|ty| closed_type_id(self.b, &ty))
                                .map_err(|_| DataPhaseError::Emission)
                        })
                    })
                    .transpose()?
            } else {
                None
            };
            for (&ordinal, &argument) in step.operand_ordinals.iter().zip(&step.argument_ordinals) {
                let operand = find(node, ordinal).ok_or(fail.clone())?;
                let value = if let Some(id) = structural_null_type
                    .as_ref()
                    .filter(|_| operand.operation.constant() == Some("null"))
                {
                    body.literal(MonomorphicValue::Option {
                        type_id: id.clone(),
                        arm: OptionArm::None,
                        value: None,
                    })
                } else {
                    self.expression(body, operand)?
                };
                values.insert(argument, value);
            }
            let args = values.into_values().collect::<Vec<_>>();
            if step.family == "structural" {
                if args.len() != 2
                    || args[0].type_id != args[1].type_id
                    || !matches!(
                        step.operation.as_str(),
                        "structural_equal" | "structural_not_equal"
                    )
                {
                    return Err(fail);
                }
                let id = args[0].type_id.clone();
                generate_structural_program(self.b, self.r, self.c, &id)
                    .map_err(|_| fail.clone())?;
                let equal = self.invoke(
                    body,
                    ClosedOperationSignature {
                        id: format!("structural.equal.{id}"),
                        tag: ClosedOperationTag::StructuralEqual,
                        argument_type_ids: vec![id.clone(), id],
                        normal_result_type_id: BOOL_TYPE_ID.into(),
                        ordered_checks: vec![],
                    },
                    args,
                )?;
                return if step.operation == "structural_not_equal" {
                    self.invoke(
                        body,
                        scalar_operation_signature("boolean.not").map_err(|_| fail.clone())?,
                        vec![equal],
                    )
                } else {
                    Ok(equal)
                };
            }
            if step.family == "string" {
                let id = if step.operation == "string.interpolation.restricted" {
                    let shape = args
                        .iter()
                        .map(|arg| {
                            if arg.type_id == "mpk.csharp.value.char.v1" {
                                'c'
                            } else {
                                's'
                            }
                        })
                        .collect::<String>();
                    format!(
                        "string.interpolation.restricted.{}",
                        if shape.is_empty() { "empty" } else { &shape }
                    )
                } else {
                    step.operation
                };
                return self.string_invoke(body, &id, args);
            }
            let ty = parse_data_type_key(self.b, op.type_key().ok_or(fail.clone())?)?;
            let result = closed_type_id(self.b, &ClosedType::parse(&ty).map_err(|_| fail.clone())?)
                .map_err(|_| fail.clone())?;
            if step.family == "domain" {
                if step.operation.starts_with("lifted.") {
                    let types = args.iter().map(|a| a.type_id.clone()).collect::<Vec<_>>();
                    let signature = domain::lifted_operation_signature(
                        self.r,
                        self.c,
                        &step.operation,
                        &types,
                        &result,
                    )
                    .map_err(|_| fail.clone())?;
                    return self.invoke(body, signature, args);
                }
                let mut args = args;
                let (id, operation) = match step.operation.as_str() {
                    "nullable.none" => (result.clone(), "none"),
                    "nullable.some" => (result.clone(), "some"),
                    "nullable.has_value" => (
                        args.first().ok_or(fail.clone())?.type_id.clone(),
                        "has_value",
                    ),
                    "nullable.value" => {
                        (args.first().ok_or(fail.clone())?.type_id.clone(), "value")
                    }
                    "nullable.value_or" | "nullable.value_or_default" => {
                        let id = args.first().ok_or(fail.clone())?.type_id.clone();
                        if step.operation == "nullable.value_or_default" {
                            args.push(
                                body.literal(
                                    domain_default(self.b, self.r, self.c, &result)
                                        .map_err(|_| fail.clone())?,
                                ),
                            );
                        }
                        (id, "value_or")
                    }
                    _ => return Err(fail),
                };
                let signature = self
                    .signatures
                    .get(&format!("{id}.{operation}"))
                    .cloned()
                    .ok_or(fail.clone())?;
                if signature.normal_result_type_id != result {
                    return Err(fail);
                }
                return self.invoke(body, signature, args);
            }
            let types = args.iter().map(|a| a.type_id.clone()).collect::<Vec<_>>();
            let exceptions = if step.family == "numeric" {
                NumericOperation::new(
                    &step.operation,
                    &types,
                    &result,
                    (!step.rounding.is_empty()).then_some(step.rounding.as_str()),
                )
                .map_err(|_| fail.clone())?
                .exception_types()
            } else {
                BusinessOperation::new(&step.operation, &types, &result)
                    .map_err(|_| fail.clone())?
                    .exception_types()
            };
            return self.invoke(
                body,
                ClosedOperationSignature {
                    id: if step.operation == "decimal.round" {
                        format!("decimal.round.{}.{}", step.rounding, types.len())
                    } else {
                        step.operation
                    },
                    tag: ClosedOperationTag::Data,
                    argument_type_ids: types,
                    normal_result_type_id: result,
                    ordered_checks: exceptions
                        .into_iter()
                        .map(exception_check)
                        .collect::<Result<_, _>>()?,
                },
                args,
            );
        }
        let mut result = match op.kind() {
            "Argument" | "VariableInitializer" | "Parenthesized" => {
                self.expression(body, get(0)?)?
            }
            "ParameterReference" | "LocalReference" => body
                .variables
                .get(op.symbol())
                .cloned()
                .ok_or(fail.clone())?,
            "InstanceReference" => body.variables.get("this").cloned().ok_or(fail.clone())?,
            "ConditionalAccessInstance" => body
                .variables
                .get("conditional_receiver")
                .cloned()
                .ok_or(fail.clone())?,
            "Literal" | "FieldReference" if node.children.is_empty() && op.constant().is_some() => {
                let ty = parse_data_type_key(self.b, op.type_key().ok_or(fail.clone())?)?;
                let id = ClosedType::parse(&ty)
                    .and_then(|ty| closed_type_id(self.b, &ty))
                    .map_err(|_| fail.clone())?;
                let mut value = source_literal(&id, op.constant().ok_or(fail.clone())?)?;
                if let MonomorphicValue::Signed { value: carrier, .. }
                | MonomorphicValue::Unsigned { value: carrier, .. } = &value
                {
                    if id == "mpk.csharp.value.day_of_week.v1" {
                        value = MonomorphicValue::Enum {
                            type_id: id.clone(),
                            underlying: "i32".into(),
                            carrier: carrier.clone(),
                        };
                    } else if let Some(source) = self
                        .r
                        .source_types
                        .get(&id)
                        .filter(|s| s.kind == SourceKind::Enum)
                    {
                        value = MonomorphicValue::Enum {
                            type_id: id.clone(),
                            underlying: source.enum_underlying.clone().ok_or(fail.clone())?,
                            carrier: carrier.clone(),
                        };
                    }
                }
                body.literal(value)
            }
            "Conditional" => {
                let condition = self.expression(body, get(0)?)?;
                self.branch(
                    body,
                    condition,
                    Some(get(1)?),
                    Some(get(2)?),
                    None,
                    None,
                    false,
                )?
                .ok_or(fail.clone())?
            }
            "IsPattern" => {
                let value = self.receiver_expression(body, get(0)?)?;
                let mut pattern = get(1)?;
                let mut negate = false;
                while pattern.operation.kind() == "NegatedPattern" {
                    negate = !negate;
                    pattern = pattern.children.first().ok_or(fail.clone())?;
                }
                if pattern.operation.kind() != "ConstantPattern"
                    || pattern
                        .children
                        .first()
                        .is_none_or(|n| n.operation.constant() != Some("null"))
                {
                    return Err(fail);
                }
                let present = if let Some(signature) = self
                    .signatures
                    .get(&format!("{}.has_value", value.type_id))
                    .cloned()
                {
                    self.invoke(body, signature, vec![value])?
                } else {
                    body.literal(MonomorphicValue::Bool {
                        type_id: BOOL_TYPE_ID.into(),
                        value: true,
                    })
                };
                if negate {
                    present
                } else {
                    self.invoke(
                        body,
                        scalar_operation_signature("boolean.not").map_err(|_| fail.clone())?,
                        vec![present],
                    )?
                }
            }
            "DefaultValue" => {
                let ty = parse_data_type_key(self.b, op.type_key().ok_or(fail.clone())?)?;
                let id = ClosedType::parse(&ty)
                    .and_then(|ty| closed_type_id(self.b, &ty))
                    .map_err(|_| fail.clone())?;
                body.literal(self.default_value(&id)?)
            }
            "CompoundAssignment" | "Increment" | "Decrement" => {
                let target = get(0)?;
                let (old, address) = if target.operation.kind() == "ArrayElementReference" {
                    let receiver =
                        self.expression(body, target.children.first().ok_or(fail.clone())?)?;
                    let index =
                        self.expression(body, target.children.get(1).ok_or(fail.clone())?)?;
                    let origin = body
                        .live_constructions
                        .iter()
                        .find(|(_, v)| v.id == receiver.id)
                        .map(|(k, _)| k.clone())
                        .ok_or(fail.clone())?;
                    let read = self
                        .signatures
                        .get(&format!("{}.read", receiver.type_id))
                        .cloned()
                        .ok_or(fail.clone())?;
                    (
                        self.invoke(body, read, vec![receiver, index.clone()])?,
                        Some((origin, index)),
                    )
                } else if matches!(
                    target.operation.kind(),
                    "LocalReference" | "ParameterReference"
                ) {
                    (self.expression(body, target)?, None)
                } else {
                    return Err(fail);
                };
                let token = |id: &str| {
                    id.strip_prefix("mpk.csharp.value.")
                        .and_then(|s| s.strip_suffix(".v1"))
                        .map(str::to_owned)
                        .ok_or(DataPhaseError::Emission)
                };
                let target_token = token(&old.type_id)?;
                let traits = op.traits().split('|').collect::<Vec<_>>();
                if traits.get(2) != Some(&"False") {
                    return Err(fail);
                }
                let checked = match traits.get(1) {
                    Some(&"True") => "checked",
                    Some(&"False") => "unchecked",
                    _ => return Err(fail),
                };
                let name = match *traits.first().ok_or(fail.clone())? {
                    "Add" | "Increment" => "add",
                    "Subtract" | "Decrement" => "subtract",
                    "Multiply" => "multiply",
                    "Divide" => "divide",
                    "Remainder" => "remainder",
                    "And" => "and",
                    "Or" => "or",
                    "ExclusiveOr" => "xor",
                    "LeftShift" => "left_shift",
                    "RightShift" => "right_shift",
                    "UnsignedRightShift" => "unsigned_right_shift",
                    _ => return Err(fail),
                };
                let mut right = if op.kind() == "CompoundAssignment" {
                    self.expression(body, get(1)?)?
                } else {
                    body.literal(MonomorphicValue::Signed {
                        type_id: I32_TYPE_ID.into(),
                        value: "1".into(),
                    })
                };
                let right_token = token(&right.type_id)?;
                if target_token == "bool" {
                    if op.kind() != "CompoundAssignment"
                        || right_token != "bool"
                        || !matches!(name, "and" | "or" | "xor")
                    {
                        return Err(fail);
                    }
                    let value = self.invoke(
                        body,
                        scalar_operation_signature(&format!("boolean.{name}"))
                            .map_err(|_| fail.clone())?,
                        vec![old, right],
                    )?;
                    if let Some((origin, index)) = address {
                        let current = body
                            .live_constructions
                            .get(&origin)
                            .cloned()
                            .ok_or(fail.clone())?;
                        let rewrite = self
                            .signatures
                            .get(&format!("{}.rewrite", current.type_id))
                            .cloned()
                            .ok_or(fail.clone())?;
                        self.invoke(body, rewrite, vec![current, index, value.clone()])?;
                    } else {
                        body.variables
                            .insert(target.operation.symbol().into(), value.clone());
                    }
                    return Ok(value);
                }
                let shift = name.ends_with("shift");
                let promoted = if shift || op.kind() != "CompoundAssignment" {
                    match target_token.as_str() {
                        "i8" | "u8" | "i16" | "u16" | "char" => "i32",
                        t => t,
                    }
                } else if target_token == "u64" || right_token == "u64" {
                    "u64"
                } else if target_token == "i64" || right_token == "i64" {
                    "i64"
                } else if target_token == "u32" || right_token == "u32" {
                    if matches!(target_token.as_str(), "i8" | "i16" | "i32")
                        || matches!(right_token.as_str(), "i8" | "i16" | "i32")
                    {
                        "i64"
                    } else {
                        "u32"
                    }
                } else {
                    "i32"
                };
                let mut left = old.clone();
                if target_token != promoted {
                    left = self.invoke(
                        body,
                        scalar_operation_signature(&format!(
                            "integer.convert.{target_token}.{promoted}.{checked}"
                        ))
                        .map_err(|_| fail.clone())?,
                        vec![left],
                    )?;
                }
                let right_target = if shift { "i32" } else { promoted };
                if right_token != right_target {
                    right = self.invoke(
                        body,
                        scalar_operation_signature(&format!(
                            "integer.convert.{right_token}.{right_target}.{checked}"
                        ))
                        .map_err(|_| fail.clone())?,
                        vec![right],
                    )?;
                }
                let mut value = self.invoke(
                    body,
                    scalar_operation_signature(&format!("integer.{promoted}.{name}.{checked}"))
                        .map_err(|_| fail.clone())?,
                    vec![left, right],
                )?;
                if promoted != target_token {
                    value = self.invoke(
                        body,
                        scalar_operation_signature(&format!(
                            "integer.convert.{promoted}.{target_token}.{checked}"
                        ))
                        .map_err(|_| fail.clone())?,
                        vec![value],
                    )?;
                }
                if let Some((origin, index)) = address {
                    let current = body
                        .live_constructions
                        .get(&origin)
                        .cloned()
                        .ok_or(fail.clone())?;
                    let rewrite = self
                        .signatures
                        .get(&format!("{}.rewrite", current.type_id))
                        .cloned()
                        .ok_or(fail.clone())?;
                    self.invoke(body, rewrite, vec![current, index, value.clone()])?;
                } else {
                    body.variables
                        .insert(target.operation.symbol().into(), value.clone());
                }
                if op.kind() != "CompoundAssignment" && traits.get(3) == Some(&"True") {
                    old
                } else {
                    value
                }
            }
            "SimpleAssignment" => {
                let target = get(0)?;
                if target.operation.kind() == "ArrayElementReference" {
                    let receiver =
                        self.expression(body, target.children.first().ok_or(fail.clone())?)?;
                    let index =
                        self.expression(body, target.children.get(1).ok_or(fail.clone())?)?;
                    let origin = body
                        .live_constructions
                        .iter()
                        .find(|(_, v)| v.id == receiver.id)
                        .map(|(k, _)| k.clone())
                        .ok_or(fail.clone())?;
                    // Simple assignment evaluates its RHS before the store's
                    // bounds checks; read-modify-write evaluates a read first.
                    let value = self.expression(body, get(1)?)?;
                    let value = self.publish(body, value)?;
                    let current = body
                        .live_constructions
                        .get(&origin)
                        .cloned()
                        .ok_or(fail.clone())?;
                    let mode = array_write_mode.as_deref().ok_or(fail.clone())?;
                    if mode == "fill_or_rewrite" {
                        let complete = self.invoke(
                            body,
                            sequence_construction_complete_signature(
                                self.c,
                                &format!("construction.complete.{}", current.type_id),
                            )
                            .map_err(|_| fail.clone())?,
                            vec![current.clone()],
                        )?;
                        let rewrite = self
                            .signatures
                            .get(&format!("{}.rewrite", current.type_id))
                            .cloned()
                            .ok_or(fail.clone())?;
                        let fill = self
                            .signatures
                            .get(&format!("{}.fill", current.type_id))
                            .cloned()
                            .ok_or(fail.clone())?;
                        let args = vec![current, index, value.clone()];
                        self.branch_with_value(
                            body,
                            complete,
                            None,
                            None,
                            None,
                            None,
                            true,
                            Some((rewrite, args.clone())),
                            None,
                            Some((fill, args)),
                        )?;
                        return Ok(value);
                    }
                    if !matches!(mode, "fill" | "rewrite") {
                        return Err(fail);
                    }
                    let rewrite = self
                        .signatures
                        .get(&format!("{}.{mode}", current.type_id))
                        .cloned()
                        .ok_or(fail.clone())?;
                    self.invoke(body, rewrite, vec![current, index, value.clone()])?;
                    return Ok(value);
                }
                let value = self.expression(body, get(1)?)?;
                let value = if target.operation.kind() == "LocalReference"
                    && fresh_array_expression(get(1)?)
                {
                    value
                } else {
                    self.publish(body, value)?
                };
                if matches!(
                    target.operation.kind(),
                    "LocalReference" | "ParameterReference"
                ) {
                    body.variables
                        .insert(target.operation.symbol().into(), value.clone());
                } else if target.operation.kind() == "Discard" {
                    // The RHS is still evaluated once; it introduces no name.
                } else if body.constructor
                    && matches!(
                        target.operation.kind(),
                        "FieldReference" | "PropertyReference"
                    )
                    && target
                        .children
                        .first()
                        .is_some_and(|n| n.operation.kind() == "InstanceReference")
                {
                    let name = target
                        .operation
                        .symbol()
                        .strip_prefix(&format!("{}.", body.owner))
                        .ok_or(fail.clone())?;
                    let source = &self.r.source_types[&body.owner];
                    let member = source
                        .members
                        .iter()
                        .find(|m| m.name == name)
                        .ok_or(fail.clone())?;
                    if value.type_id
                        != closed_type_id(self.b, &member.ty).map_err(|_| fail.clone())?
                    {
                        return Err(fail);
                    }
                    if body.object_constructor {
                        let signature = object_construction_signature(
                            self.r,
                            self.c,
                            &format!("object.write.{}", member.id),
                        )
                        .map_err(|_| DataPhaseError::Emission)?;
                        let state = body
                            .variables
                            .get("construction:this")
                            .cloned()
                            .ok_or(DataPhaseError::Emission)?;
                        self.invoke(body, signature, vec![state, value.clone()])?;
                    } else {
                        body.fields.insert(name.into(), value.clone());
                    }
                } else {
                    return Err(fail);
                }
                value
            }
            "FieldReference" | "PropertyReference" => {
                let receiver = get(0)?;
                if body.constructor && receiver.operation.kind() == "InstanceReference" {
                    let name = op
                        .symbol()
                        .strip_prefix(&format!("{}.", body.owner))
                        .ok_or(fail.clone())?;
                    if body.object_constructor {
                        let member = self.r.source_types[&body.owner]
                            .members
                            .iter()
                            .find(|m| m.name == name)
                            .ok_or(fail.clone())?;
                        let signature = object_construction_signature(
                            self.r,
                            self.c,
                            &format!("object.read.{}", member.id),
                        )
                        .map_err(|_| fail.clone())?;
                        let state = body
                            .variables
                            .get("construction:this")
                            .cloned()
                            .ok_or(fail.clone())?;
                        self.invoke(body, signature, vec![state])?
                    } else {
                        body.fields.get(name).cloned().ok_or(fail.clone())?
                    }
                } else {
                    let mut value = self.expression(body, receiver)?;
                    if let Ok(signature) = domain::reference_value_signature(
                        self.r,
                        self.c,
                        &format!("reference.value.{}", value.type_id),
                    ) {
                        value = self.invoke(body, signature, vec![value])?;
                    }
                    if op.symbol() == "System.Runtime|System.Array.Length"
                        && self.c.metadata.get(&value.type_id).is_some_and(|m| {
                            template_name(&m.template_id) == Some("sequence_construction")
                        })
                    {
                        let origin = body
                            .live_constructions
                            .iter()
                            .find(|(_, v)| v.id == value.id)
                            .map(|(k, _)| k)
                            .ok_or(fail.clone())?;
                        return body.construction_lengths.get(origin).cloned().ok_or(fail);
                    }
                    if self
                        .c
                        .metadata
                        .get(&value.type_id)
                        .is_some_and(|m| template_name(&m.template_id) == Some("option"))
                    {
                        let name = if op.symbol().ends_with(".Value") {
                            "value"
                        } else if op.symbol().ends_with(".HasValue") {
                            "has_value"
                        } else {
                            return Err(fail);
                        };
                        // Exact framework ownership was checked by the captured Roslyn data gate.
                        if !op.symbol().starts_with("System.Runtime|") {
                            return Err(fail);
                        }
                        return self.invoke(
                            body,
                            self.signatures[&format!("{}.{name}", value.type_id)].clone(),
                            vec![value],
                        );
                    }
                    if self
                        .c
                        .metadata
                        .get(&value.type_id)
                        .is_some_and(|m| template_name(&m.template_id) == Some("bounded_sequence"))
                        && op.symbol() == "System.Runtime|System.Array.Length"
                    {
                        let signature = self
                            .signatures
                            .get(&format!("{}.length", value.type_id))
                            .cloned()
                            .ok_or(fail.clone())?;
                        let length = self.invoke(body, signature, vec![value])?;
                        return self.invoke(
                            body,
                            scalar_operation_signature("integer.convert.u32.i32.unchecked")
                                .map_err(|_| fail.clone())?,
                            vec![length],
                        );
                    }
                    let source = self
                        .r
                        .source_types
                        .get(&value.type_id)
                        .ok_or(fail.clone())?;
                    if op.kind() == "PropertyReference" {
                        if let Some(getter) = self.source.callables().iter().find(|c| {
                            c.identity()["owner"] == source.id
                                && c.identity()["name"].as_str().is_some_and(|name| {
                                    name.strip_prefix("get_").is_some_and(|name| {
                                        op.symbol() == format!("{}.{}", source.id, name)
                                    })
                                })
                        }) {
                            let id = getter.id().to_owned();
                            self.compile(&id)?;
                            return self.invoke(body, self.signatures[&id].clone(), vec![value]);
                        }
                    }
                    if let Some(member) = source
                        .members
                        .iter()
                        .find(|m| format!("{}.{}", source.id, m.name) == op.symbol())
                    {
                        self.invoke(
                            body,
                            source_field_operation(self.r, self.c, &member.id)
                                .map_err(|_| fail.clone())?,
                            vec![value],
                        )?
                    } else {
                        let name = op
                            .symbol()
                            .strip_prefix(&format!("{}.", source.id))
                            .ok_or(fail.clone())?;
                        let getter = self
                            .source
                            .callables()
                            .iter()
                            .find(|c| {
                                c.identity()["owner"] == source.id
                                    && c.identity()["name"] == format!("get_{name}")
                            })
                            .ok_or(fail.clone())?
                            .id()
                            .to_owned();
                        self.compile(&getter)?;
                        self.invoke(body, self.signatures[&getter].clone(), vec![value])?
                    }
                }
            }
            "Conversion" => {
                let ty = parse_data_type_key(self.b, op.type_key().ok_or(fail.clone())?)?;
                let id = ClosedType::parse(&ty)
                    .and_then(|ty| closed_type_id(self.b, &ty))
                    .map_err(|_| fail.clone())?;
                if op.constant() == Some("null") {
                    let meta = self
                        .c
                        .metadata
                        .get(&id)
                        .filter(|m| template_name(&m.template_id) == Some("option"))
                        .ok_or(fail.clone())?;
                    let _ = meta;
                    body.literal(MonomorphicValue::Option {
                        type_id: id,
                        arm: OptionArm::None,
                        value: None,
                    })
                } else {
                    let value = self.expression(body, get(0)?)?;
                    if value.type_id == id {
                        value
                    } else if self.c.metadata.get(&id).is_some_and(|m| {
                        template_name(&m.template_id) == Some("option")
                            && m.argument_ids == [value.type_id.as_str()]
                    }) {
                        self.invoke(
                            body,
                            self.signatures[&format!("{id}.some")].clone(),
                            vec![value],
                        )?
                    } else {
                        let from = value
                            .type_id
                            .strip_prefix("mpk.csharp.value.")
                            .and_then(|s| s.strip_suffix(".v1"))
                            .ok_or(fail.clone())?;
                        let to = id
                            .strip_prefix("mpk.csharp.value.")
                            .and_then(|s| s.strip_suffix(".v1"))
                            .ok_or(fail.clone())?;
                        let checked = if op.traits().split('|').next() == Some("True") {
                            "checked"
                        } else {
                            "unchecked"
                        };
                        let signature = scalar_operation_signature(&format!(
                            "integer.convert.{from}.{to}.{checked}"
                        ))
                        .map_err(|_| fail.clone())?;
                        self.invoke(body, signature, vec![value])?
                    }
                }
            }
            "ArrayCreation" => {
                let length_node = get(0)?;
                let length = self.expression(body, length_node)?;
                let ty = parse_data_type_key(self.b, op.type_key().ok_or(fail.clone())?)?;
                let ClosedType::Instance {
                    template,
                    arguments,
                } = ClosedType::parse(&ty).map_err(|_| fail.clone())?
                else {
                    return Err(fail);
                };
                if template != "bounded_sequence" {
                    return Err(fail);
                }
                let element = closed_type_id(self.b, &arguments[0]).map_err(|_| fail.clone())?;
                let construction = closed_type_id(
                    self.b,
                    &ClosedType::Instance {
                        template: "sequence_construction".into(),
                        arguments,
                    },
                )
                .map_err(|_| fail.clone())?;
                let default = body.literal(MonomorphicValue::Bool {
                    type_id: BOOL_TYPE_ID.into(),
                    value: node.children.len() == 1
                        && domain_default(self.b, self.r, self.c, &element).is_ok(),
                });
                let signature = self
                    .signatures
                    .get(&format!("{construction}.allocate"))
                    .cloned()
                    .ok_or(fail.clone())?;
                let mut value = self.invoke(body, signature, vec![length, default])?;
                if let Some(initializer) = node.children.get(1) {
                    if initializer.operation.kind() != "ArrayInitializer"
                        || node.children.len() != 2
                    {
                        return Err(fail);
                    }
                    for (index, expression) in initializer.children.iter().enumerate() {
                        let item = self.expression(body, expression)?;
                        let item = self.publish(body, item)?;
                        let index = body.literal(MonomorphicValue::Signed {
                            type_id: I32_TYPE_ID.into(),
                            value: index.to_string(),
                        });
                        let fill = self
                            .signatures
                            .get(&format!("{}.fill", value.type_id))
                            .cloned()
                            .ok_or(fail.clone())?;
                        value = self.invoke(body, fill, vec![value, index, item])?;
                    }
                }
                value
            }
            "ArrayElementReference" => {
                let mut value = self.receiver_expression(body, get(0)?)?;
                let index = self.expression(body, get(1)?)?;
                if let Ok(signature) = domain::reference_value_signature(
                    self.r,
                    self.c,
                    &format!("reference.value.{}", value.type_id),
                ) {
                    value = self.invoke(body, signature, vec![value])?;
                }
                let signature = self
                    .signatures
                    .get(&format!("{}.read", value.type_id))
                    .cloned()
                    .ok_or(fail.clone())?;
                self.invoke(body, signature, vec![value, index])?
            }
            "ObjectCreation" | "Invocation" => {
                self.compile(op.symbol())?;
                let signature = self
                    .signatures
                    .get(op.symbol())
                    .cloned()
                    .ok_or(fail.clone())?;
                let mut positional = BTreeMap::new();
                let mut receiver = None;
                let mut initializer = None;
                for child in &node.children {
                    if child.operation.kind() == "ObjectOrCollectionInitializer" {
                        if initializer.replace(child).is_some() {
                            return Err(fail);
                        }
                        continue;
                    }
                    let value = if child.operation.kind() == "Argument" {
                        self.expression(body, child)?
                    } else {
                        self.receiver_expression(body, child)?
                    };
                    if child.operation.kind() == "Argument" {
                        let ordinal = child
                            .operation
                            .symbol()
                            .strip_prefix("argument:")
                            .and_then(|n| n.parse::<usize>().ok())
                            .ok_or(fail.clone())?;
                        if positional.insert(ordinal, value).is_some() {
                            return Err(fail);
                        }
                    } else if receiver.replace(value).is_some() {
                        return Err(fail);
                    }
                }
                let receiver = receiver
                    .map(|value| {
                        if let Ok(signature) = domain::reference_value_signature(
                            self.r,
                            self.c,
                            &format!("reference.value.{}", value.type_id),
                        ) {
                            self.invoke(body, signature, vec![value])
                        } else {
                            Ok(value)
                        }
                    })
                    .transpose()?;
                let mut args = receiver.into_iter().collect::<Vec<_>>();
                for (expected, (ordinal, value)) in positional.into_iter().enumerate() {
                    if expected != ordinal {
                        return Err(fail);
                    }
                    args.push(value);
                }
                let args = args
                    .into_iter()
                    .map(|v| self.publish(body, v))
                    .collect::<Result<Vec<_>, _>>()?;
                if signature.tag == ClosedOperationTag::ConstructorExecute {
                    if op.kind() != "ObjectCreation" {
                        return Err(fail);
                    }
                    let plan = self
                        .source
                        .callables()
                        .iter()
                        .find(|c| c.id() == body.function.id)
                        .and_then(|c| {
                            c.initialization_plans()
                                .iter()
                                .find(|p| p.node_ordinal == node.ordinal)
                        })
                        .ok_or(fail.clone())?;
                    let owner = plan.type_id.clone();
                    let begin = object_construction_signature(
                        self.r,
                        self.c,
                        &format!("object.begin.{owner}"),
                    )
                    .map_err(|_| fail.clone())?;
                    let begin_node_id = body.function.blocks[body.current].node.id.clone();
                    let state = self.invoke(body, begin, vec![])?;
                    let mut private_args = vec![state];
                    private_args.extend(args);
                    let private_args = self.call_arguments(body, &signature, private_args)?;
                    let constructor_node_id = body.function.blocks[body.current].node.id.clone();
                    let mut state = self.invoke(body, signature, private_args)?;
                    let mut assignment_node_ids = vec![];
                    if let Some(initializer) = initializer {
                        for assignment in &initializer.children {
                            if assignment.operation.kind() != "SimpleAssignment"
                                || assignment.children.len() != 2
                            {
                                return Err(fail);
                            }
                            let target = &assignment.children[0];
                            let name = target
                                .operation
                                .symbol()
                                .strip_prefix(&format!("{owner}."))
                                .ok_or(fail.clone())?;
                            let member = self.r.source_types[&owner]
                                .members
                                .iter()
                                .find(|m| m.name == name)
                                .ok_or(fail.clone())?;
                            let write = object_construction_signature(
                                self.r,
                                self.c,
                                &format!("object.write.{}", member.id),
                            )
                            .map_err(|_| fail.clone())?;
                            let value = self.expression(body, &assignment.children[1])?;
                            let value = self.publish(body, value)?;
                            let value = self.coerce(body, value, &write.argument_type_ids[1])?;
                            assignment_node_ids
                                .push(body.function.blocks[body.current].node.id.clone());
                            state = self.invoke(body, write, vec![state, value])?;
                        }
                    }
                    let finalize = object_construction_signature(
                        self.r,
                        self.c,
                        &format!("object.finalize.{owner}"),
                    )
                    .map_err(|_| fail.clone())?;
                    let finalize_node_id = body.function.blocks[body.current].node.id.clone();
                    let result = self.invoke(body, finalize, vec![state])?;
                    body.object_protocol()
                        .initializations
                        .push(v::PracticalObjectInitialization {
                            source_node_ordinal: node.ordinal,
                            begin_node_id,
                            constructor_node_id,
                            assignment_node_ids,
                            finalize_node_id,
                        });
                    result
                } else {
                    if initializer.is_some() {
                        return Err(fail);
                    }
                    let args = self.call_arguments(body, &signature, args)?;
                    self.invoke(body, signature, args)?
                }
            }
            "Binary" | "BinaryOperator" | "BinaryOperation" | "Unary" | "UnaryOperator"
            | "UnaryOperation" => {
                if matches!(
                    op.traits().split('|').next(),
                    Some("ConditionalAnd" | "ConditionalOr")
                ) {
                    let condition = self.expression(body, get(0)?)?;
                    return if op.traits().starts_with("ConditionalAnd|") {
                        self.branch(
                            body,
                            condition,
                            Some(get(1)?),
                            None,
                            None,
                            Some(false),
                            false,
                        )?
                        .ok_or(fail)
                    } else {
                        self.branch(
                            body,
                            condition,
                            None,
                            Some(get(1)?),
                            Some(true),
                            None,
                            false,
                        )?
                        .ok_or(fail)
                    };
                }
                let args = node
                    .children
                    .iter()
                    .map(|n| self.expression(body, n))
                    .collect::<Result<Vec<_>, _>>()?;
                let token = args
                    .first()
                    .and_then(|v| v.type_id.strip_prefix("mpk.csharp.value."))
                    .and_then(|s| s.strip_suffix(".v1"))
                    .ok_or(fail.clone())?;
                let traits = op.traits().split('|').collect::<Vec<_>>();
                let name = match *traits.first().ok_or(fail.clone())? {
                    "Add" => "add",
                    "Subtract" => "subtract",
                    "Multiply" => "multiply",
                    "Divide" => "divide",
                    "Remainder" => "remainder",
                    "Plus" => "plus",
                    "Minus" => "negate",
                    "Not" | "BitwiseNegation" => "not",
                    "Equals" => "equal",
                    "NotEquals" => "not_equal",
                    "LessThan" => "less",
                    "LessThanOrEqual" => "less_equal",
                    "GreaterThan" => "greater",
                    "GreaterThanOrEqual" => "greater_equal",
                    "And" => "and",
                    "Or" => "or",
                    "ExclusiveOr" => "xor",
                    "LeftShift" => "left_shift",
                    "RightShift" => "right_shift",
                    "UnsignedRightShift" => "unsigned_right_shift",
                    _ => return Err(fail),
                };
                let checked = if traits.get(1) == Some(&"True") {
                    "checked"
                } else {
                    "unchecked"
                };
                if matches!(token, "f32" | "f64" | "decimal" | "string") {
                    return Err(fail);
                } else {
                    let id = if token == "bool" {
                        format!("boolean.{name}")
                    } else {
                        format!("integer.{token}.{name}.{checked}")
                    };
                    self.invoke(
                        body,
                        scalar_operation_signature(&id).map_err(|_| fail.clone())?,
                        args,
                    )?
                }
            }
            _ => return Err(fail),
        };
        if let Some(key) = op.type_key() {
            let ty = parse_data_type_key(self.b, key)?;
            let expected = ClosedType::parse(&ty)
                .and_then(|ty| closed_type_id(self.b, &ty))
                .map_err(|_| DataPhaseError::Emission)?;
            if result.type_id != expected
                && self.c.metadata.get(&result.type_id).is_some_and(|m| {
                    template_name(&m.template_id) == Some("sequence_construction")
                        && m.dependency_ids.contains(&expected)
                })
            {
                return Ok(result);
            }
            if result.type_id != expected
                && matches!(op.kind(), "ParameterReference" | "LocalReference")
            {
                let signature = domain::reference_value_signature(
                    self.r,
                    self.c,
                    &format!("reference.value.{}", result.type_id),
                )
                .map_err(|_| DataPhaseError::Emission)?;
                if signature.normal_result_type_id != expected {
                    return Err(DataPhaseError::Emission);
                }
                result = self.invoke(body, signature, vec![result])?;
            }
            if result.type_id != expected {
                return Err(DataPhaseError::Emission);
            }
        }
        Ok(result)
    }
}
fn source_literal(type_id: &str, constant: &str) -> Result<MonomorphicValue, DataPhaseError> {
    let fail = DataPhaseError::Emission;
    let type_id = type_id.to_owned();
    let (kind, text) = constant.split_once(':').ok_or(fail.clone())?;
    Ok(match kind {
        "bool" if matches!(text, "true" | "false") => MonomorphicValue::Bool {
            type_id,
            value: text == "true",
        },
        "string_utf16" => {
            if text.len() % 4 != 0 || text.len() / 4 > STRING_VALUE_LENGTH_MAX as usize {
                return Err(fail);
            }
            let utf16 = text
                .as_bytes()
                .chunks_exact(4)
                .map(|b| {
                    std::str::from_utf8(b)
                        .ok()
                        .and_then(|s| u16::from_str_radix(s, 16).ok())
                        .ok_or(DataPhaseError::Emission)
                })
                .collect::<Result<_, _>>()?;
            MonomorphicValue::String { type_id, utf16 }
        }
        "char" => MonomorphicValue::Char {
            type_id,
            utf16: text.parse().map_err(|_| fail.clone())?,
        },
        "decimal" => {
            let bits = text
                .split(',')
                .map(|s| u32::from_str_radix(s, 16).map_err(|_| DataPhaseError::Emission))
                .collect::<Result<Vec<_>, _>>()?;
            if bits.len() != 4 || bits[3] & 0x7f00ffff != 0 {
                return Err(fail);
            }
            MonomorphicValue::DecimalBits {
                type_id,
                negative: bits[3] >> 31 != 0,
                scale: ((bits[3] >> 16) & 255) as u8,
                coefficient: (u128::from(bits[0])
                    | (u128::from(bits[1]) << 32)
                    | (u128::from(bits[2]) << 64))
                    .to_string(),
            }
        }
        "f32" => MonomorphicValue::F32Bits {
            type_id,
            bits: text.into(),
        },
        "f64" => MonomorphicValue::F64Bits {
            type_id,
            bits: text.into(),
        },
        "System.SByte" | "System.Int16" | "System.Int32" | "System.Int64" => {
            MonomorphicValue::Signed {
                type_id,
                value: text.into(),
            }
        }
        "System.Byte" | "System.UInt16" | "System.UInt32" | "System.UInt64" => {
            MonomorphicValue::Unsigned {
                type_id,
                value: text.into(),
            }
        }
        _ => return Err(fail),
    })
}

fn foundation_signatures(
    r: &ValidatedClosedRootSet,
    c: &ClosedInstanceSet,
) -> Result<BTreeMap<String, ClosedOperationSignature>, DataPhaseError> {
    let mut signatures = BTreeMap::new();
    for entry in c.entries() {
        for operation in entry["operation_definitions"]
            .as_array()
            .ok_or(DataPhaseError::Emission)?
        {
            let text = |key: &str| {
                operation[key]
                    .as_str()
                    .map(str::to_owned)
                    .ok_or(DataPhaseError::Emission)
            };
            let ordered_checks = operation["error_precedence"]
                .as_array()
                .ok_or(DataPhaseError::Emission)?
                .iter()
                .map(|value| {
                    let id = value.as_str().ok_or(DataPhaseError::Emission)?;
                    let contract = check_contract(id).ok_or(DataPhaseError::Emission)?;
                    let failure_type_id = match contract.failure {
                        CheckFailureType::None => None,
                        CheckFailureType::Exact(id) => Some(id.into()),
                    };
                    Ok(RequiredCheck {
                        id: id.into(),
                        tag: contract.tag,
                        failure_type_id,
                    })
                })
                .collect::<Result<Vec<_>, DataPhaseError>>()?;
            let signature = ClosedOperationSignature {
                id: text("id")?,
                tag: ClosedOperationTag::Foundation,
                argument_type_ids: operation["argument_type_ids"]
                    .as_array()
                    .ok_or(DataPhaseError::Emission)?
                    .iter()
                    .map(|v| {
                        v.as_str()
                            .map(str::to_owned)
                            .ok_or(DataPhaseError::Emission)
                    })
                    .collect::<Result<_, _>>()?,
                normal_result_type_id: text("normal_result_type_id")?,
                ordered_checks,
            };
            validate_closed_operation_signature(r, c, &signature)
                .map_err(|_| DataPhaseError::Emission)?;
            if signatures.insert(signature.id.clone(), signature).is_some() {
                return Err(DataPhaseError::Emission);
            }
        }
    }
    Ok(signatures)
}

pub fn emit_data_phase(
    b: &ValidatedFoundationBundle,
    context: &PracticalArtifactContext,
    captures: &CapturedInputSet,
    source: &ValidatedDataSource,
) -> Result<EmittedDataPhase, DataPhaseError> {
    // Bound parser/emitter recursion independently of an API caller's thread
    // stack. The source importer bounds operation depth before this worker.
    std::thread::scope(|scope| {
        std::thread::Builder::new()
            .stack_size(64 * 1024 * 1024)
            .spawn_scoped(scope, || {
                emit_data_phase_inner(b, context, captures, source)
            })
            .map_err(|_| DataPhaseError::Emission)?
            .join()
            .map_err(|_| DataPhaseError::Emission)?
    })
}
fn emit_data_phase_inner(
    b: &ValidatedFoundationBundle,
    context: &PracticalArtifactContext,
    captures: &CapturedInputSet,
    source: &ValidatedDataSource,
) -> Result<EmittedDataPhase, DataPhaseError> {
    source.require_lineage(context, captures)?;
    let sidecars = DataSidecars::capture(context, captures)?;
    let reachable = source
        .source_types()
        .as_object()
        .ok_or(DataPhaseError::Source)?
        .keys()
        .cloned()
        .collect();
    let contract_roots = derive_data_contract_roots(b, source.source_roots(), &sidecars)?;
    let closure = DataBindingClosure::derive(
        b,
        context,
        captures,
        &contract_roots,
        sidecars.bindings(),
        &reachable,
    )?;
    let mut emitter = Emitter {
        b,
        r: closure.roots(),
        c: closure.closed(),
        source,
        control: source
            .control_lowering()
            .map(|_| validate_control_source(b, source).map(std::sync::Arc::new))
            .transpose()?,
        signatures: foundation_signatures(closure.roots(), closure.closed())?,
        functions: BTreeMap::new(),
        active: BTreeSet::new(),
    };
    emitter.compile_all()?;
    let (binding_projections, binding_commutations) =
        binding_emission(&closure, &sidecars, &mut emitter.signatures)?;
    data_phase::attach_data_contracts(
        b,
        context,
        captures,
        &closure,
        source,
        &sidecars,
        &emitter.signatures,
    )?;
    if source.control_lowering().is_some() {
        let used = emitter
            .functions
            .values()
            .flat_map(|f| {
                std::iter::once(f.id.clone()).chain(
                    f.blocks
                        .iter()
                        .filter_map(|b| b.invocation.as_ref().map(|i| i.operation_id.clone())),
                )
            })
            .chain(
                binding_projections
                    .iter()
                    .flat_map(|p| [p.project.id.clone(), p.reconstruct.id.clone()]),
            )
            .chain(binding_commutations.iter().flat_map(|c| {
                [
                    c.source_operation.id.clone(),
                    c.semantic_operation.id.clone(),
                ]
            }))
            .collect::<BTreeSet<_>>();
        emitter.signatures.retain(|id, signature| {
            signature.tag == ClosedOperationTag::Foundation || used.contains(id)
        });
    }
    let boundaries = boundary::attach_boundary_contracts(
        b,
        context,
        source,
        &closure,
        &sidecars,
        &emitter.signatures,
    )
    .map_err(DataPhaseError::Boundary)?;
    let transitions = transition::attach_transition_contracts(
        b,
        context,
        source,
        &closure,
        &sidecars,
        &emitter.signatures,
        None,
    )?;
    let transition_refs = transitions
        .iter()
        .map(|t| t.artifact().artifact_ref())
        .collect::<Vec<_>>();
    let signatures = emitter.signatures.into_values().collect();
    let functions = emitter.functions.into_values().collect();
    let closed_ref =
        a::bind_closed_instances(context, b, captures, closure.roots(), closure.closed())
            .map_err(|_| DataPhaseError::Emission)?;
    let operations = a::build_concrete_operation_tables(
        context,
        closure.roots(),
        closure.closed(),
        &closed_ref,
        signatures,
    )
    .map_err(|_| DataPhaseError::Emission)?;
    let input = v::PracticalVirImportContext {
        data_source_facts: Some(source.captured_facts()),
        artifact_context: context,
        captured_inputs: captures,
        foundation_descriptor_transport: registered_foundation_descriptor_transport(),
        foundation_definitions_transport: registered_foundation_definitions_transport(),
        closed_roots_transport: closure.roots().canonical_json(),
        closed_instances_transport: closure.closed().canonical_json(),
        semantic_bindings_transport: closure.bindings().canonical_bytes(),
        required_checks_transport: operations.required_checks().canonical_bytes(),
        operations_transport: operations.operations().canonical_bytes(),
    };
    let bytes = v::canonical_csharp_practical_vir_transport(
        input,
        v::PracticalVirContents {
            functions,
            binding_projections,
            binding_commutations,
            source_obligations: source.source_obligations().to_vec(),
            source_exceptions: if source.control_lowering().is_some() {
                validate_control_source(b, source)?.definitions().to_vec()
            } else {
                vec![]
            },
            data_contracts: sidecars
                .contracts()
                .iter()
                .map(|c| {
                    String::from_utf8(c.canonical_bytes().to_vec())
                        .map_err(|_| DataPhaseError::Contract)
                })
                .collect::<Result<_, _>>()?,
        },
    )
    .map_err(|_| DataPhaseError::Emission)?;
    let vir =
        v::import_csharp_practical_vir_json(&bytes, input).map_err(|e| DataPhaseError::Import {
            phase: e.phase().as_str(),
            code: e.code().as_str(),
        })?;
    let boundary_refs = boundaries
        .iter()
        .map(|b| b.artifact().artifact_ref())
        .collect::<Vec<_>>();
    let routes = derive_data_type_routes(b, closure.roots(), closure.closed())?;
    let source_map = a::build_practical_source_map(
        context,
        captures,
        &vir.artifact_ref(),
        source.source_map_declarations(&vir)?,
    )
    .map_err(|_| DataPhaseError::Emission)?;
    let manifest = a::build_frontend_source_manifest(
        context,
        b,
        captures,
        a::FrontendManifestArtifacts {
            type_contracts: sidecars
                .contracts()
                .iter()
                .filter(|c| c.schema() == a::TYPE_CONTRACT_SCHEMA)
                .map(|c| c.artifact_ref())
                .collect(),
            method_contracts: sidecars
                .contracts()
                .iter()
                .filter(|c| c.schema() == a::METHOD_CONTRACT_SCHEMA)
                .map(|c| c.artifact_ref())
                .collect(),
            semantic_bindings: closure.bindings().artifact_ref(),
            boundary_contracts: boundary_refs.clone(),
            boundary_inputs: vec![],
            boundary_outputs: vec![],
            transition_contracts: transition_refs.clone(),
            closed_instances: closed_ref.clone(),
            operations: operations.operations().artifact_ref(),
            required_checks: operations.required_checks().artifact_ref(),
            vir: vir.artifact_ref(),
            source_map: source_map.artifact_ref(),
        },
    )
    .map_err(|_| DataPhaseError::Emission)?;
    let artifacts = a::build_frontend_source_artifacts(
        context,
        b,
        a::FrontendSourceArtifactLinks {
            vir: &vir.artifact_ref(),
            source_map: &source_map.artifact_ref(),
            source_manifest: &manifest,
            semantic_bindings: &closure.bindings().artifact_ref(),
            closed_instances: &closed_ref,
            boundary_contracts: boundary_refs.clone(),
            transition_contracts: transition_refs.clone(),
        },
    )
    .map_err(|_| DataPhaseError::Emission)?;
    Ok(EmittedDataPhase {
        closure,
        operations,
        vir,
        routes,
        boundaries,
        transitions,
        source_map,
        manifest,
        artifacts,
    })
}

fn exception_check(exception: &str) -> Result<RequiredCheck, DataPhaseError> {
    let id = match exception {
        "System.OverflowException" => "exception.overflow",
        "System.DivideByZeroException" => "exception.division_by_zero",
        "System.ArgumentOutOfRangeException" => "exception.range",
        "System.NullReferenceException" => "exception.null_receiver",
        "System.ArgumentNullException" => "exception.null_argument",
        "System.IndexOutOfRangeException" => "index_range",
        "System.InvalidOperationException" => "invalid_operation",
        _ => return Err(DataPhaseError::Emission),
    };
    Ok(RequiredCheck {
        id: id.into(),
        tag: RequiredCheckTag::Exception,
        failure_type_id: Some(exception.into()),
    })
}

fn binding_emission(
    closure: &DataBindingClosure,
    sidecars: &DataSidecars,
    signatures: &mut BTreeMap<String, ClosedOperationSignature>,
) -> Result<(Vec<BindingTypeProjection>, Vec<BindingOperationCommutation>), DataPhaseError> {
    use a::PracticalJsonValue as J;
    fn projection(
        source: &str,
        semantic: &str,
        suffix: &str,
        binding_id: String,
        id: String,
    ) -> BindingTypeProjection {
        BindingTypeProjection {
            id,
            binding_id,
            source_type_id: source.into(),
            semantic_type_id: semantic.into(),
            project: ClosedOperationSignature {
                id: format!("binding.project.{suffix}"),
                tag: ClosedOperationTag::BindingProject,
                argument_type_ids: vec![source.into()],
                normal_result_type_id: semantic.into(),
                ordered_checks: vec![],
            },
            reconstruct: ClosedOperationSignature {
                id: format!("binding.reconstruct.{suffix}"),
                tag: ClosedOperationTag::BindingReconstruct,
                argument_type_ids: vec![semantic.into()],
                normal_result_type_id: source.into(),
                ordered_checks: vec![],
            },
        }
    }
    let mut projections = BTreeMap::<String, BindingTypeProjection>::new();
    for row in closure
        .bindings()
        .value()
        .get("bindings")
        .and_then(J::as_array)
        .ok_or(DataPhaseError::Binding)?
    {
        let source = row
            .get("source_type_id")
            .and_then(J::as_str)
            .ok_or(DataPhaseError::Binding)?;
        let hash = row
            .get("binding_sha256")
            .and_then(J::as_str)
            .ok_or(DataPhaseError::Binding)?;
        let semantic = closure
            .projections()
            .get(source)
            .ok_or(DataPhaseError::Binding)?;
        let item = projection(
            source,
            semantic,
            hash,
            format!("binding.{hash}"),
            format!("projection.{hash}"),
        );
        projections.insert(item.id.clone(), item);
    }
    fn project_pair(
        projections: &mut BTreeMap<String, BindingTypeProjection>,
        source: &str,
        semantic: &str,
    ) -> Result<String, DataPhaseError> {
        if let Some(p) = projections
            .values()
            .find(|p| p.source_type_id == source && p.semantic_type_id == semantic)
        {
            return Ok(p.id.clone());
        }
        if source != semantic {
            return Err(DataPhaseError::Binding);
        }
        let id = format!("projection.identity.{source}");
        projections.insert(
            id.clone(),
            projection(
                source,
                semantic,
                &format!("identity.{source}"),
                "binding.identity".into(),
                id.clone(),
            ),
        );
        Ok(id)
    }
    let mut commutations = vec![];
    for binding in sidecars.bindings() {
        let own = projections
            .values()
            .find(|p| p.source_type_id == binding.source_type_id)
            .ok_or(DataPhaseError::Binding)?
            .clone();
        for mapping in &binding.operation_map {
            let source = signatures
                .get(&mapping.member_id)
                .filter(|s| s.tag == ClosedOperationTag::SourceCall)
                .ok_or(DataPhaseError::Unreachable)?
                .clone();
            let semantic_id =
                binding_semantic_operation_id(&own.semantic_type_id, &mapping.operation);
            // Instant is a primitive semantic domain, so its operation recipes
            // are supplied by the W13 business registry, not a template row.
            if binding.role == "instant" && !signatures.contains_key(&semantic_id) {
                let instant = "mpk.csharp.value.instant.v1";
                let duration = "mpk.csharp.value.duration.v1";
                let (args, result) = match mapping.operation.as_str() {
                    "milliseconds" => (vec![instant], "mpk.csharp.value.i64.v1"),
                    "compare" => (vec![instant, instant], "mpk.csharp.value.i32.v1"),
                    "add_duration" | "subtract_duration" => (vec![instant, duration], instant),
                    "difference" => (vec![instant, instant], duration),
                    _ => return Err(DataPhaseError::Binding),
                };
                let args = args.into_iter().map(str::to_owned).collect::<Vec<_>>();
                let op = BusinessOperation::new(&semantic_id, &args, result)
                    .map_err(|_| DataPhaseError::Binding)?;
                let signature = ClosedOperationSignature {
                    id: semantic_id.clone(),
                    tag: ClosedOperationTag::Data,
                    argument_type_ids: args,
                    normal_result_type_id: result.into(),
                    ordered_checks: op
                        .ordered_errors()
                        .into_iter()
                        .map(|id| RequiredCheck {
                            id: id.into(),
                            tag: RequiredCheckTag::ErrorOutcome,
                            failure_type_id: None,
                        })
                        .collect(),
                };
                validate_closed_operation_signature(closure.roots(), closure.closed(), &signature)
                    .map_err(|_| DataPhaseError::Binding)?;
                signatures.insert(semantic_id.clone(), signature);
            }
            let semantic = signatures
                .get(&semantic_id)
                .ok_or(DataPhaseError::Binding)?
                .clone();
            if source.argument_type_ids.len() != semantic.argument_type_ids.len() {
                return Err(DataPhaseError::Binding);
            }
            let mut operands = vec![];
            let mut rounding_operands = vec![];
            for (ordinal, (s, t)) in source
                .argument_type_ids
                .iter()
                .zip(&semantic.argument_type_ids)
                .enumerate()
            {
                if t == "mpk.csharp.value.u32.v1"
                    && binding
                        .enum_arms
                        .get(s)
                        .is_some_and(|arms| arms.contains_key("ToEven"))
                {
                    rounding_operands.push(RoundingOperandCommutation {
                        ordinal: ordinal as u32,
                        source_type_id: s.clone(),
                        enum_arms: binding.enum_arms[s].clone(),
                    });
                    operands.push(format!("enum.{s}"));
                } else {
                    operands.push(project_pair(&mut projections, s, t)?);
                }
            }
            let errors = semantic
                .ordered_checks
                .iter()
                .filter(|c| c.tag == RequiredCheckTag::ErrorOutcome)
                .collect::<Vec<_>>();
            let (result, returned_result) = if errors.is_empty() {
                (
                    project_pair(
                        &mut projections,
                        &source.normal_result_type_id,
                        &semantic.normal_result_type_id,
                    )?,
                    None,
                )
            } else {
                let result_binding = sidecars
                    .bindings()
                    .iter()
                    .find(|b| {
                        b.source_type_id == source.normal_result_type_id && b.role == "result"
                    })
                    .ok_or(DataPhaseError::Binding)?;
                let complete = projections
                    .values()
                    .find(|p| p.source_type_id == result_binding.source_type_id)
                    .ok_or(DataPhaseError::Binding)?
                    .clone();
                let raw = &closure.roots().source_types[&result_binding.source_type_id];
                let payload_member = result_binding
                    .member_map
                    .iter()
                    .find(|m| m.role == "value")
                    .ok_or(DataPhaseError::Binding)?;
                let payload = raw
                    .members
                    .iter()
                    .find(|m| m.id == payload_member.member_id)
                    .ok_or(DataPhaseError::Binding)?;
                let payload_id = type_id_in_closed_set(&payload.ty, closure.closed())
                    .ok_or(DataPhaseError::Binding)?;
                let success = project_pair(
                    &mut projections,
                    &payload_id,
                    &semantic.normal_result_type_id,
                )?;
                let error_id = result_binding
                    .inferred_argument_ids
                    .get(1)
                    .ok_or(DataPhaseError::Binding)?
                    .clone();
                let arms = binding
                    .enum_arms
                    .get(&error_id)
                    .ok_or(DataPhaseError::Binding)?;
                let ordered_errors = errors
                    .iter()
                    .enumerate()
                    .map(|(ordinal, c)| {
                        Ok(ReturnedErrorCommutation {
                            ordinal: ordinal as u32,
                            semantic_check_id: c.id.clone(),
                            source_carrier: arms.get(&c.id).ok_or(DataPhaseError::Binding)?.clone(),
                        })
                    })
                    .collect::<Result<_, DataPhaseError>>()?;
                (
                    complete.id,
                    Some(ReturnedResultCommutation {
                        success_projection_id: success,
                        error_type_id: error_id,
                        ordered_errors,
                    }),
                )
            };
            let outcomes = semantic
                .ordered_checks
                .iter()
                .filter(|c| c.tag != RequiredCheckTag::ErrorOutcome)
                .enumerate()
                .map(|(ordinal, t)| {
                    let s = source
                        .ordered_checks
                        .iter()
                        .find(|s| s.id == t.id && s.tag == t.tag)
                        .ok_or(DataPhaseError::Binding)?;
                    let failure_projection_id = match (&s.failure_type_id, &t.failure_type_id) {
                        (a, b) if a == b => None,
                        (Some(a), Some(b)) => Some(project_pair(&mut projections, a, b)?),
                        _ => return Err(DataPhaseError::Binding),
                    };
                    Ok(CheckCommutation {
                        ordinal: ordinal as u32,
                        source_check_id: s.id.clone(),
                        semantic_check_id: t.id.clone(),
                        failure_projection_id,
                    })
                })
                .collect::<Result<Vec<_>, DataPhaseError>>()?;
            let unmatched_source_check_ids = source
                .ordered_checks
                .iter()
                .filter(|s| !outcomes.iter().any(|o| o.source_check_id == s.id))
                .map(|s| s.id.clone())
                .collect();
            commutations.push(BindingOperationCommutation {
                binding_id: own.binding_id.clone(),
                source_operation: source,
                semantic_operation: semantic,
                operand_projection_ids: operands,
                result_projection_id: result,
                ordered_outcomes: outcomes,
                returned_result,
                rounding_operands,
                unmatched_source_check_ids,
            });
        }
    }
    let projections = projections.into_values().collect::<Vec<_>>();
    for projection in &projections {
        for signature in [&projection.project, &projection.reconstruct] {
            validate_closed_operation_signature(closure.roots(), closure.closed(), signature)
                .map_err(|_| DataPhaseError::Binding)?;
            if signatures
                .insert(signature.id.clone(), signature.clone())
                .is_some()
            {
                return Err(DataPhaseError::Binding);
            }
        }
    }
    for commutation in &commutations {
        validate_binding_operation_commutation(
            closure.roots(),
            closure.closed(),
            &projections,
            commutation,
        )
        .map_err(|_| DataPhaseError::Binding)?;
    }
    commutations.sort_by(|a, b| {
        (&a.binding_id, &a.source_operation.id).cmp(&(&b.binding_id, &b.source_operation.id))
    });
    Ok((projections, commutations))
}

/// Reconstruct source-bound control using captured operations, without Roslyn,
/// artifact parsing, or an invocation of the VIR importer.
pub(crate) fn derive_source_control_functions(
    b: &ValidatedFoundationBundle,
    r: &ValidatedClosedRootSet,
    c: &ClosedInstanceSet,
    source: &ValidatedDataSource,
) -> Result<Vec<v::PracticalVirFunction>, DataPhaseError> {
    let mut emitter = Emitter {
        b,
        r,
        c,
        source,
        control: source
            .control_lowering()
            .map(|_| validate_control_source(b, source).map(std::sync::Arc::new))
            .transpose()?,
        signatures: foundation_signatures(r, c)?,
        functions: BTreeMap::new(),
        active: BTreeSet::new(),
    };
    emitter.compile_all()?;
    Ok(emitter.functions.into_values().collect())
}
