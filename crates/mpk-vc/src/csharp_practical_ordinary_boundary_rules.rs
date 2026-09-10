//! Exact W07 missing/null predicates over projected source values.
//! Source domains, input admission and reconstruction proofs remain obligations.
use super::*;

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct OrdinaryBoundaryRuleProgram {
    schema: String,
    source_ir_sha256: String,
    foundation_sha256: String,
    boundary_vc_sha256: String,
    binding_vc_sha256: String,
    projections: Vec<OrdinaryBindingProjectionDefinition>,
    predicates: Vec<OrdinaryBindingPredicate>,
    values: Vec<OrdinaryLiteralDefinition>,
    unresolved_boundary_vc_symbols: Vec<String>,
    static_transformers: usize,
    certificate_sha256: String,
    #[serde(skip)]
    certificate: Vec<u8>,
}
impl OrdinaryBoundaryRuleProgram {
    pub fn canonical_bytes(&self) -> Vec<u8> {
        serde_json::to_vec(self).expect("ordinary boundary rules")
    }
    pub fn certificate_bytes(&self) -> &[u8] {
        &self.certificate
    }
    pub fn predicates(&self) -> &[OrdinaryBindingPredicate] {
        &self.predicates
    }
    pub fn values(&self) -> &[OrdinaryLiteralDefinition] {
        &self.values
    }
    pub fn unresolved_boundary_vc_symbols(&self) -> &[String] {
        &self.unresolved_boundary_vc_symbols
    }
}

struct Rule {
    symbol: String,
    source: String,
    semantic: String,
    value: Option<String>,
}

