//! Type-directed contract literals. Scalar conversion uses the W10 relation;
//! structural validation uses the sole monomorphic-value validator.
use super::*;
use crate::csharp_practical_source_artifacts::PracticalJsonValue as J;

pub(super) fn decode_contract_value(
    b: &ValidatedFoundationBundle,
    r: &ValidatedClosedRootSet,
    c: &ClosedInstanceSet,
    id: &str,
    value: &J,
) -> Result<MonomorphicValue, DataPhaseError> {
    fn decode(
        b: &ValidatedFoundationBundle,
        r: &ValidatedClosedRootSet,
        c: &ClosedInstanceSet,
        id: &str,
        v: &J,
        cells: &mut u64,
        depth: usize,
    ) -> Result<MonomorphicValue, DataPhaseError> {
        let fail = DataPhaseError::Contract;
        *cells += 1;
        if *cells > TOTAL_VALUE_CELLS_MAX || depth > 64 {
            return Err(fail);
        }
        let type_id = id.to_owned();
        let text = || v.as_str().ok_or(DataPhaseError::Contract);
        let utf16 = || match v {
            J::String(s) => Ok(s.encode_utf16().collect::<Vec<_>>()),
            J::Utf16String(s) => Ok(s.clone()),
            _ => Err(DataPhaseError::Contract),
        };
        let mut child = |ty: &str, v: &J| decode(b, r, c, ty, v, cells, depth + 1);
        if let Some(token) = id
            .strip_prefix("mpk.csharp.value.")
            .and_then(|s| s.strip_suffix(".v1"))
        {
            return match token {
                "unit" if v == &J::Null => Ok(MonomorphicValue::Unit { type_id }),
                "bool" => {
                    if let J::Bool(value) = v {
                        Ok(MonomorphicValue::Bool {
                            type_id,
                            value: *value,
                        })
                    } else {
                        Err(fail)
                    }
                }
                "char" => {
                    let s = utf16()?;
                    if s.len() != 1 {
                        return Err(fail);
                    }
                    Ok(MonomorphicValue::Char {
                        type_id,
                        utf16: s[0],
                    })
                }
                "string" => Ok(MonomorphicValue::String {
                    type_id,
                    utf16: utf16()?,
                }),
                "parse_error" => {
                    let arm = match text()? {
                        "input_bound" => ParseErrorArm::InputBound,
                        "syntax" => ParseErrorArm::Syntax,
                        "noncanonical" => ParseErrorArm::Noncanonical,
                        "scale_precision" => ParseErrorArm::ScalePrecision,
                        "range" => ParseErrorArm::Range,
                        _ => return Err(fail),
                    };
                    Ok(MonomorphicValue::ParseError { type_id, arm })
                }
                _ => {
                    let codec = match token {
                        "i8" | "u8" | "i16" | "u16" | "i32" | "u32" | "i64" | "u64" => {
                            format!("integer.{token}")
                        }
                        "f32" => "binary32".into(),
                        "f64" => "binary64".into(),
                        "decimal" => "decimal.normalized".into(),
                        "date" => "date".into(),
                        "time" => "time".into(),
                        "duration" => "duration_ticks".into(),
                        "instant" => "unix_milliseconds".into(),
                        "guid" => "guid.n".into(),
                        _ => return Err(fail),
                    };
                    BoundaryCodec::new(&codec, id, None, None)
                        .map_err(|_| fail.clone())?
                        .parse(&utf16()?)
                        .map_err(|_| fail)
                }
            };
        }
        if let Some(source) = r.source_types.get(id) {
            if source.kind == SourceKind::Enum {
                return Ok(MonomorphicValue::Enum {
                    type_id,
                    underlying: source.enum_underlying.clone().ok_or(fail.clone())?,
                    carrier: text()?.into(),
                });
            }
            let fields = v.as_object().ok_or(fail.clone())?;
            if fields
                .iter()
                .map(|(name, _)| name)
                .ne(source.members.iter().map(|m| &m.name))
            {
                return Err(fail);
            }
            let mut result = vec![];
            for (m, (_, v)) in source.members.iter().zip(fields) {
                let ty = closed_type_id(b, &m.ty).map_err(|_| fail.clone())?;
                result.push(NamedMonomorphicValue {
                    name: m.name.clone(),
                    value: Box::new(child(&ty, v)?),
                });
            }
            return Ok(MonomorphicValue::Product {
                type_id,
                fields: result,
            });
        }
        let meta = c.metadata.get(id).ok_or(fail.clone())?;
        let args = &meta.argument_ids;
        let fields = |names: &[&str]| -> Result<Vec<&J>, DataPhaseError> {
            let fields = v.as_object().ok_or(DataPhaseError::Contract)?;
            if fields
                .iter()
                .map(|(k, _)| k.as_str())
                .ne(names.iter().copied())
            {
                return Err(DataPhaseError::Contract);
            }
            Ok(fields.iter().map(|(_, v)| v).collect())
        };
        match template_name(&meta.template_id).ok_or(fail.clone())? {
            "bounded_sequence" | "ordered_set" => {
                let values = v.as_array().ok_or(fail.clone())?;
                if values.len() as u64 > TOTAL_VALUE_CELLS_MAX {
                    return Err(fail);
                }
                let elements = values
                    .iter()
                    .map(|v| child(&args[0], v))
                    .collect::<Result<_, _>>()?;
                if template_name(&meta.template_id) == Some("ordered_set") {
                    Ok(MonomorphicValue::OrderedSet { type_id, elements })
                } else {
                    Ok(MonomorphicValue::Sequence { type_id, elements })
                }
            }
            "ordered_entry" => {
                let f = fields(&["key", "value"])?;
                Ok(MonomorphicValue::OrderedEntry {
                    type_id,
                    key: Box::new(child(&args[0], f[0])?),
                    value: Box::new(child(&args[1], f[1])?),
                })
            }
            "ordered_map" => {
                let values = v.as_array().ok_or(fail.clone())?;
                let mut entries = vec![];
                for value in values {
                    let f = value.as_object().ok_or(fail.clone())?;
                    if f.iter().map(|(k, _)| k.as_str()).ne(["key", "value"]) {
                        return Err(fail);
                    }
                    entries.push(MonomorphicMapEntry {
                        key: Box::new(child(&args[0], &f[0].1)?),
                        value: Box::new(child(&args[1], &f[1].1)?),
                    });
                }
                Ok(MonomorphicValue::OrderedMap { type_id, entries })
            }
            "money" => {
                let f = fields(&["amount", "currency"])?;
                Ok(MonomorphicValue::Money {
                    type_id,
                    amount: Box::new(child("mpk.csharp.value.decimal.v1", f[0])?),
                    currency: Box::new(child(&args[0], f[1])?),
                })
            }
            "transition" => {
                let f = fields(&["state", "events", "response"])?;
                let events = f[1]
                    .as_array()
                    .ok_or(fail.clone())?
                    .iter()
                    .map(|v| child(&args[1], v))
                    .collect::<Result<_, _>>()?;
                Ok(MonomorphicValue::Transition {
                    type_id,
                    state: Box::new(child(&args[0], f[0])?),
                    events,
                    response: Box::new(child(&args[2], f[2])?),
                })
            }
            name @ ("option" | "lookup" | "result" | "validation" | "boundary_field") => {
                let arm = v.get("tag").and_then(J::as_str).ok_or(fail.clone())?;
                let payload_ty = match (name, arm) {
                    ("option", "none")
                    | ("lookup", "missing_key")
                    | ("boundary_field", "missing" | "null") => None,
                    ("option", "some")
                    | ("lookup", "found")
                    | ("result", "ok")
                    | ("validation", "valid")
                    | ("boundary_field", "value") => Some(args[0].clone()),
                    ("result", "error") => Some(args[1].clone()),
                    ("validation", "invalid") => {
                        let ty = concrete_type(r, c, &args[1])?;
                        Some(
                            closed_type_id(
                                b,
                                &ClosedType::Instance {
                                    template: "bounded_sequence".into(),
                                    arguments: vec![ty],
                                },
                            )
                            .map_err(|_| fail.clone())?,
                        )
                    }
                    _ => return Err(fail),
                };
                let payload = if let Some(ty) = payload_ty {
                    let f = fields(&["tag", "payload"])?;
                    Some(child(&ty, f[1])?)
                } else {
                    fields(&["tag"])?;
                    None
                };
                match name {
                    "option" => Ok(MonomorphicValue::Option {
                        type_id,
                        arm: if arm == "none" {
                            OptionArm::None
                        } else {
                            OptionArm::Some
                        },
                        value: payload.map(Box::new),
                    }),
                    "boundary_field" => Ok(MonomorphicValue::BoundaryPresence {
                        type_id,
                        arm: match arm {
                            "missing" => BoundaryArm::Missing,
                            "null" => BoundaryArm::Null,
                            _ => BoundaryArm::Value,
                        },
                        value: payload.map(Box::new),
                    }),
                    _ => Ok(MonomorphicValue::TaggedSum {
                        type_id,
                        arm: arm.into(),
                        payload: payload.into_iter().collect(),
                    }),
                }
            }
            _ => Err(fail),
        }
    }
    let decoded = decode(b, r, c, id, value, &mut 0, 0)?;
    validate_monomorphic_value(b, r, c, &decoded).map_err(|_| DataPhaseError::Contract)?;
    Ok(decoded)
}
