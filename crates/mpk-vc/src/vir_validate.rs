//! Complete semantic validation for VIR v0.

use crate::safety_check::{
    required_safety_checks, validate_safety_check_sequence, SafetyCheckError, VirSafetyOperation,
};
use crate::semantic_profile::{
    validate_semantic_context, validate_semantic_parameters, PointerWidth, SemanticParameters,
    SemanticProfile, SourceLanguage,
};
use crate::vir::{
    LowercaseSha256, VirBinaryOperator, VirBlock, VirConstDecl, VirContract, VirContractExpr,
    VirFeature, VirFunction, VirInstruction, VirModule, VirSafetyCheck, VirStructDecl,
    VirTerminator, VirType, VirUnit, VirValue, VIR_SCHEMA_VERSION,
};
use crate::vir_canonical::{canonical_vir_json, contract_hash, vir_hash, VirCanonicalError};
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::error::Error;
use std::fmt;

pub const VIR_CANONICAL_JSON_BYTES_MAX: u64 = 201_326_592;
pub const VIR_UNITS_MAX: usize = 256;
pub const VIR_TYPE_DECLS_MAX: usize = 4_096;
pub const VIR_CONST_DECLS_MAX: usize = 65_536;
pub const VIR_FUNCTIONS_MAX: usize = 8_192;
pub const VIR_PARAMS_MAX: usize = 256;
pub const VIR_RESULTS_MAX: usize = 16;
pub const VIR_LOCALS_MAX: usize = 65_536;
pub const VIR_BLOCKS_PER_FUNCTION_MAX: usize = 8_192;
pub const VIR_BLOCKS_PER_MODULE_MAX: usize = 65_536;
pub const VIR_BLOCK_PARAMETERS_MAX: usize = 4_096;
pub const VIR_INSTRUCTIONS_PER_BLOCK_MAX: usize = 100_000;
pub const VIR_INSTRUCTIONS_PER_FUNCTION_MAX: usize = 100_000;
pub const VIR_INSTRUCTIONS_PER_MODULE_MAX: usize = 250_000;
pub const VIR_CFG_EDGES_PER_FUNCTION_MAX: usize = 16_000;
pub const VIR_CALL_ARGS_MAX: usize = 256;
pub const VIR_ARRAY_ELEMENTS_MAX: usize = 256;
pub const VIR_STRUCT_FIELDS_MAX: usize = 64;
pub const VIR_AGGREGATE_TYPE_NESTING_MAX: usize = 16;
pub const VIR_CONTRACT_CLAUSES_MAX: usize = 64;
pub const VIR_CONTRACT_EXPR_NODES_PER_FUNCTION_MAX: usize = 1_024;
pub const VIR_CONTRACT_EXPR_NODES_PER_MODULE_MAX: usize = 8_192;
pub const VIR_CONTRACT_EXPR_NESTING_MAX: usize = 32;
pub const VIR_LOOPS_MAX: usize = 1_024;
pub const VIR_LOOP_INVARIANTS_MAX: usize = 64;
pub const VIR_LOOP_DECREASES_MAX: usize = 64;
pub const VIR_IDENTIFIER_BYTES_MAX: usize = 1_024;

const RUST_FUNCTIONS_MAX: usize = 128;
const RUST_BLOCKS_PER_FUNCTION_MAX: usize = 1_024;
const RUST_BLOCKS_PER_MODULE_MAX: usize = 8_192;

pub fn validate_vir(module: &VirModule) -> Result<(), VirValidationError> {
    if module.schema != VIR_SCHEMA_VERSION {
        return Err(invalid(
            "VIR_SCHEMA_UNSUPPORTED",
            format!("unsupported VIR schema {:?}", module.schema),
        ));
    }
    validate_profile(module)?;
    validate_limits(module)?;
    let index = ModuleIndex::build(module)?;
    validate_declarations(module, &index)?;

    let mut function_analysis = BTreeMap::new();
    for (unit_index, unit) in module.units.iter().enumerate() {
        for function in &unit.functions {
            let analysis = validate_function(module, unit_index, unit, function, &index)?;
            function_analysis.insert(function.id.clone(), analysis);
        }
    }
    validate_call_graph_and_order(module, &function_analysis)?;
    validate_module_safety_checks(module, &function_analysis)?;
    validate_hashes(module, &index)?;
    Ok(())
}

fn validate_profile(module: &VirModule) -> Result<(), VirValidationError> {
    validate_semantic_context(
        module.source_language,
        module.semantic_profile,
        &module.semantic_parameters,
    )
    .map_err(|error| invalid("VIR_PROFILE_MISMATCH", error.to_string()))?;

    match (&module.semantic_profile, &module.semantic_parameters) {
        (SemanticProfile::GoFixedV0, SemanticParameters::GoFixed(parameters))
            if parameters.target_id == "linux/amd64"
                && parameters.pointer_width == PointerWidth::Bits64 => {}
        (SemanticProfile::RustCheckedV0, SemanticParameters::RustChecked(parameters))
            if (parameters.target_id == "i686-unknown-linux-gnu"
                && parameters.pointer_width == PointerWidth::Bits32)
                || (parameters.target_id == "x86_64-unknown-linux-gnu"
                    && parameters.pointer_width == PointerWidth::Bits64) => {}
        (SemanticProfile::GoFixedV0, SemanticParameters::GoFixed(_))
        | (SemanticProfile::RustCheckedV0, SemanticParameters::RustChecked(_)) => {
            return Err(invalid(
                "VIR_TARGET_WIDTH",
                "target identifier and pointer width are not a registered profile pair",
            ));
        }
        _ => {
            return Err(invalid(
                "VIR_SEMANTIC_PARAMETERS",
                "semantic parameter shape does not match the selected profile",
            ));
        }
    }
    Ok(())
}

fn validate_limits(module: &VirModule) -> Result<(), VirValidationError> {
    limit_nonempty(&module.units, "VIR_EMPTY_UNITS", "units")?;
    limit_max(&module.units, VIR_UNITS_MAX, "VIR_LIMIT_UNITS", "units")?;
    if module.source_language == SourceLanguage::Rust && module.units.len() != 1 {
        return Err(invalid(
            "VIR_RUST_UNIT_COUNT",
            "Rust VIR v0 contains exactly one unit",
        ));
    }
    validate_identifier_limits(module)?;
    validate_aggregate_depth_limits(module)?;

    let mut type_decls = 0_usize;
    let mut const_decls = 0_usize;
    let mut functions = 0_usize;
    let mut blocks = 0_usize;
    let mut instructions = 0_usize;
    let mut contract_nodes = 0_usize;
    let mut loops = 0_usize;
    for unit in &module.units {
        limit_nonempty(&unit.functions, "VIR_EMPTY_FUNCTIONS", "functions")?;
        type_decls = checked_add(type_decls, unit.type_decls.len(), "type declarations")?;
        const_decls = checked_add(const_decls, unit.const_decls.len(), "constant declarations")?;
        functions = checked_add(functions, unit.functions.len(), "functions")?;
        for decl in &unit.type_decls {
            limit_max(
                &decl.fields,
                VIR_STRUCT_FIELDS_MAX,
                "VIR_LIMIT_STRUCT_FIELDS",
                "struct fields",
            )?;
        }
        for function in &unit.functions {
            limit_max(
                &function.params,
                VIR_PARAMS_MAX,
                "VIR_LIMIT_PARAMS",
                "parameters",
            )?;
            limit_max(
                &function.results,
                VIR_RESULTS_MAX,
                "VIR_LIMIT_RESULTS",
                "results",
            )?;
            limit_max(
                &function.locals,
                VIR_LOCALS_MAX,
                "VIR_LIMIT_LOCALS",
                "locals",
            )?;
            limit_nonempty(&function.blocks, "VIR_EMPTY_BLOCKS", "blocks")?;
            let per_function_block_max =
                if module.semantic_profile == SemanticProfile::RustCheckedV0 {
                    RUST_BLOCKS_PER_FUNCTION_MAX
                } else {
                    VIR_BLOCKS_PER_FUNCTION_MAX
                };
            limit_max(
                &function.blocks,
                per_function_block_max,
                "VIR_LIMIT_BLOCKS_PER_FUNCTION",
                "blocks per function",
            )?;
            limit_max(
                &function.contracts.requires,
                VIR_CONTRACT_CLAUSES_MAX,
                "VIR_LIMIT_CONTRACT_CLAUSES",
                "contract clauses",
            )?;
            let clause_count = checked_add(
                function.contracts.requires.len(),
                function.contracts.ensures.len(),
                "contract clauses",
            )?;
            if clause_count > VIR_CONTRACT_CLAUSES_MAX {
                return Err(limit(
                    "VIR_LIMIT_CONTRACT_CLAUSES",
                    "requires plus ensures exceeds the function limit",
                ));
            }
            limit_nonempty(&function.contracts.ensures, "VIR_EMPTY_ENSURES", "ensures")?;
            loops = checked_add(loops, function.contracts.loops.len(), "loops")?;
            let (nodes, nesting) = contract_metrics(&function.contracts)?;
            if nodes > VIR_CONTRACT_EXPR_NODES_PER_FUNCTION_MAX {
                return Err(limit(
                    "VIR_LIMIT_CONTRACT_EXPR_NODES_PER_FUNCTION",
                    "contract expression node count exceeds the function limit",
                ));
            }
            if nesting > VIR_CONTRACT_EXPR_NESTING_MAX {
                return Err(limit(
                    "VIR_LIMIT_CONTRACT_EXPR_NESTING",
                    "contract expression nesting exceeds the limit",
                ));
            }
            contract_nodes = checked_add(contract_nodes, nodes, "contract expression nodes")?;
            let mut function_instructions = 0_usize;
            for block in &function.blocks {
                limit_max(
                    &block.parameters,
                    VIR_BLOCK_PARAMETERS_MAX,
                    "VIR_LIMIT_BLOCK_PARAMETERS",
                    "block parameters",
                )?;
                limit_max(
                    &block.instructions,
                    VIR_INSTRUCTIONS_PER_BLOCK_MAX,
                    "VIR_LIMIT_INSTRUCTIONS_PER_BLOCK",
                    "instructions per block",
                )?;
                function_instructions = checked_add(
                    function_instructions,
                    block.instructions.len(),
                    "instructions per function",
                )?;
                for instruction in &block.instructions {
                    match instruction {
                        VirInstruction::MakeArray { elements, .. } => limit_max(
                            elements,
                            VIR_ARRAY_ELEMENTS_MAX,
                            "VIR_LIMIT_ARRAY_ELEMENTS",
                            "array elements",
                        )?,
                        VirInstruction::CallStatic { args, .. } => limit_max(
                            args,
                            VIR_CALL_ARGS_MAX,
                            "VIR_LIMIT_CALL_ARGS",
                            "call arguments",
                        )?,
                        _ => {}
                    }
                }
            }
            if function_instructions > VIR_INSTRUCTIONS_PER_FUNCTION_MAX {
                return Err(limit(
                    "VIR_LIMIT_INSTRUCTIONS_PER_FUNCTION",
                    "instruction count exceeds the function limit",
                ));
            }
            blocks = checked_add(blocks, function.blocks.len(), "module blocks")?;
            instructions = checked_add(instructions, function_instructions, "module instructions")?;
        }
    }
    check_total(type_decls, VIR_TYPE_DECLS_MAX, "VIR_LIMIT_TYPE_DECLS")?;
    check_total(const_decls, VIR_CONST_DECLS_MAX, "VIR_LIMIT_CONST_DECLS")?;
    check_total(functions, VIR_FUNCTIONS_MAX, "VIR_LIMIT_FUNCTIONS")?;
    if module.semantic_profile == SemanticProfile::RustCheckedV0 && functions > RUST_FUNCTIONS_MAX {
        return Err(limit(
            "VIR_LIMIT_FUNCTIONS",
            "Rust call closure exceeds 128 functions",
        ));
    }
    let module_block_max = if module.semantic_profile == SemanticProfile::RustCheckedV0 {
        RUST_BLOCKS_PER_MODULE_MAX
    } else {
        VIR_BLOCKS_PER_MODULE_MAX
    };
    check_total(blocks, module_block_max, "VIR_LIMIT_BLOCKS_PER_MODULE")?;
    check_total(
        instructions,
        VIR_INSTRUCTIONS_PER_MODULE_MAX,
        "VIR_LIMIT_INSTRUCTIONS_PER_MODULE",
    )?;
    check_total(
        contract_nodes,
        VIR_CONTRACT_EXPR_NODES_PER_MODULE_MAX,
        "VIR_LIMIT_CONTRACT_EXPR_NODES_PER_MODULE",
    )?;
    check_total(loops, VIR_LOOPS_MAX, "VIR_LIMIT_LOOPS")?;
    Ok(())
}

