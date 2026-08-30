use crate::contract::{
    exact_fields, parse_contract, valid_function_id, valid_identifier, ContractCode, ContractError,
    ContractFunction, ContractInput, ContractSet, ContractType, NormalizedContract, ParsedContract,
    CONTRACT_CLAUSES_MAX, CONTRACT_EXPRESSION_DEPTH_MAX, CONTRACT_NODES_CLOSURE_MAX,
    CONTRACT_NODES_FUNCTION_MAX, RUST_SEMANTIC_PROFILE,
};
use crate::json::{self, JsonValue};
use crate::limits::RustLimitId;
use crate::sha256::{hex, Sha256};
use std::collections::BTreeMap;

const CONTRACT_HASH_DOMAIN: &[u8] = b"MPK-CONTRACT-1.0";

pub fn attach_contracts(
    inputs: Vec<ContractInput>,
    functions: &[ContractFunction],
    target_id: &str,
    pointer_width: u8,
) -> Result<ContractSet, ContractError> {
    let mut inputs = inputs;
    inputs.sort_by(|left, right| left.normalized_path.cmp(&right.normalized_path));

    let contract_files_max = RustLimitId::ContractFiles.maximum() as usize;
    if inputs.len() > contract_files_max {
        return Err(ContractError::new(
            ContractCode::Limit,
            inputs.get(contract_files_max),
        ));
    }
    let mut total_bytes = 0_u64;
    let mut prescan_nodes = 0_usize;
    let mut prescan_errors = Vec::new();
    for input in &inputs {
        total_bytes = total_bytes
            .checked_add(input.bytes.len() as u64)
            .ok_or_else(|| ContractError::new(ContractCode::Limit, Some(input)))?;
        if total_bytes > RustLimitId::ContractTotalBytes.maximum() {
            return Err(ContractError::new(ContractCode::Limit, Some(input)));
        }
        match crate::contract::prescan_contract(input) {
            Ok(metrics) => {
                prescan_nodes = prescan_nodes
                    .checked_add(metrics.nodes)
                    .ok_or_else(|| ContractError::new(ContractCode::Limit, Some(input)))?;
                if prescan_nodes > CONTRACT_NODES_CLOSURE_MAX {
                    prescan_errors.push(ContractError::new(ContractCode::Limit, Some(input)));
                    break;
                }
            }
            Err(error) => prescan_errors.push(error),
        }
    }
    if let Some(error) = first_error(prescan_errors) {
        return Err(error);
    }

    let mut parsed = Vec::with_capacity(inputs.len());
    let mut errors = Vec::new();
    for input in inputs {
        match parse_contract(input) {
            Ok(contract) => parsed.push(contract),
            Err(error) => errors.push(error),
        }
    }
    if let Some(error) = first_error(errors) {
        return Err(error);
    }

    let function_map = functions
        .iter()
        .map(|function| (function.function_id.as_str(), function))
        .collect::<BTreeMap<_, _>>();
    let mut by_function = BTreeMap::<String, Vec<ParsedContract>>::new();
    let mut identity_errors = Vec::new();
    for contract in parsed {
        if !valid_function_id(&contract.function) {
            identity_errors.push(ContractError::for_function(
                ContractCode::Identity,
                Some(&contract.input),
                &contract.function,
            ));
            continue;
        }
        by_function
            .entry(contract.function.clone())
            .or_default()
            .push(contract);
    }
    if let Some(error) = first_error(identity_errors) {
        return Err(error);
    }

    let mut closure_errors = Vec::new();
    for (function_id, contracts) in &by_function {
        if contracts.len() > 1 {
            closure_errors.push(ContractError::for_function(
                ContractCode::Duplicate,
                contracts.get(1).map(|contract| &contract.input),
                function_id,
            ));
        }
        if !function_map.contains_key(function_id.as_str()) {
            closure_errors.push(ContractError::for_function(
                ContractCode::Unused,
                contracts.first().map(|contract| &contract.input),
                function_id,
            ));
        }
    }
    for function in functions {
        if !by_function.contains_key(&function.function_id) {
            closure_errors.push(ContractError::for_function(
                ContractCode::Missing,
                None,
                &function.function_id,
            ));
        }
    }
    if let Some(error) = first_error(closure_errors) {
        return Err(error);
    }

    let mut normalized = Vec::with_capacity(functions.len());
    let mut type_errors = Vec::new();
    let mut closure_nodes = 0_usize;
    for (function_id, contracts) in by_function {
        let contract = contracts
            .into_iter()
            .next()
            .expect("validated one contract per function");
        let function = function_map[function_id.as_str()];
        let remaining_nodes = CONTRACT_NODES_CLOSURE_MAX
            .checked_sub(closure_nodes)
            .expect("checked closure-node counter cannot exceed its maximum");
        match normalize_contract(
            contract,
            function,
            target_id,
            pointer_width,
            remaining_nodes,
        ) {
            Ok((contract, nodes)) => {
                closure_nodes = match closure_nodes.checked_add(nodes) {
                    Some(nodes) => nodes,
                    None => {
                        type_errors.push(ContractError::for_function(
                            ContractCode::Limit,
                            None,
                            function_id.as_str(),
                        ));
                        continue;
                    }
                };
                normalized.push(contract);
            }
            Err(error) => {
                let limit = error.code == ContractCode::Limit;
                type_errors.push(error);
                if limit {
                    break;
                }
            }
        }
    }
    if let Some(error) = first_error(type_errors) {
        return Err(error);
    }
    normalized.sort_by(|left, right| left.function_id.cmp(&right.function_id));
    Ok(ContractSet::new(normalized))
}