pub fn generate_csharp_practical_ordinary_boundary_rules(
    emitted: &EmittedDataPhase,
) -> R<OrdinaryBoundaryRuleProgram> {
    let vir = emitted.vir();
    let boundary = generate_boundary_vcs(vir).map_err(|_| OrdinaryCarrierError::Linkage)?;
    let construction = generate_construction_vcs(vir).map_err(|_| OrdinaryCarrierError::Linkage)?;
    let binding =
        generate_binding_vcs(vir, &construction).map_err(|_| OrdinaryCarrierError::Linkage)?;
    let layouts = generate_csharp_practical_ordinary_carriers(vir)?;
    let mut a = assemble(vir, &binding, &layouts)?;
    let mut rules = vec![];
    let mut values = BTreeMap::new();
    for contract in emitted.boundaries() {
        let artifact = contract.artifact();
        // An immutable emitted boundary must belong to the reconstructed VC set.
        let doc = std::str::from_utf8(artifact.canonical_bytes())
            .map_err(|_| OrdinaryCarrierError::Linkage)?;
        if !boundary.contracts().iter().any(|c| c == doc) {
            return Err(OrdinaryCarrierError::Linkage);
        }
        let id = artifact
            .value()
            .get("contract_sha256")
            .and_then(crate::csharp_practical_source_artifacts::PracticalJsonValue::as_str)
            .ok_or(OrdinaryCarrierError::Linkage)?;
        for field in contract.input_fields() {
            let source = field.source_type_id();
            let semantic = emitted
                .closure()
                .projections()
                .get(source)
                .map(String::as_str)
                .unwrap_or(source);
            let missing = if field.required() {
                None
            } else {
                match field.missing_rule() {
                    BoundaryMissingRule::Reject => None,
                    BoundaryMissingRule::FrozenDefault(value) => Some(value.clone()),
                    BoundaryMissingRule::ExposeMissing => {
                        Some(MonomorphicValue::BoundaryPresence {
                            type_id: semantic.into(),
                            arm: BoundaryArm::Missing,
                            value: None,
                        })
                    }
                }
            };
            let null = if !field.nullable() {
                None
            } else if field.presence_binding().is_some() {
                Some(MonomorphicValue::BoundaryPresence {
                    type_id: semantic.into(),
                    arm: BoundaryArm::Null,
                    value: None,
                })
            } else {
                Some(MonomorphicValue::Option {
                    type_id: semantic.into(),
                    arm: OptionArm::None,
                    value: None,
                })
            };
            for (kind, value) in [("MissingRule", missing), ("NullRule", null)] {
                let symbol = format!("Mpk.CSharp.Boundary.{kind}.{id}.{}", field.id());
                if !boundary.definition_names().contains(&symbol) {
                    return Err(OrdinaryCarrierError::Linkage);
                }
                let value = value.map(|value| {
                    let name = format!(
                        "{PREFIX}.BoundaryRuleValue.H{:x}",
                        Sha256::digest(serde_json::to_vec(&value).expect("frozen boundary value"))
                    );
                    values.entry(name.clone()).or_insert(value);
                    name
                });
                rules.push(Rule {
                    symbol,
                    source: source.into(),
                    semantic: semantic.into(),
                    value,
                });
            }
        }
    }
    let definitions;
    (a.r.b, definitions) =
        super::super::super::literals::emit_named_values(vir, &layouts, a.r.b, values)?;
    for rule in rules {
        let source =
            a.r.carriers
                .get(&rule.source)
                .ok_or(OrdinaryCarrierError::Shape)?
                .clone();
        let body = if let Some(value) = rule.value {
            let literal = definitions
                .iter()
                .find(|d| d.name == value)
                .ok_or(OrdinaryCarrierError::Linkage)?;
            if literal.value.type_id() != rule.semantic {
                return Err(OrdinaryCarrierError::Shape);
            }
            let mut actual = a.r.b.var(0)?;
            if rule.source != rule.semantic {
                let projection = a
                    .projections
                    .iter()
                    .find(|p| {
                        p.source_carrier.type_id == rule.source
                            && p.semantic_carrier.type_id == rule.semantic
                    })
                    .ok_or(OrdinaryCarrierError::Linkage)?;
                actual = call(&mut a.r.b, &projection.project_definition, vec![actual])?;
            }
            let expected = a.r.b.constant(&value)?;
            // Proof-level typed agreement is reflexive on NaNs and observes all
            // active storage; it is not the non-reflexive C# `.equal` operation.
            let equal = a.r.source_observation(&rule.semantic)?.equal;
            call(&mut a.r.b, &equal, vec![actual, expected])?
        } else {
            bit(&mut a.r.b, false)?
        };
        let definition = name(&rule.symbol);
        define(&mut a.r.b, &definition, &[source.depth], 0, body)?;
        insert(
            &mut a.predicates,
            rule.symbol,
            definition,
            vec![rule.source],
        )?;
    }
    let unresolved_boundary_vc_symbols = boundary
        .definition_names()
        .iter()
        .filter(|s| !a.predicates.contains_key(*s))
        .cloned()
        .collect();
    let static_transformers = a.r.b.static_transformers;
    let certificate = a.r.b.finish()?;
    let p = OrdinaryBoundaryRuleProgram {
        schema: "mpk.csharp.ordinary_boundary_rules.v1".into(),
        source_ir_sha256: vir.hash().into(),
        foundation_sha256: vir.construction_context().0.content_sha256().into(),
        boundary_vc_sha256: boundary.hash(),
        binding_vc_sha256: binding.hash(),
        projections: a.projections,
        predicates: a.predicates.into_values().collect(),
        values: definitions,
        unresolved_boundary_vc_symbols,
        static_transformers,
        certificate_sha256: mpk_cert::hash_hex(&mpk_cert::certificate_hash(&certificate)),
        certificate,
    };
    if p.canonical_bytes().len() > 16 * 1024 * 1024 {
        return Err(OrdinaryCarrierError::Limit);
    }
    Ok(p)
}

pub fn import_csharp_practical_ordinary_boundary_rules(
    input: &[u8],
    certificate: &[u8],
    emitted: &EmittedDataPhase,
) -> R<OrdinaryBoundaryRuleProgram> {
    if input.len() > 16 * 1024 * 1024 || certificate.len() > 16 * 1024 * 1024 {
        return Err(OrdinaryCarrierError::Limit);
    }
    let p = generate_csharp_practical_ordinary_boundary_rules(emitted)?;
    if input != p.canonical_bytes() || certificate != p.certificate_bytes() {
        return Err(OrdinaryCarrierError::Linkage);
    }
    Ok(p)
}
