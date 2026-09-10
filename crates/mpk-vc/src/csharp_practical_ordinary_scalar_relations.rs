//! Reuse reviewed scalar circuits for structural relations; equality and total
//! ordering are distinct capabilities (IEEE values never gain total ordering).
use super::*;
#[derive(Clone, Debug)]
pub(in super::super) struct ScalarRelations {
    pub equal: String,
    pub compare: Option<String>,
}
fn signature(key: &str, suffix: &str, result: &str) -> ClosedOperationSignature {
    ClosedOperationSignature {
        id: format!("structural.scalar.{key}.{suffix}"),
        tag: ClosedOperationTag::Data,
        argument_type_ids: vec![key.into(); 2],
        normal_result_type_id: format!("mpk.csharp.value.{result}.v1"),
        ordered_checks: vec![],
    }
}
pub(in super::super) fn bits_relation(
    b: &mut Builder,
    key: &str,
    width: u32,
    signed: bool,
) -> R<ScalarRelations> {
    if width > 128 {
        return Err(OrdinaryCarrierError::Shape);
    }
    let mut equal = String::new();
    let mut compare = String::new();
    for comparison in [false, true] {
        let mut c = Circuit::new(&[width.max(1) as usize; 2]);
        let left = c.inputs[0][..width as usize].to_vec();
        let right = c.inputs[1][..width as usize].to_vec();
        let eq = if width == 0 {
            T
        } else {
            c.equal(&left, &right)
        };
        let output = if comparison {
            let less = if width == 0 {
                F
            } else {
                c.lt(&left, &right, signed)
            };
            let mut one = vec![F; 32];
            one[0] = T;
            let other = c.select(eq, &[F; 32], &one);
            c.select(less, &[T; 32], &other)
        } else {
            vec![eq]
        };
        let p = IntegerCircuit {
            signature: signature(
                key,
                if comparison { "compare" } else { "equal" },
                if comparison { "i32" } else { "bool" },
            ),
            circuit: c,
            output,
            failures: vec![],
        };
        let d = emit_circuit(b, p, "StructuralScalar")?;
        if comparison {
            compare = d.result_definition;
        } else {
            equal = d.result_definition;
        }
    }
    Ok(ScalarRelations {
        equal,
        compare: Some(compare),
    })
}
pub(in super::super) fn special_relation(b: &mut Builder, token: &str) -> R<ScalarRelations> {
    match token {
        "f32" | "f64" => {
            let id = if token == "f32" {
                "floating.single.equal"
            } else {
                "floating.double.equal"
            };
            let d = emit_circuit(b, floating::floating_circuit(id)?, "StructuralScalar")?;
            Ok(ScalarRelations {
                equal: d.result_definition,
                compare: None,
            })
        }
        "decimal" => {
            let eq = decimal::emit_decimal(b, "decimal.equal")?.result_definition;
            let lt = decimal::emit_decimal(b, "decimal.less")?.result_definition;
            // Normalize the value comparison to -1/0/1 without observing scale/sign storage.
            let mut c = Circuit::new(&[1, 1]);
            let eq_bit = c.inputs[0][0];
            let lt_bit = c.inputs[1][0];
            let mut one = vec![F; 32];
            one[0] = T;
            let rest = c.select(eq_bit, &[F; 32], &one);
            let output = c.select(lt_bit, &[T; 32], &rest);
            let mut s = signature("mpk.csharp.value.bool.v1", "decimal_order", "i32");
            s.id = "structural.decimal.order_from_flags".into();
            let flags = emit_circuit(
                b,
                IntegerCircuit {
                    signature: s,
                    circuit: c,
                    output,
                    failures: vec![],
                },
                "StructuralScalar",
            )?
            .result_definition;
            let left = b.var(1)?;
            let right = b.var(0)?;
            let eq_fn = b.constant(&eq)?;
            let eq_value = b.app(eq_fn, vec![left, right])?;
            let lt_fn = b.constant(&lt)?;
            let lt_value = b.app(lt_fn, vec![left, right])?;
            let f = b.constant(&flags)?;
            let result = b.app(f, vec![eq_value, lt_value])?;
            let body = bind_inputs(b, &[512, 512], result)?;
            let word = b.cube(5)?;
            let ty = input_type(b, &[512, 512], word)?;
            let compare = format!("{PREFIX}.StructuralScalar.DecimalCompare");
            b.define(&compare, ty, body)?;
            Ok(ScalarRelations {
                equal: eq,
                compare: Some(compare),
            })
        }
        _ => Err(OrdinaryCarrierError::Shape),
    }
}