fn validate_identifier_limits(module: &VirModule) -> Result<(), VirValidationError> {
    for unit in &module.units {
        check_identifier_strings([unit.id.as_str(), unit.name.as_str()])?;
        for declaration in &unit.type_decls {
            check_identifier_strings([declaration.id.as_str(), declaration.name.as_str()])?;
            for field in &declaration.fields {
                validate_identifier_bytes(&field.name)?;
                validate_type_identifier_limits(&field.r#type)?;
            }
        }
        for declaration in &unit.const_decls {
            check_identifier_strings([declaration.id.as_str(), declaration.name.as_str()])?;
            validate_type_identifier_limits(&declaration.r#type)?;
        }
        for function in &unit.functions {
            check_identifier_strings([
                function.id.as_str(),
                function.unit_id.as_str(),
                function.name.as_str(),
                function.contracts.unit_id.as_str(),
                function.contracts.function_id.as_str(),
            ])?;
            for binding in function
                .params
                .iter()
                .chain(&function.results)
                .chain(&function.locals)
                .chain(function.blocks.iter().flat_map(|block| &block.parameters))
            {
                validate_identifier_bytes(&binding.id)?;
                validate_type_identifier_limits(&binding.r#type)?;
            }
            for block in &function.blocks {
                validate_identifier_bytes(&block.label)?;
                for instruction in &block.instructions {
                    validate_identifier_bytes(instruction_id(instruction))?;
                    validate_type_identifier_limits(instruction_type(instruction))?;
                    match instruction {
                        VirInstruction::Copy { target, .. } => validate_identifier_bytes(target)?,
                        VirInstruction::Field { field, .. } => validate_identifier_bytes(field)?,
                        VirInstruction::CallStatic { function, .. } => {
                            validate_identifier_bytes(function)?
                        }
                        _ => {}
                    }
                    for value in instruction_values(instruction) {
                        validate_value_identifier_limits(value)?;
                    }
                }
                match &block.terminator {
                    VirTerminator::Jump { label, .. } => validate_identifier_bytes(label)?,
                    VirTerminator::Branch {
                        then_label,
                        else_label,
                        ..
                    } => check_identifier_strings([then_label.as_str(), else_label.as_str()])?,
                    VirTerminator::Return { .. } => {}
                }
                for value in terminator_values(&block.terminator) {
                    validate_value_identifier_limits(value)?;
                }
            }
            for modified in &function.contracts.modifies {
                validate_identifier_bytes(modified)?;
            }
            for expression in function
                .contracts
                .requires
                .iter()
                .chain(&function.contracts.ensures)
            {
                validate_contract_identifier_limits(expression)?;
            }
            for loop_contract in &function.contracts.loops {
                validate_identifier_bytes(&loop_contract.header)?;
                for expression in loop_contract
                    .invariants
                    .iter()
                    .chain(&loop_contract.decreases)
                {
                    validate_contract_identifier_limits(expression)?;
                }
            }
        }
    }
    Ok(())
}

fn check_identifier_strings<'a>(
    identifiers: impl IntoIterator<Item = &'a str>,
) -> Result<(), VirValidationError> {
    for identifier in identifiers {
        validate_identifier_bytes(identifier)?;
    }
    Ok(())
}

fn validate_type_identifier_limits(r#type: &VirType) -> Result<(), VirValidationError> {
    match r#type {
        VirType::Array { element, .. } => validate_type_identifier_limits(element),
        VirType::Struct { id } => validate_identifier_bytes(id),
        VirType::Bool {} | VirType::Bv { .. } => Ok(()),
    }
}

fn validate_value_identifier_limits(value: &VirValue) -> Result<(), VirValidationError> {
    match value {
        VirValue::Variable(reference) => validate_identifier_bytes(&reference.var),
        VirValue::Constant(reference) => validate_identifier_bytes(&reference.constant),
        VirValue::Boolean(_) | VirValue::Integer(_) => Ok(()),
    }
}

fn validate_contract_identifier_limits(
    expression: &VirContractExpr,
) -> Result<(), VirValidationError> {
    match expression {
        VirContractExpr::Variable(reference) => validate_identifier_bytes(&reference.var),
        VirContractExpr::Unary(expression) => {
            validate_contract_identifier_limits(&expression.value)
        }
        VirContractExpr::Nary(expression) => {
            for argument in &expression.args {
                validate_contract_identifier_limits(argument)?;
            }
            Ok(())
        }
        VirContractExpr::Binary(expression) => {
            validate_contract_identifier_limits(&expression.lhs)?;
            validate_contract_identifier_limits(&expression.rhs)
        }
        VirContractExpr::Convert(expression) => {
            validate_type_identifier_limits(&expression.r#type)?;
            validate_contract_identifier_limits(&expression.value)
        }
        VirContractExpr::Result(_) | VirContractExpr::Boolean(_) | VirContractExpr::Integer(_) => {
            Ok(())
        }
    }
}

fn validate_aggregate_depth_limits(module: &VirModule) -> Result<(), VirValidationError> {
    for unit in &module.units {
        let analysis = analyze_struct_declarations(&unit.type_decls, false)?;
        for declaration in &unit.type_decls {
            for field in &declaration.fields {
                validate_cached_aggregate_depth(&field.r#type, &analysis.depths, 1)?;
            }
        }
        for declaration in &unit.const_decls {
            validate_cached_aggregate_depth(&declaration.r#type, &analysis.depths, 0)?;
        }
        for function in &unit.functions {
            for binding in function
                .params
                .iter()
                .chain(&function.results)
                .chain(&function.locals)
                .chain(function.blocks.iter().flat_map(|block| &block.parameters))
            {
                validate_cached_aggregate_depth(&binding.r#type, &analysis.depths, 0)?;
            }
            for instruction in function
                .blocks
                .iter()
                .flat_map(|block| block.instructions.iter())
            {
                validate_cached_aggregate_depth(
                    instruction_type(instruction),
                    &analysis.depths,
                    0,
                )?;
            }
        }
    }
    Ok(())
}

fn validate_cached_aggregate_depth(
    r#type: &VirType,
    depths: &BTreeMap<String, usize>,
    depth: usize,
) -> Result<(), VirValidationError> {
    match r#type {
        VirType::Bool {} | VirType::Bv { .. } => Ok(()),
        VirType::Array { element, .. } => {
            let depth = depth + 1;
            check_aggregate_depth(depth)?;
            validate_cached_aggregate_depth(element, depths, depth)
        }
        VirType::Struct { id } => {
            let nested = depths.get(id).copied().unwrap_or(1);
            let depth = depth
                .checked_add(nested)
                .ok_or_else(|| limit("VIR_LIMIT_OVERFLOW", "aggregate type depth overflow"))?;
            check_aggregate_depth(depth)
        }
    }
}

struct StructAnalysis {
    order: Vec<String>,
    depths: BTreeMap<String, usize>,
}

fn analyze_struct_declarations(
    declarations: &[VirStructDecl],
    strict_references: bool,
) -> Result<StructAnalysis, VirValidationError> {
    let by_id: BTreeMap<_, _> = declarations
        .iter()
        .map(|declaration| (declaration.id.as_str(), declaration))
        .collect();
    let ids: BTreeSet<_> = by_id.keys().copied().collect();
    let mut dependencies: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for declaration in declarations {
        let mut referenced = BTreeSet::new();
        for field in &declaration.fields {
            collect_struct_type_ids(&field.r#type, &mut referenced);
        }
        let mut known = BTreeSet::new();
        for reference in referenced {
            if reference == declaration.id {
                if strict_references {
                    return Err(invalid("VIR_TYPE_CYCLE", "recursive aggregate type"));
                }
                known.insert(reference.to_owned());
            } else if ids.contains(reference) {
                known.insert(reference.to_owned());
            } else if strict_references {
                return Err(invalid(
                    "VIR_UNKNOWN_TYPE",
                    format!("unknown struct {reference:?}"),
                ));
            }
        }
        dependencies.insert(declaration.id.clone(), known);
    }

    let mut dependents: BTreeMap<String, Vec<String>> = BTreeMap::new();
    let mut remaining = BTreeMap::new();
    let mut ready = BTreeSet::new();
    for (id, required) in &dependencies {
        remaining.insert(id.clone(), required.len());
        if required.is_empty() {
            ready.insert(id.clone());
        }
        for dependency in required {
            dependents
                .entry(dependency.clone())
                .or_default()
                .push(id.clone());
        }
    }

    let mut order = Vec::with_capacity(dependencies.len());
    let mut depths = BTreeMap::new();
    while let Some(next) = ready.pop_first() {
        let declaration = by_id
            .get(next.as_str())
            .ok_or_else(|| invalid("VIR_UNKNOWN_TYPE", "missing struct declaration"))?;
        let mut depth = 1_usize;
        for field in &declaration.fields {
            let field_depth = aggregate_type_depth(&field.r#type, &depths)?;
            depth = depth.max(
                1_usize
                    .checked_add(field_depth)
                    .ok_or_else(|| limit("VIR_LIMIT_OVERFLOW", "aggregate type depth overflow"))?,
            );
        }
        check_aggregate_depth(depth)?;
        depths.insert(next.clone(), depth);
        order.push(next.clone());
        for dependent in dependents.get(&next).into_iter().flatten() {
            let count = remaining
                .get_mut(dependent)
                .ok_or_else(|| invalid("VIR_UNKNOWN_TYPE", "missing dependent declaration"))?;
            *count -= 1;
            if *count == 0 {
                ready.insert(dependent.clone());
            }
        }
    }
    if strict_references && order.len() != dependencies.len() {
        return Err(invalid(
            "VIR_TYPE_CYCLE",
            "recursive aggregate declarations",
        ));
    }
    Ok(StructAnalysis { order, depths })
}

fn aggregate_type_depth(
    r#type: &VirType,
    depths: &BTreeMap<String, usize>,
) -> Result<usize, VirValidationError> {
    match r#type {
        VirType::Bool {} | VirType::Bv { .. } => Ok(0),
        VirType::Array { element, .. } => 1_usize
            .checked_add(aggregate_type_depth(element, depths)?)
            .ok_or_else(|| limit("VIR_LIMIT_OVERFLOW", "aggregate type depth overflow")),
        VirType::Struct { id } => Ok(depths.get(id).copied().unwrap_or(1)),
    }
}

fn check_aggregate_depth(depth: usize) -> Result<(), VirValidationError> {
    if depth > VIR_AGGREGATE_TYPE_NESTING_MAX {
        Err(limit(
            "VIR_LIMIT_AGGREGATE_TYPE_NESTING",
            "aggregate type nesting exceeds 16",
        ))
    } else {
        Ok(())
    }
}

fn checked_add(left: usize, right: usize, label: &str) -> Result<usize, VirValidationError> {
    left.checked_add(right)
        .ok_or_else(|| limit("VIR_LIMIT_OVERFLOW", format!("{label} count overflow")))
}

fn check_total(
    actual: usize,
    maximum: usize,
    code: &'static str,
) -> Result<(), VirValidationError> {
    if actual > maximum {
        Err(limit(code, format!("count {actual} exceeds {maximum}")))
    } else {
        Ok(())
    }
}

fn limit_nonempty<T>(
    values: &[T],
    code: &'static str,
    label: &str,
) -> Result<(), VirValidationError> {
    if values.is_empty() {
        Err(invalid(code, format!("{label} must be nonempty")))
    } else {
        Ok(())
    }
}

fn limit_max<T>(
    values: &[T],
    maximum: usize,
    code: &'static str,
    label: &str,
) -> Result<(), VirValidationError> {
    if values.len() > maximum {
        Err(limit(
            code,
            format!("{label} count {} exceeds {maximum}", values.len()),
        ))
    } else {
        Ok(())
    }
}

struct ModuleIndex<'a> {
    structs: Vec<BTreeMap<&'a str, &'a VirStructDecl>>,
    struct_orders: Vec<Vec<String>>,
    struct_depths: Vec<BTreeMap<String, usize>>,
    constants: Vec<BTreeMap<&'a str, &'a VirConstDecl>>,
    functions: BTreeMap<&'a str, (&'a VirUnit, &'a VirFunction)>,
}

impl<'a> ModuleIndex<'a> {
    fn build(module: &'a VirModule) -> Result<Self, VirValidationError> {
        let mut units = BTreeMap::new();
        let mut structs = Vec::with_capacity(module.units.len());
        let mut struct_orders = Vec::with_capacity(module.units.len());
        let mut struct_depths = Vec::with_capacity(module.units.len());
        let mut constants = Vec::with_capacity(module.units.len());
        let mut functions = BTreeMap::new();
        let mut previous_unit: Option<&str> = None;
        for unit in &module.units {
            validate_unit_identity(module.source_language, unit)?;
            if previous_unit.is_some_and(|previous| previous.as_bytes() >= unit.id.as_bytes()) {
                return Err(invalid(
                    "VIR_UNIT_ORDER",
                    "units are not strictly sorted by id",
                ));
            }
            previous_unit = Some(&unit.id);
            if units.insert(unit.id.as_str(), unit).is_some() {
                return Err(invalid("VIR_DUPLICATE_UNIT", "duplicate unit id"));
            }

            let mut unit_structs = BTreeMap::new();
            let mut unit_constants = BTreeMap::new();
            let mut declarations = BTreeSet::new();
            for declaration in &unit.type_decls {
                validate_public_declaration_id(
                    module.source_language,
                    unit,
                    &declaration.id,
                    &declaration.name,
                    false,
                )?;
                if !declarations.insert(declaration.id.as_str())
                    || unit_structs
                        .insert(declaration.id.as_str(), declaration)
                        .is_some()
                {
                    return Err(invalid("VIR_DUPLICATE_DECLARATION", "duplicate type id"));
                }
            }
            let mut previous_const: Option<&str> = None;
            for declaration in &unit.const_decls {
                validate_public_declaration_id(
                    module.source_language,
                    unit,
                    &declaration.id,
                    &declaration.name,
                    false,
                )?;
                if previous_const
                    .is_some_and(|previous| previous.as_bytes() >= declaration.id.as_bytes())
                {
                    return Err(invalid(
                        "VIR_CONST_ORDER",
                        "constant declarations are not strictly sorted by id",
                    ));
                }
                previous_const = Some(&declaration.id);
                if !declarations.insert(declaration.id.as_str())
                    || unit_constants
                        .insert(declaration.id.as_str(), declaration)
                        .is_some()
                {
                    return Err(invalid(
                        "VIR_DUPLICATE_DECLARATION",
                        "duplicate constant id",
                    ));
                }
            }
            for function in &unit.functions {
                validate_public_declaration_id(
                    module.source_language,
                    unit,
                    &function.id,
                    &function.name,
                    true,
                )?;
                if !declarations.insert(function.id.as_str()) {
                    return Err(invalid(
                        "VIR_DUPLICATE_DECLARATION",
                        "function id collides with another declaration",
                    ));
                }
                if functions
                    .insert(function.id.as_str(), (unit, function))
                    .is_some()
                {
                    return Err(invalid("VIR_DUPLICATE_FUNCTION", "duplicate function id"));
                }
            }
            let struct_analysis = analyze_struct_declarations(&unit.type_decls, true)?;
            structs.push(unit_structs);
            struct_orders.push(struct_analysis.order);
            struct_depths.push(struct_analysis.depths);
            constants.push(unit_constants);
        }
        Ok(Self {
            structs,
            struct_orders,
            struct_depths,
            constants,
            functions,
        })
    }
}

fn validate_unit_identity(
    language: SourceLanguage,
    unit: &VirUnit,
) -> Result<(), VirValidationError> {
    validate_identifier_bytes(&unit.id)?;
    validate_identifier_bytes(&unit.name)?;
    match language {
        SourceLanguage::Rust => {
            if !is_ascii_ident(&unit.id) || !is_cargo_package_name(&unit.name) {
                return Err(invalid(
                    "VIR_IDENTIFIER",
                    "Rust unit id or Cargo package name is invalid",
                ));
            }
        }
        SourceLanguage::Go => {
            if !is_ascii_ident(&unit.name) || !is_go_unit_id(&unit.id) {
                return Err(invalid("VIR_IDENTIFIER", "invalid Go unit id"));
            }
        }
    }
    Ok(())
}

fn validate_public_declaration_id(
    language: SourceLanguage,
    unit: &VirUnit,
    id: &str,
    name: &str,
    allow_go_method: bool,
) -> Result<(), VirValidationError> {
    validate_identifier_bytes(id)?;
    if !is_ascii_ident(name) {
        return Err(invalid(
            "VIR_IDENTIFIER",
            "declaration name is not AsciiIdent",
        ));
    }
    let valid = match language {
        SourceLanguage::Rust => {
            let prefix = format!("{}::", unit.id);
            id.strip_prefix(&prefix).is_some_and(|suffix| {
                let segments: Vec<_> = suffix.split("::").collect();
                !segments.is_empty()
                    && segments.iter().all(|segment| is_ascii_ident(segment))
                    && segments.last() == Some(&name)
            })
        }
        SourceLanguage::Go => {
            let prefix = format!("{}.", unit.id);
            id.strip_prefix(&prefix).is_some_and(|suffix| {
                let segments: Vec<_> = suffix.split('.').collect();
                (segments.len() == 1 || (allow_go_method && segments.len() == 2))
                    && segments.iter().all(|segment| is_ascii_ident(segment))
                    && segments.last() == Some(&name)
            })
        }
    };
    if valid {
        Ok(())
    } else {
        Err(invalid("VIR_IDENTIFIER", "invalid public declaration id"))
    }
}

fn validate_identifier_bytes(identifier: &str) -> Result<(), VirValidationError> {
    if identifier.len() > VIR_IDENTIFIER_BYTES_MAX {
        Err(limit(
            "VIR_LIMIT_IDENTIFIER_BYTES",
            "identifier exceeds 1,024 UTF-8 bytes",
        ))
    } else {
        Ok(())
    }
}

fn is_ascii_ident(value: &str) -> bool {
    if value.is_empty() || value == "_" || value.len() > 255 {
        return false;
    }
    let mut bytes = value.bytes();
    let Some(first) = bytes.next() else {
        return false;
    };
    (first.is_ascii_alphabetic() || first == b'_')
        && bytes.all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
}

fn is_cargo_package_name(value: &str) -> bool {
    let mut bytes = value.bytes();
    bytes.next().is_some_and(|byte| byte.is_ascii_alphabetic())
        && bytes.all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-'))
}

fn is_go_unit_id(value: &str) -> bool {
    !value.is_empty()
        && !value.starts_with('/')
        && !value.ends_with('/')
        && !value.contains('\\')
        && !value.contains("://")
        && value.split('/').all(|segment| {
            !matches!(segment, "" | "." | "..")
                && segment
                    .bytes()
                    .next()
                    .is_some_and(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
                && segment.bytes().all(|byte| {
                    byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'.' | b'~' | b'-')
                })
        })
}

fn validate_declarations(
    module: &VirModule,
    index: &ModuleIndex<'_>,
) -> Result<(), VirValidationError> {
    for (unit_index, unit) in module.units.iter().enumerate() {
        let structs = &index.structs[unit_index];
        let depths = &index.struct_depths[unit_index];
        let mut field_names = BTreeSet::new();
        for declaration in &unit.type_decls {
            field_names.clear();
            for field in &declaration.fields {
                if !is_ascii_ident(&field.name) || !field_names.insert(field.name.as_str()) {
                    return Err(invalid(
                        "VIR_DUPLICATE_FIELD",
                        "struct field names must be unique AsciiIdent values",
                    ));
                }
                validate_type(&field.r#type, structs, depths, 1)?;
            }
        }
        validate_type_declaration_order(unit, &index.struct_orders[unit_index])?;
        for declaration in &unit.const_decls {
            validate_const_decl(declaration, structs, depths)?;
        }
    }
    Ok(())
}

fn validate_type_declaration_order(
    unit: &VirUnit,
    expected: &[String],
) -> Result<(), VirValidationError> {
    if !expected.iter().map(String::as_str).eq(unit
        .type_decls
        .iter()
        .map(|declaration| declaration.id.as_str()))
    {
        return Err(invalid(
            "VIR_TYPE_DECL_ORDER",
            "struct declarations are not in canonical dependency order",
        ));
    }
    Ok(())
}

fn collect_struct_type_ids<'a>(r#type: &'a VirType, output: &mut BTreeSet<&'a str>) {
    match r#type {
        VirType::Array { element, .. } => collect_struct_type_ids(element, output),
        VirType::Struct { id } => {
            output.insert(id);
        }
        VirType::Bool {} | VirType::Bv { .. } => {}
    }
}

fn validate_const_decl(
    declaration: &VirConstDecl,
    structs: &BTreeMap<&str, &VirStructDecl>,
    depths: &BTreeMap<String, usize>,
) -> Result<(), VirValidationError> {
    validate_type(&declaration.r#type, structs, depths, 0)?;
    if !matches!(declaration.r#type, VirType::Bool {} | VirType::Bv { .. })
        || literal_type(&declaration.value)? != declaration.r#type
    {
        return Err(invalid(
            "VIR_CONST_TYPE",
            "constant declaration type and literal do not match",
        ));
    }
    Ok(())
}

pub fn validate_vir_type_fragment(
    r#type: &VirType,
    declarations: &[VirStructDecl],
) -> Result<(), VirValidationError> {
    let analysis = analyze_struct_declarations(declarations, true)?;
    let structs: BTreeMap<_, _> = declarations
        .iter()
        .map(|declaration| (declaration.id.as_str(), declaration))
        .collect();
    validate_type(r#type, &structs, &analysis.depths, 0)
}

pub fn validate_vir_struct_decl_fragment(
    source_language: SourceLanguage,
    unit_id: &str,
    declaration: &VirStructDecl,
    declarations: &[VirStructDecl],
) -> Result<(), VirValidationError> {
    let unit = fragment_unit(unit_id);
    validate_public_declaration_id(
        source_language,
        &unit,
        &declaration.id,
        &declaration.name,
        false,
    )?;
    limit_max(
        &declaration.fields,
        VIR_STRUCT_FIELDS_MAX,
        "VIR_LIMIT_STRUCT_FIELDS",
        "struct fields",
    )?;
    let mut all = declarations.to_vec();
    if !all.iter().any(|candidate| candidate.id == declaration.id) {
        all.push(declaration.clone());
    }
    let structs: BTreeMap<_, _> = all
        .iter()
        .map(|candidate| (candidate.id.as_str(), candidate))
        .collect();
    let analysis = analyze_struct_declarations(&all, true)?;
    let mut fields = BTreeSet::new();
    for field in &declaration.fields {
        if !is_ascii_ident(&field.name) || !fields.insert(field.name.as_str()) {
            return Err(invalid(
                "VIR_DUPLICATE_FIELD",
                "struct field names must be unique AsciiIdent values",
            ));
        }
        validate_type(&field.r#type, &structs, &analysis.depths, 1)?;
    }
    Ok(())
}

pub fn validate_vir_const_decl_fragment(
    source_language: SourceLanguage,
    unit_id: &str,
    declaration: &VirConstDecl,
    declarations: &[VirStructDecl],
) -> Result<(), VirValidationError> {
    let unit = fragment_unit(unit_id);
    validate_public_declaration_id(
        source_language,
        &unit,
        &declaration.id,
        &declaration.name,
        false,
    )?;
    let structs: BTreeMap<_, _> = declarations
        .iter()
        .map(|candidate| (candidate.id.as_str(), candidate))
        .collect();
    let analysis = analyze_struct_declarations(declarations, true)?;
    validate_const_decl(declaration, &structs, &analysis.depths)
}

fn fragment_unit(unit_id: &str) -> VirUnit {
    VirUnit {
        id: unit_id.to_owned(),
        name: unit_id.to_owned(),
        type_decls: Vec::new(),
        const_decls: Vec::new(),
        functions: Vec::new(),
    }
}

pub fn validate_vir_contract_expr_fragment(
    expression: &VirContractExpr,
    profile: SemanticProfile,
    variables: &[crate::vir::VirBinding],
    results: &[crate::vir::VirBinding],
    declarations: &[VirStructDecl],
) -> Result<VirType, VirValidationError> {
    let analysis = analyze_struct_declarations(declarations, true)?;
    let structs: BTreeMap<_, _> = declarations
        .iter()
        .map(|declaration| (declaration.id.as_str(), declaration))
        .collect();
    for binding in variables.iter().chain(results) {
        validate_type(&binding.r#type, &structs, &analysis.depths, 0)?;
    }
    let variables: BTreeMap<_, _> = variables
        .iter()
        .map(|binding| (binding.id.as_str(), &binding.r#type))
        .collect();
    validate_contract_expr(expression, profile, &variables, results)
}

fn validate_type(
    r#type: &VirType,
    structs: &BTreeMap<&str, &VirStructDecl>,
    depths: &BTreeMap<String, usize>,
    aggregate_depth: usize,
) -> Result<(), VirValidationError> {
    match r#type {
        VirType::Bool {} | VirType::Bv { .. } => Ok(()),
        VirType::Array { length, element } => {
            if usize::from(length.get()) > VIR_ARRAY_ELEMENTS_MAX {
                return Err(limit(
                    "VIR_LIMIT_ARRAY_ELEMENTS",
                    "array type length exceeds 256",
                ));
            }
            let depth = aggregate_depth + 1;
            if depth > VIR_AGGREGATE_TYPE_NESTING_MAX {
                return Err(limit(
                    "VIR_LIMIT_AGGREGATE_TYPE_NESTING",
                    "aggregate type nesting exceeds 16",
                ));
            }
            validate_type(element, structs, depths, depth)
        }
        VirType::Struct { id } => {
            validate_identifier_bytes(id)?;
            if !structs.contains_key(id.as_str()) {
                return Err(invalid(
                    "VIR_UNKNOWN_TYPE",
                    format!("unknown struct {id:?}"),
                ));
            }
            let nested = depths.get(id).copied().ok_or_else(|| {
                invalid(
                    "VIR_TYPE_CYCLE",
                    format!("struct {id:?} has no acyclic aggregate depth"),
                )
            })?;
            let depth = aggregate_depth
                .checked_add(nested)
                .ok_or_else(|| limit("VIR_LIMIT_OVERFLOW", "aggregate type depth overflow"))?;
            check_aggregate_depth(depth)
        }
    }
}

fn literal_type(literal: &crate::vir::VirLiteral) -> Result<VirType, VirValidationError> {
    match literal {
        crate::vir::VirLiteral::Boolean(_) => Ok(VirType::Bool {}),
        crate::vir::VirLiteral::Integer(value) => {
            validate_integer_literal(&value.int)?;
            Ok(VirType::Bv {
                width: value.int.width,
                signed: value.int.signed,
            })
        }
    }
}

fn value_literal_type(value: &VirValue) -> Result<Option<VirType>, VirValidationError> {
    match value {
        VirValue::Boolean(_) => Ok(Some(VirType::Bool {})),
        VirValue::Integer(value) => {
            validate_integer_literal(&value.int)?;
            Ok(Some(VirType::Bv {
                width: value.int.width,
                signed: value.int.signed,
            }))
        }
        VirValue::Variable(_) | VirValue::Constant(_) => Ok(None),
    }
}

fn validate_integer_literal(literal: &crate::vir::VirIntLiteral) -> Result<(), VirValidationError> {
    let text = literal.value.as_str();
    let valid = if literal.signed {
        let value = text
            .parse::<i128>()
            .map_err(|_| invalid("VIR_LITERAL_RANGE", "signed literal is not representable"))?;
        let width = literal.width.bits();
        let bound = 1_i128 << (width - 1);
        value >= -bound && value < bound
    } else {
        let value = text
            .parse::<u128>()
            .map_err(|_| invalid("VIR_LITERAL_RANGE", "unsigned literal is not representable"))?;
        value < (1_u128 << literal.width.bits())
    };
    if valid {
        Ok(())
    } else {
        Err(invalid(
            "VIR_LITERAL_RANGE",
            "integer literal lies outside its declared bitvector type",
        ))
    }
}

fn contract_metrics(contract: &VirContract) -> Result<(usize, usize), VirValidationError> {
    let mut nodes = 0_usize;
    let mut nesting = 0_usize;
    for expression in contract
        .requires
        .iter()
        .chain(&contract.ensures)
        .chain(
            contract
                .loops
                .iter()
                .flat_map(|loop_contract| loop_contract.invariants.iter()),
        )
        .chain(
            contract
                .loops
                .iter()
                .flat_map(|loop_contract| loop_contract.decreases.iter()),
        )
    {
        let (expression_nodes, expression_nesting) = expression_metrics(expression)?;
        nodes = checked_add(nodes, expression_nodes, "contract expression nodes")?;
        nesting = nesting.max(expression_nesting);
    }
    for loop_contract in &contract.loops {
        limit_max(
            &loop_contract.invariants,
            VIR_LOOP_INVARIANTS_MAX,
            "VIR_LIMIT_LOOP_INVARIANTS",
            "loop invariants",
        )?;
        limit_max(
            &loop_contract.decreases,
            VIR_LOOP_DECREASES_MAX,
            "VIR_LIMIT_LOOP_DECREASES",
            "loop decreases",
        )?;
    }
    Ok((nodes, nesting))
}

fn expression_metrics(expression: &VirContractExpr) -> Result<(usize, usize), VirValidationError> {
    let children: Vec<&VirContractExpr> = match expression {
        VirContractExpr::Unary(value) => vec![&value.value],
        VirContractExpr::Nary(value) => value.args.iter().collect(),
        VirContractExpr::Binary(value) => vec![&value.lhs, &value.rhs],
        VirContractExpr::Convert(value) => vec![&value.value],
        VirContractExpr::Variable(_)
        | VirContractExpr::Result(_)
        | VirContractExpr::Boolean(_)
        | VirContractExpr::Integer(_) => Vec::new(),
    };
    let mut nodes = 1_usize;
    let mut depth = 1_usize;
    for child in children {
        let (child_nodes, child_depth) = expression_metrics(child)?;
        nodes = checked_add(nodes, child_nodes, "contract expression nodes")?;
        depth = depth.max(child_depth + 1);
    }
    Ok((nodes, depth))
}

#[derive(Default)]
struct FunctionAnalysis {
    callees: BTreeSet<String>,
    safety_checks: Vec<(Vec<VirSafetyCheck>, Vec<VirSafetyCheck>)>,
}

struct FunctionContext<'a> {
    module: &'a VirModule,
    unit: &'a VirUnit,
    structs: &'a BTreeMap<&'a str, &'a VirStructDecl>,
    struct_depths: &'a BTreeMap<String, usize>,
    constants: &'a BTreeMap<&'a str, &'a VirConstDecl>,
    functions: &'a BTreeMap<&'a str, (&'a VirUnit, &'a VirFunction)>,
    arguments: BTreeMap<&'a str, &'a VirType>,
    locals: BTreeMap<&'a str, &'a VirType>,
    block_parameters: Vec<BTreeMap<&'a str, &'a VirType>>,
    block_parameter_types: Vec<Vec<&'a VirType>>,
    block_indices: BTreeMap<&'a str, usize>,
}

fn validate_function(
    module: &VirModule,
    unit_index: usize,
    unit: &VirUnit,
    function: &VirFunction,
    index: &ModuleIndex<'_>,
) -> Result<FunctionAnalysis, VirValidationError> {
    if function.unit_id != unit.id {
        return Err(invalid("VIR_FUNCTION_UNIT", "function unit_id mismatch"));
    }
    validate_dense_bindings(&function.params, "arg")?;
    validate_dense_bindings(&function.results, "result")?;
    validate_dense_bindings(&function.locals, "local")?;
    if module.semantic_profile == SemanticProfile::RustCheckedV0 && function.results.len() != 1 {
        return Err(invalid(
            "VIR_RUST_RESULT_COUNT",
            "Rust v0 functions have exactly one result",
        ));
    }

    let structs = &index.structs[unit_index];
    let struct_depths = &index.struct_depths[unit_index];
    for binding in function
        .params
        .iter()
        .chain(&function.results)
        .chain(&function.locals)
    {
        validate_type(&binding.r#type, structs, struct_depths, 0)?;
    }

    let mut block_indices = BTreeMap::new();
    let mut block_parameters = Vec::with_capacity(function.blocks.len());
    let mut block_parameter_types = Vec::with_capacity(function.blocks.len());
    let mut next_parameter = 0_usize;
    let mut next_instruction = 0_usize;
    for (block_index, block) in function.blocks.iter().enumerate() {
        let expected_label = format!("bb{block_index}");
        if block.label != expected_label
            || block_indices
                .insert(block.label.as_str(), block_index)
                .is_some()
        {
            return Err(invalid(
                "VIR_BLOCK_ID",
                "block labels must be dense bbN ids",
            ));
        }
        if block_index == 0 && !block.parameters.is_empty() {
            return Err(invalid(
                "VIR_ENTRY_PARAMETERS",
                "entry block has parameters",
            ));
        }
        let mut parameters = BTreeMap::new();
        let mut parameter_types = Vec::with_capacity(block.parameters.len());
        for parameter in &block.parameters {
            let expected = format!("p{next_parameter}");
            if parameter.id != expected
                || parameters
                    .insert(parameter.id.as_str(), &parameter.r#type)
                    .is_some()
            {
                return Err(invalid(
                    "VIR_BLOCK_PARAMETER_ID",
                    "block parameter IDs must be function-wide dense pN values",
                ));
            }
            next_parameter += 1;
            validate_type(&parameter.r#type, structs, struct_depths, 0)?;
            parameter_types.push(&parameter.r#type);
        }
        block_parameters.push(parameters);
        block_parameter_types.push(parameter_types);
        for instruction in &block.instructions {
            let id = instruction_id(instruction);
            if id != format!("t{next_instruction}") {
                return Err(invalid(
                    "VIR_INSTRUCTION_ID",
                    "instruction IDs must be function-wide dense tN values",
                ));
            }
            next_instruction += 1;
        }
    }

    validate_canonical_bfs(function, &block_indices)?;
    let cfg = cfg_analysis(function, &block_indices)?;
    let cyclic = cfg.cyclic;
    if cyclic && module.semantic_profile == SemanticProfile::RustCheckedV0 {
        return Err(invalid("VIR_RUST_CYCLIC_CFG", "Rust CFG is cyclic"));
    }

    let arguments = function
        .params
        .iter()
        .map(|binding| (binding.id.as_str(), &binding.r#type))
        .collect();
    let locals = function
        .locals
        .iter()
        .map(|binding| (binding.id.as_str(), &binding.r#type))
        .collect();
    let context = FunctionContext {
        module,
        unit,
        structs,
        struct_depths,
        constants: &index.constants[unit_index],
        functions: &index.functions,
        arguments,
        locals,
        block_parameters,
        block_parameter_types,
        block_indices,
    };

    let initialized = initialized_locals(function, &cfg.predecessors, &context.locals);
    let mut analysis = FunctionAnalysis {
        callees: BTreeSet::new(),
        safety_checks: Vec::new(),
    };
    for (block_index, block) in function.blocks.iter().enumerate() {
        validate_block(
            function,
            block_index,
            block,
            &context,
            initialized[block_index].clone(),
            &mut analysis,
        )?;
    }
    validate_contract(function, &context, cyclic, &cfg)?;
    validate_features(function, cyclic)?;
    Ok(analysis)
}

fn validate_dense_bindings(
    bindings: &[crate::vir::VirBinding],
    prefix: &str,
) -> Result<(), VirValidationError> {
    for (index, binding) in bindings.iter().enumerate() {
        validate_identifier_bytes(&binding.id)?;
        if binding.id != format!("{prefix}{index}") {
            return Err(invalid(
                "VIR_BINDING_ID",
                format!("{prefix} binding IDs are not dense"),
            ));
        }
    }
    Ok(())
}

fn instruction_id(instruction: &VirInstruction) -> &str {
    match instruction {
        VirInstruction::Const { id, .. }
        | VirInstruction::Copy { id, .. }
        | VirInstruction::BinOp { id, .. }
        | VirInstruction::UnaryOp { id, .. }
        | VirInstruction::Convert { id, .. }
        | VirInstruction::Field { id, .. }
        | VirInstruction::Index { id, .. }
        | VirInstruction::MakeStruct { id, .. }
        | VirInstruction::MakeArray { id, .. }
        | VirInstruction::CallStatic { id, .. } => id,
    }
}

struct CfgAnalysis {
    edges: Vec<Vec<usize>>,
    predecessors: Vec<Vec<usize>>,
    cyclic: bool,
}

fn cfg_analysis(
    function: &VirFunction,
    indices: &BTreeMap<&str, usize>,
) -> Result<CfgAnalysis, VirValidationError> {
    let mut edges = vec![Vec::new(); function.blocks.len()];
    let mut predecessors = vec![Vec::new(); function.blocks.len()];
    let mut edge_count = 0_usize;
    for (source, block) in function.blocks.iter().enumerate() {
        for label in successor_labels(&block.terminator) {
            let Some(&target) = indices.get(label) else {
                return Err(invalid(
                    "VIR_UNKNOWN_BLOCK",
                    "terminator targets unknown block",
                ));
            };
            edges[source].push(target);
            predecessors[target].push(source);
            edge_count += 1;
        }
    }
    if edge_count > VIR_CFG_EDGES_PER_FUNCTION_MAX {
        return Err(limit(
            "VIR_LIMIT_CFG_EDGES_PER_FUNCTION",
            "CFG edge count exceeds 16,000",
        ));
    }
    let cyclic = !is_acyclic(&edges, &BTreeSet::new());
    Ok(CfgAnalysis {
        edges,
        predecessors,
        cyclic,
    })
}

fn successor_labels(terminator: &VirTerminator) -> Vec<&str> {
    match terminator {
        VirTerminator::Return { .. } => Vec::new(),
        VirTerminator::Jump { label, .. } => vec![label],
        VirTerminator::Branch {
            else_label,
            then_label,
            ..
        } => vec![else_label, then_label],
    }
}

fn validate_canonical_bfs(
    function: &VirFunction,
    indices: &BTreeMap<&str, usize>,
) -> Result<(), VirValidationError> {
    let mut discovered = vec![false; function.blocks.len()];
    let mut queue = VecDeque::new();
    discovered[0] = true;
    queue.push_back(0_usize);
    let mut traversal = Vec::new();
    while let Some(block_index) = queue.pop_front() {
        traversal.push(block_index);
        for label in successor_labels(&function.blocks[block_index].terminator) {
            let Some(&target) = indices.get(label) else {
                return Err(invalid(
                    "VIR_UNKNOWN_BLOCK",
                    "terminator targets unknown block",
                ));
            };
            if !discovered[target] {
                discovered[target] = true;
                queue.push_back(target);
            }
        }
    }
    if traversal != (0..function.blocks.len()).collect::<Vec<_>>() {
        return Err(invalid(
            "VIR_BLOCK_ORDER",
            "blocks are unreachable or not in canonical breadth-first order",
        ));
    }
    Ok(())
}

fn is_acyclic(edges: &[Vec<usize>], ignored_edges: &BTreeSet<(usize, usize)>) -> bool {
    let mut indegree = vec![0_usize; edges.len()];
    for (source, targets) in edges.iter().enumerate() {
        for &target in targets {
            if !ignored_edges.contains(&(source, target)) {
                indegree[target] += 1;
            }
        }
    }
    let mut ready: VecDeque<_> = indegree
        .iter()
        .enumerate()
        .filter_map(|(index, degree)| (*degree == 0).then_some(index))
        .collect();
    let mut visited = 0_usize;
    while let Some(source) = ready.pop_front() {
        visited += 1;
        for &target in &edges[source] {
            if ignored_edges.contains(&(source, target)) {
                continue;
            }
            indegree[target] -= 1;
            if indegree[target] == 0 {
                ready.push_back(target);
            }
        }
    }
    visited == edges.len()
}

fn initialized_locals(
    function: &VirFunction,
    predecessors: &[Vec<usize>],
    locals: &BTreeMap<&str, &VirType>,
) -> Vec<BTreeSet<String>> {
    let all: BTreeSet<_> = locals.keys().map(|id| (*id).to_owned()).collect();
    let mut input = vec![all.clone(); function.blocks.len()];
    input[0].clear();
    loop {
        let output: Vec<_> = function
            .blocks
            .iter()
            .enumerate()
            .map(|(index, block)| {
                let mut initialized = input[index].clone();
                for instruction in &block.instructions {
                    if let VirInstruction::Copy { target, .. } = instruction {
                        initialized.insert(target.clone());
                    }
                }
                initialized
            })
            .collect();
        let mut changed = false;
        for index in 1..function.blocks.len() {
            let mut next = all.clone();
            for predecessor in &predecessors[index] {
                next = next.intersection(&output[*predecessor]).cloned().collect();
            }
            if next != input[index] {
                input[index] = next;
                changed = true;
            }
        }
        if !changed {
            return input;
        }
    }
}

fn validate_block(
    function: &VirFunction,
    block_index: usize,
    block: &VirBlock,
    context: &FunctionContext<'_>,
    mut initialized: BTreeSet<String>,
    analysis: &mut FunctionAnalysis,
) -> Result<(), VirValidationError> {
    let mut temporaries: BTreeMap<&str, &VirType> = BTreeMap::new();
    for instruction in &block.instructions {
        validate_type(
            instruction_type(instruction),
            context.structs,
            context.struct_depths,
            0,
        )?;
        validate_instruction(
            instruction,
            block_index,
            context,
            &initialized,
            &temporaries,
            analysis,
        )?;
        if let VirInstruction::Copy { target, .. } = instruction {
            initialized.insert(target.clone());
        }
        temporaries.insert(instruction_id(instruction), instruction_type(instruction));
    }
    validate_terminator(
        function,
        block_index,
        &block.terminator,
        context,
        &initialized,
        &temporaries,
    )
}

fn instruction_type(instruction: &VirInstruction) -> &VirType {
    match instruction {
        VirInstruction::Const { r#type, .. }
        | VirInstruction::Copy { r#type, .. }
        | VirInstruction::BinOp { r#type, .. }
        | VirInstruction::UnaryOp { r#type, .. }
        | VirInstruction::Convert { r#type, .. }
        | VirInstruction::Field { r#type, .. }
        | VirInstruction::Index { r#type, .. }
        | VirInstruction::MakeStruct { r#type, .. }
        | VirInstruction::MakeArray { r#type, .. }
        | VirInstruction::CallStatic { r#type, .. } => r#type,
    }
}

fn resolve_value_type(
    value: &VirValue,
    block_index: usize,
    context: &FunctionContext<'_>,
    initialized: &BTreeSet<String>,
    temporaries: &BTreeMap<&str, &VirType>,
) -> Result<VirType, VirValidationError> {
    if let Some(literal) = value_literal_type(value)? {
        return Ok(literal);
    }
    match value {
        VirValue::Variable(reference) => {
            if let Some(r#type) = context.arguments.get(reference.var.as_str()) {
                return Ok((*r#type).clone());
            }
            if let Some(r#type) = context.locals.get(reference.var.as_str()) {
                if initialized.contains(&reference.var) {
                    return Ok((*r#type).clone());
                }
                return Err(invalid(
                    "VIR_UNINITIALIZED_LOCAL",
                    format!("local {:?} is not definitely initialized", reference.var),
                ));
            }
            if let Some(r#type) = context.block_parameters[block_index].get(reference.var.as_str())
            {
                return Ok((*r#type).clone());
            }
            if let Some(r#type) = temporaries.get(reference.var.as_str()) {
                return Ok((*r#type).clone());
            }
            Err(invalid(
                "VIR_UNKNOWN_VALUE",
                format!("unknown or out-of-block value {:?}", reference.var),
            ))
        }
        VirValue::Constant(reference) => context
            .constants
            .get(reference.constant.as_str())
            .map(|declaration| declaration.r#type.clone())
            .ok_or_else(|| invalid("VIR_UNKNOWN_CONST", "unknown constant reference")),
        VirValue::Boolean(_) | VirValue::Integer(_) => unreachable!(),
    }
}

fn validate_instruction(
    instruction: &VirInstruction,
    block_index: usize,
    context: &FunctionContext<'_>,
    initialized: &BTreeSet<String>,
    temporaries: &BTreeMap<&str, &VirType>,
    analysis: &mut FunctionAnalysis,
) -> Result<(), VirValidationError> {
    let resolve = |value: &VirValue| {
        resolve_value_type(value, block_index, context, initialized, temporaries)
    };
    let expected_checks = match instruction {
        VirInstruction::Const { r#type, value, .. } => {
            if literal_type(value)? != *r#type {
                return Err(invalid(
                    "VIR_INSTRUCTION_TYPE",
                    "Const literal type mismatch",
                ));
            }
            Vec::new()
        }
        VirInstruction::Copy {
            r#type,
            target,
            value,
            ..
        } => {
            let Some(target_type) = context.locals.get(target.as_str()) else {
                return Err(invalid("VIR_COPY_TARGET", "Copy target is not a local"));
            };
            if **target_type != *r#type || resolve(value)? != *r#type {
                return Err(invalid("VIR_INSTRUCTION_TYPE", "Copy type mismatch"));
            }
            Vec::new()
        }
        VirInstruction::BinOp {
            op,
            r#type,
            lhs,
            rhs,
            ..
        } => {
            let lhs_type = resolve(lhs)?;
            let rhs_type = resolve(rhs)?;
            validate_binary_operation(
                *op,
                &lhs_type,
                &rhs_type,
                r#type,
                context.module.semantic_profile,
            )?;
            required_safety_checks(
                context.module.semantic_profile,
                VirSafetyOperation::Binary(*op),
                &[lhs_type, rhs_type],
            )
            .map_err(safety_check_error)?
        }
        VirInstruction::UnaryOp {
            op, r#type, value, ..
        } => {
            let value_type = resolve(value)?;
            validate_unary_operation(*op, &value_type, r#type, context.module.semantic_profile)?;
            required_safety_checks(
                context.module.semantic_profile,
                VirSafetyOperation::Unary(*op),
                &[value_type],
            )
            .map_err(safety_check_error)?
        }
        VirInstruction::Convert { r#type, value, .. } => {
            if context.module.semantic_profile != SemanticProfile::GoFixedV0
                || !matches!(r#type, VirType::Bv { .. })
                || !matches!(resolve(value)?, VirType::Bv { .. })
            {
                return Err(invalid(
                    "VIR_PROFILE_OPERATION",
                    "invalid Convert operation",
                ));
            }
            Vec::new()
        }
        VirInstruction::Field {
            r#type,
            base,
            field,
            ..
        } => {
            let VirType::Struct { id } = resolve(base)? else {
                return Err(invalid(
                    "VIR_INSTRUCTION_TYPE",
                    "Field base is not a struct",
                ));
            };
            let declaration = context
                .structs
                .get(id.as_str())
                .ok_or_else(|| invalid("VIR_UNKNOWN_TYPE", "unknown Field struct type"))?;
            let field_type = declaration
                .fields
                .iter()
                .find(|candidate| candidate.name == *field)
                .map(|candidate| &candidate.r#type)
                .ok_or_else(|| invalid("VIR_UNKNOWN_FIELD", "unknown struct field"))?;
            if field_type != r#type {
                return Err(invalid(
                    "VIR_INSTRUCTION_TYPE",
                    "Field result type mismatch",
                ));
            }
            Vec::new()
        }
        VirInstruction::Index {
            r#type,
            base,
            index,
            ..
        } => {
            let base_type = resolve(base)?;
            let VirType::Array { element, .. } = &base_type else {
                return Err(invalid(
                    "VIR_INSTRUCTION_TYPE",
                    "Index base is not an array",
                ));
            };
            let index_type = resolve(index)?;
            let VirType::Bv { width, signed } = &index_type else {
                return Err(invalid("VIR_INSTRUCTION_TYPE", "Index is not a bitvector"));
            };
            if element.as_ref() != r#type {
                return Err(invalid(
                    "VIR_INSTRUCTION_TYPE",
                    "Index result type mismatch",
                ));
            }
            if context.module.semantic_profile == SemanticProfile::RustCheckedV0
                && (*signed
                    || width.bits() != context.module.semantic_parameters.pointer_width().bits())
            {
                return Err(invalid(
                    "VIR_RUST_INDEX_TYPE",
                    "Rust Index requires unsigned pointer-width index",
                ));
            }
            required_safety_checks(
                context.module.semantic_profile,
                VirSafetyOperation::Index,
                &[base_type, index_type],
            )
            .map_err(safety_check_error)?
        }
        VirInstruction::MakeStruct { r#type, fields, .. } => {
            let VirType::Struct { id } = r#type else {
                return Err(invalid(
                    "VIR_INSTRUCTION_TYPE",
                    "MakeStruct result is not struct",
                ));
            };
            let declaration = context
                .structs
                .get(id.as_str())
                .ok_or_else(|| invalid("VIR_UNKNOWN_TYPE", "unknown MakeStruct type"))?;
            if fields.len() != declaration.fields.len() {
                return Err(invalid(
                    "VIR_STRUCT_FIELDS",
                    "MakeStruct field count mismatch",
                ));
            }
            for (field, expected) in fields.iter().zip(&declaration.fields) {
                if field.name != expected.name || resolve(&field.value)? != expected.r#type {
                    return Err(invalid("VIR_STRUCT_FIELDS", "MakeStruct field mismatch"));
                }
            }
            Vec::new()
        }
        VirInstruction::MakeArray {
            r#type, elements, ..
        } => {
            let VirType::Array { length, element } = r#type else {
                return Err(invalid(
                    "VIR_INSTRUCTION_TYPE",
                    "MakeArray result is not array",
                ));
            };
            if elements.len() != usize::from(length.get()) {
                return Err(invalid("VIR_ARRAY_ELEMENTS", "MakeArray element mismatch"));
            }
            for value in elements {
                if resolve(value)? != **element {
                    return Err(invalid("VIR_ARRAY_ELEMENTS", "MakeArray element mismatch"));
                }
            }
            Vec::new()
        }
        VirInstruction::CallStatic {
            r#type,
            function,
            args,
            ..
        } => {
            let Some((_, callee)) = context.functions.get(function.as_str()) else {
                return Err(invalid("VIR_UNKNOWN_CALLEE", "unknown CallStatic function"));
            };
            if callee.results.len() != 1
                || callee.results[0].r#type != *r#type
                || callee.params.len() != args.len()
            {
                return Err(invalid(
                    "VIR_CALL_SIGNATURE",
                    "CallStatic signature mismatch",
                ));
            }
            for (argument, parameter) in args.iter().zip(&callee.params) {
                if resolve(argument)? != parameter.r#type {
                    return Err(invalid(
                        "VIR_CALL_SIGNATURE",
                        "CallStatic signature mismatch",
                    ));
                }
            }
            analysis.callees.insert(function.clone());
            Vec::new()
        }
    };
    analysis
        .safety_checks
        .push((instruction_checks(instruction).to_vec(), expected_checks));
    Ok(())
}

fn instruction_checks(instruction: &VirInstruction) -> &[VirSafetyCheck] {
    match instruction {
        VirInstruction::Const { safety_checks, .. }
        | VirInstruction::Copy { safety_checks, .. }
        | VirInstruction::BinOp { safety_checks, .. }
        | VirInstruction::UnaryOp { safety_checks, .. }
        | VirInstruction::Convert { safety_checks, .. }
        | VirInstruction::Field { safety_checks, .. }
        | VirInstruction::Index { safety_checks, .. }
        | VirInstruction::MakeStruct { safety_checks, .. }
        | VirInstruction::MakeArray { safety_checks, .. }
        | VirInstruction::CallStatic { safety_checks, .. } => safety_checks,
    }
}

fn validate_binary_operation(
    op: VirBinaryOperator,
    lhs: &VirType,
    rhs: &VirType,
    result: &VirType,
    profile: SemanticProfile,
) -> Result<(), VirValidationError> {
    use VirBinaryOperator as Op;
    let bool_type = VirType::Bool {};
    match op {
        Op::Eq | Op::NotEq => {
            if lhs != rhs || result != &bool_type {
                return Err(invalid("VIR_INSTRUCTION_TYPE", "equality type mismatch"));
            }
            if profile == SemanticProfile::RustCheckedV0
                && !matches!(lhs, VirType::Bool {} | VirType::Bv { .. })
            {
                return Err(invalid(
                    "VIR_PROFILE_OPERATION",
                    "Rust program equality accepts only bool and bitvector values",
                ));
            }
        }
        Op::BvShl | Op::BvAshr | Op::BvLshr => {
            let VirType::Bv {
                signed: lhs_signed, ..
            } = lhs
            else {
                return Err(invalid(
                    "VIR_INSTRUCTION_TYPE",
                    "shift LHS is not bitvector",
                ));
            };
            if !matches!(rhs, VirType::Bv { .. }) || result != lhs {
                return Err(invalid("VIR_INSTRUCTION_TYPE", "shift type mismatch"));
            }
            if (op == Op::BvAshr && !lhs_signed) || (op == Op::BvLshr && *lhs_signed) {
                return Err(invalid("VIR_INSTRUCTION_TYPE", "shift signedness mismatch"));
            }
        }
        Op::SignedLt | Op::SignedLe | Op::SignedGt | Op::SignedGe => {
            if lhs != rhs
                || result != &bool_type
                || !matches!(lhs, VirType::Bv { signed: true, .. })
            {
                return Err(invalid(
                    "VIR_INSTRUCTION_TYPE",
                    "signed comparison mismatch",
                ));
            }
        }
        Op::UnsignedLt | Op::UnsignedLe | Op::UnsignedGt | Op::UnsignedGe => {
            if lhs != rhs
                || result != &bool_type
                || !matches!(lhs, VirType::Bv { signed: false, .. })
            {
                return Err(invalid(
                    "VIR_INSTRUCTION_TYPE",
                    "unsigned comparison mismatch",
                ));
            }
        }
        Op::BvSdiv | Op::BvSrem => {
            require_matching_bv(lhs, rhs, result, Some(true))?;
        }
        Op::BvUdiv | Op::BvUrem => {
            require_matching_bv(lhs, rhs, result, Some(false))?;
        }
        Op::BvAdd | Op::BvSub | Op::BvMul | Op::BvAnd | Op::BvOr | Op::BvXor => {
            require_matching_bv(lhs, rhs, result, None)?
        }
    }
    Ok(())
}

fn require_matching_bv(
    lhs: &VirType,
    rhs: &VirType,
    result: &VirType,
    signed: Option<bool>,
) -> Result<(), VirValidationError> {
    if lhs != rhs || lhs != result || !matches!(lhs, VirType::Bv { .. }) {
        return Err(invalid(
            "VIR_INSTRUCTION_TYPE",
            "bitvector operand/result types do not match",
        ));
    }
    if let (Some(expected), VirType::Bv { signed, .. }) = (signed, lhs) {
        if *signed != expected {
            return Err(invalid(
                "VIR_INSTRUCTION_TYPE",
                "bitvector operation signedness mismatch",
            ));
        }
    }
    Ok(())
}

fn validate_unary_operation(
    op: crate::vir::VirUnaryOperator,
    operand: &VirType,
    result: &VirType,
    profile: SemanticProfile,
) -> Result<(), VirValidationError> {
    use crate::vir::VirUnaryOperator as Op;
    match op {
        Op::Not if matches!(operand, VirType::Bool {}) && operand == result => Ok(()),
        Op::BvNot if matches!(operand, VirType::Bv { .. }) && operand == result => Ok(()),
        Op::BvNeg if matches!(operand, VirType::Bv { .. }) && operand == result => {
            if profile == SemanticProfile::RustCheckedV0
                && matches!(operand, VirType::Bv { signed: false, .. })
            {
                Err(invalid(
                    "VIR_PROFILE_OPERATION",
                    "Rust unsigned negation is not accepted",
                ))
            } else {
                Ok(())
            }
        }
        _ => Err(invalid(
            "VIR_INSTRUCTION_TYPE",
            "unary operation type mismatch",
        )),
    }
}

pub fn validate_vir_safety_fragment(
    profile: SemanticProfile,
    operation: VirSafetyOperation,
    operand_types: &[VirType],
    actual: &[VirSafetyCheck],
) -> Result<(), VirValidationError> {
    let expected =
        required_safety_checks(profile, operation, operand_types).map_err(safety_check_error)?;
    validate_safety_checks(actual, &expected)
}

pub fn validate_safety_checks(
    actual: &[VirSafetyCheck],
    expected: &[VirSafetyCheck],
) -> Result<(), VirValidationError> {
    validate_safety_check_sequence(actual, expected).map_err(safety_check_error)
}

pub fn validate_vir_limit_count(limit_id: &str, count: u64) -> Result<(), VirValidationError> {
    let (maximum, code) = match limit_id {
        "canonical_json_bytes" => (
            VIR_CANONICAL_JSON_BYTES_MAX,
            "VIR_LIMIT_CANONICAL_JSON_BYTES",
        ),
        "input_json_bytes" => (
            crate::vir::VIR_INPUT_JSON_BYTES_MAX,
            "VIR_LIMIT_INPUT_JSON_BYTES",
        ),
        "json_nesting" => (crate::vir::VIR_JSON_NESTING_MAX, "VIR_LIMIT_JSON_NESTING"),
        "string_bytes" => (crate::vir::VIR_STRING_BYTES_MAX, "VIR_LIMIT_STRING_BYTES"),
        "identifier_bytes" => (
            VIR_IDENTIFIER_BYTES_MAX as u64,
            "VIR_LIMIT_IDENTIFIER_BYTES",
        ),
        "units" => (VIR_UNITS_MAX as u64, "VIR_LIMIT_UNITS"),
        "type_decls" => (VIR_TYPE_DECLS_MAX as u64, "VIR_LIMIT_TYPE_DECLS"),
        "const_decls" => (VIR_CONST_DECLS_MAX as u64, "VIR_LIMIT_CONST_DECLS"),
        "functions" => (VIR_FUNCTIONS_MAX as u64, "VIR_LIMIT_FUNCTIONS"),
        "params" => (VIR_PARAMS_MAX as u64, "VIR_LIMIT_PARAMS"),
        "results" => (VIR_RESULTS_MAX as u64, "VIR_LIMIT_RESULTS"),
        "locals" => (VIR_LOCALS_MAX as u64, "VIR_LIMIT_LOCALS"),
        "blocks_per_function" => (
            VIR_BLOCKS_PER_FUNCTION_MAX as u64,
            "VIR_LIMIT_BLOCKS_PER_FUNCTION",
        ),
        "blocks_per_module" => (
            VIR_BLOCKS_PER_MODULE_MAX as u64,
            "VIR_LIMIT_BLOCKS_PER_MODULE",
        ),
        "block_parameters" => (
            VIR_BLOCK_PARAMETERS_MAX as u64,
            "VIR_LIMIT_BLOCK_PARAMETERS",
        ),
        "instructions_per_block" => (
            VIR_INSTRUCTIONS_PER_BLOCK_MAX as u64,
            "VIR_LIMIT_INSTRUCTIONS_PER_BLOCK",
        ),
        "instructions_per_function" => (
            VIR_INSTRUCTIONS_PER_FUNCTION_MAX as u64,
            "VIR_LIMIT_INSTRUCTIONS_PER_FUNCTION",
        ),
        "instructions_per_module" => (
            VIR_INSTRUCTIONS_PER_MODULE_MAX as u64,
            "VIR_LIMIT_INSTRUCTIONS_PER_MODULE",
        ),
        "cfg_edges_per_function" => (
            VIR_CFG_EDGES_PER_FUNCTION_MAX as u64,
            "VIR_LIMIT_CFG_EDGES_PER_FUNCTION",
        ),
        "call_args" => (VIR_CALL_ARGS_MAX as u64, "VIR_LIMIT_CALL_ARGS"),
        "array_elements" => (VIR_ARRAY_ELEMENTS_MAX as u64, "VIR_LIMIT_ARRAY_ELEMENTS"),
        "struct_fields" => (VIR_STRUCT_FIELDS_MAX as u64, "VIR_LIMIT_STRUCT_FIELDS"),
        "aggregate_type_nesting" => (
            VIR_AGGREGATE_TYPE_NESTING_MAX as u64,
            "VIR_LIMIT_AGGREGATE_TYPE_NESTING",
        ),
        "contract_clauses" => (
            VIR_CONTRACT_CLAUSES_MAX as u64,
            "VIR_LIMIT_CONTRACT_CLAUSES",
        ),
        "contract_expr_nodes_per_function" => (
            VIR_CONTRACT_EXPR_NODES_PER_FUNCTION_MAX as u64,
            "VIR_LIMIT_CONTRACT_EXPR_NODES_PER_FUNCTION",
        ),
        "contract_expr_nodes_per_module" => (
            VIR_CONTRACT_EXPR_NODES_PER_MODULE_MAX as u64,
            "VIR_LIMIT_CONTRACT_EXPR_NODES_PER_MODULE",
        ),
        "contract_expr_nesting" => (
            VIR_CONTRACT_EXPR_NESTING_MAX as u64,
            "VIR_LIMIT_CONTRACT_EXPR_NESTING",
        ),
        "loops" => (VIR_LOOPS_MAX as u64, "VIR_LIMIT_LOOPS"),
        "loop_invariants" => (VIR_LOOP_INVARIANTS_MAX as u64, "VIR_LIMIT_LOOP_INVARIANTS"),
        "loop_decreases" => (VIR_LOOP_DECREASES_MAX as u64, "VIR_LIMIT_LOOP_DECREASES"),
        _ => return Err(invalid("VIR_LIMIT_UNKNOWN", "unknown VIR limit identifier")),
    };
    if count > maximum {
        Err(limit(code, format!("count {count} exceeds {maximum}")))
    } else {
        Ok(())
    }
}

fn validate_terminator(
    function: &VirFunction,
    block_index: usize,
    terminator: &VirTerminator,
    context: &FunctionContext<'_>,
    initialized: &BTreeSet<String>,
    temporaries: &BTreeMap<&str, &VirType>,
) -> Result<(), VirValidationError> {
    let resolve = |value: &VirValue| {
        resolve_value_type(value, block_index, context, initialized, temporaries)
    };
    match terminator {
        VirTerminator::Return { values } => {
            if values.len() != function.results.len() {
                return Err(invalid("VIR_RETURN_TYPE", "Return value count mismatch"));
            }
            for (value, result) in values.iter().zip(&function.results) {
                if resolve(value)? != result.r#type {
                    return Err(invalid("VIR_RETURN_TYPE", "Return value type mismatch"));
                }
            }
        }
        VirTerminator::Jump { label, args } => {
            validate_successor_args(label, args, context, &resolve)?;
        }
        VirTerminator::Branch {
            cond,
            then_label,
            then_args,
            else_label,
            else_args,
        } => {
            if then_label == else_label {
                return Err(invalid(
                    "VIR_BRANCH_NONCANONICAL",
                    "Branch targets must differ",
                ));
            }
            if resolve(cond)? != (VirType::Bool {}) {
                return Err(invalid("VIR_BRANCH_TYPE", "Branch condition is not bool"));
            }
            validate_successor_args(then_label, then_args, context, &resolve)?;
            validate_successor_args(else_label, else_args, context, &resolve)?;
        }
    }
    Ok(())
}

fn validate_successor_args<F>(
    label: &str,
    args: &[VirValue],
    context: &FunctionContext<'_>,
    resolve: &F,
) -> Result<(), VirValidationError>
where
    F: Fn(&VirValue) -> Result<VirType, VirValidationError>,
{
    let Some(&target) = context.block_indices.get(label) else {
        return Err(invalid("VIR_UNKNOWN_BLOCK", "unknown successor label"));
    };
    let parameters = &context.block_parameter_types[target];
    if args.len() != parameters.len() {
        return Err(invalid(
            "VIR_SUCCESSOR_ARGUMENTS",
            "successor argument count mismatch",
        ));
    }
    for (argument, parameter_type) in args.iter().zip(parameters) {
        if resolve(argument)? != (**parameter_type).clone() {
            return Err(invalid(
                "VIR_SUCCESSOR_ARGUMENTS",
                "successor argument type mismatch",
            ));
        }
    }
    Ok(())
}

fn validate_contract(
    function: &VirFunction,
    context: &FunctionContext<'_>,
    cyclic: bool,
    cfg: &CfgAnalysis,
) -> Result<(), VirValidationError> {
    let contract = &function.contracts;
    if contract.unit_id != context.unit.id || contract.function_id != function.id {
        return Err(invalid(
            "VIR_CONTRACT_IDENTITY",
            "contract identity mismatch",
        ));
    }
    validate_semantic_parameters(contract.semantic_profile, &contract.semantic_parameters)
        .map_err(|error| invalid("VIR_PROFILE_MISMATCH", error.to_string()))?;
    if contract.semantic_profile != context.module.semantic_profile
        || contract.semantic_parameters != context.module.semantic_parameters
    {
        return Err(invalid(
            "VIR_PROFILE_MISMATCH",
            "contract semantic context differs from module",
        ));
    }
    if contract.ensures.is_empty() {
        return Err(invalid("VIR_EMPTY_ENSURES", "contract ensures is empty"));
    }
    if !contract.modifies.is_empty() {
        return Err(invalid(
            "VIR_CONTRACT_MODIFIES",
            "VIR v0 modifies must be empty",
        ));
    }

    let parameter_types: BTreeMap<_, _> = function
        .params
        .iter()
        .map(|binding| (binding.id.as_str(), &binding.r#type))
        .collect();
    for expression in &contract.requires {
        if validate_contract_expr(
            expression,
            context.module.semantic_profile,
            &parameter_types,
            &[],
        )? != (VirType::Bool {})
        {
            return Err(invalid(
                "VIR_CONTRACT_TYPE",
                "requires expression is not bool",
            ));
        }
    }
    for expression in &contract.ensures {
        if validate_contract_expr(
            expression,
            context.module.semantic_profile,
            &parameter_types,
            &function.results,
        )? != (VirType::Bool {})
        {
            return Err(invalid(
                "VIR_CONTRACT_TYPE",
                "ensures expression is not bool",
            ));
        }
    }

    if context.module.semantic_profile == SemanticProfile::RustCheckedV0 {
        if !contract.loops.is_empty() {
            return Err(invalid("VIR_RUST_LOOPS", "Rust contract contains loops"));
        }
        if contract.termination != crate::vir::VirTermination::Total {
            return Err(invalid(
                "VIR_LOOP_TERMINATION",
                "Rust termination is not total",
            ));
        }
    } else {
        validate_go_loops(function, contract, context, cyclic, cfg, &parameter_types)?;
    }

    Ok(())
}

fn validate_contract_expr(
    expression: &VirContractExpr,
    profile: SemanticProfile,
    variables: &BTreeMap<&str, &VirType>,
    results: &[crate::vir::VirBinding],
) -> Result<VirType, VirValidationError> {
    match expression {
        VirContractExpr::Variable(reference) => variables
            .get(reference.var.as_str())
            .map(|r#type| (*r#type).clone())
            .ok_or_else(|| invalid("VIR_CONTRACT_NAME", "unknown contract variable")),
        VirContractExpr::Result(reference) => results
            .get(reference.result as usize)
            .map(|binding| binding.r#type.clone())
            .ok_or_else(|| invalid("VIR_CONTRACT_NAME", "unknown contract result")),
        VirContractExpr::Boolean(_) => Ok(VirType::Bool {}),
        VirContractExpr::Integer(value) => {
            validate_integer_literal(&value.int)?;
            Ok(VirType::Bv {
                width: value.int.width,
                signed: value.int.signed,
            })
        }
        VirContractExpr::Unary(value) => {
            let operand = validate_contract_expr(&value.value, profile, variables, results)?;
            match value.op {
                crate::vir::VirContractUnaryOperator::Not if operand == (VirType::Bool {}) => {
                    Ok(VirType::Bool {})
                }
                crate::vir::VirContractUnaryOperator::BvNeg
                | crate::vir::VirContractUnaryOperator::BvNot
                    if matches!(operand, VirType::Bv { .. }) =>
                {
                    Ok(operand)
                }
                _ => Err(invalid("VIR_CONTRACT_TYPE", "contract unary type mismatch")),
            }
        }
        VirContractExpr::Nary(value) => {
            if !(2..=64).contains(&value.args.len()) {
                return Err(invalid(
                    "VIR_CONTRACT_TYPE",
                    "and/or arity is outside 2..64",
                ));
            }
            for argument in &value.args {
                if validate_contract_expr(argument, profile, variables, results)?
                    != (VirType::Bool {})
                {
                    return Err(invalid("VIR_CONTRACT_TYPE", "and/or argument is not bool"));
                }
            }
            Ok(VirType::Bool {})
        }
        VirContractExpr::Binary(value) => {
            if profile == SemanticProfile::RustCheckedV0
                && matches!(
                    value.op,
                    VirBinaryOperator::BvSdiv
                        | VirBinaryOperator::BvSrem
                        | VirBinaryOperator::BvUdiv
                        | VirBinaryOperator::BvUrem
                )
            {
                return Err(invalid(
                    "VIR_CONTRACT_OPERATOR",
                    "Rust contract division/remainder is not accepted",
                ));
            }
            let lhs = validate_contract_expr(&value.lhs, profile, variables, results)?;
            let rhs = validate_contract_expr(&value.rhs, profile, variables, results)?;
            contract_binary_result(value.op, &lhs, &rhs)
        }
        VirContractExpr::Convert(value) => {
            if profile != SemanticProfile::GoFixedV0 {
                return Err(invalid(
                    "VIR_CONTRACT_OPERATOR",
                    "Rust contract convert is not accepted",
                ));
            }
            let operand = validate_contract_expr(&value.value, profile, variables, results)?;
            if !matches!(operand, VirType::Bv { .. }) || !matches!(value.r#type, VirType::Bv { .. })
            {
                return Err(invalid(
                    "VIR_CONTRACT_TYPE",
                    "contract convert type mismatch",
                ));
            }
            Ok(value.r#type.clone())
        }
    }
}

fn contract_binary_result(
    op: VirBinaryOperator,
    lhs: &VirType,
    rhs: &VirType,
) -> Result<VirType, VirValidationError> {
    use VirBinaryOperator as Op;
    match op {
        Op::Eq | Op::NotEq if lhs == rhs => Ok(VirType::Bool {}),
        Op::SignedLt | Op::SignedLe | Op::SignedGt | Op::SignedGe
            if lhs == rhs && matches!(lhs, VirType::Bv { signed: true, .. }) =>
        {
            Ok(VirType::Bool {})
        }
        Op::UnsignedLt | Op::UnsignedLe | Op::UnsignedGt | Op::UnsignedGe
            if lhs == rhs && matches!(lhs, VirType::Bv { signed: false, .. }) =>
        {
            Ok(VirType::Bool {})
        }
        Op::BvShl | Op::BvAshr | Op::BvLshr
            if matches!(lhs, VirType::Bv { .. }) && matches!(rhs, VirType::Bv { .. }) =>
        {
            if (op == Op::BvAshr && !matches!(lhs, VirType::Bv { signed: true, .. }))
                || (op == Op::BvLshr && !matches!(lhs, VirType::Bv { signed: false, .. }))
            {
                Err(invalid(
                    "VIR_CONTRACT_TYPE",
                    "contract shift signedness mismatch",
                ))
            } else {
                Ok(lhs.clone())
            }
        }
        Op::BvSdiv | Op::BvSrem
            if lhs == rhs && matches!(lhs, VirType::Bv { signed: true, .. }) =>
        {
            Ok(lhs.clone())
        }
        Op::BvUdiv | Op::BvUrem
            if lhs == rhs && matches!(lhs, VirType::Bv { signed: false, .. }) =>
        {
            Ok(lhs.clone())
        }
        Op::BvAdd | Op::BvSub | Op::BvMul | Op::BvAnd | Op::BvOr | Op::BvXor
            if lhs == rhs && matches!(lhs, VirType::Bv { .. }) =>
        {
            Ok(lhs.clone())
        }
        _ => Err(invalid(
            "VIR_CONTRACT_TYPE",
            "contract binary type mismatch",
        )),
    }
}

fn validate_go_loops(
    function: &VirFunction,
    contract: &VirContract,
    context: &FunctionContext<'_>,
    cyclic: bool,
    cfg: &CfgAnalysis,
    parameters: &BTreeMap<&str, &VirType>,
) -> Result<(), VirValidationError> {
    if !cyclic {
        if !contract.loops.is_empty() || contract.termination != crate::vir::VirTermination::Total {
            return Err(invalid(
                "VIR_LOOP_TERMINATION",
                "acyclic Go function must be total with no loop contracts",
            ));
        }
        return Ok(());
    }

    let components = strongly_connected_components(&cfg.edges);
    let cyclic_components: Vec<_> = components
        .into_iter()
        .filter(|component| {
            component.len() > 1
                || cfg.edges[component[0]]
                    .iter()
                    .any(|target| *target == component[0])
        })
        .collect();
    if cyclic_components.len() != contract.loops.len() {
        return Err(invalid(
            "VIR_LOOP_CUTPOINT",
            "cyclic component and loop-contract counts differ",
        ));
    }

    let contract_by_header: BTreeMap<_, _> = contract
        .loops
        .iter()
        .map(|loop_contract| (loop_contract.header.as_str(), loop_contract))
        .collect();
    if contract_by_header.len() != contract.loops.len() {
        return Err(invalid(
            "VIR_LOOP_CUTPOINT",
            "duplicate loop header contract",
        ));
    }
    let mut expected_headers = Vec::new();
    let mut ignored_backedges = BTreeSet::new();
    for component in cyclic_components {
        let members: BTreeSet<_> = component.iter().copied().collect();
        let headers: Vec<_> = component
            .iter()
            .copied()
            .filter(|index| contract_by_header.contains_key(function.blocks[*index].label.as_str()))
            .collect();
        if headers.len() != 1 {
            return Err(invalid(
                "VIR_LOOP_CUTPOINT",
                "cyclic component must have exactly one contracted header",
            ));
        }
        let header = headers[0];
        expected_headers.push(header);
        let VirTerminator::Branch {
            then_label,
            else_label,
            ..
        } = &function.blocks[header].terminator
        else {
            return Err(invalid("VIR_LOOP_CUTPOINT", "loop header must branch"));
        };
        let then_index = context.block_indices[then_label.as_str()];
        let else_index = context.block_indices[else_label.as_str()];
        if !members.contains(&then_index) || members.contains(&else_index) {
            return Err(invalid(
                "VIR_LOOP_CUTPOINT",
                "loop header true edge must enter and false edge must leave",
            ));
        }
        for (source, targets) in cfg.edges.iter().enumerate() {
            for &target in targets {
                if !members.contains(&source) && members.contains(&target) && target != header {
                    return Err(invalid("VIR_LOOP_CUTPOINT", "loop has a non-header entry"));
                }
                if members.contains(&source)
                    && !members.contains(&target)
                    && (source != header || target != else_index)
                {
                    return Err(invalid("VIR_LOOP_CUTPOINT", "loop has an invalid exit"));
                }
                if members.contains(&source) && target == header && source != header {
                    if !matches!(
                        function.blocks[source].terminator,
                        VirTerminator::Jump { .. }
                    ) {
                        return Err(invalid(
                            "VIR_LOOP_CUTPOINT",
                            "loop backedge must be a Jump to its header",
                        ));
                    }
                    ignored_backedges.insert((source, target));
                }
            }
        }
    }
    expected_headers.sort_unstable();
    let actual_headers: Vec<_> = contract
        .loops
        .iter()
        .map(|loop_contract| context.block_indices[loop_contract.header.as_str()])
        .collect();
    if actual_headers != expected_headers || !is_acyclic(&cfg.edges, &ignored_backedges) {
        return Err(invalid(
            "VIR_LOOP_CUTPOINT",
            "loop contracts are not canonical or cut backedges do not make the CFG acyclic",
        ));
    }

    let total = contract.termination == crate::vir::VirTermination::Total;
    for loop_contract in &contract.loops {
        if loop_contract.invariants.is_empty() {
            return Err(invalid("VIR_LOOP_CONTRACT", "loop invariant list is empty"));
        }
        if total == loop_contract.decreases.is_empty() {
            return Err(invalid(
                "VIR_LOOP_TERMINATION",
                "loop decreases do not match termination mode",
            ));
        }
        let header = context.block_indices[loop_contract.header.as_str()];
        let mut visible = parameters.clone();
        visible.extend(context.locals.iter().map(|(id, r#type)| (*id, *r#type)));
        visible.extend(
            context.block_parameters[header]
                .iter()
                .map(|(id, r#type)| (*id, *r#type)),
        );
        for invariant in &loop_contract.invariants {
            if validate_contract_expr(invariant, context.module.semantic_profile, &visible, &[])?
                != (VirType::Bool {})
            {
                return Err(invalid("VIR_CONTRACT_TYPE", "loop invariant is not bool"));
            }
        }
        for decreases in &loop_contract.decreases {
            if !matches!(
                validate_contract_expr(decreases, context.module.semantic_profile, &visible, &[],)?,
                VirType::Bv { .. }
            ) {
                return Err(invalid("VIR_CONTRACT_TYPE", "loop decreases is not BV"));
            }
        }
    }
    if !total
        && contract
            .loops
            .iter()
            .any(|loop_contract| !loop_contract.decreases.is_empty())
    {
        return Err(invalid(
            "VIR_LOOP_TERMINATION",
            "partial function has a nonempty decreases list",
        ));
    }
    Ok(())
}

fn strongly_connected_components(edges: &[Vec<usize>]) -> Vec<Vec<usize>> {
    let mut reverse = vec![Vec::new(); edges.len()];
    for (source, targets) in edges.iter().enumerate() {
        for &target in targets {
            reverse[target].push(source);
        }
    }
    let mut seen = vec![false; edges.len()];
    let mut order = Vec::with_capacity(edges.len());
    for source in 0..edges.len() {
        if seen[source] {
            continue;
        }
        seen[source] = true;
        let mut stack = vec![(source, 0_usize)];
        while let Some((node, next_edge)) = stack.last_mut() {
            if let Some(&target) = edges[*node].get(*next_edge) {
                *next_edge += 1;
                if !seen[target] {
                    seen[target] = true;
                    stack.push((target, 0));
                }
            } else {
                order.push(*node);
                stack.pop();
            }
        }
    }
    seen.fill(false);
    let mut components = Vec::new();
    while let Some(source) = order.pop() {
        if !seen[source] {
            let mut component = Vec::new();
            seen[source] = true;
            let mut stack = vec![source];
            while let Some(node) = stack.pop() {
                component.push(node);
                for &target in reverse[node].iter().rev() {
                    if !seen[target] {
                        seen[target] = true;
                        stack.push(target);
                    }
                }
            }
            components.push(component);
        }
    }
    components
}

fn validate_features(function: &VirFunction, cyclic: bool) -> Result<(), VirValidationError> {
    let mut expected = BTreeSet::new();
    for binding in function
        .params
        .iter()
        .chain(&function.results)
        .chain(&function.locals)
        .chain(function.blocks.iter().flat_map(|block| &block.parameters))
    {
        collect_type_features(&binding.r#type, &mut expected);
    }
    if !function.locals.is_empty() {
        expected.insert(VirFeature::MutableLocal);
    }
    if cyclic {
        expected.insert(VirFeature::CyclicCfg);
    }
    for block in &function.blocks {
        if matches!(block.terminator, VirTerminator::Branch { .. }) {
            expected.insert(VirFeature::Branch);
        }
        for value in terminator_values(&block.terminator) {
            collect_value_features(value, &mut expected);
        }
        for instruction in &block.instructions {
            collect_type_features(instruction_type(instruction), &mut expected);
            for value in instruction_values(instruction) {
                collect_value_features(value, &mut expected);
            }
            match instruction {
                VirInstruction::Copy { .. } => {
                    expected.insert(VirFeature::MutableLocal);
                }
                VirInstruction::Convert { .. } => {
                    expected.insert(VirFeature::Conversion);
                }
                VirInstruction::MakeArray { .. } | VirInstruction::Index { .. } => {
                    expected.insert(VirFeature::Array);
                }
                VirInstruction::MakeStruct { .. } | VirInstruction::Field { .. } => {
                    expected.insert(VirFeature::Struct);
                }
                VirInstruction::CallStatic { .. } => {
                    expected.insert(VirFeature::CallStatic);
                }
                _ => {}
            }
        }
    }
    let actual: BTreeSet<_> = function.features_used.iter().copied().collect();
    if actual.len() != function.features_used.len()
        || function
            .features_used
            .windows(2)
            .any(|pair| feature_name(pair[0]) >= feature_name(pair[1]))
        || actual != expected
    {
        return Err(invalid(
            "VIR_FEATURES",
            "features_used is not the exact sorted derived set",
        ));
    }
    Ok(())
}

fn collect_type_features(r#type: &VirType, output: &mut BTreeSet<VirFeature>) {
    match r#type {
        VirType::Array { element, .. } => {
            output.insert(VirFeature::Array);
            collect_type_features(element, output);
        }
        VirType::Struct { .. } => {
            output.insert(VirFeature::Struct);
        }
        VirType::Bool {} | VirType::Bv { .. } => {}
    }
}

fn collect_value_features(value: &VirValue, output: &mut BTreeSet<VirFeature>) {
    if matches!(value, VirValue::Constant(_)) {
        output.insert(VirFeature::ConstantDecl);
    }
}

fn instruction_values(instruction: &VirInstruction) -> Vec<&VirValue> {
    match instruction {
        VirInstruction::Const { .. } => Vec::new(),
        VirInstruction::Copy { value, .. }
        | VirInstruction::UnaryOp { value, .. }
        | VirInstruction::Convert { value, .. } => vec![value],
        VirInstruction::BinOp { lhs, rhs, .. } => vec![lhs, rhs],
        VirInstruction::Field { base, .. } => vec![base],
        VirInstruction::Index { base, index, .. } => vec![base, index],
        VirInstruction::MakeStruct { fields, .. } => {
            fields.iter().map(|field| &field.value).collect()
        }
        VirInstruction::MakeArray { elements, .. } => elements.iter().collect(),
        VirInstruction::CallStatic { args, .. } => args.iter().collect(),
    }
}

fn terminator_values(terminator: &VirTerminator) -> Vec<&VirValue> {
    match terminator {
        VirTerminator::Return { values } | VirTerminator::Jump { args: values, .. } => {
            values.iter().collect()
        }
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

fn feature_name(feature: VirFeature) -> &'static str {
    match feature {
        VirFeature::Array => "array",
        VirFeature::Branch => "branch",
        VirFeature::CallStatic => "call_static",
        VirFeature::ConstantDecl => "constant_decl",
        VirFeature::Conversion => "conversion",
        VirFeature::CyclicCfg => "cyclic_cfg",
        VirFeature::MutableLocal => "mutable_local",
        VirFeature::Struct => "struct",
    }
}

fn validate_call_graph_and_order(
    module: &VirModule,
    analyses: &BTreeMap<String, FunctionAnalysis>,
) -> Result<(), VirValidationError> {
    let mut callers_by_callee: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
    let mut remaining_callees: BTreeMap<&str, usize> = BTreeMap::new();
    for (function, analysis) in analyses {
        remaining_callees.insert(function, analysis.callees.len());
        for callee in &analysis.callees {
            callers_by_callee.entry(callee).or_default().push(function);
        }
    }
    let mut ready: BTreeSet<&str> = remaining_callees
        .iter()
        .filter_map(|(id, count)| (*count == 0).then_some(*id))
        .collect();
    let mut expected = Vec::with_capacity(remaining_callees.len());
    while let Some(next) = ready.pop_first() {
        expected.push(next);
        for caller in callers_by_callee.get(next).into_iter().flatten() {
            let Some(count) = remaining_callees.get_mut(caller) else {
                return Err(invalid(
                    "VIR_UNKNOWN_CALLEE",
                    "call graph contains an unindexed caller",
                ));
            };
            *count -= 1;
            if *count == 0 {
                ready.insert(caller);
            }
        }
    }
    if expected.len() != analyses.len() {
        return Err(invalid("VIR_CALL_CYCLE", "module call graph is cyclic"));
    }
    for unit in &module.units {
        let expected_unit: Vec<_> = expected
            .iter()
            .copied()
            .filter(|id| unit.functions.iter().any(|function| function.id == **id))
            .collect();
        let actual: Vec<_> = unit
            .functions
            .iter()
            .map(|function| function.id.as_str())
            .collect();
        if actual != expected_unit {
            return Err(invalid(
                "VIR_FUNCTION_ORDER",
                "functions are not in canonical callee-first order",
            ));
        }
    }
    Ok(())
}

fn validate_module_safety_checks(
    module: &VirModule,
    analyses: &BTreeMap<String, FunctionAnalysis>,
) -> Result<(), VirValidationError> {
    for function in module.units.iter().flat_map(|unit| unit.functions.iter()) {
        let Some(analysis) = analyses.get(&function.id) else {
            return Err(invalid(
                "VIR_UNKNOWN_FUNCTION",
                "validated function is absent from the safety index",
            ));
        };
        for (actual, expected) in &analysis.safety_checks {
            validate_safety_checks(actual, expected)?;
        }
    }
    Ok(())
}

fn validate_hashes(module: &VirModule, index: &ModuleIndex<'_>) -> Result<(), VirValidationError> {
    let canonical = canonical_vir_json(module).map_err(canonical_error)?;
    if canonical.len() as u64 > VIR_CANONICAL_JSON_BYTES_MAX {
        return Err(limit(
            "VIR_LIMIT_CANONICAL_JSON_BYTES",
            "complete-root canonical JSON exceeds 192 MiB",
        ));
    }
    let mut contract_hashes = BTreeMap::new();
    for (id, (_, function)) in &index.functions {
        contract_hashes.insert(
            *id,
            contract_hash(&function.contracts).map_err(canonical_error)?,
        );
    }
    for function in module.units.iter().flat_map(|unit| unit.functions.iter()) {
        let actual_hash = contract_hashes.get(function.id.as_str()).ok_or_else(|| {
            invalid(
                "VIR_UNKNOWN_FUNCTION",
                "function is absent from the contract hash index",
            )
        })?;
        if !hashes_equal(actual_hash, &function.contracts.contract_hash) {
            return Err(invalid(
                "VIR_CONTRACT_HASH_MISMATCH",
                "contract_hash does not match recomputation",
            ));
        }
        for instruction in function
            .blocks
            .iter()
            .flat_map(|block| block.instructions.iter())
        {
            let VirInstruction::CallStatic {
                function,
                contract_hash: repeated_hash,
                ..
            } = instruction
            else {
                continue;
            };
            let Some(actual_hash) = contract_hashes.get(function.as_str()) else {
                return Err(invalid("VIR_UNKNOWN_CALLEE", "unknown CallStatic function"));
            };
            if !hashes_equal(actual_hash, repeated_hash) {
                return Err(invalid(
                    "VIR_CALLEE_CONTRACT_HASH",
                    "CallStatic contract hash mismatch",
                ));
            }
        }
    }
    let actual = vir_hash(module).map_err(canonical_error)?;
    if !hashes_equal(&actual, &module.vir_hash) {
        return Err(invalid(
            "VIR_HASH_MISMATCH",
            "vir_hash does not match recomputation",
        ));
    }
    Ok(())
}

fn hashes_equal(left: &LowercaseSha256, right: &LowercaseSha256) -> bool {
    let mut difference = 0_u8;
    for (left, right) in left
        .as_str()
        .as_bytes()
        .chunks_exact(2)
        .zip(right.as_str().as_bytes().chunks_exact(2))
    {
        difference |= decode_hex_pair(left) ^ decode_hex_pair(right);
    }
    difference == 0
}

fn decode_hex_pair(pair: &[u8]) -> u8 {
    (decode_lower_hex(pair[0]) << 4) | decode_lower_hex(pair[1])
}

fn decode_lower_hex(byte: u8) -> u8 {
    match byte {
        b'0'..=b'9' => byte - b'0',
        b'a'..=b'f' => byte - b'a' + 10,
        _ => 0,
    }
}

fn canonical_error(error: VirCanonicalError) -> VirValidationError {
    invalid("VIR_CANONICAL", error.to_string())
}

fn safety_check_error(error: SafetyCheckError) -> VirValidationError {
    invalid(error.code(), error.detail())
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VirValidationError {
    code: &'static str,
    detail: String,
}

impl VirValidationError {
    pub(crate) fn new(code: &'static str, detail: impl Into<String>) -> Self {
        Self {
            code,
            detail: detail.into(),
        }
    }

    pub const fn code(&self) -> &'static str {
        self.code
    }

    pub fn detail(&self) -> &str {
        &self.detail
    }
}

impl fmt::Display for VirValidationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}: {}", self.code, self.detail)
    }
}

impl Error for VirValidationError {}

fn invalid(code: &'static str, detail: impl Into<String>) -> VirValidationError {
    VirValidationError::new(code, detail)
}

fn limit(code: &'static str, detail: impl Into<String>) -> VirValidationError {
    invalid(code, detail)
}