fn normalize_contract(
    contract: ParsedContract,
    function: &ContractFunction,
    target_id: &str,
    pointer_width: u8,
    closure_nodes_remaining: usize,
) -> Result<(NormalizedContract, usize), ContractError> {
    if contract.semantic_profile != RUST_SEMANTIC_PROFILE
        || contract.target_pointer_width != i64::from(pointer_width)
        || !matches!(
            (target_id, pointer_width),
            ("i686-unknown-linux-gnu", 32) | ("x86_64-unknown-linux-gnu", 64)
        )
    {
        return Err(error_for(
            ContractCode::Profile,
            &contract,
            &function.function_id,
        ));
    }
    if contract
        .requires
        .len()
        .checked_add(contract.ensures.len())
        .is_none_or(|clauses| clauses > CONTRACT_CLAUSES_MAX)
    {
        return Err(error_for(
            ContractCode::Limit,
            &contract,
            &function.function_id,
        ));
    }
    if function.parameter_names.len() != function.parameter_types.len() {
        return Err(error_for(
            ContractCode::Resolution,
            &contract,
            &function.function_id,
        ));
    }
    let mut parameters = BTreeMap::new();
    for (index, (name, parameter_type)) in function
        .parameter_names
        .iter()
        .zip(&function.parameter_types)
        .enumerate()
    {
        if !valid_identifier(name)
            || parameters
                .insert(name.as_str(), (format!("arg{index}"), parameter_type))
                .is_some()
        {
            return Err(error_for(
                ContractCode::Resolution,
                &contract,
                &function.function_id,
            ));
        }
    }

    let context = ExpressionContext {
        parameters,
        result_type: &function.result_type,
    };
    let mut metrics = ExpressionMetrics {
        nodes: 0,
        maximum_nodes: CONTRACT_NODES_FUNCTION_MAX.min(closure_nodes_remaining),
    };
    let mut requires = Vec::with_capacity(contract.requires.len());
    for expression in &contract.requires {
        let (normalized, expression_type) = normalize_expression(
            expression,
            &context,
            false,
            &contract,
            &function.function_id,
            &mut metrics,
            1,
        )?;
        if !expression_type.is_bool() {
            return Err(error_for(
                ContractCode::Type,
                &contract,
                &function.function_id,
            ));
        }
        requires.push(normalized);
    }
    let mut ensures = Vec::with_capacity(contract.ensures.len());
    for expression in &contract.ensures {
        let (normalized, expression_type) = normalize_expression(
            expression,
            &context,
            true,
            &contract,
            &function.function_id,
            &mut metrics,
            1,
        )?;
        if !expression_type.is_bool() {
            return Err(error_for(
                ContractCode::Type,
                &contract,
                &function.function_id,
            ));
        }
        ensures.push(normalized);
    }

    let unit_id = function
        .function_id
        .split("::")
        .next()
        .ok_or_else(|| error_for(ContractCode::Identity, &contract, &function.function_id))?;
    let mut root = BTreeMap::from([
        ("unit_id".to_owned(), JsonValue::String(unit_id.to_owned())),
        (
            "function_id".to_owned(),
            JsonValue::String(function.function_id.clone()),
        ),
        (
            "semantic_context".to_owned(),
            crate::successor::semantic_context(target_id, pointer_width),
        ),
        ("requires".to_owned(), JsonValue::Array(requires)),
        ("ensures".to_owned(), JsonValue::Array(ensures)),
        ("modifies".to_owned(), JsonValue::Array(Vec::new())),
        (
            "panic".to_owned(),
            JsonValue::String("forbidden".to_owned()),
        ),
        (
            "termination".to_owned(),
            JsonValue::String("total".to_owned()),
        ),
        ("loops".to_owned(), JsonValue::Array(Vec::new())),
    ]);
    let hash = hash_contract_value(&JsonValue::Object(root.clone()))
        .map_err(|_| error_for(ContractCode::Hash, &contract, &function.function_id))?;
    root.insert("contract_hash".to_owned(), JsonValue::String(hash.clone()));
    let value = JsonValue::Object(root);
    if recompute_contract_hash(&value).as_deref() != Ok(hash.as_str()) {
        return Err(error_for(
            ContractCode::Hash,
            &contract,
            &function.function_id,
        ));
    }
    Ok((
        NormalizedContract {
            normalized_path: contract.input.normalized_path,
            raw_input_sha256: contract.input.raw_input_sha256,
            function_id: function.function_id.clone(),
            contract_hash: hash,
            value,
        },
        metrics.nodes,
    ))
}

