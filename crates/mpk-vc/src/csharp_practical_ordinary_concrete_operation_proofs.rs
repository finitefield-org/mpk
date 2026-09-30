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

#[derive(Clone, Copy)]
enum Replacement {
    Global(u32),
    Binder,
}

// Capture-preserving transformation of the existing ordinary DAG. This changes
// no original declaration or source expression; it instantiates an Eq motive.
fn transform(
    b: &mut Builder,
    root: u32,
    amount: u32,
    replacements: &BTreeMap<u32, Replacement>,
) -> R<u32> {
    fn visit(
        b: &mut Builder,
        t: u32,
        depth: u32,
        amount: u32,
        replacements: &BTreeMap<u32, Replacement>,
        memo: &mut BTreeMap<(u32, u32), u32>,
    ) -> R<u32> {
        if let Some(&v) = memo.get(&(t, depth)) {
            return Ok(v);
        }
        let node = b.c.term_table[t as usize].clone();
        let result = match node {
            TermNode::Var(i) if i >= depth => {
                b.var(i.checked_add(amount).ok_or(OrdinaryCarrierError::Limit)?)?
            }
            TermNode::Const { global, levels } if replacements.contains_key(&global) => {
                if !levels.is_empty() {
                    return Err(OrdinaryCarrierError::Linkage);
                }
                match replacements[&global] {
                    Replacement::Global(global) => b.term(TermNode::Const { global, levels })?,
                    Replacement::Binder => b.var(depth)?,
                }
            }
            TermNode::App {
                function,
                arguments,
            } => {
                let f = visit(b, function, depth, amount, replacements, memo)?;
                let a = arguments
                    .into_iter()
                    .map(|t| visit(b, t, depth, amount, replacements, memo))
                    .collect::<R<Vec<_>>>()?;
                b.app(f, a)?
            }
            TermNode::Lam { ty, body } => {
                let ty = visit(b, ty, depth, amount, replacements, memo)?;
                let body = visit(b, body, depth + 1, amount, replacements, memo)?;
                b.lam(ty, body)?
            }
            TermNode::Pi { ty, body } => {
                let ty = visit(b, ty, depth, amount, replacements, memo)?;
                let body = visit(b, body, depth + 1, amount, replacements, memo)?;
                b.pi(ty, body)?
            }
            TermNode::Let { ty, value, body } => {
                let ty = visit(b, ty, depth, amount, replacements, memo)?;
                let value = visit(b, value, depth, amount, replacements, memo)?;
                let body = visit(b, body, depth + 1, amount, replacements, memo)?;
                b.term(TermNode::Let { ty, value, body })?
            }
            other => b.term(other)?,
        };
        memo.insert((t, depth), result);
        Ok(result)
    }
    visit(b, root, 0, amount, replacements, &mut BTreeMap::new())
}

fn global(b: &Builder, name: &str) -> R<u32> {
    b.globals
        .get(name)
        .copied()
        .ok_or(OrdinaryCarrierError::Linkage)
}

fn function_equality(
    b: &mut Builder,
    left: &str,
    right: &str,
    program_hash: &str,
) -> R<(u32, u32)> {
    let l = global(b, left)?;
    let r = global(b, right)?;
    let DeclarationKind::Def { ty, .. } = b.c.declarations[l as usize].kind else {
        return Err(OrdinaryCarrierError::Linkage);
    };
    let DeclarationKind::Def { ty: right_ty, .. } = b.c.declarations[r as usize].kind else {
        return Err(OrdinaryCarrierError::Linkage);
    };
    if ty != right_ty {
        return Err(OrdinaryCarrierError::Linkage);
    }
    let hash = format!(
        "{:x}",
        Sha256::digest(
            serde_json::to_vec(&(program_hash, left, right))
                .map_err(|_| OrdinaryCarrierError::Linkage)?
        )
    );
    let name = format!("{PREFIX}.ConcreteOperationProof.Function.H{hash}");
    if !b.globals.contains_key(&name) {
        let actual = b.constant(left)?;
        let concrete = b.constant(right)?;
        let (proposition, proof) = reflexive(b, ty, actual, concrete)?;
        publish_theorem(b, &name, proposition, proof)?;
    }
    Ok((ty, b.constant(&name)?))
}

