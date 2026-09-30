//! Original W06 operation sequents: guarded normal equality and every outcome.
//! Proof candidates preserve input domains and ordered first-failure selection.
use super::super::super::super::ownership_proofs::{equality, logic, publish_theorem};
use super::*;
use sha2::{Digest, Sha256};

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct OrdinaryConcreteOperationAgreement {
    pub outcome: String,
    pub failure_label: Option<String>,
    pub actual_definition: String,
    pub concrete_definition: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct OrdinaryConcreteOperationProof {
    pub sequent: BindingSequent,
    /// Every ordered selected failure, success flag and complete normal value.
    pub all_outcomes: Vec<OrdinaryConcreteOperationAgreement>,
    pub proposition_definition: String,
    pub theorem: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct OrdinaryConcreteOperationProofProgram {
    schema: String,
    source_ir_sha256: String,
    foundation_sha256: String,
    binding_vc_sha256: String,
    original_program_sha256: String,
    original_certificate_sha256: String,
    proofs: Vec<OrdinaryConcreteOperationProof>,
    proof_check_pending: bool,
    application_scope_pending: bool,
    pending_operations: Vec<OrdinaryConcreteOperationPending>,
    pending_condition_ids: Vec<String>,
    /// Complete application assembly must still check every original proof ID.
    pending_proof_ids: Vec<String>,
    certificate_sha256: String,
    #[serde(skip)]
    certificate: Vec<u8>,
}

impl OrdinaryConcreteOperationProofProgram {
    pub fn proofs(&self) -> &[OrdinaryConcreteOperationProof] {
        &self.proofs
    }
    pub fn pending_operations(&self) -> &[OrdinaryConcreteOperationPending] {
        &self.pending_operations
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
        serde_json::to_vec(self).expect("original concrete operation proof candidates")
    }
}

fn application(t: &ContractTerm) -> R<(&str, Vec<&ContractTerm>)> {
    let mut function = t;
    let mut arguments = vec![];
    while let ContractTerm::App {
        function: f,
        argument,
        ..
    } = function
    {
        arguments.push(argument.as_ref());
        function = f;
    }
    let ContractTerm::Const { name, .. } = function else {
        return Err(OrdinaryCarrierError::Linkage);
    };
    arguments.reverse();
    Ok((name, arguments))
}

fn term(
    b: &mut Builder,
    t: &ContractTerm,
    subjects: &[TypedValueRef],
    symbols: &BTreeMap<String, String>,
    offset: u32,
) -> R<u32> {
    match t {
        ContractTerm::Var { index, type_id }
            if subjects.get(*index).is_some_and(|s| s.type_id == *type_id) =>
        {
            b.var((subjects.len() - 1 - *index) as u32 + offset)
        }
        ContractTerm::Const { name, .. } => {
            b.constant(symbols.get(name).ok_or(OrdinaryCarrierError::Linkage)?)
        }
        ContractTerm::App {
            function, argument, ..
        } => {
            let f = term(b, function, subjects, symbols, offset)?;
            let a = term(b, argument, subjects, symbols, offset)?;
            b.app(f, vec![a])
        }
        _ => Err(OrdinaryCarrierError::Linkage),
    }
}

fn truth(b: &mut Builder, value: u32) -> R<u32> {
    let boolean = b.boolean;
    let yes = bit(b, true)?;
    call(b, "Std.Eq", vec![boolean, value, yes])
}

fn reflexive(b: &mut Builder, carrier: u32, left: u32, right: u32) -> R<(u32, u32)> {
    let proposition = call(b, "Std.Eq", vec![carrier, left, right])?;
    // The right operand is retained in the type. Both unchanged kernels must
    // actually check conversion; generation and reimport are only linkage.
    let proof = call(b, "Std.Eq.refl", vec![carrier, left])?;
    Ok((proposition, proof))
}

fn conjoin(b: &mut Builder, statements: &[(u32, u32)]) -> R<(u32, u32)> {
    let (&last, rest) = statements
        .split_last()
        .ok_or(OrdinaryCarrierError::Linkage)?;
    let (mut ty, mut proof) = last;
    for &(left, left_proof) in rest.iter().rev() {
        proof = call(b, "Std.Logic.And.intro", vec![left, ty, left_proof, proof])?;
        ty = call(b, "Std.Logic.And", vec![left, ty])?;
    }
    Ok((ty, proof))
}

struct Operands<'a> {
    guard: &'a ContractTerm,
    left: &'a ContractTerm,
    right: &'a ContractTerm,
}

fn operands<'a>(
    sequent: &'a BindingSequent,
    d: &OrdinaryConcreteOperationDefinition,
) -> R<Operands<'a>> {
    let [normal, all_outcomes] = sequent.goals.as_slice() else {
        return Err(OrdinaryCarrierError::Linkage);
    };
    let (name, outer) = application(normal)?;
    let [negative, agreement] = outer.as_slice() else {
        return Err(OrdinaryCarrierError::Linkage);
    };
    let (negative_name, negative_args) = application(negative)?;
    let [guard] = negative_args.as_slice() else {
        return Err(OrdinaryCarrierError::Linkage);
    };
    let (equal_name, equal_args) = application(agreement)?;
    let [left, right] = equal_args.as_slice() else {
        return Err(OrdinaryCarrierError::Linkage);
    };
    let (outcomes_name, outcomes_args) = application(all_outcomes)?;
    if name != "Mpk.CSharp.Bool.Or"
        || negative_name != "Mpk.CSharp.Bool.Not"
        || equal_name != format!("Mpk.CSharp.Binding.Equal.{}", d.component.result_type_id)
        || left.type_id() != d.component.result_type_id
        || right.type_id() != d.component.result_type_id
        || normal.type_id() != BOOL_TYPE_ID
        || guard.type_id() != BOOL_TYPE_ID
        || all_outcomes.type_id() != BOOL_TYPE_ID
        || outcomes_name != d.all_outcomes_symbol
        || outcomes_args.len() != sequent.subjects.len()
        || !outcomes_args.iter().enumerate().all(|(i, t)| {
            matches!(t, ContractTerm::Var {index,type_id} if *index==i && *type_id==sequent.subjects[i].type_id)
        })
    {
        return Err(OrdinaryCarrierError::Linkage);
    }
    Ok(Operands { guard, left, right })
}

