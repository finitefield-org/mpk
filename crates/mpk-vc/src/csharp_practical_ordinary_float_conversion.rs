//! The six frozen numeric conversions, using ordinary finite Boolean circuits.
use super::*;
const IDS: &[(&str, &str, &str)] = &[
    ("int32_to_single", "i32", "f32"),
    ("int64_to_double", "i64", "f64"),
    ("single_to_double", "f32", "f64"),
    ("double_to_single", "f64", "f32"),
    ("single_to_int32.checked", "f32", "i32"),
    ("double_to_int64.checked", "f64", "i64"),
];
fn format(token: &str) -> Option<Format> {
    match token {
        "f32" => Some(Format { f: 23, e: 8 }),
        "f64" => Some(Format { f: 52, e: 11 }),
        _ => None,
    }
}
pub(super) fn signature(id: &str) -> R<ClosedOperationSignature> {
    let suffix = id
        .strip_prefix("numeric.conversion.")
        .ok_or(OrdinaryCarrierError::Shape)?;
    let &(_, from, to) = IDS
        .iter()
        .find(|(s, _, _)| *s == suffix)
        .ok_or(OrdinaryCarrierError::Shape)?;
    let ty = |s| format!("mpk.csharp.value.{s}.v1");
    let args = vec![ty(from)];
    let result = ty(to);
    let recipe =
        NumericOperation::new(id, &args, &result, None).map_err(|_| OrdinaryCarrierError::Shape)?;
    let checks = recipe
        .exception_types()
        .into_iter()
        .map(|exception| {
            if exception != "System.OverflowException" {
                return Err(OrdinaryCarrierError::Shape);
            }
            Ok(RequiredCheck {
                id: "exception.overflow".into(),
                tag: RequiredCheckTag::Exception,
                failure_type_id: Some(exception.into()),
            })
        })
        .collect::<R<Vec<_>>>()?;
    Ok(ClosedOperationSignature {
        id: id.into(),
        tag: ClosedOperationTag::Data,
        argument_type_ids: args,
        normal_result_type_id: result,
        ordered_checks: checks,
    })
}
// Unlike source integer shifts, conversion shifts saturate instead of masking
// the count. Every count bit participates, including huge float exponents.
fn shift(c: &mut Circuit, a: &[Bit], count: &[Bit], right: bool) -> Word {
    let mut x = a.to_vec();
    for (i, &take) in count.iter().enumerate() {
        let k = 1usize << i;
        let y = (0..x.len())
            .map(|j| {
                if right {
                    x.get(j + k).copied().unwrap_or(F)
                } else {
                    j.checked_sub(k).map_or(F, |n| x[n])
                }
            })
            .collect::<Word>();
        x = c.select(take, &y, &x);
    }
    x
}
pub(super) fn circuit(id: &str) -> R<IntegerCircuit> {
    let signature = signature(id)?;
    let suffix = id.strip_prefix("numeric.conversion.").unwrap();
    let &(_, from, to) = IDS.iter().find(|(s, _, _)| *s == suffix).unwrap();
    let width = |s| if matches!(s, "i32" | "f32") { 32 } else { 64 };
    let mut c = Circuit::new(&[width(from)]);
    let raw = c.inputs[0].clone();
    let mut failures = vec![];
    let output = match (format(from), format(to)) {
        (None, Some(target)) => {
            let sign = *raw.last().unwrap();
            let negative = c.neg(&raw);
            let magnitude = c.select(sign, &negative, &raw);
            pack(&mut c, target, sign, &magnitude, &exp(0))
        }
        (Some(source), Some(target)) => {
            let p = parts(&mut c, &raw, source);
            let finite = pack(&mut c, target, p.sign, &p.sig, &p.exponent);
            let mut special = target.infinity(p.sign);
            let payload = if target.f > source.f {
                std::iter::repeat_n(F, target.f - source.f)
                    .chain(raw[..source.f].iter().copied())
                    .collect::<Word>()
            } else {
                raw[source.f - target.f..source.f].to_vec()
            };
            special[..target.f].copy_from_slice(&payload);
            special[target.f - 1] = c.or(special[target.f - 1], p.nan);
            c.select(p.special, &special, &finite)
        }
        (Some(source), None) => {
            let p = parts(&mut c, &raw, source);
            let w = width(to);
            let a = Circuit::extend(&p.sig, w + 1, false);
            let negative_exp = p.exponent[EW - 1];
            let opposite = c.neg(&p.exponent);
            let left = shift(&mut c, &a, &p.exponent, false);
            let right = shift(&mut c, &a, &opposite, true);
            let magnitude = c.select(negative_exp, &right, &left);
            // A saturated left shift may discard all ones; independently bound
            // the highest set bit before comparing the retained magnitude.
            let (_, leading) = normalize(&mut c, &p.sig);
            let top = add_exp(&mut c, &p.exponent, source.f as i32);
            let top = c.sub(&top, &leading).0;
            let below_width = c.lt(&top, &exp(w as i32), true);
            let too_large = c.not(below_width);
            let nonzero = c.not(p.zero);
            let too_large = c.and(too_large, nonzero);
            let maximum = literal((1u128 << (w - 1)) - 1, w + 1);
            let minimum_magnitude = literal(1u128 << (w - 1), w + 1);
            let limit = c.select(p.sign, &minimum_magnitude, &maximum);
            let exceeds = c.lt(&limit, &magnitude, false);
            let exceeds = c.or(exceeds, too_large);
            failures.push(c.or(p.special, exceeds));
            let negative = c.neg(&magnitude[..w]);
            c.select(p.sign, &negative, &magnitude[..w])
        }
        _ => return Err(OrdinaryCarrierError::Shape),
    };
    Ok(IntegerCircuit {
        signature,
        circuit: c,
        output,
        failures,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    fn value(token: &str, raw: u128) -> MonomorphicValue {
        let type_id = format!("mpk.csharp.value.{token}.v1");
        match token {
            "f32" => MonomorphicValue::F32Bits {
                type_id,
                bits: format!("{raw:08x}"),
            },
            "f64" => MonomorphicValue::F64Bits {
                type_id,
                bits: format!("{raw:016x}"),
            },
            "i32" => MonomorphicValue::Signed {
                type_id,
                value: (raw as i32).to_string(),
            },
            "i64" => MonomorphicValue::Signed {
                type_id,
                value: (raw as i64).to_string(),
            },
            _ => unreachable!(),
        }
    }
    fn raw(value: MonomorphicValue, width: usize) -> u128 {
        let bits = match value {
            MonomorphicValue::F32Bits { bits, .. } | MonomorphicValue::F64Bits { bits, .. } => {
                u128::from_str_radix(&bits, 16).unwrap()
            }
            MonomorphicValue::Signed { value, .. } => value.parse::<i128>().unwrap() as u128,
            _ => unreachable!(),
        };
        bits & ((1u128 << width) - 1)
    }
    fn samples(token: &str) -> Vec<u128> {
        let width = if matches!(token, "f32" | "i32") {
            32
        } else {
            64
        };
        let mask = (1u128 << width) - 1;
        let mut samples = vec![0, 1, mask, mask >> 1, 1 << (width - 1)];
        if let Some(f) = format(token) {
            for e in 0..1u128 << f.e {
                for fraction in [0, 1, 1 << (f.f - 1), (1 << f.f) - 1] {
                    for sign in [0, 1 << (width - 1)] {
                        samples.push(sign | (e << f.f) | fraction);
                    }
                }
            }
            // Neighbours of integer conversion limits and float32 rounding,
            // underflow, overflow and subnormal-to-normal halfway points.
            let pivots: &[u128] = if width == 32 {
                &[0x4f000000, 0xcf000000, 0x3f800000, 0xbf800000]
            } else {
                &[
                    0x43e0000000000000,
                    0xc3e0000000000000,
                    0x3ff0000010000000,
                    0x3ff0000030000000,
                    0x3690000000000000,
                    0x380fffffe0000000,
                    0x47effffff0000000,
                ]
            };
            for &pivot in pivots {
                for offset in -4i128..=4 {
                    samples.push((pivot as i128 + offset) as u128);
                }
            }
        } else {
            for bit in 0..width {
                for offset in -4i128..=4 {
                    let n = (1i128 << bit) + offset;
                    samples.push(n as u128 & mask);
                    samples.push((-n) as u128 & mask);
                }
            }
        }
        let mut seed = 0x734eda179036fd81u64;
        for _ in 0..4096 {
            seed ^= seed << 13;
            seed ^= seed >> 7;
            seed ^= seed << 17;
            samples.push(seed as u128 & mask);
        }
        samples
    }
    #[test]
    fn floating_conversion_circuits_match_numeric_oracle() {
        let bundle = validate_registered_foundation_bundle(
            registered_foundation_descriptor_transport(),
            registered_foundation_definitions_transport(),
        )
        .unwrap();
        let roots = serde_json::json!(["i32", "i64", "f32", "f64"].map(|id| serde_json::json!({"origin":"semantic_binding","provenance_id":format!("conversion.{id}"),"type":{"kind":"primitive","id":id}})));
        let bytes =
            canonical_closed_root_set_transport(&bundle, &roots, &serde_json::json!({})).unwrap();
        let roots = validate_closed_root_set(&bundle, &bytes).unwrap();
        let closed = derive_closed_instances(&bundle, &roots).unwrap();
        let mut count = 0;
        for &(suffix, from, to) in IDS {
            let id = format!("numeric.conversion.{suffix}");
            let mut p = circuit(&id).unwrap();
            let mut outputs = p
                .output
                .iter()
                .chain(&p.failures)
                .copied()
                .collect::<Vec<_>>();
            p.circuit.prune(&mut outputs);
            let (output, failures) = outputs.split_at(p.output.len());
            let recipe = NumericOperation::new(
                &id,
                &p.signature.argument_type_ids,
                &p.signature.normal_result_type_id,
                None,
            )
            .unwrap();
            let mut successes = 0;
            let mut overflows = 0;
            for input in samples(from) {
                let observed = p.circuit.evaluate(&[input]);
                let failure = failures.iter().any(|&bit| observed[bit]);
                let result = output
                    .iter()
                    .enumerate()
                    .fold(0u128, |v, (i, &b)| v | (u128::from(observed[b]) << i));
                match recipe.evaluate(&bundle, &roots, &closed, &[value(from, input)]) {
                    Ok(expected) => {
                        successes += 1;
                        assert!(!failure, "unexpected overflow {id}({input:x})");
                        assert_eq!(
                            result,
                            raw(expected, output.len()),
                            "{id}({input:x}) -> {to}"
                        );
                    }
                    Err(e) => {
                        assert_eq!(e, NumericError::Overflow);
                        overflows += 1;
                        assert!(failure, "missing overflow {id}({input:x})");
                    }
                }
                count += 1;
            }
            assert!(successes > 0);
            assert_eq!(overflows > 0, suffix.ends_with(".checked"));
        }
        eprintln!("numeric conversion oracle evaluations: {count}");
    }
    #[test]
    fn floating_conversion_circuits_emit_all_signatures() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../develop/migrations/csharp-03/ordinary-foundation/conversion-circuits");
        let out = std::env::var_os("MPK_W09_CONVERSION_OUT").map(std::path::PathBuf::from);
        if let Some(dir) = &out {
            std::fs::create_dir_all(dir).unwrap();
        }
        let mut metrics = vec![];
        for &(suffix, _, _) in IDS {
            let id = format!("numeric.conversion.{suffix}");
            let mut b = Builder::new().unwrap();
            let definition = emit_floating(&mut b, &id).unwrap();
            let bytes = b.finish().unwrap();
            let cert = decode_canonical_certificate(&bytes).unwrap();
            metrics.push(serde_json::json!({"id": id, "definition": definition, "terms": cert.term_table.len(), "declarations":cert.declarations.len(), "hash":mpk_cert::hash_hex(&mpk_cert::certificate_hash(&bytes))}));
            let hex = bytes.iter().map(|b| format!("{b:02x}")).collect::<String>() + "\n";
            if let Some(dir) = &out {
                std::fs::write(dir.join(format!("{id}.hex")), hex).unwrap();
            } else {
                assert_eq!(
                    std::fs::read_to_string(root.join(format!("{id}.hex"))).unwrap(),
                    hex,
                    "{id}"
                );
            }
        }
        if let Some(dir) = &out {
            std::fs::write(
                dir.join("metrics.json"),
                serde_json::to_vec_pretty(&metrics).unwrap(),
            )
            .unwrap();
        } else {
            let expected: Value =
                serde_json::from_slice(&std::fs::read(root.join("metrics.json")).unwrap()).unwrap();
            assert_eq!(serde_json::json!(metrics), expected);
        }
    }
    #[test]
    fn floating_conversion_circuits_core_success_and_overflow() {
        std::thread::Builder::new()
            .stack_size(64 * 1024 * 1024)
            .spawn(|| {
                use super::super::super::super::tests::{apply, bit, run, V};
                for (suffix, cases) in [
                    (
                        "int32_to_single",
                        vec![
                            (0x01000001u128, Some(0x4b800000u128)),
                            (0x01000003, Some(0x4b800002)),
                            (0x80000000, Some(0xcf000000)),
                        ],
                    ),
                    (
                        "int64_to_double",
                        vec![
                            (0x0020000000000001, Some(0x4340000000000000)),
                            (0x0020000000000003, Some(0x4340000000000002)),
                            (0x8000000000000000, Some(0xc3e0000000000000)),
                        ],
                    ),
                    (
                        "single_to_double",
                        vec![
                            (0x80000000, Some(0x8000000000000000)),
                            (1, Some(0x36a0000000000000)),
                            (0xff800001, Some(0xfff8000020000000)),
                        ],
                    ),
                    (
                        "double_to_single",
                        vec![
                            (0x3690000000000000, Some(0)),
                            (0x3690000000000001, Some(1)),
                            (0x7ff0000000000001, Some(0x7fc00000)),
                        ],
                    ),
                    (
                        "single_to_int32.checked",
                        vec![
                            (0xcf000000, Some(0x80000000)),
                            (0x4f000000, None),
                            (0xbff00000, Some(0xffffffff)),
                            (0x7f800001, None),
                        ],
                    ),
                    (
                        "double_to_int64.checked",
                        vec![
                            (0xc3e0000000000000, Some(0x8000000000000000)),
                            (0x43e0000000000000, None),
                            (0x7fefffffffffffff, None),
                            (0xbffe000000000000, Some(0xffffffffffffffff)),
                        ],
                    ),
                ] {
                    let id = format!("numeric.conversion.{suffix}");
                    let p = circuit(&id).unwrap();
                    let width = p.output.len();
                    let input_width = p.circuit.inputs[0].len();
                    let mut b = Builder::new().unwrap();
                    let d = emit_floating(&mut b, &id).unwrap();
                    let cert = decode_canonical_certificate(&b.finish().unwrap()).unwrap();
                    for (input, expected) in cases {
                        let args = vec![V::Cube(
                            (0..input_width).map(|i| input & (1 << i) != 0).collect(),
                        )];
                        assert_eq!(
                            bit(run(&cert, &d.success_definition, args.clone())),
                            expected.is_some(),
                            "{id}({input:x})"
                        );
                        for name in &d.ordered_failure_definitions {
                            assert_eq!(bit(run(&cert, name, args.clone())), expected.is_none());
                        }
                        let value = run(&cert, &d.result_definition, args);
                        for i in 0..width {
                            let mut v = value.clone();
                            for j in 0..address_bits(width as u32) {
                                v = apply(&cert, v, V::Bit(i & (1 << j) != 0));
                            }
                            assert_eq!(
                                bit(v),
                                expected.unwrap_or(0) & (1 << i) != 0,
                                "{id}({input:x}) bit {i}"
                            );
                        }
                    }
                }
            })
            .unwrap()
            .join()
            .unwrap();
    }
}
