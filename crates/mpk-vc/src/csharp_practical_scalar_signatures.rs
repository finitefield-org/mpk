//! Closed recipes for the retained scalar operations. This records types and
//! frozen failure order; it does not introduce an arithmetic evaluator.
use super::*;
fn scalar(token: &str) -> String {
    format!("mpk.csharp.value.{token}.v1")
}
fn integer(token: &str) -> Option<(u32, bool)> {
    Some(match token {
        "i8" => (8, true),
        "u8" => (8, false),
        "i16" => (16, true),
        "u16" | "char" => (16, false),
        "i32" => (32, true),
        "u32" => (32, false),
        "i64" => (64, true),
        "u64" => (64, false),
        _ => return None,
    })
}
pub fn scalar_operation_signature(
    id: &str,
) -> Result<ClosedOperationSignature, PracticalVirValidationError> {
    let fail = || {
        vir_failure(
            PracticalVirValidationPhase::Operation,
            PracticalVirErrorCode::UnknownOperation,
        )
    };
    let mut checks = vec![];
    let (arguments, result) = if let Some(op) = id.strip_prefix("boolean.") {
        match op {
            "not" => (vec![scalar("bool")], scalar("bool")),
            "and" | "or" | "xor" | "equal" | "not_equal" => {
                (vec![scalar("bool"); 2], scalar("bool"))
            }
            _ => return Err(fail()),
        }
    } else if let Some(rest) = id.strip_prefix("integer.convert.") {
        let parts = rest.split('.').collect::<Vec<_>>();
        if parts.len() != 3 || !matches!(parts[2], "checked" | "unchecked") {
            return Err(fail());
        }
        let (source_width, source_signed) = integer(parts[0]).ok_or_else(fail)?;
        let (target_width, target_signed) = integer(parts[1]).ok_or_else(fail)?;
        let fits = if source_signed {
            target_signed && target_width >= source_width
        } else if target_signed {
            target_width > source_width
        } else {
            target_width >= source_width
        };
        if parts[2] == "checked" && !fits {
            checks.push(("exception.overflow", "System.OverflowException"));
        }
        (vec![scalar(parts[0])], scalar(parts[1]))
    } else if let Some(rest) = id.strip_prefix("integer.") {
        let parts = rest.split('.').collect::<Vec<_>>();
        if parts.len() != 3
            || !matches!(parts[0], "i32" | "u32" | "i64" | "u64")
            || !matches!(parts[2], "checked" | "unchecked")
        {
            return Err(fail());
        }
        let ty = scalar(parts[0]);
        let unary = matches!(parts[1], "plus" | "negate" | "not");
        let comparison = matches!(
            parts[1],
            "equal" | "not_equal" | "less" | "less_equal" | "greater" | "greater_equal"
        );
        let shift = matches!(
            parts[1],
            "left_shift" | "right_shift" | "unsigned_right_shift"
        );
        if !(unary
            || comparison
            || shift
            || matches!(
                parts[1],
                "add" | "subtract" | "multiply" | "divide" | "remainder" | "and" | "or" | "xor"
            ))
        {
            return Err(fail());
        }
        if matches!(parts[1], "divide" | "remainder") {
            checks.push(("exception.division_by_zero", "System.DivideByZeroException"));
            if integer(parts[0]).unwrap().1 {
                checks.push(("exception.overflow", "System.OverflowException"));
            }
        } else if parts[2] == "checked"
            && matches!(parts[1], "add" | "subtract" | "multiply" | "negate")
        {
            checks.push(("exception.overflow", "System.OverflowException"));
        }
        let mut args = vec![ty.clone()];
        if !unary {
            args.push(if shift { scalar("i32") } else { ty.clone() });
        }
        (args, if comparison { scalar("bool") } else { ty })
    } else {
        return Err(fail());
    };
    Ok(ClosedOperationSignature {
        id: id.into(),
        tag: ClosedOperationTag::Data,
        argument_type_ids: arguments,
        normal_result_type_id: result,
        ordered_checks: checks
            .into_iter()
            .map(|(id, exception)| RequiredCheck {
                id: id.into(),
                tag: RequiredCheckTag::Exception,
                failure_type_id: Some(exception.into()),
            })
            .collect(),
    })
}
