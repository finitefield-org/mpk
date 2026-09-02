//! Java-specific admission of untrusted successor VIR. No source execution.
//!
//! Common structural/type/dominance validation runs after these closed rules.
//! The shared projection remains crate-private; validated artifacts retain
//! Java IDs, context, hashes, and the original required-check arrays throughout.

use crate::java_profile::{is_integer, is_scalar, method_id, valid_compilation, valid_source_path};
use crate::safety_check::{
    required_safety_checks_for_profile, validate_safety_check_sequence,
    CompiledRequiredCheckProfile, VirSafetyOperation,
};
use crate::successor_source_artifacts::{SuccessorVirFunction, ValidatedSuccessorVir};
use crate::vir::{
    BitVectorWidth, VirBinaryOperator as Op, VirContractExpr, VirInstruction, VirLiteral,
    VirSafetyCheck, VirTermination, VirTerminator, VirType, VirValue,
};
use std::collections::{BTreeMap, BTreeSet, VecDeque};

pub(crate) fn validate(vir: &ValidatedSuccessorVir) -> Result<(), &'static str> {
    let [unit] = vir.module().units() else {
        return Err("Java requires one compilation");
    };
    if !valid_compilation(unit.id())
        || unit.id() != unit.name()
        || !unit.type_decls().is_empty()
        || !unit.const_decls().is_empty()
        || unit.functions().is_empty()
        || unit.functions().len() > 128
    {
        return Err("invalid Java compilation shape");
    }
    let mut names = BTreeSet::new();
    let mut instructions = 0_usize;
    let mut blocks = 0_usize;
    let mut contract_nodes = 0_usize;
    for function in unit.functions() {
        let method = method_id(function.id()).ok_or("invalid Java method ID")?;
        if method.name != function.name()
            || !valid_source_path(&method.source_path())
            || !names.insert((method.owner, method.name))
            || method.parameters.len() != function.params().len()
            || !method
                .parameters
                .iter()
                .eq(function.params().iter().map(|p| &p.r#type))
            || function.results().len() != 1
            || function.results()[0].r#type != method.result
            || function.locals().iter().any(|p| !is_scalar(&p.r#type))
        {
            return Err("Java signature, declaration or result differs");
        }
        if function.blocks().is_empty() || function.blocks().len() > 1024 {
            return Err("Java block limit");
        }
        blocks += function.blocks().len();
        let count: usize = function.blocks().iter().map(|b| b.instructions.len()).sum();
        if count > 100_000 {
            return Err("Java instruction limit");
        }
        instructions += count;
        if blocks > 8192 || instructions > 250_000 {
            return Err("Java closure limit");
        }
        contract_nodes += validate_contract(function)?;
        if contract_nodes > 8192 {
            return Err("Java contract closure limit");
        }
        validate_acyclic(function)?;
        validate_instructions(function)?;
    }
    crate::successor_vc::validate_java_vir_structure(vir)
        .map_err(|_| "invalid Java VIR structure, types, dominance or features")
}

fn validate_contract(function: &SuccessorVirFunction) -> Result<usize, &'static str> {
    let contract = function.contracts();
    if contract.ensures().is_empty()
        || contract.requires().len() + contract.ensures().len() > 64
        || contract.termination() != VirTermination::Total
        || !contract.loops().is_empty()
    {
        return Err("invalid Java normalized contract shape");
    }
    let mut stack = contract
        .requires()
        .iter()
        .chain(contract.ensures())
        .map(|e| (e, 1))
        .collect::<Vec<_>>();
    let mut count = 0;
    while let Some((expr, depth)) = stack.pop() {
        count += 1;
        if count > 1024 || depth > 32 {
            return Err("Java normalized contract limit");
        }
        match expr {
            VirContractExpr::Variable(_)
            | VirContractExpr::Result(_)
            | VirContractExpr::Boolean(_) => {}
            VirContractExpr::Integer(value)
                if matches!(
                    value.int.width,
                    BitVectorWidth::Bits32 | BitVectorWidth::Bits64
                ) && value.int.signed => {}
            VirContractExpr::Unary(value) => stack.push((&value.value, depth + 1)),
            VirContractExpr::Nary(value) if (2..=64).contains(&value.args.len()) => {
                stack.extend(value.args.iter().map(|e| (e, depth + 1)));
            }
            VirContractExpr::Binary(value)
                if matches!(
                    value.op,
                    Op::Eq
                        | Op::NotEq
                        | Op::SignedLt
                        | Op::SignedLe
                        | Op::SignedGt
                        | Op::SignedGe
                        | Op::BvAdd
                        | Op::BvSub
                        | Op::BvMul
                        | Op::BvAnd
                        | Op::BvOr
                        | Op::BvXor
                ) =>
            {
                stack.push((&value.lhs, depth + 1));
                stack.push((&value.rhs, depth + 1));
            }
            _ => return Err("Java normalized contract operation or type is excluded"),
        }
    }
    Ok(count)
}

fn validate_acyclic(function: &SuccessorVirFunction) -> Result<(), &'static str> {
    let indices: BTreeMap<_, _> = function
        .blocks()
        .iter()
        .enumerate()
        .map(|(i, b)| (b.label.as_str(), i))
        .collect();
    let mut edges = vec![Vec::new(); function.blocks().len()];
    let mut indegree = vec![0; edges.len()];
    for (i, block) in function.blocks().iter().enumerate() {
        let labels = match &block.terminator {
            VirTerminator::Return { .. } => Vec::new(),
            VirTerminator::Jump { label, .. } => vec![label],
            VirTerminator::Branch {
                else_label,
                then_label,
                ..
            } => vec![else_label, then_label],
        };
        for label in labels {
            let &target = indices
                .get(label.as_str())
                .ok_or("unknown Java CFG target")?;
            edges[i].push(target);
            indegree[target] += 1;
        }
    }
    let mut ready: VecDeque<_> = indegree
        .iter()
        .enumerate()
        .filter_map(|(i, n)| (*n == 0).then_some(i))
        .collect();
    let mut visited = 0;
    while let Some(i) = ready.pop_front() {
        visited += 1;
        for &target in &edges[i] {
            indegree[target] -= 1;
            if indegree[target] == 0 {
                ready.push_back(target)
            }
        }
    }
    if visited != edges.len() {
        return Err("Java CFG cycle");
    }
    Ok(())
}

fn validate_instructions(function: &SuccessorVirFunction) -> Result<(), &'static str> {
    for block in function.blocks() {
        if block.parameters.iter().any(|p| !is_scalar(&p.r#type)) {
            return Err("Java block parameters must be source scalars");
        }
        let mut types: BTreeMap<_, _> = function
            .params()
            .iter()
            .chain(function.locals())
            .chain(&block.parameters)
            .map(|p| (p.id.as_str(), &p.r#type))
            .collect();
        let mut definitions = BTreeMap::new();
        let mut unsigned = BTreeSet::new();
        let mut admitted_unsigned = BTreeSet::new();
        let mut uses = BTreeMap::new();
        for instruction in &block.instructions {
            for value in instruction_values(instruction) {
                if let VirValue::Variable(reference) = value {
                    *uses.entry(reference.var.as_str()).or_insert(0_usize) += 1;
                }
            }
        }
        for value in terminator_values(&block.terminator) {
            if let VirValue::Variable(reference) = value {
                *uses.entry(reference.var.as_str()).or_insert(0_usize) += 1;
            }
        }
        for (index, instruction) in block.instructions.iter().enumerate() {
            let (id, ty, checks) = instruction_parts(instruction);
            let (operation, operands) = match instruction {
                VirInstruction::BinOp { op, lhs, rhs, .. } => (
                    VirSafetyOperation::Binary(*op),
                    vec![value_type(lhs, &types)?, value_type(rhs, &types)?],
                ),
                VirInstruction::UnaryOp { op, value, .. } => (
                    VirSafetyOperation::Unary(*op),
                    vec![value_type(value, &types)?],
                ),
                other => (VirSafetyOperation::None(other.kind()), Vec::new()),
            };
            let expected = required_safety_checks_for_profile(
                CompiledRequiredCheckProfile::JavaScalarV0,
                operation,
                &operands,
                checks,
            )
            .map_err(|_| "excluded Java operation or operand type")?;
            validate_safety_check_sequence(checks, &expected)
                .map_err(|_| "Java required-check sequence differs")?;
            if !is_scalar(ty) {
                if matches!(
                    ty,
                    VirType::Bv {
                        width: BitVectorWidth::Bits32 | BitVectorWidth::Bits64,
                        signed: false
                    }
                ) {
                    unsigned.insert(id);
                } else {
                    return Err("excluded Java instruction result type");
                }
            }
            match instruction {
                VirInstruction::Const { .. } if is_scalar(ty) => {}
                VirInstruction::Copy { .. } | VirInstruction::CallStatic { .. }
                    if is_scalar(ty) =>
                {
                    // All call/copy arguments remain source scalars. Exact type,
                    // arity and local-target linkage are checked structurally.
                    for value in instruction_values(instruction) {
                        if !is_scalar(&value_type(value, &types)?) {
                            return Err("unsigned Java value escapes into call/copy");
                        }
                    }
                }
                VirInstruction::UnaryOp { .. } | VirInstruction::BinOp { .. } => {
                    if let VirInstruction::BinOp {
                        op: Op::BvShl | Op::BvAshr | Op::BvLshr,
                        lhs,
                        rhs,
                        ..
                    } = instruction
                    {
                        let width = match operands[0] {
                            VirType::Bv { width, .. } => width,
                            _ => return Err("invalid Java shift carrier"),
                        };
                        validate_mask(rhs, width, &definitions)?;
                        validate_shift_sequence(&block.instructions, index)?;
                        if let VirInstruction::BinOp { op: Op::BvLshr, .. } = instruction {
                            let source_id = variable(lhs)?;
                            let Some(VirInstruction::Convert { r#type, value, .. }) =
                                definitions.get(source_id).copied()
                            else {
                                return Err("Java logical shift needs unsigned conversion");
                            };
                            if r#type != ty
                                || value_type(value, &types)? != signed_type(width)
                                || uses.get(source_id) != Some(&1)
                            {
                                return Err("Java unsigned input conversion differs or escapes");
                            }
                            admitted_unsigned.insert(source_id);
                        }
                    }
                }
                VirInstruction::Convert { value, .. } => {
                    let from = value_type(value, &types)?;
                    if is_integer(&from) && is_integer(ty) && from != *ty {
                        // Exactly int<->long. Identity conversions emit nothing.
                    } else if let (
                        VirType::Bv {
                            width: a,
                            signed: true,
                        },
                        VirType::Bv {
                            width: b,
                            signed: false,
                        },
                    ) = (&from, ty)
                    {
                        if a != b {
                            return Err("Java unsigned conversion changes width");
                        }
                    } else if let (
                        VirType::Bv {
                            width: a,
                            signed: false,
                        },
                        VirType::Bv {
                            width: b,
                            signed: true,
                        },
                    ) = (&from, ty)
                    {
                        let source_id = variable(value)?;
                        if a != b
                            || uses.get(source_id) != Some(&1)
                            || !matches!(
                                definitions.get(source_id),
                                Some(VirInstruction::BinOp { op: Op::BvLshr, .. })
                            )
                        {
                            return Err("Java signed restoration is not a closed logical shift");
                        }
                        admitted_unsigned.insert(source_id);
                    } else {
                        return Err("excluded Java conversion");
                    }
                }
                _ => return Err("excluded Java instruction"),
            }
            definitions.insert(id, instruction);
            types.insert(id, ty);
        }
        for value in terminator_values(&block.terminator) {
            if !is_scalar(&value_type(value, &types)?) {
                return Err("unsigned intermediate escapes Java block");
            }
        }
        if unsigned != admitted_unsigned {
            return Err("unpaired Java unsigned intermediate");
        }
    }
    Ok(())
}

fn validate_shift_sequence(
    instructions: &[VirInstruction],
    index: usize,
) -> Result<(), &'static str> {
    let VirInstruction::BinOp {
        id, op, lhs, rhs, ..
    } = &instructions[index]
    else {
        return Err("Java shift instruction absent");
    };
    let logical = *op == Op::BvLshr;
    let start = index
        .checked_sub(if logical { 3 } else { 2 })
        .ok_or("Java shift helper sequence is incomplete")?;
    let constant = instruction_parts(&instructions[start]).0;
    let mask = instruction_parts(&instructions[start + 1]).0;
    let VirInstruction::BinOp {
        rhs: mask_constant, ..
    } = &instructions[start + 1]
    else {
        return Err("Java mask is not in canonical position");
    };
    if variable(rhs)? != mask || variable(mask_constant)? != constant {
        return Err("Java shift helpers are reordered, interleaved or reused");
    }
    if logical {
        if variable(lhs)? != instruction_parts(&instructions[start + 2]).0 {
            return Err("Java unsigned conversion is reordered");
        }
        let Some(VirInstruction::Convert { value, .. }) = instructions.get(index + 1) else {
            return Err("Java logical shift must immediately restore signedness");
        };
        if variable(value)? != id {
            return Err("Java signed restoration references another value");
        }
    }
    Ok(())
}

fn signed_type(width: BitVectorWidth) -> VirType {
    VirType::Bv {
        width,
        signed: true,
    }
}

fn variable(value: &VirValue) -> Result<&str, &'static str> {
    if let VirValue::Variable(reference) = value {
        Ok(&reference.var)
    } else {
        Err("Java helper must reference the exact producer")
    }
}

fn value_type(value: &VirValue, types: &BTreeMap<&str, &VirType>) -> Result<VirType, &'static str> {
    match value {
        VirValue::Variable(reference) => types
            .get(reference.var.as_str())
            .map(|ty| (*ty).clone())
            .ok_or("unknown Java value"),
        VirValue::Boolean(_) => Ok(VirType::Bool {}),
        VirValue::Integer(value)
            if value.int.signed
                && matches!(
                    value.int.width,
                    BitVectorWidth::Bits32 | BitVectorWidth::Bits64
                ) =>
        {
            Ok(signed_type(value.int.width))
        }
        _ => Err("excluded Java literal or constant reference"),
    }
}

fn validate_mask(
    count: &VirValue,
    width: BitVectorWidth,
    definitions: &BTreeMap<&str, &VirInstruction>,
) -> Result<(), &'static str> {
    let Some(VirInstruction::BinOp {
        op: Op::BvAnd,
        r#type,
        rhs,
        ..
    }) = definitions.get(variable(count)?).copied()
    else {
        return Err("Java shift count is not the linked mask");
    };
    if *r#type != signed_type(BitVectorWidth::Bits32) {
        return Err("Java mask width differs");
    }
    let Some(VirInstruction::Const {
        r#type,
        value: VirLiteral::Integer(mask),
        ..
    }) = definitions.get(variable(rhs)?).copied()
    else {
        return Err("Java mask is not the linked constant");
    };
    let expected = if width == BitVectorWidth::Bits32 {
        "31"
    } else {
        "63"
    };
    if *r#type != signed_type(BitVectorWidth::Bits32)
        || mask.int.width != BitVectorWidth::Bits32
        || !mask.int.signed
        || mask.int.value.as_str() != expected
    {
        return Err("Java mask constant differs");
    }
    Ok(())
}

