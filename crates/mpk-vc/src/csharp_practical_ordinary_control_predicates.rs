//! Original W04 predicate bodies and their logical implication, with exact
//! cutpoint bindings. Integrated native definitions retain their exact guards;
//! execution scopes, loop induction and application proofs remain open.
use super::*;
use crate::csharp_practical_vir_model::{ControlBinding, ControlPredicate, ControlSequent};
use sha2::{Digest, Sha256};

#[path = "csharp_practical_ordinary_control_pattern_scopes.rs"]
mod pattern_scopes;
pub use pattern_scopes::{
    OrdinaryControlPatternExecutionScope, OrdinaryControlPatternObservation,
    OrdinaryControlPatternRouteObservation, OrdinaryControlPatternScope,
};
#[path = "csharp_practical_ordinary_control_pattern_captures.rs"]
mod pattern_captures;
pub use pattern_captures::{
    OrdinaryControlPatternCapture, OrdinaryControlPatternCaptureAlternative,
    OrdinaryControlPatternCaptureDependency,
};
#[path = "csharp_practical_ordinary_control_pattern_sources.rs"]
mod pattern_sources;
pub use pattern_sources::OrdinaryControlPatternSourceDefinition;

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct OrdinaryControlMeasureDefinition {
    pub source_name: String,
    pub type_id: String,
    pub operation: String,
    pub definition: String,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct OrdinaryControlGuardDependency {
    pub function_id: String,
    pub edge_id: String,
    pub guard_definition: String,
    /// Retain the exact receiver/flow prerequisite, not a context-free
    /// interpretation of the template's ownership predicate.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ownership: Option<OrdinaryConstructionOwnershipUse>,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct OrdinaryControlPredicateDefinition {
    pub source: ControlPredicate,
    /// Original binding order, projected from the sequent's shared arguments.
    pub argument_indices: Vec<usize>,
    pub definition: Option<String>,
    pub pending_constant_names: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub native_guard_dependency: Option<OrdinaryControlGuardDependency>,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct OrdinaryControlSequentDefinition {
    pub source: ControlSequent,
    /// Equality includes kind, edge, node, value and nominal type. In particular
    /// a previous header and the state after its backedge never share a binder.
    pub arguments: Vec<ControlBinding>,
    pub assumptions: Vec<OrdinaryControlPredicateDefinition>,
    pub goals: Vec<OrdinaryControlPredicateDefinition>,
    /// Only the original predicates' Boolean implication. Its connection to
    /// native execution and a loop-induction proof is a separate requirement.
    pub logical_implication_definition: Option<String>,
    pub pending_implication_reasons: Vec<String>,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct OrdinaryControlPredicateProgram {
    schema: String,
    source_ir_sha256: String,
    foundation_sha256: String,
    control_vc_sha256: String,
    /// Independently reconstructed native execution program, whose ordinary
    /// definitions retain the same names and bodies in the integrated certificate.
    #[serde(skip_serializing_if = "Option::is_none")]
    native_source_program_sha256: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    native_source_certificate_sha256: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pattern_scopes: Vec<OrdinaryControlPatternScope>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pattern_captures: Vec<OrdinaryControlPatternCapture>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pattern_capture_scopes: Vec<OrdinaryControlPatternScope>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pattern_sources: Vec<OrdinaryControlPatternSourceDefinition>,
    measures: Vec<OrdinaryControlMeasureDefinition>,
    sequents: Vec<OrdinaryControlSequentDefinition>,
    unresolved_regions: Vec<String>,
    /// No predicate definition or free-state implication discharges a source VC.
    application_scope_pending: bool,
    certificate_sha256: String,
    #[serde(skip)]
    certificate: Vec<u8>,
}
impl OrdinaryControlPredicateProgram {
    pub fn pattern_scopes(&self) -> &[OrdinaryControlPatternScope] {
        &self.pattern_scopes
    }
    pub fn pattern_captures(&self) -> &[OrdinaryControlPatternCapture] {
        &self.pattern_captures
    }
    pub fn pattern_capture_scopes(&self) -> &[OrdinaryControlPatternScope] {
        &self.pattern_capture_scopes
    }
    pub fn pattern_sources(&self) -> &[OrdinaryControlPatternSourceDefinition] {
        &self.pattern_sources
    }
    pub fn measures(&self) -> &[OrdinaryControlMeasureDefinition] {
        &self.measures
    }
    pub fn sequents(&self) -> &[OrdinaryControlSequentDefinition] {
        &self.sequents
    }
    pub fn unresolved_regions(&self) -> &[String] {
        &self.unresolved_regions
    }
    pub fn certificate_bytes(&self) -> &[u8] {
        &self.certificate
    }
    pub fn canonical_bytes(&self) -> Vec<u8> {
        serde_json::to_vec(self).expect("ordinary control predicates")
    }
}
fn name(role: &str, identity: &impl Serialize) -> String {
    format!(
        "{PREFIX}.ControlPredicate.{role}.H{:x}",
        Sha256::digest(serde_json::to_vec(identity).expect("typed control predicate identity"))
    )
}
fn constants(t: &ContractTerm, out: &mut BTreeMap<String, String>) -> R<()> {
    match t {
        ContractTerm::Const { name, type_id } => {
            if out
                .insert(name.clone(), type_id.clone())
                .is_some_and(|old| old != *type_id)
            {
                return Err(OrdinaryCarrierError::Linkage);
            }
        }
        ContractTerm::App {
            function, argument, ..
        } => {
            constants(function, out)?;
            constants(argument, out)?;
        }
        ContractTerm::Lam { body, .. } => constants(body, out)?,
        ContractTerm::Let { value, body, .. } => {
            constants(value, out)?;
            constants(body, out)?;
        }
        ContractTerm::Var { .. } => {}
    }
    Ok(())
}
fn read(b: &mut Builder, input: u32, index: u32, depth: u32) -> R<u32> {
    let selectors = (0..depth)
        .map(|i| bit(b, index & (1 << i) != 0))
        .collect::<R<Vec<_>>>()?;
    b.app(input, selectors)
}
fn measure(b: &mut Builder, symbol: &str) -> R<Option<OrdinaryControlMeasureDefinition>> {
    let Some((operation, ty)) = ["NonNegative", "MathLess", "MathEqual"]
        .into_iter()
        .find_map(|op| {
            symbol
                .strip_prefix(&format!("Mpk.CSharp.Integer.{op}."))
                .map(|ty| (op, ty))
        })
    else {
        return Ok(None);
    };
    let (width, signed) = match ty {
        "mpk.csharp.value.i8.v1" => (8, true),
        "mpk.csharp.value.u8.v1" => (8, false),
        "mpk.csharp.value.i16.v1" => (16, true),
        "mpk.csharp.value.u16.v1" => (16, false),
        "mpk.csharp.value.i32.v1" => (32, true),
        "mpk.csharp.value.u32.v1" => (32, false),
        "mpk.csharp.value.i64.v1" => (64, true),
        "mpk.csharp.value.u64.v1" => (64, false),
        _ => return Err(OrdinaryCarrierError::Linkage),
    };
    let depth = address_bits(width);
    let arity = if operation == "NonNegative" { 1 } else { 2 };
    let definition = name("Measure", &symbol);
    let mut body = bit(b, operation != "MathLess")?;
    if operation == "NonNegative" && signed {
        let input = b.var(0)?;
        let sign = read(b, input, width - 1, depth)?;
        body = call(b, "Std.Bool.not", vec![sign])?;
    } else if operation != "NonNegative" {
        let left = b.var(1)?;
        let right = b.var(0)?;
        for i in 0..width {
            let mut a = read(b, left, i, depth)?;
            let mut d = read(b, right, i, depth)?;
            if operation == "MathLess" {
                // Flipping the sign bit maps two's-complement signed order to
                // unsigned order, without subtraction or modular overflow.
                if signed && i == width - 1 {
                    a = call(b, "Std.Bool.not", vec![a])?;
                    d = call(b, "Std.Bool.not", vec![d])?;
                }
                let inverse = call(b, "Std.Bool.not", vec![d])?;
                let same = mux(b, a, d, inverse)?;
                // Equal higher bits preserve the lower-bit comparison. A
                // differing higher bit selects the right bit. The previous
                // body occurs once, also keeping the ordinary term DAG small.
                body = mux(b, same, body, d)?;
            } else {
                let inverse = call(b, "Std.Bool.not", vec![d])?;
                let same = mux(b, a, d, inverse)?;
                body = call(b, "Std.Bool.and", vec![body, same])?;
            }
        }
    }
    define(b, &definition, &vec![depth; arity], 0, body)?;
    Ok(Some(OrdinaryControlMeasureDefinition {
        source_name: symbol.into(),
        type_id: ty.into(),
        operation: operation.into(),
        definition,
    }))
}
// W04 free indices address bindings[index], whereas lower() consumes ordinary
// de Bruijn indices under argument lambdas. Local lambda/let indices stay local.
fn point_term(t: &ContractTerm, bindings: &[ControlBinding], depth: usize) -> R<ContractTerm> {
    if depth > 256 {
        return Err(OrdinaryCarrierError::Limit);
    }
    Ok(match t {
        ContractTerm::Var { index, type_id } if *index >= depth => {
            let free = index - depth;
            if bindings.get(free).map(|b| &b.type_id) != Some(type_id) {
                return Err(OrdinaryCarrierError::Linkage);
            }
            ContractTerm::Var {
                index: depth + bindings.len() - 1 - free,
                type_id: type_id.clone(),
            }
        }
        ContractTerm::Var { .. } | ContractTerm::Const { .. } => t.clone(),
        ContractTerm::App {
            function,
            argument,
            type_id,
        } => ContractTerm::App {
            function: Box::new(point_term(function, bindings, depth)?),
            argument: Box::new(point_term(argument, bindings, depth)?),
            type_id: type_id.clone(),
        },
        ContractTerm::Lam {
            parameter_type,
            body,
            type_id,
        } => ContractTerm::Lam {
            parameter_type: parameter_type.clone(),
            body: Box::new(point_term(body, bindings, depth + 1)?),
            type_id: type_id.clone(),
        },
        ContractTerm::Let {
            value,
            body,
            type_id,
        } => ContractTerm::Let {
            value: Box::new(point_term(value, bindings, depth)?),
            body: Box::new(point_term(body, bindings, depth + 1)?),
            type_id: type_id.clone(),
        },
    })
}
fn predicate(
    c: &mut Clauses<'_>,
    source: &ControlPredicate,
    definition: String,
    arguments: &mut Vec<ControlBinding>,
    native_guard: Option<&control_edges::OrdinaryControlEdgeDefinition>,
    function_id: &str,
) -> R<OrdinaryControlPredicateDefinition> {
    if source.term.type_id() != SOURCE_BOOL {
        return Err(OrdinaryCarrierError::Linkage);
    }
    let mut argument_indices = vec![];
    for binding in &source.bindings {
        let index = arguments
            .iter()
            .position(|a| a == binding)
            .unwrap_or_else(|| {
                arguments.push(binding.clone());
                arguments.len() - 1
            });
        argument_indices.push(index);
    }
    let term = point_term(&source.term, &source.bindings, 0)?;
    let mut requested = BTreeMap::new();
    constants(&term, &mut requested)?;
    let mut pending_constant_names = vec![];
    for (symbol, ty) in requested {
        if let Some((actual, _)) = c.constants.get(&symbol) {
            if actual != &ty {
                return Err(OrdinaryCarrierError::Linkage);
            }
        } else {
            pending_constant_names.push(symbol);
        }
    }
    let native_guard_dependency = if let Some(edge) = native_guard {
        if &edge.source.guard != source || !edge.pending_constant_names.is_empty() {
            return Err(OrdinaryCarrierError::Linkage);
        }
        Some(OrdinaryControlGuardDependency {
            function_id: function_id.into(),
            edge_id: edge.source.id.clone(),
            guard_definition: edge
                .guard_definition
                .clone()
                .ok_or(OrdinaryCarrierError::Linkage)?,
            ownership: edge.ownership.clone(),
        })
    } else {
        None
    };
    let definition = if let Some(dependency) = &native_guard_dependency {
        // The existing guard has this exact original argument order and point.
        // Keep its failure semantics and its recorded ownership prerequisites.
        pending_constant_names.clear();
        Some(dependency.guard_definition.clone())
    } else if pending_constant_names.is_empty() {
        let args = source
            .bindings
            .iter()
            .map(|b| b.type_id.clone())
            .collect::<Vec<_>>();
        let mut body = c.lower(&term, &mut args.clone(), 0)?;
        for ty in args.iter().rev() {
            let ty = c.ty(ty, 0)?;
            body = c.b.lam(ty, body)?;
        }
        let ty = c.ty(&signature(&args, SOURCE_BOOL), 0)?;
        c.b.define(&definition, ty, body)?;
        Some(definition)
    } else {
        None
    };
    Ok(OrdinaryControlPredicateDefinition {
        source: source.clone(),
        argument_indices,
        definition,
        pending_constant_names,
        native_guard_dependency,
    })
}
fn conjunction(
    b: &mut Builder,
    predicates: &[OrdinaryControlPredicateDefinition],
    count: usize,
) -> R<u32> {
    let mut body = bit(b, true)?;
    for p in predicates {
        let args = p
            .argument_indices
            .iter()
            .map(|&i| b.var((count - 1 - i) as u32))
            .collect::<R<Vec<_>>>()?;
        let value = call(
            b,
            p.definition
                .as_deref()
                .ok_or(OrdinaryCarrierError::Linkage)?,
            args,
        )?;
        body = call(b, "Std.Bool.and", vec![body, value])?;
    }
    Ok(body)
}
fn sequent(
    c: &mut Clauses<'_>,
    source: &ControlSequent,
    native: Option<&OrdinaryControlEdgeProgram>,
) -> R<OrdinaryControlSequentDefinition> {
    let mut arguments = vec![];
    let mut emit = |role: &str, predicates: &[ControlPredicate]| {
        predicates
            .iter()
            .enumerate()
            .map(|(i, p)| {
                let guard = native
                    .filter(|_| {
                        role == "Assumption" && i == 0 && source.id.starts_with("control.loop.")
                    })
                    .and_then(|native| {
                        native
                            .functions()
                            .iter()
                            .find(|f| f.source.function_id == source.function_id)
                    })
                    .and_then(|f| {
                        f.edges.iter().find(|edge| {
                            edge.guard_definition.is_some()
                                && edge.pending_constant_names.is_empty()
                                && edge.source.source_node_id == source.source_node_id
                                && edge.source.target_node_id == source.target_node_id
                                && source.id.ends_with(&format!(".{}", edge.source.id))
                                && edge.source.guard == *p
                        })
                    });
                predicate(
                    c,
                    p,
                    name(role, &(&source.id, i, p)),
                    &mut arguments,
                    guard,
                    &source.function_id,
                )
            })
            .collect::<R<Vec<_>>>()
    };
    let assumptions = emit("Assumption", &source.assumptions)?;
    let goals = emit("Goal", &source.goals)?;
    let mut pending_implication_reasons = vec![];
    if assumptions
        .iter()
        .chain(&goals)
        .any(|p| p.definition.is_none())
    {
        pending_implication_reasons.push("predicate_definitions".into());
    }
    if arguments.len() > 256 {
        pending_implication_reasons.push("combined_binder_limit".into());
    }
    let logical_implication_definition = if pending_implication_reasons.is_empty() {
        let prior = conjunction(&mut c.b, &assumptions, arguments.len())?;
        let goal = conjunction(&mut c.b, &goals, arguments.len())?;
        let no = call(&mut c.b, "Std.Bool.not", vec![prior])?;
        let mut body = call(&mut c.b, "Std.Bool.or", vec![no, goal])?;
        let args = arguments
            .iter()
            .map(|b| b.type_id.clone())
            .collect::<Vec<_>>();
        for ty in args.iter().rev() {
            let ty = c.ty(ty, 0)?;
            body = c.b.lam(ty, body)?;
        }
        let ty = c.ty(&signature(&args, SOURCE_BOOL), 0)?;
        let definition = name("LogicalImplication", source);
        c.b.define(&definition, ty, body)?;
        Some(definition)
    } else {
        None
    };
    Ok(OrdinaryControlSequentDefinition {
        source: source.clone(),
        arguments,
        assumptions,
        goals,
        logical_implication_definition,
        pending_implication_reasons,
    })
}
pub fn generate_csharp_practical_ordinary_control_predicates(
    vir: &ValidatedPracticalVir,
) -> R<OrdinaryControlPredicateProgram> {
    generate(vir, false, false, false, false, false)
}
/// Share exact native source definitions and reuse guards only at their source
/// edges. Source execution/loop induction still require application proofs.
pub fn generate_csharp_practical_ordinary_control_predicates_with_execution(
    vir: &ValidatedPracticalVir,
) -> R<OrdinaryControlPredicateProgram> {
    generate(vir, true, false, false, false, false)
}
/// Retain complete native path premises and physical observation transport for
/// original pattern steps. Pattern predicates and their proofs remain pending.
pub fn generate_csharp_practical_ordinary_control_predicates_with_pattern_scopes(
    vir: &ValidatedPracticalVir,
) -> R<OrdinaryControlPredicateProgram> {
    generate(vir, true, true, false, false, false)
}
/// Compose the original governing producer and consuming native path under
/// explicit premises. Establishing those premises still requires source proofs.
pub fn generate_csharp_practical_ordinary_control_predicates_with_pattern_captures(
    vir: &ValidatedPracticalVir,
) -> R<OrdinaryControlPredicateProgram> {
    generate(vir, true, true, true, false, false)
}
/// Compile original source conditions with exact operands and slot phases.
/// Execution establishment and native/source equivalence still require proofs.
pub fn generate_csharp_practical_ordinary_control_predicates_with_pattern_observations(
    vir: &ValidatedPracticalVir,
) -> R<OrdinaryControlPredicateProgram> {
    generate(vir, true, true, true, true, false)
}
/// Observe exact source successor choices under explicit native path premises.
pub fn generate_csharp_practical_ordinary_control_predicates_with_pattern_routes(
    vir: &ValidatedPracticalVir,
) -> R<OrdinaryControlPredicateProgram> {
    generate(vir, true, true, true, true, true)
}

fn generate(
    vir: &ValidatedPracticalVir,
    with_execution: bool,
    with_pattern_scopes: bool,
    with_pattern_captures: bool,
    with_pattern_observations: bool,
    with_pattern_routes: bool,
) -> R<OrdinaryControlPredicateProgram> {
    use crate::csharp_practical_vir_model::data_vc::DataDefinitionFamily;
    let data = crate::csharp_practical_vir_model::data_vc::generate_data_vcs(vir)
        .map_err(|_| OrdinaryCarrierError::Linkage)?;
    let control = if with_pattern_routes {
        crate::csharp_practical_vir_model::generate_csharp_practical_control_vcs_with_pattern_routes(vir)
            .map_err(|_| OrdinaryCarrierError::Linkage)?
    } else if with_pattern_observations {
        crate::csharp_practical_vir_model::generate_csharp_practical_control_vcs_with_pattern_observations(vir)
            .map_err(|_| OrdinaryCarrierError::Linkage)?
    } else {
        crate::csharp_practical_vir_model::generate_control_vcs(vir, &data)
            .map_err(|_| OrdinaryCarrierError::Linkage)?
    };
    let layouts = generate_csharp_practical_ordinary_carriers(vir)?;
    let mut requested = BTreeMap::new();
    for s in control.sequents() {
        for p in s.assumptions.iter().chain(&s.goals) {
            constants(&p.term, &mut requested)?;
        }
    }
    let needed = |d: &ContractDefinition| {
        requested.contains_key(&d.name)
            || requested
                .keys()
                .any(|n| n.starts_with(&format!("Mpk.CSharp.Data.ContractFails.{}.", d.name)))
    };
    let expressions = vir
        .contract_expressions()
        .iter()
        .filter(|e| e.definitions().iter().any(needed))
        .collect::<Vec<_>>();
    let (mut c, native) = if with_execution {
        let (c, native) = control_edges::emit_complete_program(vir, &layouts)?;
        let mut recipes = compiler(vir, &layouts, c.b, &expressions)?;
        recipes.relations = c.relations;
        recipes.storage = c.storage;
        (recipes, Some(native))
    } else {
        (
            compiler(vir, &layouts, Builder::new()?, &expressions)?,
            None,
        )
    };
    c.definedness_logic()?;
    for e in expressions {
        for d in e.definitions().iter().filter(|d| needed(d)) {
            c.recipe(d)?;
        }
    }
    for d in data.definitions().iter().filter(|d| {
        d.family == DataDefinitionFamily::IntegerBoolean
            && (requested.contains_key(&d.relation_name)
                || d.failure_names.iter().any(|n| requested.contains_key(n)))
    }) {
        if native.is_none() {
            integer_data::emit_definition(&mut c, d)?;
        }
    }
    let mut measures = vec![];
    for (symbol, expected) in &requested {
        let Some(d) = measure(&mut c.b, symbol)? else {
            continue;
        };
        let count = if d.operation == "NonNegative" { 1 } else { 2 };
        let ty = signature(&vec![d.type_id.clone(); count], SOURCE_BOOL);
        if &ty != expected
            || c.constants
                .insert(symbol.clone(), (ty, d.definition.clone()))
                .is_some()
        {
            return Err(OrdinaryCarrierError::Linkage);
        }
        measures.push(d);
    }
    let pattern_sources = if with_pattern_observations {
        pattern_sources::emit(
            &mut c,
            vir,
            &control,
            &layouts,
            native.as_ref().ok_or(OrdinaryCarrierError::Linkage)?,
        )?
    } else {
        vec![]
    };
    let sequents = control
        .sequents()
        .iter()
        .map(|s| sequent(&mut c, s, native.as_ref()))
        .collect::<R<Vec<_>>>()?;
    let pattern_scopes = if with_pattern_scopes {
        pattern_scopes::emit(
            &mut c,
            vir,
            &control,
            native.as_ref().ok_or(OrdinaryCarrierError::Linkage)?,
            None,
        )?
    } else {
        vec![]
    };
    let (pattern_captures, pattern_capture_scopes) = if with_pattern_captures {
        let native = native.as_ref().ok_or(OrdinaryCarrierError::Linkage)?;
        let captures = pattern_captures::emit(&mut c, vir, &control, native)?;
        let scopes = pattern_scopes::emit(&mut c, vir, &control, native, Some(&captures))?;
        (captures, scopes)
    } else {
        (vec![], vec![])
    };
    let certificate = c.b.finish()?;
    let p = OrdinaryControlPredicateProgram {
        schema: "mpk.csharp.ordinary_control_predicates.v1".into(),
        source_ir_sha256: vir.hash().into(),
        foundation_sha256: vir.construction_context().0.content_sha256().into(),
        control_vc_sha256: control.hash(),
        native_source_program_sha256: native
            .as_ref()
            .map(|p| format!("{:x}", Sha256::digest(p.canonical_bytes()))),
        native_source_certificate_sha256: native
            .as_ref()
            .map(|p| mpk_cert::hash_hex(&mpk_cert::certificate_hash(p.certificate_bytes()))),
        pattern_scopes,
        pattern_captures,
        pattern_capture_scopes,
        pattern_sources,
        measures,
        sequents,
        unresolved_regions: control.unresolved_regions().to_vec(),
        application_scope_pending: true,
        certificate_sha256: mpk_cert::hash_hex(&mpk_cert::certificate_hash(&certificate)),
        certificate,
    };
    if p.canonical_bytes().len() > 16 * 1024 * 1024 {
        return Err(OrdinaryCarrierError::Limit);
    }
    Ok(p)
}
pub fn import_csharp_practical_ordinary_control_predicates(
    input: &[u8],
    certificate: &[u8],
    vir: &ValidatedPracticalVir,
) -> R<OrdinaryControlPredicateProgram> {
    if input.len() > 16 * 1024 * 1024 || certificate.len() > 16 * 1024 * 1024 {
        return Err(OrdinaryCarrierError::Limit);
    }
    let p = generate_csharp_practical_ordinary_control_predicates(vir)?;
    if input != p.canonical_bytes() || certificate != p.certificate_bytes() {
        return Err(OrdinaryCarrierError::Linkage);
    }
    Ok(p)
}
pub fn import_csharp_practical_ordinary_control_predicates_with_execution(
    input: &[u8],
    certificate: &[u8],
    vir: &ValidatedPracticalVir,
) -> R<OrdinaryControlPredicateProgram> {
    if input.len() > 16 * 1024 * 1024 || certificate.len() > 16 * 1024 * 1024 {
        return Err(OrdinaryCarrierError::Limit);
    }
    let p = generate_csharp_practical_ordinary_control_predicates_with_execution(vir)?;
    if input != p.canonical_bytes() || certificate != p.certificate_bytes() {
        return Err(OrdinaryCarrierError::Linkage);
    }
    Ok(p)
}
pub fn import_csharp_practical_ordinary_control_predicates_with_pattern_scopes(
    input: &[u8],
    certificate: &[u8],
    vir: &ValidatedPracticalVir,
) -> R<OrdinaryControlPredicateProgram> {
    if input.len() > 16 * 1024 * 1024 || certificate.len() > 16 * 1024 * 1024 {
        return Err(OrdinaryCarrierError::Limit);
    }
    let p = generate_csharp_practical_ordinary_control_predicates_with_pattern_scopes(vir)?;
    if input != p.canonical_bytes() || certificate != p.certificate_bytes() {
        return Err(OrdinaryCarrierError::Linkage);
    }
    Ok(p)
}
pub fn import_csharp_practical_ordinary_control_predicates_with_pattern_captures(
    input: &[u8],
    certificate: &[u8],
    vir: &ValidatedPracticalVir,
) -> R<OrdinaryControlPredicateProgram> {
    if input.len() > 16 * 1024 * 1024 || certificate.len() > 16 * 1024 * 1024 {
        return Err(OrdinaryCarrierError::Limit);
    }
    let p = generate_csharp_practical_ordinary_control_predicates_with_pattern_captures(vir)?;
    if input != p.canonical_bytes() || certificate != p.certificate_bytes() {
        return Err(OrdinaryCarrierError::Linkage);
    }
    Ok(p)
}
pub fn import_csharp_practical_ordinary_control_predicates_with_pattern_routes(
    input: &[u8],
    certificate: &[u8],
    vir: &ValidatedPracticalVir,
) -> R<OrdinaryControlPredicateProgram> {
    if input.len() > 16 * 1024 * 1024 || certificate.len() > 16 * 1024 * 1024 {
        return Err(OrdinaryCarrierError::Limit);
    }
    let program = generate_csharp_practical_ordinary_control_predicates_with_pattern_routes(vir)?;
    if input != program.canonical_bytes() || certificate != program.certificate_bytes() {
        return Err(OrdinaryCarrierError::Linkage);
    }
    Ok(program)
}

pub fn import_csharp_practical_ordinary_control_predicates_with_pattern_observations(
    input: &[u8],
    certificate: &[u8],
    vir: &ValidatedPracticalVir,
) -> R<OrdinaryControlPredicateProgram> {
    if input.len() > 16 * 1024 * 1024 || certificate.len() > 16 * 1024 * 1024 {
        return Err(OrdinaryCarrierError::Limit);
    }
    let p = generate_csharp_practical_ordinary_control_predicates_with_pattern_observations(vir)?;
    if input != p.canonical_bytes() || certificate != p.certificate_bytes() {
        return Err(OrdinaryCarrierError::Linkage);
    }
    Ok(p)
}

#[cfg(test)]
mod tests {
    use super::super::super::super::test_eval::{bit as observed, run, V};
    use super::*;

    #[test]
    fn ordinary_control_measures_match_mathematical_integer_boundaries() {
        let mut b = Builder::new().unwrap();
        let mut cases = vec![];
        let mut metadata = vec![];
        for (token, width, signed) in [
            ("i8", 8, true),
            ("u8", 8, false),
            ("i16", 16, true),
            ("u16", 16, false),
            ("i32", 32, true),
            ("u32", 32, false),
            ("i64", 64, true),
            ("u64", 64, false),
        ] {
            let mut names = BTreeMap::new();
            for op in ["NonNegative", "MathLess", "MathEqual"] {
                let d = measure(
                    &mut b,
                    &format!("Mpk.CSharp.Integer.{op}.mpk.csharp.value.{token}.v1"),
                )
                .unwrap()
                .unwrap();
                names.insert(op, d.definition.clone());
                metadata.push(d);
            }
            cases.push((width, signed, names));
        }
        let bytes = b.finish().unwrap();
        let cert = mpk_cert::decode_canonical_certificate(&bytes).unwrap();
        let word = |n: u64, width: u32| V::Cube((0..width).map(|i| n & (1u64 << i) != 0).collect());
        let mut observations = 0;
        for (width, signed, names) in cases {
            let mask = u64::MAX >> (64 - width);
            let sign = 1u64 << (width - 1);
            let samples = [0, 1, sign - 1, sign, sign + 1, mask - 1, mask];
            let mathematical = |n: u64| {
                if signed && n & sign != 0 {
                    n as i128 - (1i128 << width)
                } else {
                    n as i128
                }
            };
            for left in samples {
                assert_eq!(
                    observed(run(&cert, &names["NonNegative"], vec![word(left, width)])),
                    mathematical(left) >= 0,
                    "width {width}, signed {signed}, value {left}"
                );
                observations += 1;
            }
            // Mathematical values are the test oracle, not core definitions.
            // Single-bit changes cover every leaf, including the high u64 bit.
            for (left, right) in samples
                .into_iter()
                .flat_map(|a| samples.into_iter().map(move |d| (a, d)))
                .chain((0..width).flat_map(|i| [(0, 1u64 << i), (1u64 << i, 0)]))
            {
                let args = vec![word(left, width), word(right, width)];
                assert_eq!(
                    observed(run(&cert, &names["MathLess"], args.clone())),
                    mathematical(left) < mathematical(right),
                    "width {width}, signed {signed}: {left} < {right}"
                );
                assert_eq!(
                    observed(run(&cert, &names["MathEqual"], args)),
                    mathematical(left) == mathematical(right),
                    "width {width}, signed {signed}: {left} == {right}"
                );
                observations += 2;
            }
        }
        assert_eq!(observations, 1800);
        let metadata = serde_json::to_vec(&serde_json::json!({
            "definitions": metadata, "observations": observations
        }))
        .unwrap();
        let hex = bytes.iter().map(|v| format!("{v:02x}")).collect::<String>() + "\n";
        if let Some(dir) = std::env::var_os("MPK_W09_CONTROL_PREDICATE_OUTPUT") {
            let dir = std::path::PathBuf::from(dir);
            std::fs::create_dir_all(&dir).unwrap();
            std::fs::write(dir.join("measures.mpcert"), &bytes).unwrap();
            std::fs::write(dir.join("measures.hex"), hex).unwrap();
            std::fs::write(dir.join("measures.json"), metadata).unwrap();
        } else {
            let dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../../develop/migrations/csharp-03/ordinary-foundation/control-predicates");
            assert_eq!(
                std::fs::read(dir.join("measures.hex")).unwrap(),
                hex.as_bytes()
            );
            assert_eq!(std::fs::read(dir.join("measures.json")).unwrap(), metadata);
        }
    }

    fn binding(ty: &str, edge: Option<&str>) -> ControlBinding {
        ControlBinding {
            kind: "current_slot".into(),
            edge_id: edge.map(str::to_owned),
            node_id: "header".into(),
            value_id: "local:0".into(),
            type_id: ty.into(),
        }
    }
    #[test]
    fn ordinary_control_predicate_mapping_preserves_local_binders_and_snapshots() {
        let int = "mpk.csharp.value.i32.v1";
        let wide = "mpk.csharp.value.u64.v1";
        let bindings = vec![
            binding(int, None),
            binding(SOURCE_BOOL, None),
            binding(wide, None),
        ];
        // Two local binders followed by three original free binding indices.
        for (index, ty, expected) in [
            (0, int, 0),
            (1, SOURCE_BOOL, 1),
            (2, int, 4),
            (3, SOURCE_BOOL, 3),
            (4, wide, 2),
        ] {
            let term = ContractTerm::Var {
                index,
                type_id: ty.into(),
            };
            assert_eq!(
                point_term(&term, &bindings, 2).unwrap(),
                ContractTerm::Var {
                    index: expected,
                    type_id: ty.into(),
                }
            );
        }
        let nested = ContractTerm::Lam {
            parameter_type: SOURCE_BOOL.into(),
            body: Box::new(ContractTerm::Let {
                value: Box::new(ContractTerm::Var {
                    index: 1,
                    type_id: int.into(),
                }),
                body: Box::new(ContractTerm::Var {
                    index: 4,
                    type_id: wide.into(),
                }),
                type_id: wide.into(),
            }),
            type_id: format!("({SOURCE_BOOL}->{wide})"),
        };
        let ContractTerm::Lam { body, .. } = point_term(&nested, &bindings, 0).unwrap() else {
            panic!("lambda")
        };
        let ContractTerm::Let { value, body, .. } = *body else {
            panic!("let")
        };
        assert_eq!(
            *value,
            ContractTerm::Var {
                index: 3,
                type_id: int.into()
            }
        );
        assert_eq!(
            *body,
            ContractTerm::Var {
                index: 2,
                type_id: wide.into()
            }
        );
        let source = ControlPredicate {
            bindings: vec![binding(int, None), binding(int, Some("backedge"))],
            term: ContractTerm::Const {
                name: "Mpk.CSharp.Control.GeneratedLoop.pending.entry".into(),
                type_id: SOURCE_BOOL.into(),
            },
        };
        let layouts = OrdinaryCarrierProgram {
            schema: String::new(),
            source_ir_sha256: String::new(),
            foundation_sha256: String::new(),
            carriers: vec![],
            certificate_sha256: String::new(),
            certificate: vec![],
        };
        let mut c = Clauses {
            vir: None,
            relations: Default::default(),
            b: Builder::new().unwrap(),
            carriers: layouts
                .carriers()
                .iter()
                .map(|c| (c.type_id.as_str(), c))
                .collect(),
            storage: Default::default(),
            constants: Default::default(),
            scalars: Default::default(),
            strings: Default::default(),
            finite: None,
            codecs: Default::default(),
            binding_projections: None,
            quantifiers: Default::default(),
        };
        let mut arguments = vec![];
        let p = predicate(
            &mut c,
            &source,
            "unused".into(),
            &mut arguments,
            None,
            "function",
        )
        .unwrap();
        assert_eq!(arguments, source.bindings);
        assert_eq!(p.argument_indices, [0, 1]);
        assert!(p.definition.is_none());
        assert_eq!(
            p.pending_constant_names,
            ["Mpk.CSharp.Control.GeneratedLoop.pending.entry"]
        );
        let again = predicate(
            &mut c,
            &source,
            "unused".into(),
            &mut arguments,
            None,
            "function",
        )
        .unwrap();
        assert_eq!(arguments.len(), 2);
        assert_eq!(again.argument_indices, [0, 1]);
        assert!(point_term(
            &ContractTerm::Var {
                index: 2,
                type_id: SOURCE_BOOL.into()
            },
            &source.bindings,
            0,
        )
        .is_err());
        for token in ["bool", "char", "f32", "decimal"] {
            assert!(measure(
                &mut c.b,
                &format!("Mpk.CSharp.Integer.MathLess.mpk.csharp.value.{token}.v1")
            )
            .is_err());
        }
    }
}