fn normal_equality(
    b: &mut Builder,
    operands: &Operands<'_>,
    sequent: &BindingSequent,
    symbols: &BTreeMap<String, String>,
    carrier: u32,
    offset: u32,
) -> R<(u32, u32)> {
    let left = term(b, operands.left, &sequent.subjects, symbols, offset)?;
    let right = term(b, operands.right, &sequent.subjects, symbols, offset)?;
    reflexive(b, carrier, left, right)
}

fn named(b: &mut Builder, name: &str, count: usize, offset: u32) -> R<u32> {
    let arguments = (0..count)
        .map(|i| b.var((count - 1 - i) as u32 + offset))
        .collect::<R<Vec<_>>>()?;
    call(b, name, arguments)
}

fn emit(
    program: &OrdinaryConcreteOperationProgram,
    carriers: &BTreeMap<String, u32>,
) -> R<OrdinaryConcreteOperationProofProgram> {
    if program.conditions.len() != program.definitions.len() {
        return Err(OrdinaryCarrierError::Linkage);
    }
    let mut b = Builder::resume(program.certificate_bytes())?;
    if !program.conditions.is_empty() {
        equality(&mut b)?;
        logic(&mut b)?;
    }
    let mut symbols = program
        .public_domains
        .iter()
        .map(|d| (d.symbol.clone(), d.valid_definition.clone()))
        .collect::<BTreeMap<_, _>>();
    super::super::conditions::boolean_symbols(&mut symbols);
    for d in &program.definitions {
        for (symbol, definition) in [
            (&d.component.operation_id, &d.normal_definition),
            (&d.concrete_symbol, &d.concrete_definition),
            (&d.all_outcomes_symbol, &d.all_outcomes_definition),
        ] {
            if symbols.insert(symbol.clone(), definition.clone()).is_some() {
                return Err(OrdinaryCarrierError::Linkage);
            }
        }
        for f in &d.failures {
            if symbols
                .insert(f.check_symbol.clone(), f.failure_definition.clone())
                .is_some()
            {
                return Err(OrdinaryCarrierError::Linkage);
            }
        }
    }
    let original_program_sha256 = format!("{:x}", Sha256::digest(program.canonical_bytes()));
    let mut seen = BTreeSet::new();
    let mut proofs = vec![];
    for condition in &program.conditions {
        let sequent = &condition.sequent;
        let d = program
            .definitions
            .iter()
            .find(|d| d.component.operation_id == sequent.owner_id)
            .ok_or(OrdinaryCarrierError::Linkage)?;
        if sequent.kind != "concrete_definition_equivalence"
            || sequent
                .subjects
                .iter()
                .map(|s| &s.type_id)
                .ne(&d.component.argument_type_ids)
            || !seen.insert(sequent.owner_id.clone())
        {
            return Err(OrdinaryCarrierError::Linkage);
        }
        let operands = operands(sequent, d)?;
        let mut value_binders = vec![];
        for subject in &sequent.subjects {
            value_binders.push(
                b.cube(
                    *carriers
                        .get(&subject.type_id)
                        .ok_or(OrdinaryCarrierError::Linkage)?,
                )?,
            );
        }
        let carrier = b.cube(
            *carriers
                .get(&d.component.result_type_id)
                .ok_or(OrdinaryCarrierError::Linkage)?,
        )?;
        let mut premises = vec![];
        for (i, assumption) in sequent.assumptions.iter().enumerate() {
            if assumption.type_id() != BOOL_TYPE_ID {
                return Err(OrdinaryCarrierError::Linkage);
            }
            let value = term(&mut b, assumption, &sequent.subjects, &symbols, i as u32)?;
            premises.push(truth(&mut b, value)?);
        }
        let offset = premises.len() as u32;
        let guard = term(&mut b, operands.guard, &sequent.subjects, &symbols, offset)?;
        let guard = truth(&mut b, guard)?;
        let (normal_type, normal_proof) =
            normal_equality(&mut b, &operands, sequent, &symbols, carrier, offset + 1)?;
        let guarded_normal = (b.pi(guard, normal_type)?, b.lam(guard, normal_proof)?);
        let mut statements = vec![];
        let mut all_outcomes = vec![];
        for f in &d.failures {
            let actual = named(
                &mut b,
                &f.first_failure_definition,
                sequent.subjects.len(),
                offset,
            )?;
            let concrete = named(
                &mut b,
                &f.concrete_first_failure_definition,
                sequent.subjects.len(),
                offset,
            )?;
            let boolean = b.boolean;
            statements.push(reflexive(&mut b, boolean, actual, concrete)?);
            all_outcomes.push(OrdinaryConcreteOperationAgreement {
                outcome: "first_failure".into(),
                failure_label: Some(f.label.clone()),
                actual_definition: f.first_failure_definition.clone(),
                concrete_definition: f.concrete_first_failure_definition.clone(),
            });
        }
        let actual = named(
            &mut b,
            &d.success_definition,
            sequent.subjects.len(),
            offset,
        )?;
        let concrete = named(
            &mut b,
            &d.concrete_success_definition,
            sequent.subjects.len(),
            offset,
        )?;
        let boolean = b.boolean;
        statements.push(reflexive(&mut b, boolean, actual, concrete)?);
        all_outcomes.push(OrdinaryConcreteOperationAgreement {
            outcome: "success".into(),
            failure_label: None,
            actual_definition: d.success_definition.clone(),
            concrete_definition: d.concrete_success_definition.clone(),
        });
        let success = truth(&mut b, actual)?;
        statements.push((b.pi(success, normal_type)?, b.lam(success, normal_proof)?));
        all_outcomes.push(OrdinaryConcreteOperationAgreement {
            outcome: "normal".into(),
            failure_label: None,
            actual_definition: d.normal_definition.clone(),
            concrete_definition: d.concrete_definition.clone(),
        });
        let all = conjoin(&mut b, &statements)?;
        let (mut ty, mut proof) = conjoin(&mut b, &[guarded_normal, all])?;
        for &premise in premises.iter().rev() {
            ty = b.pi(premise, ty)?;
            proof = b.lam(premise, proof)?;
        }
        for &binder in value_binders.iter().rev() {
            ty = b.pi(binder, ty)?;
            proof = b.lam(binder, proof)?;
        }
        let hash = format!(
            "{:x}",
            Sha256::digest(
                serde_json::to_vec(&(&original_program_sha256, sequent))
                    .map_err(|_| OrdinaryCarrierError::Linkage)?
            )
        );
        let proposition_definition = format!("{PREFIX}.ConcreteOperationProof.Type.H{hash}");
        b.define(&proposition_definition, b.sort, ty)?;
        // Check the same complete proposition through its explicit Pi spine.
        // This lets the unchanged checker reuse its local-context checks instead
        // of inferring a nested lambda against a named type constant. The public
        // theorem still has the exact original named proposition as its type.
        let checked_body = format!("{PREFIX}.ConcreteOperationProof.Body.H{hash}");
        publish_theorem(&mut b, &checked_body, ty, proof)?;
        let proof = b.constant(&checked_body)?;
        let theorem = format!("{PREFIX}.ConcreteOperationProof.Theorem.H{hash}");
        let ty = b.constant(&proposition_definition)?;
        publish_theorem(&mut b, &theorem, ty, proof)?;
        proofs.push(OrdinaryConcreteOperationProof {
            sequent: sequent.clone(),
            all_outcomes,
            proposition_definition,
            theorem,
        });
    }
    let certificate = b.finish()?;
    let result = OrdinaryConcreteOperationProofProgram {
        schema: "mpk.csharp.ordinary_concrete_operation_proofs.v1".into(),
        source_ir_sha256: program.source_ir_sha256.clone(),
        foundation_sha256: program.foundation_sha256.clone(),
        binding_vc_sha256: program.binding_vc_sha256.clone(),
        original_program_sha256,
        original_certificate_sha256: program.certificate_sha256.clone(),
        proofs,
        proof_check_pending: true,
        application_scope_pending: true,
        pending_operations: program.pending_operations.clone(),
        pending_condition_ids: program.pending_condition_ids.clone(),
        pending_proof_ids: program.pending_proof_ids.clone(),
        certificate_sha256: mpk_cert::hash_hex(&mpk_cert::certificate_hash(&certificate)),
        certificate,
    };
    if result.canonical_bytes().len() > 16 * 1024 * 1024 {
        return Err(OrdinaryCarrierError::Limit);
    }
    Ok(result)
}

pub fn generate_csharp_practical_ordinary_concrete_operation_proofs(
    vir: &ValidatedPracticalVir,
) -> R<OrdinaryConcreteOperationProofProgram> {
    let original = generate_csharp_practical_ordinary_concrete_operations(vir)?;
    let layouts = generate_csharp_practical_ordinary_carriers(vir)?;
    emit(
        &original,
        &layouts
            .carriers()
            .iter()
            .map(|c| (c.type_id.clone(), c.depth))
            .collect(),
    )
}

pub fn import_csharp_practical_ordinary_concrete_operation_proofs(
    input: &[u8],
    certificate: &[u8],
    vir: &ValidatedPracticalVir,
) -> R<OrdinaryConcreteOperationProofProgram> {
    if input.len() > 16 * 1024 * 1024 || certificate.len() > 16 * 1024 * 1024 {
        return Err(OrdinaryCarrierError::Limit);
    }
    let expected = generate_csharp_practical_ordinary_concrete_operation_proofs(vir)?;
    if input != expected.canonical_bytes() || certificate != expected.certificate_bytes() {
        return Err(OrdinaryCarrierError::Linkage);
    }
    Ok(expected)
}
