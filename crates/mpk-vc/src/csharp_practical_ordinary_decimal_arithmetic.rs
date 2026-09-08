//! Finite alignment, arithmetic and single-round fitting for decimal values.
use super::*;
const W: usize = 288;
const ALIGN: usize = 410;
const SCALE: usize = 490;
const DIVIDE: usize = 971;
const FIT: usize = 308;
pub(super) const OPS: &[&str] = &[
    "equal",
    "not_equal",
    "value_equality",
    "less",
    "less_equal",
    "greater",
    "greater_equal",
    "add",
    "subtract",
    "multiply",
    "divide",
    "remainder",
];
pub(super) fn operation(id: &str) -> Option<&str> {
    id.strip_prefix("decimal.").filter(|op| OPS.contains(op))
}
pub(super) fn comparison(op: &str) -> bool {
    !matches!(op, "add" | "subtract" | "multiply" | "divide" | "remainder")
}
struct Phase {
    circuit: IntegerCircuit,
    repeats: usize,
}
fn phase(id: &str, name: &str, c: Circuit, output: Word, repeats: usize) -> Phase {
    Phase {
        circuit: helper(id, name, c, output, vec![], vec![]),
        repeats,
    }
}
fn times_ten(c: &mut Circuit, a: &[Bit]) -> Word {
    let shift = |n| {
        std::iter::repeat_n(F, n)
            .chain(a[..a.len() - n].iter().copied())
            .collect::<Word>()
    };
    c.add(&shift(1), &shift(3), F).0
}
fn align(id: &str) -> Vec<Phase> {
    let mut c = Circuit::new(&[WIDTH, WIDTH]);
    let (an, as_, a) = read(&c.inputs[0]);
    let (bn, bs, b) = read(&c.inputs[1]);
    let less = c.lt(&as_, &bs, false);
    let scale = c.select(less, &bs, &as_);
    let ac = c.sub(&scale, &as_).0;
    let bc = c.sub(&scale, &bs).0;
    let initial = Circuit::extend(&a, 192, false)
        .into_iter()
        .chain(Circuit::extend(&b, 192, false))
        .chain(scale)
        .chain(ac)
        .chain(bc)
        .chain([an, bn])
        .collect();
    let initial = phase(id, "AlignInit", c, initial, 1);
    let mut c = Circuit::new(&[ALIGN]);
    let s = c.inputs[0].clone();
    let mut next = s.clone();
    for (start, count) in [(0, 392), (192, 400)] {
        let enabled = c.nonzero(&s[count..count + 8]);
        let value = times_ten(&mut c, &s[start..start + 192]);
        let value = c.select(enabled, &value, &s[start..start + 192]);
        next[start..start + 192].copy_from_slice(&value);
        let dec = c.sub(&s[count..count + 8], &literal(1, 8)).0;
        let dec = c.select(enabled, &dec, &s[count..count + 8]);
        next[count..count + 8].copy_from_slice(&dec);
    }
    vec![initial, phase(id, "AlignStep", c, next, 28)]
}
fn fit_state(
    n: &[Bit],
    scale: &[Bit],
    sign: Bit,
    sticky: Bit,
    above: Bit,
    tie: Bit,
    zero: Bit,
) -> Word {
    Circuit::extend(n, W, false)
        .into_iter()
        .chain(scale.iter().copied())
        .chain([sign])
        .chain([F; 4])
        .chain([sticky, above, tie, T, F, F, zero])
        .collect()
}
fn fit_step(id: &str) -> Phase {
    // q[0..288], scale[288..296], sign[296], digit[297..301],
    // sticky[301], initial-half flags[302..304], first/done/overflow/zero[304..308].
    let mut c = Circuit::new(&[FIT]);
    let s = c.inputs[0].clone();
    let above = c.lt(&literal(5, 4), &s[297..301], false);
    let tie = c.equal(&s[297..301], &literal(5, 4));
    let tail = c.or(s[301], s[0]);
    let tie_up = c.and(tie, tail);
    let reduced_up = c.or(above, tie_up);
    let initial_tie = c.and(s[303], s[0]);
    let initial_up = c.or(s[302], initial_tie);
    let up = c.mux(s[304], initial_up, reduced_up);
    let rounded = c.add(&s[..W], &vec![F; W], up).0;
    let too_wide = c.nonzero(&rounded[96..]);
    let scale_large = c.lt(&literal(28, 8), &s[288..296], false);
    let bad = c.or(too_wide, scale_large);
    let fits = c.not(bad);
    let has_scale = c.nonzero(&s[288..296]);
    let zero_scale = c.not(has_scale);
    let stop = c.or(fits, zero_scale);
    let active = c.not(s[305]);
    let accept = c.and(active, fits);
    let reject = c.and(active, bad);
    let reduce = c.and(reject, has_scale);
    let overflow = c.and(reject, zero_scale);
    let (q, digit) = divide_constant(&mut c, &s[..W], 10);
    let keep = c.select(accept, &rounded, &s[..W]);
    let value = c.select(reduce, &q, &keep);
    let mut next = s.clone();
    next[..W].copy_from_slice(&value);
    let scale = c.sub(&s[288..296], &literal(1, 8)).0;
    let scale = c.select(reduce, &scale, &s[288..296]);
    next[288..296].copy_from_slice(&scale);
    let digit = c.select(reduce, &digit[..4], &s[297..301]);
    next[297..301].copy_from_slice(&digit);
    let prior = c.nonzero(&s[297..301]);
    let sticky = c.or(s[301], prior);
    next[301] = c.mux(reduce, sticky, s[301]);
    next[304] = c.mux(reduce, F, s[304]);
    next[305] = c.or(s[305], stop);
    next[306] = c.or(s[306], overflow);
    phase(id, "FitStep", c, next, 57)
}
fn finish(id: &str) -> Phase {
    let signature = signature(id).unwrap();
    let c = Circuit::new(&[FIT]);
    let s = c.inputs[0].clone();
    let output = product(s[296], &s[288..296], &s[..96]);
    let failures = signature
        .ordered_checks
        .iter()
        .map(|check| match check.id.as_str() {
            "exception.division_by_zero" => s[307],
            "exception.overflow" => s[306],
            _ => unreachable!(),
        })
        .collect();
    Phase {
        circuit: helper(id, "Result", c, output, signature.ordered_checks, failures),
        repeats: 1,
    }
}
fn multiplication(id: &str) -> Vec<Phase> {
    // Accumulator/multiplicand/multiplier/count/scale/sign.
    const N: usize = 496;
    let mut c = Circuit::new(&[WIDTH, WIDTH]);
    let (an, as_, a) = read(&c.inputs[0]);
    let (bn, bs, b) = read(&c.inputs[1]);
    let scale = c.add(&as_, &bs, F).0;
    let sign = c.xor(an, bn);
    let initial = vec![F; 192]
        .into_iter()
        .chain(Circuit::extend(&a, 192, false))
        .chain(b)
        .chain(literal(96, 7))
        .chain(scale)
        .chain([sign])
        .collect();
    let initial = phase(id, "MultiplyInit", c, initial, 1);
    let mut c = Circuit::new(&[N]);
    let s = c.inputs[0].clone();
    let sum = c.add(&s[..192], &s[192..384], F).0;
    let sum = c.select(s[384], &sum, &s[..192]);
    let count = c.sub(&s[480..487], &literal(1, 7)).0;
    let next = sum
        .into_iter()
        .chain([F])
        .chain(s[192..383].iter().copied())
        .chain(s[385..480].iter().copied())
        .chain([F])
        .chain(count)
        .chain(s[487..].iter().copied())
        .collect::<Word>();
    let enabled = c.nonzero(&s[480..487]);
    let next = c.select(enabled, &next, &s);
    let step = phase(id, "MultiplyStep", c, next, 96);
    let c = Circuit::new(&[N]);
    let s = c.inputs[0].clone();
    let output = fit_state(&s[..192], &s[487..495], s[495], F, F, F, F);
    vec![initial, step, phase(id, "MultiplyFit", c, output, 1)]
}
fn division(id: &str, op: &str) -> Vec<Phase> {
    let mut phases = align(id);
    let mut c = Circuit::new(&[ALIGN]);
    let s = c.inputs[0].clone();
    let nonzero = c.nonzero(&s[192..384]);
    let zero = c.not(nonzero);
    let sign = if op == "divide" {
        c.xor(s[408], s[409])
    } else {
        s[408]
    };
    let scale = if op == "divide" {
        literal(28, 8)
    } else {
        s[384..392].to_vec()
    };
    let output = Circuit::extend(&s[..192], W, false)
        .into_iter()
        .chain(s[192..384].iter().copied())
        .chain(scale)
        .chain([sign, zero])
        .collect();
    phases.push(phase(id, "ScaleInit", c, output, 1));
    if op == "divide" {
        let mut c = Circuit::new(&[SCALE]);
        let s = c.inputs[0].clone();
        let scaled = times_ten(&mut c, &s[..W]);
        let out = scaled.into_iter().chain(s[W..].iter().copied()).collect();
        phases.push(phase(id, "ScaleStep", c, out, 28));
    }
    let c = Circuit::new(&[SCALE]);
    let s = c.inputs[0].clone();
    let output = s[..480]
        .iter()
        .copied()
        .chain(vec![F; 193 + W])
        .chain(s[480..].iter().copied())
        .collect();
    phases.push(phase(id, "DivideInit", c, output, 1));
    // Numerator[0..288], divisor[288..480], remainder[480..673],
    // quotient[673..961], scale/sign/zero[961..971].
    let mut c = Circuit::new(&[DIVIDE]);
    let s = c.inputs[0].clone();
    let r = std::iter::once(s[W - 1])
        .chain(s[480..672].iter().copied())
        .collect::<Word>();
    let divisor = Circuit::extend(&s[288..480], 193, false);
    let (delta, ge) = c.sub(&r, &divisor);
    let r = c.select(ge, &delta, &r);
    let out = std::iter::once(F)
        .chain(s[..W - 1].iter().copied())
        .chain(s[288..480].iter().copied())
        .chain(r)
        .chain([ge])
        .chain(s[673..960].iter().copied())
        .chain(s[961..].iter().copied())
        .collect();
    phases.push(phase(id, "DivideStep", c, out, 288));
    let mut c = Circuit::new(&[DIVIDE]);
    let s = c.inputs[0].clone();
    let out = if op == "divide" {
        let twice = std::iter::once(F)
            .chain(s[480..673].iter().copied())
            .collect::<Word>();
        let divisor = Circuit::extend(&s[288..480], 194, false);
        let above = c.lt(&divisor, &twice, false);
        let tie = c.equal(&divisor, &twice);
        let sticky = c.nonzero(&s[480..673]);
        fit_state(
            &s[673..961],
            &s[961..969],
            s[969],
            sticky,
            above,
            tie,
            s[970],
        )
    } else {
        fit_state(&s[480..673], &s[961..969], s[969], F, F, F, s[970])
    };
    phases.push(phase(id, "DivideFit", c, out, 1));
    phases
}
fn phases(id: &str) -> R<Vec<Phase>> {
    let op = operation(id).ok_or(OrdinaryCarrierError::Shape)?;
    let mut phases = if op == "multiply" {
        multiplication(id)
    } else if matches!(op, "divide" | "remainder") {
        division(id, op)
    } else {
        let mut phases = align(id);
        let mut c = Circuit::new(&[ALIGN]);
        let s = c.inputs[0].clone();
        let a = &s[..192];
        let b = &s[192..384];
        if comparison(op) {
            let eq = c.equal(a, b);
            let less = c.lt(a, b, false);
            let greater = c.lt(b, a, false);
            let signs = c.xor(s[408], s[409]);
            let any_a = c.nonzero(a);
            let any_b = c.nonzero(b);
            let any = c.or(any_a, any_b);
            let ordered = c.mux(s[408], greater, less);
            let lt = c.mux(signs, s[408], ordered);
            let lt = c.and(lt, any);
            let not_signs = c.not(signs);
            let zeros = c.not(any);
            let eq_signs = c.or(not_signs, zeros);
            let eq = c.and(eq, eq_signs);
            let output = match op {
                "equal" | "value_equality" => eq,
                "not_equal" => c.not(eq),
                "less" => lt,
                "less_equal" => c.or(lt, eq),
                "greater" => {
                    let le = c.or(lt, eq);
                    c.not(le)
                }
                "greater_equal" => c.not(lt),
                _ => unreachable!(),
            };
            phases.push(phase(id, "Comparison", c, vec![output], 1));
            return Ok(phases);
        }
        let bn = if op == "subtract" {
            c.not(s[409])
        } else {
            s[409]
        };
        let different_signs = c.xor(s[408], bn);
        let less = c.lt(a, b, false);
        let wide_a = Circuit::extend(a, 193, false);
        let wide_b = Circuit::extend(b, 193, false);
        let sum = c.add(&wide_a, &wide_b, F).0;
        let ab = c.sub(&wide_a, &wide_b).0;
        let ba = c.sub(&wide_b, &wide_a).0;
        let difference = c.select(less, &ba, &ab);
        let n = c.select(different_signs, &difference, &sum);
        let diff_sign = c.mux(less, bn, s[408]);
        let sign = c.mux(different_signs, diff_sign, s[408]);
        let out = fit_state(&n, &s[384..392], sign, F, F, F, F);
        phases.push(phase(id, "SumFit", c, out, 1));
        phases
    };
    phases.push(fit_step(id));
    phases.push(finish(id));
    Ok(phases)
}
pub(super) fn emit(b: &mut Builder, id: &str) -> R<OrdinaryScalarDefinition> {
    let signature = signature(id)?;
    let pipeline = phases(id)?;
    let widths = signature
        .argument_type_ids
        .iter()
        .map(|s| width(s))
        .collect::<R<Vec<_>>>()?;
    let mut value = None;
    let mut definitions = vec![];
    let mut extra = 0;
    let last = pipeline.len() - 1;
    for (index, phase) in pipeline.into_iter().enumerate() {
        let input_widths = phase
            .circuit
            .circuit
            .inputs
            .iter()
            .map(Vec::len)
            .collect::<Vec<_>>();
        let output_width = phase.circuit.output.len();
        let d = emit_shared(b, phase.circuit, "DecimalArithmetic")?;
        let f = b.constant(&d.result_definition)?;
        if index == last {
            if phase.repeats != 1
                || input_widths.len() != 1
                || output_width != width(&signature.normal_result_type_id)?
            {
                return Err(OrdinaryCarrierError::Shape);
            }
            definitions.push(d);
            break;
        }

        if value.is_none() {
            let args = (0..widths.len())
                .rev()
                .map(|i| b.var(i as u32))
                .collect::<R<Vec<_>>>()?;
            value = Some(b.app(f, args)?);
        } else if phase.repeats > 1 {
            if input_widths != vec![output_width] {
                return Err(OrdinaryCarrierError::Shape);
            }
            let depth = address_bits(output_width as u32);
            if !b
                .globals
                .contains_key(&format!("{PREFIX}.Cube.D{depth}.Compose"))
            {
                b.helpers(depth)?;
            }
            let f = b.compose(depth, &vec![f; phase.repeats])?;
            extra += phase.repeats;
            value = Some(b.app(f, vec![value.unwrap()])?);
        } else {
            value = Some(b.app(f, vec![value.unwrap()])?);
        }
        definitions.push(d);
    }
    let state = value.unwrap();
    let finalizer = definitions.last().unwrap();
    let name = format!(
        "{PREFIX}.Decimal.O{}",
        id.as_bytes()
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect::<String>()
    );
    let result_definition = format!("{name}.Result");
    let success_definition = format!("{name}.Success");
    let failures = (0..signature.ordered_checks.len())
        .map(|i| format!("{name}.Failure.F{i}"))
        .collect::<Vec<_>>();
    let outputs = std::iter::once((
        &result_definition,
        &finalizer.result_definition,
        width(&signature.normal_result_type_id)?,
    ))
    .chain(std::iter::once((
        &success_definition,
        &finalizer.success_definition,
        1,
    )))
    .chain(
        failures
            .iter()
            .zip(&finalizer.ordered_failure_definitions)
            .map(|(a, b)| (a, b, 1)),
    );
    for (name, helper, output_width) in outputs {
        let f = b.constant(helper)?;
        let result = b.app(f, vec![state])?;
        let body = bind_inputs(b, &widths, result)?;
        let ty = b.cube(address_bits(output_width as u32))?;
        let ty = input_type(b, &widths, ty)?;
        b.define(name, ty, body)?;
    }
    Ok(OrdinaryScalarDefinition {
        operation: signature,
        result_definition,
        success_definition,
        ordered_failure_definitions: failures,
        boolean_gates: definitions.iter().map(|d| d.boolean_gates).sum(),
        state_depth: definitions.iter().map(|d| d.state_depth).max().unwrap(),
        static_transformers: definitions
            .iter()
            .map(|d| d.static_transformers)
            .sum::<usize>()
            + extra,
    })
}

