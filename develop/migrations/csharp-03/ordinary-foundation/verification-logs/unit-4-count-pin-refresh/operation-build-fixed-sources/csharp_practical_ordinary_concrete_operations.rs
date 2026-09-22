//! Closed operation recipes, exact normal bodies and ordered outcome comparison.
//! Source ownership/currency predicates and application proofs are never assumed.
use super::super::domains::{extend_structural_public, StructuralDefinitionRefs};
use super::*;

const U32_TYPE_ID: &str = "mpk.csharp.value.u32.v1";

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct OrdinaryConcreteFailureComponent {
    pub label: String,
    pub definition: Option<String>,
    pub argument_indices: Vec<usize>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct OrdinaryConcreteOperationComponent {
    pub operation_id: String,
    pub argument_type_ids: Vec<String>,
    pub result_type_id: String,
    pub normal_definition: String,
    pub failures: Vec<OrdinaryConcreteFailureComponent>,
    pub currency_predicate_argument_type_id: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct OrdinaryConcreteOperationFailure {
    pub label: String,
    pub check_symbol: String,
    pub failure_definition: String,
    pub concrete_failure_definition: String,
    pub first_failure_definition: String,
    pub concrete_first_failure_definition: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct OrdinaryConcreteOperationDefinition {
    pub instance_id: String,
    pub recipe: Value,
    pub component: OrdinaryConcreteOperationComponent,
    /// Present only when this exact operation is in the original VIR signatures.
    pub signature: Option<ClosedOperationSignature>,
    pub normal_definition: String,
    pub concrete_symbol: String,
    pub concrete_definition: String,
    pub failures: Vec<OrdinaryConcreteOperationFailure>,
    pub success_definition: String,
    pub concrete_success_definition: String,
    pub normal_agreement_definition: String,
    pub all_outcomes_symbol: String,
    pub all_outcomes_definition: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum OrdinaryConcreteOperationPendingReason {
    InternalConstructionState,
    SourceOwnership,
    ApplicationCurrencyPredicate,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct OrdinaryConcreteOperationPending {
    pub instance_id: String,
    pub recipe: Value,
    pub component: OrdinaryConcreteOperationComponent,
    pub reasons: Vec<OrdinaryConcreteOperationPendingReason>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct OrdinaryConcreteOperationProgram {
    schema: String,
    source_ir_sha256: String,
    foundation_sha256: String,
    binding_vc_sha256: String,
    instances: Vec<FoundationInstanceVc>,
    public_domains: Vec<OrdinaryPublicDomainDefinition>,
    source_clauses: Vec<OrdinarySourceClauseDefinition>,
    source_observations: Vec<OrdinaryObservationDefinition>,
    definitions: Vec<OrdinaryConcreteOperationDefinition>,
    pending_operations: Vec<OrdinaryConcreteOperationPending>,
    conditions: Vec<OrdinaryBindingCondition>,
    pending_condition_ids: Vec<String>,
    pending_proof_ids: Vec<String>,
    static_transformers: usize,
    certificate_sha256: String,
    #[serde(skip)]
    certificate: Vec<u8>,
}
impl OrdinaryConcreteOperationProgram {
    pub fn instances(&self) -> &[FoundationInstanceVc] {
        &self.instances
    }
    pub fn public_domains(&self) -> &[OrdinaryPublicDomainDefinition] {
        &self.public_domains
    }
    pub fn source_clauses(&self) -> &[OrdinarySourceClauseDefinition] {
        &self.source_clauses
    }
    pub fn source_observations(&self) -> &[OrdinaryObservationDefinition] {
        &self.source_observations
    }
    pub fn definitions(&self) -> &[OrdinaryConcreteOperationDefinition] {
        &self.definitions
    }
    pub fn pending_operations(&self) -> &[OrdinaryConcreteOperationPending] {
        &self.pending_operations
    }
    pub fn conditions(&self) -> &[OrdinaryBindingCondition] {
        &self.conditions
    }
    pub fn pending_condition_ids(&self) -> &[String] {
        &self.pending_condition_ids
    }
    pub fn pending_proof_ids(&self) -> &[String] {
        &self.pending_proof_ids
    }
    pub fn certificate_bytes(&self) -> &[u8] {
        &self.certificate
    }
    pub fn canonical_bytes(&self) -> Vec<u8> {
        serde_json::to_vec(self).expect("ordinary concrete operation program")
    }
}

fn strings(v: &Value, field: &str) -> R<Vec<String>> {
    v[field]
        .as_array()
        .ok_or(OrdinaryCarrierError::Linkage)?
        .iter()
        .map(|v| {
            v.as_str()
                .map(str::to_owned)
                .ok_or(OrdinaryCarrierError::Linkage)
        })
        .collect()
}

fn component(
    id: String,
    args: Vec<String>,
    result: String,
    normal: String,
    failures: Vec<OrdinaryConcreteFailureComponent>,
) -> OrdinaryConcreteOperationComponent {
    OrdinaryConcreteOperationComponent {
        operation_id: id,
        argument_type_ids: args,
        result_type_id: result,
        normal_definition: normal,
        failures,
        currency_predicate_argument_type_id: None,
    }
}

fn failure(
    label: &str,
    definition: &str,
    arguments: Vec<usize>,
) -> OrdinaryConcreteFailureComponent {
    OrdinaryConcreteFailureComponent {
        label: label.into(),
        definition: Some(definition.into()),
        argument_indices: arguments,
    }
}

fn catalog(
    view: &StructuralDefinitionRefs<'_>,
) -> R<BTreeMap<String, OrdinaryConcreteOperationComponent>> {
    let mut result = BTreeMap::new();
    let mut add = |op: OrdinaryConcreteOperationComponent| {
        if op.failures.iter().any(|f| {
            f.argument_indices
                .iter()
                .any(|i| *i >= op.argument_type_ids.len())
        }) || result.insert(op.operation_id.clone(), op).is_some()
        {
            return Err(OrdinaryCarrierError::Linkage);
        }
        Ok(())
    };
    for d in view.sequences {
        let id = &d.carrier.type_id;
        add(component(
            format!("{id}.length"),
            vec![id.clone()],
            U32_TYPE_ID.into(),
            d.length_definition.clone(),
            vec![],
        ))?;
        add(component(
            format!("{id}.read"),
            vec![id.clone(), I32_TYPE_ID.into()],
            d.element_type_id.clone(),
            d.read_definition.clone(),
            vec![failure(
                "index_range",
                &d.index_range_definition,
                vec![0, 1],
            )],
        ))?;
        add(component(
            format!("{id}.equal"),
            vec![id.clone(), id.clone()],
            BOOL_TYPE_ID.into(),
            d.equality_definition.clone(),
            vec![],
        ))?;
        if let Some(cmp) = &d.compare_definition {
            add(component(
                format!("{id}.compare"),
                vec![id.clone(), id.clone()],
                I32_TYPE_ID.into(),
                cmp.clone(),
                vec![],
            ))?;
        }
    }
    for d in view.entries {
        let id = &d.carrier.type_id;
        add(component(
            format!("{id}.make"),
            vec![d.key_type_id.clone(), d.value_type_id.clone()],
            id.clone(),
            d.make_definition.clone(),
            vec![],
        ))?;
        add(component(
            format!("{id}.key"),
            vec![id.clone()],
            d.key_type_id.clone(),
            d.key_definition.clone(),
            vec![],
        ))?;
        add(component(
            format!("{id}.value"),
            vec![id.clone()],
            d.value_type_id.clone(),
            d.value_definition.clone(),
            vec![],
        ))?;
        add(component(
            format!("{id}.equal"),
            vec![id.clone(), id.clone()],
            BOOL_TYPE_ID.into(),
            d.equality_definition.clone(),
            vec![],
        ))?;
        if let Some(cmp) = &d.compare_definition {
            add(component(
                format!("{id}.compare"),
                vec![id.clone(), id.clone()],
                I32_TYPE_ID.into(),
                cmp.clone(),
                vec![],
            ))?;
        }
    }
    for d in view.constructions {
        for op in &d.operations {
            add(component(
                op.operation_id.clone(),
                op.argument_type_ids.clone(),
                op.result_type_id.clone(),
                op.normal_definition.clone(),
                op.failures
                    .iter()
                    .map(|f| OrdinaryConcreteFailureComponent {
                        label: f.label.clone(),
                        definition: f.definition.clone(),
                        argument_indices: f.argument_indices.clone(),
                    })
                    .collect(),
            ))?;
        }
    }
    for d in view.outcomes {
        for op in &d.operations {
            add(component(
                op.operation_id.clone(),
                op.argument_type_ids.clone(),
                op.result_type_id.clone(),
                op.normal_definition.clone(),
                op.failures
                    .iter()
                    .map(|f| failure(&f.label, &f.definition, f.argument_indices.clone()))
                    .collect(),
            ))?;
        }
    }
    for d in view.collections {
        for op in &d.operations {
            add(component(
                op.operation_id.clone(),
                op.argument_type_ids.clone(),
                op.result_type_id.clone(),
                op.normal_definition.clone(),
                op.failures
                    .iter()
                    .map(|f| failure(&f.label, &f.definition, f.argument_indices.clone()))
                    .collect(),
            ))?;
        }
    }
    for d in view.money {
        for op in &d.operations {
            let mut c = component(
                op.operation_id.clone(),
                op.argument_type_ids.clone(),
                op.result_type_id.clone(),
                op.normal_definition.clone(),
                op.failures
                    .iter()
                    .map(|f| {
                        failure(
                            &f.label,
                            &f.definition,
                            (0..op.argument_type_ids.len()).collect(),
                        )
                    })
                    .collect(),
            );
            c.currency_predicate_argument_type_id = op.currency_predicate_argument_type_id.clone();
            add(c)?;
        }
    }
    for d in view.transitions {
        let id = &d.carrier.type_id;
        add(component(
            format!("{id}.make"),
            vec![
                d.state_type_id.clone(),
                d.events_type_id.clone(),
                d.response_type_id.clone(),
            ],
            id.clone(),
            d.make_definition.clone(),
            vec![failure(
                "event_bound",
                &d.event_bound_definition,
                vec![d.event_bound_argument_index],
            )],
        ))?;
        for (suffix, result, normal) in [
            ("state", &d.state_type_id, &d.state_definition),
            ("events", &d.events_type_id, &d.events_definition),
            ("response", &d.response_type_id, &d.response_definition),
        ] {
            add(component(
                format!("{id}.{suffix}"),
                vec![id.clone()],
                result.clone(),
                normal.clone(),
                vec![],
            ))?;
        }
        add(component(
            format!("{id}.equal"),
            vec![id.clone(), id.clone()],
            BOOL_TYPE_ID.into(),
            d.equality_definition.clone(),
            vec![],
        ))?;
        if let Some(cmp) = &d.compare_definition {
            add(component(
                format!("{id}.compare"),
                vec![id.clone(), id.clone()],
                I32_TYPE_ID.into(),
                cmp.clone(),
                vec![],
            ))?;
        }
    }
    Ok(result)
}

fn args(b: &mut Builder, count: usize) -> R<Vec<u32>> {
    (0..count).map(|i| b.var((count - 1 - i) as u32)).collect()
}

fn emit_operation(
    r: &mut Relations<'_>,
    instance: &FoundationInstanceVc,
    recipe: &Value,
    c: OrdinaryConcreteOperationComponent,
    symbols: &mut BTreeMap<String, String>,
) -> R<OrdinaryConcreteOperationDefinition> {
    let count = c.argument_type_ids.len();
    let depth = |id: &str| {
        r.carriers
            .get(id)
            .map(|c| c.depth)
            .ok_or(OrdinaryCarrierError::Linkage)
    };
    let depths = c
        .argument_type_ids
        .iter()
        .map(|id| depth(id))
        .collect::<R<Vec<_>>>()?;
    let output = depth(&c.result_type_id)?;
    let hash = hash_value(
        HashDomain::new("MPK-CSHARP-CONCRETE-DEFINITION-1.0"),
        recipe,
    )
    .map_err(|_| OrdinaryCarrierError::Linkage)?;
    let concrete_symbol = format!("Mpk.CSharp.Concrete.Definition.{hash}");
    let normal_definition = name(&format!("concrete.operation.{}.normal", c.operation_id));
    let concrete_definition = name(&concrete_symbol);
    for definition in [&normal_definition, &concrete_definition] {
        let values = args(&mut r.b, count)?;
        let body = call(&mut r.b, &c.normal_definition, values)?;
        define(&mut r.b, definition, &depths, output, body)?;
    }
    symbols.insert(c.operation_id.clone(), normal_definition.clone());
    symbols.insert(concrete_symbol.clone(), concrete_definition.clone());
    let boolean_equal = r.raw(1, false)?.equal;
    let mut failures = vec![];
    let mut actual_clear = bit(&mut r.b, true)?;
    let mut concrete_clear = bit(&mut r.b, true)?;
    let mut outcomes = bit(&mut r.b, true)?;
    for f in &c.failures {
        let implementation = f.definition.as_ref().ok_or(OrdinaryCarrierError::Linkage)?;
        let check_symbol = format!("Mpk.CSharp.Concrete.Failure.{}.{}", c.operation_id, f.label);
        let failure_definition = name(&check_symbol);
        let concrete_failure_definition =
            name(&format!("concrete.definition.{hash}.failure.{}", f.label));
        for definition in [&failure_definition, &concrete_failure_definition] {
            let values = args(&mut r.b, count)?;
            let selected = f.argument_indices.iter().map(|i| values[*i]).collect();
            let body = call(&mut r.b, implementation, selected)?;
            define(&mut r.b, definition, &depths, 0, body)?;
        }
        symbols.insert(check_symbol.clone(), failure_definition.clone());
        let values = args(&mut r.b, count)?;
        let actual = call(&mut r.b, &failure_definition, values.clone())?;
        let concrete = call(&mut r.b, &concrete_failure_definition, values)?;
        let actual_first = and(&mut r.b, actual_clear, actual)?;
        let concrete_first = and(&mut r.b, concrete_clear, concrete)?;
        let first_failure_definition = name(&format!(
            "concrete.operation.{}.first.{}",
            c.operation_id, f.label
        ));
        let concrete_first_failure_definition =
            name(&format!("concrete.definition.{hash}.first.{}", f.label));
        define(
            &mut r.b,
            &first_failure_definition,
            &depths,
            0,
            actual_first,
        )?;
        define(
            &mut r.b,
            &concrete_first_failure_definition,
            &depths,
            0,
            concrete_first,
        )?;
        let same = call(&mut r.b, &boolean_equal, vec![actual_first, concrete_first])?;
        outcomes = and(&mut r.b, outcomes, same)?;
        let no_actual = call(&mut r.b, "Std.Bool.not", vec![actual])?;
        let no_concrete = call(&mut r.b, "Std.Bool.not", vec![concrete])?;
        actual_clear = and(&mut r.b, actual_clear, no_actual)?;
        concrete_clear = and(&mut r.b, concrete_clear, no_concrete)?;
        failures.push(OrdinaryConcreteOperationFailure {
            label: f.label.clone(),
            check_symbol,
            failure_definition,
            concrete_failure_definition,
            first_failure_definition,
            concrete_first_failure_definition,
        });
    }
    let success_definition = name(&format!("concrete.operation.{}.success", c.operation_id));
    let concrete_success_definition = name(&format!("concrete.definition.{hash}.success"));
    define(&mut r.b, &success_definition, &depths, 0, actual_clear)?;
    define(
        &mut r.b,
        &concrete_success_definition,
        &depths,
        0,
        concrete_clear,
    )?;
    let same = call(&mut r.b, &boolean_equal, vec![actual_clear, concrete_clear])?;
    outcomes = and(&mut r.b, outcomes, same)?;
    let values = args(&mut r.b, count)?;
    let actual = call(&mut r.b, &normal_definition, values.clone())?;
    let concrete = call(&mut r.b, &concrete_definition, values)?;
    let equal_symbol = format!("Mpk.CSharp.Binding.Equal.{}", c.result_type_id);
    // Proof-level complete value observation, including NaN payloads, is distinct
    // from a foundation's potentially non-reflexive source `.equal` operation.
    let equality = symbols
        .get(&equal_symbol)
        .ok_or(OrdinaryCarrierError::Linkage)?;
    let agreement = call(&mut r.b, equality, vec![actual, concrete])?;
    let normal_agreement_definition = name(&format!(
        "concrete.operation.{}.normal_agreement",
        c.operation_id
    ));
    define(
        &mut r.b,
        &normal_agreement_definition,
        &depths,
        0,
        agreement,
    )?;
    let failed = call(&mut r.b, "Std.Bool.not", vec![actual_clear])?;
    let normal_outcome = call(&mut r.b, "Std.Bool.or", vec![failed, agreement])?;
    outcomes = and(&mut r.b, outcomes, normal_outcome)?;
    let all_outcomes_symbol = format!("Mpk.CSharp.Concrete.AllOutcomes.{}", c.operation_id);
    let all_outcomes_definition = name(&all_outcomes_symbol);
    define(&mut r.b, &all_outcomes_definition, &depths, 0, outcomes)?;
    symbols.insert(all_outcomes_symbol.clone(), all_outcomes_definition.clone());
    let signature = r
        .vir
        .operation_signatures()
        .iter()
        .find(|s| s.id == c.operation_id)
        .cloned();
    Ok(OrdinaryConcreteOperationDefinition {
        instance_id: instance.instance_id.clone(),
        recipe: recipe.clone(),
        component: c,
        signature,
        normal_definition,
        concrete_symbol,
        concrete_definition,
        failures,
        success_definition,
        concrete_success_definition,
        normal_agreement_definition,
        all_outcomes_symbol,
        all_outcomes_definition,
    })
}

pub fn generate_csharp_practical_ordinary_concrete_operations(
    vir: &ValidatedPracticalVir,
) -> R<OrdinaryConcreteOperationProgram> {
    let construction = generate_construction_vcs(vir).map_err(|_| OrdinaryCarrierError::Linkage)?;
    let vc = generate_binding_vcs(vir, &construction).map_err(|_| OrdinaryCarrierError::Linkage)?;
    let (certificate, static_transformers, extra) = extend_structural_public(vir, |r, view| {
        let mut components = catalog(&view)?;
        let mut symbols = view
            .public_domains
            .iter()
            .map(|d| (d.symbol.clone(), d.valid_definition.clone()))
            .collect::<BTreeMap<_, _>>();
        for d in view.source_observations {
            symbols.insert(
                format!("Mpk.CSharp.Binding.Equal.{}", d.carrier.type_id),
                d.equality_definition.clone(),
            );
        }
        conditions::boolean_symbols(&mut symbols);
        let mut definitions = vec![];
        let mut pending_operations = vec![];
        for instance in vc.instances() {
            for recipe in &instance.operation_definitions {
                let id = text(recipe, "id")?;
                let c = components.remove(id).ok_or(OrdinaryCarrierError::Linkage)?;
                let errors = strings(recipe, "error_precedence")?;
                if c.argument_type_ids != strings(recipe, "argument_type_ids")?
                    || c.result_type_id != text(recipe, "normal_result_type_id")?
                    || c.failures.iter().map(|f| &f.label).ne(errors.iter())
                {
                    return Err(OrdinaryCarrierError::Linkage);
                }
                if let Some(s) = vir.operation_signatures().iter().find(|s| s.id == id) {
                    if s.argument_type_ids != c.argument_type_ids
                        || s.normal_result_type_id != c.result_type_id
                        || s.ordered_checks.iter().map(|x| &x.id).ne(errors.iter())
                    {
                        return Err(OrdinaryCarrierError::Linkage);
                    }
                }
                let mut reasons = vec![];
                if c.argument_type_ids
                    .iter()
                    .chain(std::iter::once(&c.result_type_id))
                    .any(|id| r.internal(id))
                {
                    reasons.push(OrdinaryConcreteOperationPendingReason::InternalConstructionState);
                }
                if c.failures.iter().any(|f| f.definition.is_none()) {
                    reasons.push(OrdinaryConcreteOperationPendingReason::SourceOwnership);
                }
                if c.currency_predicate_argument_type_id.is_some() {
                    reasons
                        .push(OrdinaryConcreteOperationPendingReason::ApplicationCurrencyPredicate);
                }
                if reasons.is_empty() {
                    definitions.push(emit_operation(r, instance, recipe, c, &mut symbols)?);
                } else {
                    pending_operations.push(OrdinaryConcreteOperationPending {
                        instance_id: instance.instance_id.clone(),
                        recipe: recipe.clone(),
                        component: c,
                        reasons,
                    });
                }
            }
        }
        if !components.is_empty() {
            return Err(OrdinaryCarrierError::Linkage);
        }
        let defined = definitions
            .iter()
            .map(|d| d.component.operation_id.as_str())
            .collect::<BTreeSet<_>>();
        let mut compiled = vec![];
        let mut pending_condition_ids = vec![];
        for sequent in vc.sequents() {
            if sequent.kind == "concrete_definition_equivalence"
                && defined.contains(sequent.owner_id.as_str())
            {
                compiled.push(conditions::operation_obligation(
                    r,
                    sequent,
                    &symbols,
                    "concrete_operation",
                )?);
            } else {
                pending_condition_ids.push(sequent.id.clone());
            }
        }
        Ok((
            definitions,
            pending_operations,
            compiled,
            pending_condition_ids,
            view.public_domains.to_vec(),
            view.source_clauses.to_vec(),
            view.source_observations.to_vec(),
        ))
    })?;
    let (
        definitions,
        pending_operations,
        conditions,
        pending_condition_ids,
        public_domains,
        source_clauses,
        source_observations,
    ) = extra;
    let p = OrdinaryConcreteOperationProgram {
        schema: "mpk.csharp.ordinary_concrete_operations.v1".into(),
        source_ir_sha256: vir.hash().into(),
        foundation_sha256: vir.construction_context().0.content_sha256().into(),
        binding_vc_sha256: vc.hash(),
        instances: vc.instances().to_vec(),
        public_domains,
        source_clauses,
        source_observations,
        definitions,
        pending_operations,
        conditions,
        pending_condition_ids,
        pending_proof_ids: vc.sequents().iter().map(|s| s.id.clone()).collect(),
        static_transformers,
        certificate_sha256: mpk_cert::hash_hex(&mpk_cert::certificate_hash(&certificate)),
        certificate,
    };
    if p.canonical_bytes().len() > 16 * 1024 * 1024 {
        return Err(OrdinaryCarrierError::Limit);
    }
    Ok(p)
}

pub fn import_csharp_practical_ordinary_concrete_operations(
    input: &[u8],
    certificate: &[u8],
    vir: &ValidatedPracticalVir,
) -> R<OrdinaryConcreteOperationProgram> {
    if input.len() > 16 * 1024 * 1024 || certificate.len() > 16 * 1024 * 1024 {
        return Err(OrdinaryCarrierError::Limit);
    }
    let expected = generate_csharp_practical_ordinary_concrete_operations(vir)?;
    if input != expected.canonical_bytes() || certificate != expected.certificate_bytes() {
        return Err(OrdinaryCarrierError::Linkage);
    }
    Ok(expected)
}