pub(crate) fn instruction_parts(
    instruction: &VirInstruction,
) -> (&str, &VirType, &[VirSafetyCheck]) {
    match instruction {
        VirInstruction::Const {
            id,
            r#type,
            safety_checks,
            ..
        }
        | VirInstruction::Copy {
            id,
            r#type,
            safety_checks,
            ..
        }
        | VirInstruction::BinOp {
            id,
            r#type,
            safety_checks,
            ..
        }
        | VirInstruction::UnaryOp {
            id,
            r#type,
            safety_checks,
            ..
        }
        | VirInstruction::Convert {
            id,
            r#type,
            safety_checks,
            ..
        }
        | VirInstruction::Field {
            id,
            r#type,
            safety_checks,
            ..
        }
        | VirInstruction::Index {
            id,
            r#type,
            safety_checks,
            ..
        }
        | VirInstruction::MakeStruct {
            id,
            r#type,
            safety_checks,
            ..
        }
        | VirInstruction::MakeArray {
            id,
            r#type,
            safety_checks,
            ..
        }
        | VirInstruction::CallStatic {
            id,
            r#type,
            safety_checks,
            ..
        } => (id, r#type, safety_checks),
    }
}

fn instruction_values(instruction: &VirInstruction) -> Vec<&VirValue> {
    match instruction {
        VirInstruction::Const { .. } => Vec::new(),
        VirInstruction::Copy { value, .. }
        | VirInstruction::UnaryOp { value, .. }
        | VirInstruction::Convert { value, .. } => vec![value],
        VirInstruction::BinOp { lhs, rhs, .. } => vec![lhs, rhs],
        VirInstruction::CallStatic { args, .. } => args.iter().collect(),
        // These variants are rejected before interpreting their operands.
        _ => Vec::new(),
    }
}

fn terminator_values(terminator: &VirTerminator) -> Vec<&VirValue> {
    match terminator {
        VirTerminator::Return { values } => values.iter().collect(),
        VirTerminator::Jump { args, .. } => args.iter().collect(),
        VirTerminator::Branch {
            cond,
            then_args,
            else_args,
            ..
        } => std::iter::once(cond)
            .chain(then_args)
            .chain(else_args)
            .collect(),
    }
}