fn rewrite_operand(
    b: &mut Builder,
    carrier: u32,
    left: u32,
    initial_right: u32,
    pairs: &[(String, String)],
    program_hash: &str,
) -> R<(u32, u32)> {
    let mut right = initial_right;
    // A matching complete operand is required before any transport. Host-side
    // replacement does not admit a changed mask or omitted failure.
    if right != left {
        return Err(OrdinaryCarrierError::Linkage);
    }
    let mut proof = call(b, "Std.Eq.refl", vec![carrier, left])?;
    for (actual, concrete) in pairs {
        let actual_global = global(b, actual)?;
        let concrete_global = global(b, concrete)?;
        let next = transform(
            b,
            right,
            0,
            &BTreeMap::from([(actual_global, Replacement::Global(concrete_global))]),
        )?;
        if next == right {
            continue;
        }
        let (function_type, equation) = function_equality(b, actual, concrete, program_hash)?;
        let shifted_left = transform(b, left, 1, &BTreeMap::new())?;
        let abstract_right = transform(
            b,
            right,
            1,
            &BTreeMap::from([(actual_global, Replacement::Binder)]),
        )?;
        let predicate = call(b, "Std.Eq", vec![carrier, shifted_left, abstract_right])?;
        let predicate = b.lam(function_type, predicate)?;
        let actual = b.constant(actual)?;
        let concrete = b.constant(concrete)?;
        proof = call(
            b,
            "Std.Eq.rewrite",
            vec![function_type, actual, concrete, predicate, equation, proof],
        )?;
        right = next;
    }
    Ok((proof, right))
}

fn mask_body(b: &mut Builder, name: &str, count: usize, offset: u32) -> R<u32> {
    let DeclarationKind::Def { mut value, .. } = b.c.declarations[global(b, name)? as usize].kind
    else {
        return Err(OrdinaryCarrierError::Linkage);
    };
    for _ in 0..count {
        let TermNode::Lam { body, .. } = b.c.term_table[value as usize] else {
            return Err(OrdinaryCarrierError::Linkage);
        };
        value = body;
    }
    transform(b, value, offset, &BTreeMap::new())
}

fn mask_equality(
    b: &mut Builder,
    actual: &str,
    concrete: &str,
    count: usize,
    offset: u32,
    d: &OrdinaryConcreteOperationDefinition,
    program_hash: &str,
) -> R<(u32, u32)> {
    let left = mask_body(b, actual, count, offset)?;
    let right = mask_body(b, concrete, count, offset)?;
    let pairs = d
        .failures
        .iter()
        .map(|f| {
            (
                f.failure_definition.clone(),
                f.concrete_failure_definition.clone(),
            )
        })
        .collect::<Vec<_>>();
    let replacements = pairs
        .iter()
        .map(|(a, c)| Ok((global(b, c)?, Replacement::Global(global(b, a)?))))
        .collect::<R<BTreeMap<_, _>>>()?;
    let initial = transform(b, right, 0, &replacements)?;
    let boolean = b.boolean;
    let (proof, final_right) = rewrite_operand(b, boolean, left, initial, &pairs, program_hash)?;
    if final_right != right {
        return Err(OrdinaryCarrierError::Linkage);
    }
    let ty = call(b, "Std.Eq", vec![boolean, left, right])?;
    Ok((ty, proof))
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
    sequent: &BindingSequent,
    symbols: &BTreeMap<String, String>,
    carrier: u32,
    offset: u32,
    d: &OrdinaryConcreteOperationDefinition,
    program_hash: &str,
) -> R<(u32, u32)> {
    let operands = operands(sequent, d)?;
    let left = term(b, operands.left, &sequent.subjects, symbols, offset)?;
    let right = term(b, operands.right, &sequent.subjects, symbols, offset)?;
    let replacements = BTreeMap::from([(
        global(b, &d.concrete_definition)?,
        Replacement::Global(global(b, &d.normal_definition)?),
    )]);
    let initial = transform(b, right, 0, &replacements)?;
    let pairs = vec![(d.normal_definition.clone(), d.concrete_definition.clone())];
    let (proof, final_right) = rewrite_operand(b, carrier, left, initial, &pairs, program_hash)?;
    if final_right != right {
        return Err(OrdinaryCarrierError::Linkage);
    }
    Ok((call(b, "Std.Eq", vec![carrier, left, right])?, proof))
}

fn named(b: &mut Builder, name: &str, count: usize, offset: u32) -> R<u32> {
    let arguments = (0..count)
        .map(|i| b.var((count - 1 - i) as u32 + offset))
        .collect::<R<Vec<_>>>()?;
    call(b, name, arguments)
}

pub(super) fn emit(
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
        let (normal_type, normal_proof) = normal_equality(
            &mut b,
            sequent,
            &symbols,
            carrier,
            offset + 1,
            d,
            &original_program_sha256,
        )?;
        let guarded_normal = (b.pi(guard, normal_type)?, b.lam(guard, normal_proof)?);
        let mut statements = vec![];
        let mut all_outcomes = vec![];
        for f in &d.failures {
            statements.push(mask_equality(
                &mut b,
                &f.first_failure_definition,
                &f.concrete_first_failure_definition,
                sequent.subjects.len(),
                offset,
                d,
                &original_program_sha256,
            )?);
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
        statements.push(mask_equality(
            &mut b,
            &d.success_definition,
            &d.concrete_success_definition,
            sequent.subjects.len(),
            offset,
            d,
            &original_program_sha256,
        )?);
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