struct ExpressionContext<'a> {
    parameters: BTreeMap<&'a str, (String, &'a ContractType)>,
    result_type: &'a ContractType,
}

struct ExpressionMetrics {
    nodes: usize,
    maximum_nodes: usize,
}

#[allow(clippy::too_many_arguments)]
fn normalize_expression(
    value: &JsonValue,
    context: &ExpressionContext<'_>,
    allow_result: bool,
    contract: &ParsedContract,
    function_id: &str,
    metrics: &mut ExpressionMetrics,
    depth: usize,
) -> Result<(JsonValue, ContractType), ContractError> {
    metrics.nodes = metrics
        .nodes
        .checked_add(1)
        .ok_or_else(|| error_for(ContractCode::Limit, contract, function_id))?;
    if metrics.nodes > metrics.maximum_nodes || depth > CONTRACT_EXPRESSION_DEPTH_MAX {
        return Err(error_for(ContractCode::Limit, contract, function_id));
    }
    let expression = value
        .as_object()
        .ok_or_else(|| error_for(ContractCode::Shape, contract, function_id))?;

    if expression.len() == 1 {
        if let Some(parameter) = expression.get("parameter") {
            let name = parameter
                .as_str()
                .ok_or_else(|| error_for(ContractCode::Shape, contract, function_id))?;
            if !valid_identifier(name) {
                return Err(error_for(ContractCode::Identity, contract, function_id));
            }
            let (id, parameter_type) = context
                .parameters
                .get(name)
                .ok_or_else(|| error_for(ContractCode::Resolution, contract, function_id))?;
            return Ok((
                object([("var", JsonValue::String(id.clone()))]),
                (*parameter_type).clone(),
            ));
        }
        if let Some(result) = expression.get("result") {
            if result.integer() != Some(0) || !allow_result {
                return Err(error_for(ContractCode::Resolution, contract, function_id));
            }
            return Ok((
                object([("result", JsonValue::Number("0".to_owned()))]),
                context.result_type.clone(),
            ));
        }
        if let Some(boolean) = expression.get("bool") {
            let boolean = boolean
                .as_bool()
                .ok_or_else(|| error_for(ContractCode::Shape, contract, function_id))?;
            return Ok((
                object([("bool", JsonValue::Bool(boolean))]),
                ContractType::Bool,
            ));
        }
        if let Some(integer) = expression.get("bv") {
            return normalize_integer(integer, contract, function_id);
        }
        return Err(error_for(ContractCode::Shape, contract, function_id));
    }

    if !exact_fields(expression, &["op", "args"]) {
        return Err(error_for(ContractCode::Shape, contract, function_id));
    }
    let operator = expression
        .get("op")
        .and_then(JsonValue::as_str)
        .ok_or_else(|| error_for(ContractCode::Shape, contract, function_id))?;
    let arguments = expression
        .get("args")
        .and_then(JsonValue::as_array)
        .ok_or_else(|| error_for(ContractCode::Shape, contract, function_id))?;

    match operator {
        "not" | "bv_neg" | "bv_not" => {
            if arguments.len() != 1 {
                return Err(error_for(ContractCode::Operator, contract, function_id));
            }
            let (argument, argument_type) = normalize_expression(
                &arguments[0],
                context,
                allow_result,
                contract,
                function_id,
                metrics,
                depth + 1,
            )?;
            let accepted = match operator {
                "not" => argument_type.is_bool(),
                "bv_neg" | "bv_not" => argument_type.as_bit_vector().is_some(),
                _ => false,
            };
            if !accepted {
                return Err(error_for(ContractCode::Type, contract, function_id));
            }
            Ok((
                object([
                    ("op", JsonValue::String(operator.to_owned())),
                    ("value", argument),
                ]),
                argument_type,
            ))
        }
        "and" | "or" => {
            if !(2..=64).contains(&arguments.len()) {
                return Err(error_for(ContractCode::Operator, contract, function_id));
            }
            let mut normalized = Vec::with_capacity(arguments.len());
            for argument in arguments {
                let (argument, argument_type) = normalize_expression(
                    argument,
                    context,
                    allow_result,
                    contract,
                    function_id,
                    metrics,
                    depth + 1,
                )?;
                if !argument_type.is_bool() {
                    return Err(error_for(ContractCode::Type, contract, function_id));
                }
                normalized.push(argument);
            }
            Ok((
                object([
                    ("op", JsonValue::String(operator.to_owned())),
                    ("args", JsonValue::Array(normalized)),
                ]),
                ContractType::Bool,
            ))
        }
        operator if binary_operator(operator) => {
            if arguments.len() != 2 {
                return Err(error_for(ContractCode::Operator, contract, function_id));
            }
            let (left, left_type) = normalize_expression(
                &arguments[0],
                context,
                allow_result,
                contract,
                function_id,
                metrics,
                depth + 1,
            )?;
            let (right, right_type) = normalize_expression(
                &arguments[1],
                context,
                allow_result,
                contract,
                function_id,
                metrics,
                depth + 1,
            )?;
            let result_type = check_binary_type(operator, &left_type, &right_type)
                .ok_or_else(|| error_for(ContractCode::Type, contract, function_id))?;
            Ok((
                object([
                    ("op", JsonValue::String(operator.to_owned())),
                    ("lhs", left),
                    ("rhs", right),
                ]),
                result_type,
            ))
        }
        _ => Err(error_for(ContractCode::Operator, contract, function_id)),
    }
}

