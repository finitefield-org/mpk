//! Closed seventy-head source syntax and frozen structural preflight. This is
//! deliberately distinct from expression types, lexical bindings and proofs.
use super::*;
fn exact(e: &Value, fields: &[&str]) -> OverlayResult<()> {
    let map = e
        .as_object()
        .ok_or_else(|| failure(GoSourceOverlayErrorKind::Shape))?;
    if map.len() != fields.len() || fields.iter().any(|key| !map.contains_key(*key)) {
        return Err(failure(GoSourceOverlayErrorKind::Shape));
    }
    Ok(())
}
fn text<'a>(e: &'a Value, key: &str) -> OverlayResult<&'a str> {
    e[key]
        .as_str()
        .ok_or_else(|| failure(GoSourceOverlayErrorKind::Shape))
}
fn decimal(s: &str, signed: bool) -> bool {
    let digits = if signed && s.starts_with('-') {
        let d = &s[1..];
        if d == "0" {
            return false;
        }
        d
    } else {
        s
    };
    !digits.is_empty()
        && digits.bytes().all(|b| b.is_ascii_digit())
        && (digits == "0" || !digits.starts_with('0'))
}
fn logical_name(s: &str) -> bool {
    (1..=64).contains(&s.len())
        && s.as_bytes()[0].is_ascii_lowercase()
        && s.bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_')
}
fn arity(op: &str) -> Option<usize> {
    match op {
        "bool.not" | "bool.value" | "int.neg" | "nat.to_int" | "word.neg" | "word.not"
        | "nat.from_unsigned" | "nat.from_signed" | "int.from_unsigned" | "int.from_signed"
        | "length" | "capacity" | "is_nil" => Some(1),
        "bool.and" | "bool.or" | "bool.implies" | "nat.add" | "nat.mul" | "nat.sub" | "int.add"
        | "int.mul" | "int.sub" | "eq" | "ne" | "lt" | "le" | "gt" | "ge" | "word.add"
        | "word.sub" | "word.mul" | "word.and" | "word.or" | "word.xor" | "word.andnot"
        | "word.div" | "word.rem" | "word.shl" | "word.shr" | "byte_at" | "element"
        | "bytes.prefix" | "bytes.suffix" | "bytes.concat" | "sequence.equal" => Some(2),
        "if" | "bytes.slice" | "slice.view2" => Some(3),
        "slice.view3" => Some(4),
        _ => None,
    }
}
pub(super) fn validate(wire: &Wire) -> OverlayResult<(usize, usize)> {
    let bad = || failure(GoSourceOverlayErrorKind::Shape);
    if wire.schema != "mpk.go.practical.source_overlay.v1"
        || !validate::logical_path(&wire.source)
        || !wire.function.validate()
        || !matches!(wire.termination.as_str(), "partial" | "total")
        || !wire.modifies.is_empty()
    {
        return Err(bad());
    }
    for entry in &wire.error_inputs {
        for step in &entry.path {
            if matches!(step,PathStep::Field{name}if !named(name)) {
                return Err(bad());
            }
        }
        if entry.cases.iter().any(|s| s.is_empty()) {
            return Err(bad());
        }
    }
    let mut stack = wire
        .requires
        .iter()
        .chain(&wire.ensures)
        .map(|e| (e, 0usize))
        .collect::<Vec<_>>();
    let mut limits = wire.requires.len() > 64
        || wire.ensures.is_empty()
        || wire.ensures.len() > 64
        || wire.loops.len() > 1024;
    for row in &wire.loops {
        if !matches!(row.selector.kind.as_str(), "for" | "range")
            || row.selector.start_byte >= row.selector.end_byte
        {
            return Err(bad());
        }
        limits |= row.invariants.is_empty() || row.invariants.len() > 64;
        stack.extend(row.invariants.iter().map(|e| (e, 0)));
        if let Variant::Candidate { expression } = &row.variant {
            stack.push((expression, 0));
        }
    }
    let (mut count, mut deepest) = (0, 0);
    while let Some((e, depth)) = stack.pop() {
        let op = text(e, "op")?;
        count += 1;
        deepest = deepest.max(depth);
        if let Some(n) = arity(op) {
            exact(e, &["op", "args"])?;
            let args = e["args"].as_array().ok_or_else(bad)?;
            if args.len() != n {
                return Err(bad());
            }
            stack.extend(args.iter().map(|v| (v, depth + 1)));
            continue;
        }
        match op {
            "bool.literal" => {
                exact(e, &["op", "value"])?;
                if !e["value"].is_boolean() {
                    return Err(bad());
                }
            }
            "nat.literal" | "int.literal" => {
                exact(e, &["op", "value"])?;
                if !decimal(text(e, "value")?, op == "int.literal") {
                    return Err(bad());
                }
            }
            "word.literal" | "bytes.literal" => {
                exact(e, &["op", "type", "value"])?;
                if text(e, "type")?.is_empty() {
                    return Err(bad());
                }
                let value = text(e, "value")?;
                if op == "word.literal" {
                    if !decimal(value, true) {
                        return Err(bad());
                    }
                } else {
                    if value.len() % 2 != 0
                        || !value
                            .bytes()
                            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
                    {
                        return Err(bad());
                    }
                    limits |= value.len() > 512;
                }
            }
            "source.zero" => {
                exact(e, &["op", "type"])?;
                if text(e, "type")?.is_empty() {
                    return Err(bad());
                }
            }
            "input" | "result" => {
                exact(e, &["op", "position"])?;
                if e["position"].as_u64().is_none() {
                    return Err(bad());
                }
            }
            "local" => {
                exact(e, &["op", "name"])?;
                if !named(text(e, "name")?) {
                    return Err(bad());
                }
            }
            "bound" => {
                exact(e, &["op", "name"])?;
                if !logical_name(text(e, "name")?) {
                    return Err(bad());
                }
            }
            "range.index" | "range.length" => exact(e, &["op"])?,
            "old" | "word.convert" | "field" | "error.is" | "error.payload" => {
                let fields = match op {
                    "old" => vec!["op", "value"],
                    "word.convert" => vec!["op", "type", "value"],
                    "field" => vec!["op", "value", "field"],
                    _ => vec!["op", "value", "case"],
                };
                exact(e, &fields)?;
                let invalid = match op {
                    "word.convert" => text(e, "type")?.is_empty(),
                    "field" => !named(text(e, "field")?),
                    "error.is" | "error.payload" => text(e, "case")?.is_empty(),
                    _ => false,
                };
                if invalid {
                    return Err(bad());
                }
                stack.push((&e["value"], depth + 1));
            }
            "forall" | "exists" | "nat.sum" | "int.sum" => {
                exact(e, &["op", "name", "lower", "upper", "body"])?;
                if !logical_name(text(e, "name")?) {
                    return Err(bad());
                }
                for key in ["lower", "upper", "body"] {
                    stack.push((&e[key], depth + 1));
                }
            }
            _ => return Err(bad()),
        }
    }
    if limits || count > 1024 || deepest > 32 {
        return Err(failure(GoSourceOverlayErrorKind::Limit));
    }
    Ok((count, deepest))
}