/// Exact unsigned addition saturated at the invalid logical-cell sentinel.
/// Extend to 33 bits before adding, so even two arbitrary u32 inputs cannot wrap.
pub(in super::super) fn count_addition(b: &mut Builder) -> R<String> {
    let sig = signature("mpk.csharp.value.u32.v1", "saturated_cell_add", "u32");
    let result = format!(
        "{PREFIX}.DomainCount.O{}.Result",
        sig.id
            .as_bytes()
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect::<String>()
    );
    if b.globals.contains_key(&result) {
        return Ok(result);
    }
    let mut c = Circuit::new(&[32, 32]);
    let left = Circuit::extend(&c.inputs[0], 33, false);
    let right = Circuit::extend(&c.inputs[1], 33, false);
    let (sum, _) = c.add(&left, &right, F);
    let maximum = TOTAL_VALUE_CELLS_MAX + 1;
    let bound = (0..33)
        .map(|i| if maximum & (1u64 << i) == 0 { F } else { T })
        .collect::<Vec<_>>();
    let below = c.lt(&sum, &bound, false);
    let output = c.select(below, &sum[..32], &bound[..32]);
    // Preserve left-to-right failure short-circuiting: once a child count is
    // invalid, the rest of the value need not be inspected to reject it.
    let right_valid = c.lt(&right, &bound, false);
    let output = c.select(right_valid, &output, &bound[..32]);
    let left_valid = c.lt(&left, &bound, false);
    let output = c.select(left_valid, &output, &bound[..32]);
    let d = emit_circuit(
        b,
        IntegerCircuit {
            signature: sig,
            circuit: c,
            output,
            failures: vec![],
        },
        "DomainCount",
    )?;
    Ok(d.result_definition)
}

/// Unsigned word subtraction for a sequence's right-hand offset. Its caller
/// selects it only when index >= left length; subtraction itself is total u32.
pub(in super::super) fn sequence_subtraction(b: &mut Builder) -> R<String> {
    let sig = signature("mpk.csharp.value.u32.v1", "sequence_offset", "u32");
    let result = format!(
        "{PREFIX}.SequenceIndex.O{}.Result",
        sig.id
            .as_bytes()
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect::<String>()
    );
    if b.globals.contains_key(&result) {
        return Ok(result);
    }
    let mut c = Circuit::new(&[32, 32]);
    let (output, _) = c.sub(&c.inputs[0].clone(), &c.inputs[1].clone());
    Ok(emit_circuit(
        b,
        IntegerCircuit {
            signature: sig,
            circuit: c,
            output,
            failures: vec![],
        },
        "SequenceIndex",
    )?
    .result_definition)
}

#[cfg(test)]
mod tests {
    use super::super::super::tests::{apply, bit, run, V};
    use super::*;

    #[test]
    fn ordinary_scalar_relation_signed_unsigned_boundaries() {
        let mut b = Builder::new().unwrap();
        let mut definitions = vec![];
        for width in [0u32, 1, 8, 16, 32, 64, 128] {
            for signed in [false, true] {
                let relation =
                    bits_relation(&mut b, &format!("raw.{width}.{signed}"), width, signed).unwrap();
                definitions.push((width, signed, relation));
            }
        }
        let bytes = b.finish().unwrap();
        let c = mpk_cert::decode_canonical_certificate(&bytes).unwrap();
        let mut observations = 0;
        for (width, signed, relation) in definitions {
            let mask = if width == 128 {
                u128::MAX
            } else {
                (1u128 << width) - 1
            };
            let sign = if width == 0 { 0 } else { 1u128 << (width - 1) };
            let mut cases = vec![0, 1 & mask, sign, sign.saturating_sub(1), mask];
            cases.sort();
            cases.dedup();
            let value = |n: u128| {
                let bits = (0..width.max(1).next_power_of_two())
                    .map(|i| i < width && n & (1u128 << i) != 0)
                    .collect::<Vec<_>>();
                if bits.len() == 1 {
                    V::Bit(bits[0])
                } else {
                    V::Cube(bits)
                }
            };
            for &left in &cases {
                for &right in &cases {
                    assert_eq!(
                        bit(run(&c, &relation.equal, vec![value(left), value(right)])),
                        left == right
                    );
                    let word = run(
                        &c,
                        relation.compare.as_ref().unwrap(),
                        vec![value(left), value(right)],
                    );
                    let actual = (0..32).fold(0u32, |n, i| {
                        let mut v = word.clone();
                        for selector in 0..5 {
                            v = apply(&c, v, V::Bit(i & (1 << selector) != 0));
                        }
                        n | ((bit(v) as u32) << i)
                    }) as i32;
                    let expected = if signed {
                        (left ^ sign).cmp(&(right ^ sign))
                    } else {
                        left.cmp(&right)
                    };
                    assert_eq!(actual.cmp(&0), expected, "{width} {signed} {left} {right}");
                    assert!((-1..=1).contains(&actual));
                    observations += 1;
                }
            }
        }
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../develop/migrations/csharp-03/ordinary-foundation/relations");
        let output = std::env::var_os("MPK_W09_RELATIONS_OUT").map(std::path::PathBuf::from);
        let metrics = serde_json::json!({"observations": observations, "terms": c.term_table.len(),
            "declarations": c.declarations.len(), "certificate_sha256": mpk_cert::hash_hex(&mpk_cert::certificate_hash(&bytes))});
        let hex = bytes.iter().map(|b| format!("{b:02x}")).collect::<String>() + "\n";
        if let Some(output) = output {
            std::fs::create_dir_all(&output).unwrap();
            std::fs::write(output.join("scalar-core.hex"), hex).unwrap();
            std::fs::write(
                output.join("scalar-core.json"),
                serde_json::to_vec_pretty(&metrics).unwrap(),
            )
            .unwrap();
        } else {
            assert_eq!(
                std::fs::read_to_string(root.join("scalar-core.hex")).unwrap(),
                hex
            );
            assert_eq!(
                serde_json::from_slice::<serde_json::Value>(
                    &std::fs::read(root.join("scalar-core.json")).unwrap()
                )
                .unwrap(),
                metrics
            );
        }
    }
}