fn normalize_integer(
    value: &JsonValue,
    contract: &ParsedContract,
    function_id: &str,
) -> Result<(JsonValue, ContractType), ContractError> {
    let integer = value
        .as_object()
        .filter(|integer| exact_fields(integer, &["decimal", "width", "signed"]))
        .ok_or_else(|| error_for(ContractCode::Shape, contract, function_id))?;
    let decimal = integer
        .get("decimal")
        .and_then(JsonValue::as_str)
        .ok_or_else(|| error_for(ContractCode::Shape, contract, function_id))?;
    let width = integer
        .get("width")
        .and_then(JsonValue::integer)
        .and_then(|width| u8::try_from(width).ok())
        .filter(|width| matches!(width, 8 | 16 | 32 | 64))
        .ok_or_else(|| error_for(ContractCode::Type, contract, function_id))?;
    let signed = integer
        .get("signed")
        .and_then(JsonValue::as_bool)
        .ok_or_else(|| error_for(ContractCode::Shape, contract, function_id))?;
    if !integer_fits(decimal, width, signed) {
        return Err(error_for(ContractCode::Type, contract, function_id));
    }
    Ok((
        object([(
            "int",
            object([
                ("value", JsonValue::String(decimal.to_owned())),
                ("width", JsonValue::Number(width.to_string())),
                ("signed", JsonValue::Bool(signed)),
            ]),
        )]),
        ContractType::BitVector { width, signed },
    ))
}

