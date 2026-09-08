//! T06-W02 construction sequents. These are verification conditions, never
//! proofs or public values. The ordinary assembler must discharge every goal
//! before using its normal-edge publication postcondition. Structural evidence
//! comes exclusively from the independently validated VIR/source protocol.
use super::*;
use crate::csharp_practical_source_artifacts::{self as a, PracticalJsonValue as J};
use crate::csharp_practical_vir_validation::{PracticalVirBlock, ValidatedPracticalVir};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ConstructionVcError {
    Contract,
    Limit,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ConstructionMember {
    pub id: String,
    pub type_id: String,
    pub storage: String,
    pub required: bool,
    pub clr_default: Value,
}
/// Equation for PublicDomain(type, value): structural membership (including
/// recursive member/container domains) AND all declared public clauses. Enum
/// membership is exactly enum_values, never arbitrary underlying integers.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ConstructionTypeInvariant {
    pub type_id: String,
    pub source_sha256: String,
    pub members: Vec<ConstructionMember>,
    pub enum_values: Option<Vec<String>>,
    pub enum_underlying: Option<String>,
    /// Ordinary equation body under the single public receiver binder.
    pub public_body: ContractTerm,
    pub structural_default: bool,
    pub recursive_default: Value,
    pub contract_sha256: Option<String>,
    pub construction_clause: Option<VerifiedContractExpression>,
    pub public_clauses: Vec<VerifiedContractExpression>,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ConstructionSubject {
    pub value_id: String,
    /// Actual SSA carrier type, also the type of the predicate's free binder.
    pub type_id: String,
    pub source_type_id: Option<String>,
    /// public_value or private_members. Private members are
    /// a logical observation of slots, not a cast/publication of the carrier.
    pub view: String,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ConstructionPredicate {
    pub subject: ConstructionSubject,
    pub attachment_sha256: Option<String>,
    /// One subject binder (index zero). Bind to subject at the sequent point.
    pub term: ContractTerm,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ConstructionEvidence {
    pub kind: String,
    pub node_ids: Vec<String>,
    pub value_ids: Vec<String>,
    pub member_ids: Vec<String>,
    pub definitely_assigned: u32,
    pub possibly_assigned: u32,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ConstructionSequent {
    pub id: String,
    pub function_id: String,
    pub node_id: String,
    /// entry, before_operation, normal_edge, return, or exceptional_exit.
    pub point: String,
    pub kind: String,
    /// Assumptions are permitted only at function entry. Call preconditions and
    /// all new/public values are goals, including invariant preservation.
    pub assumptions: Vec<ConstructionPredicate>,
    pub goals: Vec<ConstructionPredicate>,
    pub structural_evidence: Vec<ConstructionEvidence>,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ConstructionVcProgram {
    types: Vec<ConstructionTypeInvariant>,
    sequents: Vec<ConstructionSequent>,
    /// Binds the shared structural/container domain equations and closed types.
    closed_roots_sha256: String,
    source_ir_sha256: String,
    definition_names: Vec<String>,
    #[serde(skip)]
    node_count: usize,
}
impl ConstructionVcProgram {
    fn empty() -> Self {
        Self {
            types: vec![],
            sequents: vec![],
            closed_roots_sha256: String::new(),
            source_ir_sha256: String::new(),
            definition_names: vec![],
            node_count: 0,
        }
    }
    pub fn definition_names(&self) -> &[String] {
        &self.definition_names
    }
    pub fn types(&self) -> &[ConstructionTypeInvariant] {
        &self.types
    }
    pub fn sequents(&self) -> &[ConstructionSequent] {
        &self.sequents
    }
    pub fn canonical_bytes(&self) -> Vec<u8> {
        serde_json::to_vec(self).expect("typed construction VCs")
    }
    pub fn hash(&self) -> String {
        digest(&self.canonical_bytes())
    }
    pub(crate) fn nodes(&self) -> usize {
        self.node_count
    }
    pub(crate) fn binder_depth(&self) -> usize {
        self.types
            .iter()
            .flat_map(|t| t.construction_clause.iter().chain(&t.public_clauses))
            .map(VerifiedContractExpression::binder_depth)
            .max()
            .unwrap_or(0)
            .max(usize::from(!self.sequents.is_empty()))
    }
}
fn digest(bytes: &[u8]) -> String {
    crate::hash::hash_domain_separated_raw(HashDomain::new("MPK-CSHARP-CONSTRUCTION-VC-1.0"), bytes)
        .expect("typed VC hash")
        .to_hex()
}
fn fail() -> ConstructionVcError {
    ConstructionVcError::Contract
}
fn str_field<'a>(v: &'a J, key: &str) -> Result<&'a str, ConstructionVcError> {
    v.get(key).and_then(J::as_str).ok_or_else(fail)
}
fn clause(
    vir: &ValidatedPracticalVir,
    owner: &str,
    type_id: &str,
    v: &J,
) -> Result<VerifiedContractExpression, ConstructionVcError> {
    let bytes = a::canonical_practical_json_bytes(v).map_err(|_| fail())?;
    let e = vir
        .contract_expressions()
        .iter()
        .find(|e| e.owner() == owner && e.expression().as_bytes() == bytes)
        .ok_or_else(fail)?;
    if e.subjects() != [("current:this".into(), type_id.into())]
        || e.term().type_id() != "mpk.csharp.value.bool.v1"
    {
        return Err(fail());
    }
    Ok(e.clone())
}
fn subject(value: &TypedValueRef) -> ConstructionSubject {
    ConstructionSubject {
        value_id: value.id.clone(),
        type_id: value.type_id.clone(),
        source_type_id: None,
        view: "public_value".into(),
    }
}
const BOOL: &str = "mpk.csharp.value.bool.v1";
fn boolean(v: bool) -> ContractTerm {
    ContractTerm::Const {
        name: format!("Mpk.CSharp.Bool.{v}"),
        type_id: BOOL.into(),
    }
}
fn apply(name: String, args: Vec<ContractTerm>, result: &str) -> ContractTerm {
    let ty = args.iter().rev().fold(result.to_owned(), |ty, a| {
        format!("({}->{ty})", a.type_id())
    });
    let mut t = ContractTerm::Const { name, type_id: ty };
    for (i, arg) in args.iter().enumerate() {
        let ty = args[i + 1..].iter().rev().fold(result.to_owned(), |ty, a| {
            format!("({}->{ty})", a.type_id())
        });
        t = ContractTerm::App {
            function: Box::new(t),
            argument: Box::new(arg.clone()),
            type_id: ty,
        };
    }
    t
}
fn combine(name: &str, terms: Vec<ContractTerm>, identity: bool) -> ContractTerm {
    // Balanced ordinary conjunction/disjunction avoids growing term depth
    // linearly with the frozen member/enum limits.
    match terms.len() {
        0 => boolean(identity),
        1 => terms.into_iter().next().unwrap(),
        n => {
            let left = combine(name, terms[..n / 2].to_vec(), identity);
            let right = combine(name, terms[n / 2..].to_vec(), identity);
            apply(name.into(), vec![left, right], BOOL)
        }
    }
}
fn domain_body(t: &ConstructionTypeInvariant) -> ContractTerm {
    let receiver = ContractTerm::Var {
        index: 0,
        type_id: t.type_id.clone(),
    };
    let mut terms = if let Some(arms) = &t.enum_values {
        vec![combine(
            "Mpk.CSharp.Bool.Or",
            arms.iter()
                .map(|arm| {
                    apply(
                        format!("Mpk.CSharp.EnumCarrierEquals.{}.{arm}", t.type_id),
                        vec![receiver.clone()],
                        BOOL,
                    )
                })
                .collect(),
            false,
        )]
    } else {
        let mut terms = vec![apply(
            format!("Mpk.CSharp.SourceShape.{}", t.type_id),
            vec![receiver.clone()],
            BOOL,
        )];
        for member in &t.members {
            let field = apply(
                format!("field.read.{}", member.id),
                vec![receiver.clone()],
                &member.type_id,
            );
            terms.push(apply(
                format!("Mpk.CSharp.PublicDomain.{}", member.type_id),
                vec![field],
                BOOL,
            ));
        }
        terms
    };
    terms.extend(t.public_clauses.iter().map(|e| e.term().clone()));
    combine("Mpk.CSharp.Bool.And", terms, true)
}
fn construction_reads(
    term: &ContractTerm,
    definitions: &[ContractDefinition],
    members: &[ConstructionMember],
) -> BTreeSet<String> {
    fn visit(
        t: &ContractTerm,
        depth: usize,
        definitions: &[ContractDefinition],
        members: &[ConstructionMember],
        reads: &mut BTreeSet<String>,
    ) {
        match t {
            ContractTerm::Var { index, .. } if *index == depth => {
                reads.extend(members.iter().map(|m| m.id.clone()))
            }
            ContractTerm::App {
                function, argument, ..
            } => {
                if let (ContractTerm::Const { name, .. }, ContractTerm::Var { index, .. }) =
                    (function.as_ref(), argument.as_ref())
                {
                    if *index == depth {
                        if let Some(d) = definitions
                            .iter()
                            .find(|d| d.name == *name && d.tag == "field")
                        {
                            let fields: Value =
                                serde_json::from_str(&d.parameters).expect("verified field recipe");
                            if let Some(member) = fields["member_id"]
                                .as_str()
                                .filter(|id| members.iter().any(|m| m.id == *id))
                            {
                                reads.insert(member.into());
                                return;
                            }
                        }
                    }
                }
                visit(function, depth, definitions, members, reads);
                visit(argument, depth, definitions, members, reads);
            }
            ContractTerm::Lam { body, .. } => visit(body, depth + 1, definitions, members, reads),
            ContractTerm::Let { value, body, .. } => {
                visit(value, depth, definitions, members, reads);
                visit(body, depth + 1, definitions, members, reads);
            }
            _ => {}
        }
    }
    let mut reads = BTreeSet::new();
    visit(term, 0, definitions, members, &mut reads);
    reads
}
/// Rebind the sole source receiver to its actual private slot carrier. Direct
/// constructor field observations become typed slot reads. A whole receiver
/// observation requires all slots (construction_reads); finalization may apply
/// only the validated non-required CLR defaults. Neither recipe publishes SSA.
fn private_term(
    term: &ContractTerm,
    definitions: &[ContractDefinition],
    subject: &ConstructionSubject,
    construction: bool,
    depth: usize,
) -> ContractTerm {
    let Some(owner) = &subject.source_type_id else {
        return term.clone();
    };
    let variable = || ContractTerm::Var {
        index: depth,
        type_id: subject.type_id.clone(),
    };
    match term {
        ContractTerm::Var { index, .. } if *index == depth => apply(
            format!(
                "Mpk.CSharp.{}Snapshot.{owner}",
                if construction {
                    "Assigned"
                } else {
                    "Finalized"
                }
            ),
            vec![variable()],
            owner,
        ),
        ContractTerm::App {
            function,
            argument,
            type_id,
        } => {
            if construction {
                if let (ContractTerm::Const { name, .. }, ContractTerm::Var { index, .. }) =
                    (function.as_ref(), argument.as_ref())
                {
                    if *index == depth {
                        if let Some(d) = definitions
                            .iter()
                            .find(|d| d.name == *name && d.tag == "field")
                        {
                            let fields: Value =
                                serde_json::from_str(&d.parameters).expect("verified field recipe");
                            if let Some(member) = fields["member_id"].as_str() {
                                return apply(
                                    format!("Mpk.CSharp.SlotRead.{member}"),
                                    vec![variable()],
                                    type_id,
                                );
                            }
                        }
                    }
                }
            }
            ContractTerm::App {
                function: Box::new(private_term(
                    function,
                    definitions,
                    subject,
                    construction,
                    depth,
                )),
                argument: Box::new(private_term(
                    argument,
                    definitions,
                    subject,
                    construction,
                    depth,
                )),
                type_id: type_id.clone(),
            }
        }
        ContractTerm::Lam {
            parameter_type,
            body,
            type_id,
        } => ContractTerm::Lam {
            parameter_type: parameter_type.clone(),
            body: Box::new(private_term(
                body,
                definitions,
                subject,
                construction,
                depth + 1,
            )),
            type_id: type_id.clone(),
        },
        ContractTerm::Let {
            value,
            body,
            type_id,
        } => ContractTerm::Let {
            value: Box::new(private_term(
                value,
                definitions,
                subject,
                construction,
                depth,
            )),
            body: Box::new(private_term(
                body,
                definitions,
                subject,
                construction,
                depth + 1,
            )),
            type_id: type_id.clone(),
        },
        _ => term.clone(),
    }
}
fn predicates(
    types: &[ConstructionTypeInvariant],
    value: ConstructionSubject,
    construction: bool,
) -> Vec<ConstructionPredicate> {
    let owner = value.source_type_id.as_deref().unwrap_or(&value.type_id);
    let mut result = vec![];
    if !construction {
        let domain = apply(
            format!("Mpk.CSharp.PublicDomain.{owner}"),
            vec![ContractTerm::Var {
                index: 0,
                type_id: owner.into(),
            }],
            BOOL,
        );
        result.push(ConstructionPredicate {
            subject: value.clone(),
            attachment_sha256: None,
            term: private_term(&domain, &[], &value, false, 0),
        });
    }
    if let Some(t) = types.iter().find(|t| t.type_id == owner) {
        let clauses = if construction {
            t.construction_clause.iter().collect::<Vec<_>>()
        } else {
            t.public_clauses.iter().collect()
        };
        for e in clauses {
            if construction {
                for member in construction_reads(e.term(), e.definitions(), &t.members) {
                    result.push(ConstructionPredicate {
                        subject: value.clone(),
                        attachment_sha256: None,
                        term: apply(
                            format!("Mpk.CSharp.SlotAssigned.{member}"),
                            vec![ContractTerm::Var {
                                index: 0,
                                type_id: value.type_id.clone(),
                            }],
                            BOOL,
                        ),
                    });
                }
            }
            result.push(ConstructionPredicate {
                subject: value.clone(),
                attachment_sha256: Some(e.attachment_sha256().into()),
                term: private_term(e.term(), e.definitions(), &value, construction, 0),
            });
        }
    }
    result
}

fn value_in(
    block: &PracticalVirBlock,
    id: &str,
    f: &crate::csharp_practical_vir_validation::PracticalVirFunction,
) -> Option<TypedValueRef> {
    f.parameter_values
        .iter()
        .chain(f.blocks.iter().flat_map(|b| {
            b.literal_values
                .iter()
                .map(|v| &v.result)
                .chain(b.phi_values.iter().map(|v| &v.value))
                .chain(b.invocation.iter().map(|v| &v.result))
        }))
        .find(|v| v.id == id)
        .cloned()
        .or_else(|| {
            block
                .handler_exception_value
                .as_ref()
                .filter(|v| v.id == id)
                .cloned()
        })
}
fn evidence(
    kind: &str,
    nodes: Vec<String>,
    members: Vec<String>,
    must: u32,
    may: u32,
) -> ConstructionEvidence {
    ConstructionEvidence {
        kind: kind.into(),
        node_ids: nodes,
        value_ids: vec![],
        member_ids: members,
        definitely_assigned: must,
        possibly_assigned: may,
    }
}
impl ConstructionVcProgram {
    #[allow(clippy::too_many_arguments)]
    fn add(
        &mut self,
        function: &str,
        node: &str,
        point: &str,
        kind: &str,
        assumptions: Vec<ConstructionPredicate>,
        goals: Vec<ConstructionPredicate>,
        structural_evidence: Vec<ConstructionEvidence>,
    ) -> Result<(), ConstructionVcError> {
        if assumptions.is_empty() && goals.is_empty() && structural_evidence.is_empty() {
            return Ok(());
        }
        let mut row = ConstructionSequent {
            id: String::new(),
            function_id: function.into(),
            node_id: node.into(),
            point: point.into(),
            kind: kind.into(),
            assumptions,
            goals,
            structural_evidence,
        };
        row.id = format!(
            "construction_vc:{}",
            digest(&serde_json::to_vec(&row).map_err(|_| fail())?)
        );
        self.node_count += row
            .assumptions
            .iter()
            .chain(&row.goals)
            .map(|p| p.term.nodes())
            .sum::<usize>();
        self.sequents.push(row);
        if self.sequents.len()
            > crate::csharp_practical_vc_model::GENERATED_DECLARATIONS_MAX as usize
            || self.nodes() > crate::csharp_practical_vc_model::ORDINARY_TERM_NODES_MAX as usize
        {
            return Err(ConstructionVcError::Limit);
        }
        Ok(())
    }
}
/// Reconstructed only from validated original source and VIR. No caller-supplied
/// `discharged` metadata can suppress a goal. No separate CFG analysis is used.
pub(crate) fn generate_construction_vcs(
    vir: &ValidatedPracticalVir,
) -> Result<ConstructionVcProgram, ConstructionVcError> {
    let (b, r, source) = vir.construction_context();
    let Some(source) = source else {
        return Ok(ConstructionVcProgram::empty());
    };
    let mut program = ConstructionVcProgram {
        closed_roots_sha256: digest(r.canonical_json()),
        source_ir_sha256: vir.hash().into(),
        ..ConstructionVcProgram::empty()
    };
    for t in r.source_types.values() {
        let mut definition = ConstructionTypeInvariant {
            type_id: t.id.clone(),
            source_sha256: t.source_sha256.clone(),
            members: t
                .members
                .iter()
                .map(|m| {
                    Ok(ConstructionMember {
                        id: m.id.clone(),
                        type_id: closed_type_id(b, &m.ty).map_err(|_| fail())?,
                        storage: m.storage.clone(),
                        required: m.required,
                        clr_default: t.actual_default[&m.id].clone(),
                    })
                })
                .collect::<Result<_, ConstructionVcError>>()?,
            enum_values: (t.kind == SourceKind::Enum).then(|| t.enum_values.clone()),
            enum_underlying: t.enum_underlying.clone(),
            public_body: boolean(true),
            structural_default: source.has_structural_default(&t.id),
            recursive_default: match t.kind {
                SourceKind::SealedClass => Value::Null,
                SourceKind::Enum => Value::String("0".into()),
                SourceKind::ReadonlyStruct => Value::Object(t.actual_default.clone()),
            },
            contract_sha256: None,
            construction_clause: None,
            public_clauses: vec![],
        };
        for doc in vir.data_contracts() {
            let v = a::parse_canonical_practical_json(
                a::PracticalArtifactKind::TypeContract,
                doc.as_bytes(),
            )
            .map_err(|_| fail())?;
            if str_field(&v, "schema")? != a::TYPE_CONTRACT_SCHEMA
                || str_field(&v, "source_type_id")? != t.id
            {
                continue;
            }
            let owner = str_field(&v, "contract_sha256")?;
            definition.contract_sha256 = Some(owner.into());
            if let Some(v) = v.get("construction_invariant").filter(|v| **v != J::Null) {
                definition.construction_clause = Some(clause(vir, owner, &t.id, v)?);
            }
            for v in v.get("invariants").and_then(J::as_array).ok_or_else(fail)? {
                definition
                    .public_clauses
                    .push(clause(vir, owner, &t.id, v)?);
            }
        }
        definition.public_body = domain_body(&definition);
        program.node_count += definition.public_body.nodes();
        if program.nodes() > crate::csharp_practical_vc_model::ORDINARY_TERM_NODES_MAX as usize {
            return Err(ConstructionVcError::Limit);
        }
        program.types.push(definition);
    }
    for f in vir.functions() {
        let callable = source
            .callables()
            .iter()
            .find(|c| c.id() == f.id)
            .ok_or_else(fail)?;
        let ctor = callable.identity()["kind"] == "constructor";
        let owner = callable.identity()["owner"].as_str().unwrap_or("");
        let owner_type = program.types.iter().find(|t| t.type_id == owner);
        let has_init =
            owner_type.is_some_and(|t| t.members.iter().any(|m| m.storage == "init_auto"));
        let private_owner = f
            .object_protocol
            .as_ref()
            .and_then(|p| p.constructor_owner.as_deref());
        let entry = f
            .blocks
            .iter()
            .find(|n| n.node.tag == ControlNodeTag::Entry)
            .ok_or_else(fail)?;
        let public_parameters = f
            .parameter_values
            .iter()
            .enumerate()
            .filter(|(i, _)| !(private_owner.is_some() && *i == 0))
            .map(|(_, v)| v)
            .collect::<Vec<_>>();
        program.add(
            &f.id,
            &entry.node.id,
            "entry",
            "public_operation_assumptions",
            public_parameters
                .iter()
                .flat_map(|v| predicates(&program.types, subject(v), false))
                .collect(),
            vec![],
            vec![],
        )?;
        for block in &f.blocks {
            let node = &block.node.id;
            for literal in &block.literal_values {
                // The shared literal decoder checks enum/default structure;
                // user invariants remain goals even for recursively zero values.
                program.add(
                    &f.id,
                    node,
                    "normal_edge",
                    "literal_domain",
                    vec![],
                    predicates(&program.types, subject(&literal.result), false),
                    vec![],
                )?;
            }
            if let Some(call) = &block.invocation {
                if let Ok(op) = ObjectConstructionOperation::from_id(r, &call.operation_id) {
                    match op {
                        ObjectConstructionOperation::Begin { .. } => {}
                        ObjectConstructionOperation::Read { member, .. } => program.add(
                            &f.id,
                            node,
                            "before_operation",
                            "definite_read",
                            vec![],
                            vec![],
                            vec![evidence(
                                "validated_definite_slot_read",
                                vec![node.clone()],
                                vec![member],
                                0,
                                0,
                            )],
                        )?,
                        ObjectConstructionOperation::Write { member, .. } => program.add(
                            &f.id,
                            node,
                            "before_operation",
                            "member_assignment",
                            vec![],
                            predicates(&program.types, subject(&call.operands[1]), false),
                            vec![evidence(
                                "validated_unique_member_write",
                                vec![node.clone()],
                                vec![member],
                                0,
                                0,
                            )],
                        )?,
                        ObjectConstructionOperation::Finalize { owner } => {
                            let anchor = f
                                .object_protocol
                                .as_ref()
                                .and_then(|p| {
                                    p.initializations
                                        .iter()
                                        .find(|p| p.finalize_node_id == *node)
                                })
                                .ok_or_else(fail)?;
                            let plan = callable
                                .initialization_plans()
                                .iter()
                                .find(|p| p.node_ordinal == anchor.source_node_ordinal)
                                .ok_or_else(fail)?;
                            let mut nodes = vec![
                                anchor.begin_node_id.clone(),
                                anchor.constructor_node_id.clone(),
                            ];
                            nodes.extend(anchor.assignment_node_ids.clone());
                            nodes.push(node.clone());
                            let members = program
                                .types
                                .iter()
                                .find(|t| t.type_id == owner)
                                .ok_or_else(fail)?
                                .members
                                .iter()
                                .map(|m| m.id.clone())
                                .collect();
                            program.add(
                                &f.id,
                                node,
                                "before_operation",
                                "publication",
                                vec![],
                                predicates(
                                    &program.types,
                                    ConstructionSubject {
                                        value_id: call.operands[0].id.clone(),
                                        type_id: call.operands[0].type_id.clone(),
                                        source_type_id: Some(owner),
                                        view: "private_members".into(),
                                    },
                                    false,
                                ),
                                vec![evidence(
                                    "validated_required_init_and_defaults",
                                    nodes,
                                    members,
                                    plan.definitely_assigned,
                                    plan.possibly_assigned,
                                )],
                            )?;
                        }
                    }
                } else if vir.operation_signatures().iter().any(|s| {
                    s.id == call.operation_id && s.tag == ClosedOperationTag::ConstructorExecute
                }) {
                    // Same-type delegation and external initialization carry a
                    // private receiver; it must never receive a public assumption.
                    program.add(
                        &f.id,
                        node,
                        "before_operation",
                        "constructor_arguments",
                        vec![],
                        call.operands
                            .iter()
                            .skip(1)
                            .flat_map(|v| predicates(&program.types, subject(v), false))
                            .collect(),
                        vec![],
                    )?;
                } else {
                    program.add(
                        &f.id,
                        node,
                        "before_operation",
                        "operation_arguments",
                        vec![],
                        call.operands
                            .iter()
                            .flat_map(|v| predicates(&program.types, subject(v), false))
                            .collect(),
                        vec![],
                    )?;
                    program.add(
                        &f.id,
                        node,
                        "normal_edge",
                        "operation_result",
                        vec![],
                        predicates(&program.types, subject(&call.result), false),
                        vec![],
                    )?;
                }
            }
            if block.node.tag == ControlNodeTag::Return {
                for id in &block.return_value_ids {
                    let value = value_in(block, id, f).ok_or_else(fail)?;
                    let mut value = subject(&value);
                    if let Some(owner) = private_owner {
                        value.source_type_id = Some(owner.into());
                        value.view = "private_members".into();
                    }
                    let ev = if ctor {
                        source
                            .constructor_assignment(&f.id)
                            .map(|a| {
                                evidence(
                                    "validated_constructor_assignment",
                                    vec![node.clone()],
                                    vec![],
                                    a.definitely_assigned,
                                    a.possibly_assigned,
                                )
                            })
                            .into_iter()
                            .collect()
                    } else {
                        vec![]
                    };
                    program.add(
                        &f.id,
                        node,
                        "return",
                        if ctor && has_init {
                            "construction_invariant"
                        } else {
                            "public_return"
                        },
                        vec![],
                        predicates(&program.types, value, ctor && has_init),
                        ev,
                    )?;
                }
                program.add(
                    &f.id,
                    node,
                    "return",
                    "public_operation_preservation",
                    vec![],
                    public_parameters
                        .iter()
                        .flat_map(|v| predicates(&program.types, subject(v), false))
                        .collect(),
                    vec![],
                )?;
            }
            if block.node.tag == ControlNodeTag::Exit {
                program.add(
                    &f.id,
                    node,
                    "exceptional_exit",
                    "public_operation_preservation",
                    vec![],
                    public_parameters
                        .iter()
                        .flat_map(|v| predicates(&program.types, subject(v), false))
                        .collect(),
                    vec![],
                )?;
            }
        }
        if let Some(protocol) = &f.object_protocol {
            for discard in &protocol.exceptional_discards {
                program.add(
                    &f.id,
                    &discard.exit_node_id,
                    "exceptional_exit",
                    "discard_without_publication",
                    vec![],
                    vec![],
                    vec![ConstructionEvidence {
                        value_ids: discard.origin_value_ids.clone(),
                        ..evidence(
                            "validated_discard",
                            vec![discard.exit_node_id.clone()],
                            vec![],
                            0,
                            0,
                        )
                    }],
                )?;
            }
        }
    }
    let mut names = BTreeSet::new();
    fn constants(t: &ContractTerm, names: &mut BTreeSet<String>) {
        match t {
            ContractTerm::Const { name, .. }
                if [
                    "Mpk.CSharp.PublicDomain.",
                    "Mpk.CSharp.SourceShape.",
                    "Mpk.CSharp.EnumCarrierEquals.",
                    "Mpk.CSharp.SlotAssigned.",
                    "Mpk.CSharp.SlotRead.",
                    "Mpk.CSharp.AssignedSnapshot.",
                    "Mpk.CSharp.FinalizedSnapshot.",
                    "Mpk.CSharp.Bool.",
                    "field.read.",
                ]
                .iter()
                .any(|p| name.starts_with(p)) =>
            {
                names.insert(name.clone());
            }
            ContractTerm::App {
                function, argument, ..
            } => {
                constants(function, names);
                constants(argument, names);
            }
            ContractTerm::Lam { body, .. } => constants(body, names),
            ContractTerm::Let { value, body, .. } => {
                constants(value, names);
                constants(body, names);
            }
            _ => {}
        }
    }
    for t in &program.types {
        names.insert(format!("Mpk.CSharp.PublicDomain.{}", t.type_id));
        constants(&t.public_body, &mut names);
    }
    for p in program
        .sequents
        .iter()
        .flat_map(|s| s.assumptions.iter().chain(&s.goals))
    {
        constants(&p.term, &mut names);
    }
    program.definition_names = names.into_iter().collect();
    if program.definition_names.len() + program.sequents.len()
        > crate::csharp_practical_vc_model::GENERATED_DECLARATIONS_MAX as usize
    {
        return Err(ConstructionVcError::Limit);
    }
    program.sequents.sort_by(|a, b| a.id.cmp(&b.id));
    if program.nodes() > crate::csharp_practical_vc_model::ORDINARY_TERM_NODES_MAX as usize
        || program.canonical_bytes().len() > a::PRACTICAL_ARTIFACT_TRANSPORT_BYTES_MAX
    {
        return Err(ConstructionVcError::Limit);
    }
    Ok(program)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn construction_vc_reads_distinguish_slots_whole_receiver_and_bound_locals() {
        let members = ["a", "b"].map(|id| ConstructionMember {
            id: id.into(),
            type_id: BOOL.into(),
            storage: "init_auto".into(),
            required: false,
            clr_default: json!(false),
        });
        let receiver = ContractTerm::Var {
            index: 0,
            type_id: "owner".into(),
        };
        let defs = [ContractDefinition {
            name: "read.a".into(),
            tag: "field".into(),
            parameters: r#"{"member_id":"a"}"#.into(),
            argument_types: vec!["owner".into()],
            result_type: BOOL.into(),
            ordered_checks: vec![],
        }];
        let field = apply("read.a".into(), vec![receiver.clone()], BOOL);
        let private = ConstructionSubject {
            value_id: "private".into(),
            type_id: "slot_carrier".into(),
            source_type_id: Some("owner".into()),
            view: "private_members".into(),
        };
        let slot = ContractTerm::Var {
            index: 0,
            type_id: "slot_carrier".into(),
        };
        assert_eq!(
            private_term(&field, &defs, &private, true, 0),
            apply("Mpk.CSharp.SlotRead.a".into(), vec![slot.clone()], BOOL)
        );
        assert_eq!(
            private_term(&field, &defs, &private, false, 0),
            apply(
                "read.a".into(),
                vec![apply(
                    "Mpk.CSharp.FinalizedSnapshot.owner".into(),
                    vec![slot],
                    "owner"
                )],
                BOOL
            )
        );
        assert_eq!(
            construction_reads(&field, &defs, &members),
            BTreeSet::from(["a".into()])
        );
        assert_eq!(
            construction_reads(&receiver, &defs, &members),
            BTreeSet::from(["a".into(), "b".into()])
        );
        let local = ContractTerm::Lam {
            parameter_type: "owner".into(),
            body: Box::new(field),
            type_id: format!("(owner->{BOOL})"),
        };
        assert!(construction_reads(&local, &defs, &members).is_empty());
        let captured = ContractTerm::Lam {
            parameter_type: "owner".into(),
            body: Box::new(ContractTerm::Var {
                index: 1,
                type_id: "owner".into(),
            }),
            type_id: "(owner->owner)".into(),
        };
        assert_eq!(construction_reads(&captured, &defs, &members).len(), 2);
    }
    #[test]
    fn construction_vc_rejects_oversized_sequent_expansion() {
        let mut program = ConstructionVcProgram::empty();
        program.node_count = crate::csharp_practical_vc_model::ORDINARY_TERM_NODES_MAX as usize;
        assert!(program
            .add(
                "f",
                "n",
                "return",
                "public_return",
                vec![],
                vec![ConstructionPredicate {
                    subject: ConstructionSubject {
                        value_id: "v".into(),
                        type_id: BOOL.into(),
                        source_type_id: None,
                        view: "public_value".into()
                    },
                    attachment_sha256: None,
                    term: boolean(true)
                }],
                vec![]
            )
            .is_err());
    }
}