#[cfg(test)]
mod tests {
    use super::super::tests::{decimal as value, observe, physical, prune};
    use super::*;
    const MAX: u128 = (1u128 << 96) - 1;
    const FIXTURES: &[&str] = &[
        "equal",
        "less",
        "add",
        "subtract",
        "multiply",
        "divide",
        "remainder",
    ];
    fn samples() -> Vec<Vec<MonomorphicValue>> {
        let mut values = vec![];
        for (n, scale) in [
            (0, 0),
            (0, 28),
            (1, 0),
            (1, 1),
            (1, 28),
            (10, 1),
            (2, 0),
            (3, 0),
            (5, 0),
            (15, 1),
            (25, 1),
            (MAX, 0),
            (MAX, 1),
            (MAX, 28),
            (MAX - 1, 0),
            (MAX - 5, 2),
            (10u128.pow(28), 28),
        ] {
            for sign in [false, true] {
                values.push(value(sign, n, scale));
            }
        }
        let mut cases = values
            .iter()
            .flat_map(|a| values.iter().map(move |b| vec![a.clone(), b.clone()]))
            .collect::<Vec<_>>();
        for a in 0..=28 {
            for b in 0..=28 {
                cases.push(vec![
                    value(a % 2 != 0, if a % 3 == 0 { MAX } else { 1 }, a),
                    value(b % 2 != 0, if b % 3 == 0 { MAX } else { 3 }, b),
                ]);
            }
        }
        let mut seed = 0x826db11a7c543e91u64;
        let mut next = || {
            seed ^= seed << 13;
            seed ^= seed >> 7;
            seed ^= seed << 17;
            seed
        };
        for _ in 0..512 {
            let a = (u128::from(next()) << 32 | u128::from(next() as u32)) & MAX;
            let b = (u128::from(next()) << 32 | u128::from(next() as u32)) & MAX;
            let shape = next();
            cases.push(vec![
                value(shape & 1 != 0, a, (shape % 29) as u8),
                value(shape & 2 != 0, b, ((shape >> 8) % 29) as u8),
            ]);
        }
        cases
    }
    #[test]
    fn decimal_arithmetic_circuits_match_numeric_oracle() {
        let bundle = validate_registered_foundation_bundle(
            registered_foundation_descriptor_transport(),
            registered_foundation_definitions_transport(),
        )
        .unwrap();
        let roots=serde_json::json!(["decimal","bool"].map(|id|serde_json::json!({"origin":"semantic_binding","provenance_id":format!("decimal.arithmetic.{id}"),"type":{"kind":"primitive","id":id}})));
        let bytes =
            canonical_closed_root_set_transport(&bundle, &roots, &serde_json::json!({})).unwrap();
        let roots = validate_closed_root_set(&bundle, &bytes).unwrap();
        let closed = derive_closed_instances(&bundle, &roots).unwrap();
        let cases = samples();
        let mut count = 0;
        for op in OPS {
            let id = format!("decimal.{op}");
            let signature = signature(&id).unwrap();
            let recipe = NumericOperation::new(
                &id,
                &signature.argument_type_ids,
                &signature.normal_result_type_id,
                None,
            )
            .unwrap();
            let phases = phases(&id)
                .unwrap()
                .into_iter()
                .map(|p| (prune(p.circuit), p.repeats))
                .collect::<Vec<_>>();
            for batch in cases.chunks(64) {
                let mut inputs = vec![vec![0u64; WIDTH]; 2];
                for (lane, args) in batch.iter().enumerate() {
                    for (i, v) in args.iter().enumerate() {
                        for (j, on) in physical(v).iter().enumerate() {
                            if *on {
                                inputs[i][j] |= 1 << lane;
                            }
                        }
                    }
                }
                let mut failures = vec![];
                for (p, repeats) in &phases {
                    for _ in 0..*repeats {
                        let (result, errors) = observe(p, &inputs);
                        inputs = vec![result];
                        failures = errors;
                    }
                    if ["AlignStep", "MultiplyStep", "FitStep"]
                        .iter()
                        .any(|s| p.signature.id.ends_with(s))
                    {
                        assert_eq!(
                            observe(p, &inputs).0,
                            inputs[0],
                            "terminal identity {id} {}",
                            p.signature.id
                        );
                    }
                    if p.signature.id.ends_with("DivideStep") {
                        assert!(inputs[0][..W].iter().all(|&b| b == 0));
                    }
                }
                for (lane, args) in batch.iter().enumerate() {
                    let first = failures.iter().position(|bits| bits & (1 << lane) != 0);
                    match recipe.evaluate(&bundle, &roots, &closed, args) {
                        Ok(result) => {
                            assert!(first.is_none(), "{id} {args:?}");
                            let expected = if let MonomorphicValue::Bool { value, .. } = result {
                                vec![value]
                            } else {
                                physical(&result)
                            };
                            assert_eq!(inputs[0].len(), expected.len());
                            for (bit, on) in expected.iter().enumerate() {
                                assert_eq!(
                                    inputs[0][bit] & (1 << lane) != 0,
                                    *on,
                                    "{id} {args:?} bit {bit}"
                                );
                            }
                        }
                        Err(e) => {
                            assert!(matches!(
                                e,
                                NumericError::Overflow | NumericError::DivideByZero
                            ));
                            let first =
                                first.unwrap_or_else(|| panic!("missing {e:?}: {id} {args:?}"));
                            assert_eq!(
                                signature.ordered_checks[first].failure_type_id.as_deref(),
                                e.exception_type()
                            );
                        }
                    }
                    count += 1;
                }
            }
        }
        eprintln!("decimal arithmetic oracle cases: {count}");
    }
    #[test]
    fn decimal_arithmetic_circuits_emit_all_signatures() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(
            "../../develop/migrations/csharp-03/ordinary-foundation/decimal-arithmetic-circuits",
        );
        let out = std::env::var_os("MPK_W09_DECIMAL_ARITHMETIC_OUT").map(std::path::PathBuf::from);
        if let Some(dir) = &out {
            std::fs::create_dir_all(dir).unwrap();
        }
        let mut metrics = vec![];
        for op in OPS {
            let id = format!("decimal.{op}");
            let mut b = Builder::new().unwrap();
            let definition = emit_decimal(&mut b, &id).unwrap_or_else(|e| panic!("{id}: {e:?}"));
            let bytes = b.finish().unwrap_or_else(|e| panic!("{id}: {e:?}"));
            let cert = decode_canonical_certificate(&bytes).unwrap();
            metrics.push(serde_json::json!({"id":id,"definition":definition,"terms":cert.term_table.len(),"declarations":cert.declarations.len(),"hash":mpk_cert::hash_hex(&mpk_cert::certificate_hash(&bytes))}));
            if FIXTURES.contains(op) {
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
    fn decimal_arithmetic_circuits_core_results_and_priority() {
        std::thread::Builder::new()
            .stack_size(64 * 1024 * 1024)
            .spawn(|| {
                use super::super::super::super::tests::{apply, bit, run, V};
                let mut ids = super::super::tests::ids();
                ids.extend(OPS.iter().map(|op| format!("decimal.{op}")));
                ids.sort();
                let mut b = Builder::new().unwrap();
                let mut definitions = BTreeMap::new();
                for id in ids {
                    definitions.insert(id.clone(), emit_decimal(&mut b, &id).unwrap());
                }
                let cert = decode_canonical_certificate(&b.finish().unwrap()).unwrap();
                let boolean = |v| MonomorphicValue::Bool {
                    type_id: ty("bool"),
                    value: v,
                };
                for (op, cases) in [
                    (
                        "round.ToEven.2",
                        vec![(
                            vec![
                                value(false, 25, 1),
                                MonomorphicValue::Signed {
                                    type_id: ty("i32"),
                                    value: "0".into(),
                                },
                            ],
                            Some(value(false, 2, 0)),
                            None,
                        )],
                    ),
                    (
                        "round.AwayFromZero.2",
                        vec![(
                            vec![
                                value(false, 25, 1),
                                MonomorphicValue::Signed {
                                    type_id: ty("i32"),
                                    value: "0".into(),
                                },
                            ],
                            Some(value(false, 3, 0)),
                            None,
                        )],
                    ),
                    (
                        "conversion.decimal_to_int64",
                        vec![(
                            vec![value(true, 1, 0)],
                            Some(MonomorphicValue::Signed {
                                type_id: ty("i64"),
                                value: "-1".into(),
                            }),
                            None,
                        )],
                    ),
                    (
                        "conversion.decimal_to_uint64",
                        vec![(vec![value(true, 1, 0)], None, Some(0))],
                    ),
                    (
                        "equal",
                        vec![(
                            vec![value(false, 0, 0), value(true, 0, 28)],
                            Some(boolean(true)),
                            None,
                        )],
                    ),
                    (
                        "less",
                        vec![
                            (
                                vec![value(true, 1, 0), value(false, 0, 0)],
                                Some(boolean(true)),
                                None,
                            ),
                            (
                                vec![value(false, 1, 0), value(false, 999, 3)],
                                Some(boolean(false)),
                                None,
                            ),
                        ],
                    ),
                    (
                        "add",
                        vec![
                            (
                                vec![value(false, 1, 1), value(false, 2, 1)],
                                Some(value(false, 3, 1)),
                                None,
                            ),
                            (
                                vec![value(false, MAX, 0), value(false, 1, 0)],
                                None,
                                Some(0),
                            ),
                            (
                                vec![value(true, 0, 0), value(false, 0, 1)],
                                Some(value(true, 0, 1)),
                                None,
                            ),
                        ],
                    ),
                    (
                        "subtract",
                        vec![(
                            vec![value(false, 1, 0), value(false, 5, 1)],
                            Some(value(false, 5, 1)),
                            None,
                        )],
                    ),
                    (
                        "multiply",
                        vec![
                            (
                                vec![value(false, 125, 2), value(false, 2, 0)],
                                Some(value(false, 250, 2)),
                                None,
                            ),
                            (
                                vec![value(false, 1, 28), value(false, 1, 28)],
                                Some(value(false, 0, 28)),
                                None,
                            ),
                        ],
                    ),
                    (
                        "divide",
                        vec![
                            (
                                vec![value(false, 1, 0), value(false, 2, 0)],
                                Some(value(false, 5 * 10u128.pow(27), 28)),
                                None,
                            ),
                            (
                                vec![value(false, MAX, 0), value(false, 0, 0)],
                                None,
                                Some(0),
                            ),
                        ],
                    ),
                    (
                        "remainder",
                        vec![(
                            vec![value(true, 55, 1), value(false, 2, 0)],
                            Some(value(true, 15, 1)),
                            None,
                        )],
                    ),
                ] {
                    let id = format!("decimal.{op}");
                    let d = &definitions[&id];
                    for (args, expected, error) in cases {
                        let args = args
                            .iter()
                            .map(|v| V::Cube(physical(v)))
                            .collect::<Vec<_>>();
                        assert_eq!(
                            bit(run(&cert, &d.success_definition, args.clone())),
                            expected.is_some(),
                            "{id}"
                        );
                        for (i, failure) in d.ordered_failure_definitions.iter().enumerate() {
                            assert_eq!(
                                bit(run(&cert, failure, args.clone())),
                                error == Some(i),
                                "{id} failure {i}"
                            );
                        }
                        let result = run(&cert, &d.result_definition, args);
                        let w = width(&d.operation.normal_result_type_id).unwrap();
                        let expected = expected
                            .map(|v| {
                                if let MonomorphicValue::Bool { value, .. } = v {
                                    vec![value]
                                } else {
                                    physical(&v)
                                }
                            })
                            .unwrap_or_else(|| vec![false; w]);
                        if w == 1 {
                            assert_eq!(bit(result), expected[0]);
                        } else {
                            for (i, on) in expected.iter().enumerate() {
                                let mut v = result.clone();
                                for j in 0..address_bits(w as u32) {
                                    v = apply(&cert, v, V::Bit(i & (1 << j) != 0));
                                }
                                assert_eq!(bit(v), *on, "{id} bit {i}");
                            }
                        }
                    }
                    eprintln!("core decimal operation passed: {op}");
                }
            })
            .unwrap()
            .join()
            .unwrap();
    }
    #[test]
    fn decimal_arithmetic_circuits_all_45_share_finite_helpers() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(
            "../../develop/migrations/csharp-03/ordinary-foundation/decimal-arithmetic-circuits",
        );
        let out = std::env::var_os("MPK_W09_DECIMAL_ARITHMETIC_OUT").map(std::path::PathBuf::from);
        if let Some(dir) = &out {
            std::fs::create_dir_all(dir).unwrap();
        }
        let mut ids = super::super::tests::ids();
        ids.extend(OPS.iter().map(|op| format!("decimal.{op}")));
        ids.sort();
        assert_eq!(ids.len(), 45);
        let mut b = Builder::new().unwrap();
        let mut definitions = vec![];
        for id in ids {
            definitions.push(emit_decimal(&mut b, &id).unwrap_or_else(|e| panic!("{id}: {e:?}")));
        }
        assert_eq!(
            definitions
                .iter()
                .map(|d| &d.result_definition)
                .collect::<BTreeSet<_>>()
                .len(),
            45
        );
        let transformers = b.static_transformers;
        let shared_helpers = b.shared_scalar_circuits.len();
        let bytes = b.finish().unwrap();
        let cert = decode_canonical_certificate(&bytes).unwrap();
        let metrics = serde_json::json!({"definitions":definitions,"terms":cert.term_table.len(),"declarations":cert.declarations.len(),"static_transformers":transformers,"shared_helpers":shared_helpers,"hash":mpk_cert::hash_hex(&mpk_cert::certificate_hash(&bytes))});
        let hex = bytes.iter().map(|b| format!("{b:02x}")).collect::<String>() + "\n";
        if let Some(dir) = &out {
            std::fs::write(dir.join("decimal.all_operations.hex"), hex).unwrap();
            std::fs::write(
                dir.join("all-operations.json"),
                serde_json::to_vec_pretty(&metrics).unwrap(),
            )
            .unwrap();
        } else {
            assert_eq!(
                std::fs::read_to_string(root.join("decimal.all_operations.hex")).unwrap(),
                hex
            );
            let expected: Value =
                serde_json::from_slice(&std::fs::read(root.join("all-operations.json")).unwrap())
                    .unwrap();
            assert_eq!(metrics, expected);
        }
        eprintln!("all 45 decimal definitions: terms={}, declarations={}, transformers={transformers}, shared helpers={shared_helpers}",cert.term_table.len(),cert.declarations.len());
    }
}