fn check_binary_type(
    operator: &str,
    left: &ContractType,
    right: &ContractType,
) -> Option<ContractType> {
    match operator {
        "eq" | "not_eq" if left == right => Some(ContractType::Bool),
        "signed_lt" | "signed_le" | "signed_gt" | "signed_ge"
            if left == right && left.as_bit_vector().is_some_and(|(_, signed)| signed) =>
        {
            Some(ContractType::Bool)
        }
        "unsigned_lt" | "unsigned_le" | "unsigned_gt" | "unsigned_ge"
            if left == right && left.as_bit_vector().is_some_and(|(_, signed)| !signed) =>
        {
            Some(ContractType::Bool)
        }
        "bv_add" | "bv_sub" | "bv_mul" | "bv_and" | "bv_or" | "bv_xor"
            if left == right && left.as_bit_vector().is_some() =>
        {
            Some(left.clone())
        }
        "bv_shl" if left.as_bit_vector().is_some() && right.as_bit_vector().is_some() => {
            Some(left.clone())
        }
        "bv_ashr"
            if left.as_bit_vector().is_some_and(|(_, signed)| signed)
                && right.as_bit_vector().is_some() =>
        {
            Some(left.clone())
        }
        "bv_lshr"
            if left.as_bit_vector().is_some_and(|(_, signed)| !signed)
                && right.as_bit_vector().is_some() =>
        {
            Some(left.clone())
        }
        _ => None,
    }
}

fn binary_operator(operator: &str) -> bool {
    matches!(
        operator,
        "eq" | "not_eq"
            | "signed_lt"
            | "signed_le"
            | "signed_gt"
            | "signed_ge"
            | "unsigned_lt"
            | "unsigned_le"
            | "unsigned_gt"
            | "unsigned_ge"
            | "bv_add"
            | "bv_sub"
            | "bv_mul"
            | "bv_and"
            | "bv_or"
            | "bv_xor"
            | "bv_shl"
            | "bv_ashr"
            | "bv_lshr"
    )
}

fn integer_fits(value: &str, width: u8, signed: bool) -> bool {
    let (negative, digits) = value
        .strip_prefix('-')
        .map_or((false, value), |digits| (true, digits));
    if digits.is_empty()
        || (digits.len() > 1 && digits.starts_with('0'))
        || !digits.bytes().all(|byte| byte.is_ascii_digit())
        || negative && digits == "0"
        || negative && !signed
        || digits.len() > 20
    {
        return false;
    }
    let Ok(magnitude) = digits.parse::<u128>() else {
        return false;
    };
    if signed {
        let half = 1_u128 << (width - 1);
        if negative {
            magnitude <= half
        } else {
            magnitude < half
        }
    } else {
        magnitude < (1_u128 << width)
    }
}

fn object<const N: usize>(entries: [(&str, JsonValue); N]) -> JsonValue {
    JsonValue::Object(
        entries
            .into_iter()
            .map(|(key, value)| (key.to_owned(), value))
            .collect(),
    )
}

fn hash_contract_value(value: &JsonValue) -> Result<String, ()> {
    let payload = json::canonical(value).map_err(|_| ())?;
    let mut hasher = Sha256::new();
    hasher.update(CONTRACT_HASH_DOMAIN);
    hasher.update(&[0]);
    hasher.update(&payload);
    Ok(hex(&hasher.finish()))
}

fn recompute_contract_hash(value: &JsonValue) -> Result<String, ()> {
    let mut payload = value.clone();
    payload
        .as_object_mut()
        .ok_or(())?
        .remove("contract_hash")
        .ok_or(())?;
    hash_contract_value(&payload)
}

fn error_for(code: ContractCode, contract: &ParsedContract, function_id: &str) -> ContractError {
    ContractError::for_function(code, Some(&contract.input), function_id)
}

fn first_error(errors: Vec<ContractError>) -> Option<ContractError> {
    errors.into_iter().min_by(|left, right| {
        (
            left.code.precedence(),
            left.normalized_path.as_deref().unwrap_or(""),
            left.function_id.as_deref().unwrap_or(""),
        )
            .cmp(&(
                right.code.precedence(),
                right.normalized_path.as_deref().unwrap_or(""),
                right.function_id.as_deref().unwrap_or(""),
            ))
    })
}

#[cfg(test)]
mod tests {
    use super::integer_fits;

    #[test]
    fn integer_boundaries_are_exact() {
        assert!(integer_fits("-128", 8, true));
        assert!(integer_fits("127", 8, true));
        assert!(!integer_fits("-129", 8, true));
        assert!(!integer_fits("128", 8, true));
        assert!(integer_fits("255", 8, false));
        assert!(!integer_fits("256", 8, false));
        assert!(integer_fits("18446744073709551615", 64, false));
        assert!(!integer_fits("-0", 64, true));
        assert!(!integer_fits("01", 64, false));
    }
}
