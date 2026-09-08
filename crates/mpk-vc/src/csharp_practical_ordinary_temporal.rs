//! Finite Time, Duration and Instant circuits; no runtime observation is a proof.
use super::*;
const DAY: u128 = 864_000_000_000;
pub(super) fn literal(n: u128, width: usize) -> Word {
    (0..width)
        .map(|i| if n & (1 << i) == 0 { F } else { T })
        .collect()
}
// A constant-divisor restoring circuit needs only divisor_bits + 1 remainder
// bits. No lookup table or host evaluation of an input participates in emission.
pub(super) fn divide_constant(c: &mut Circuit, a: &[Bit], divisor: u128) -> (Word, Word) {
    assert!(divisor > 0);
    let width = (128 - divisor.leading_zeros()) as usize + 1;
    let d = literal(divisor, width);
    let mut r = vec![F; width];
    let mut q = vec![F; a.len()];
    for i in (0..a.len()).rev() {
        r.insert(0, a[i]);
        r.pop();
        let (delta, carry) = c.sub(&r, &d);
        // Subtraction returns the no-borrow carry.
        r = c.select(carry, &delta, &r);
        q[i] = carry;
    }
    (q, r)
}
fn signed_divide_constant(c: &mut Circuit, a: &[Bit], d: u128) -> (Word, Word) {
    let sign = *a.last().unwrap();
    let neg = c.neg(a);
    let mag = c.select(sign, &neg, a);
    let (q, r) = divide_constant(c, &mag, d);
    let nq = c.neg(&q);
    let nr = c.neg(&r);
    (c.select(sign, &nq, &q), c.select(sign, &nr, &r))
}
fn euclidean_day(c: &mut Circuit, a: &[Bit]) -> Word {
    let sign = *a.last().unwrap();
    let neg = c.neg(a);
    let mag = c.select(sign, &neg, a);
    let (_, r) = divide_constant(c, &mag, DAY);
    let nonzero = c.nonzero(&r);
    let negative = c.and(sign, nonzero);
    let correction = c.sub(&literal(DAY, r.len()), &r).0;
    let r = c.select(negative, &correction, &r);
    Circuit::extend(&r, 64, false)
}
fn overflows_i64(c: &mut Circuit, wide: &[Bit]) -> Bit {
    let extended = Circuit::extend(&wide[..64], wide.len(), true);
    let equal = c.equal(wide, &extended);
    c.not(equal)
}
fn temporal_signature(id: &str) -> R<ClosedOperationSignature> {
    let (token, op) = id.split_once('.').ok_or(OrdinaryCarrierError::Shape)?;
    if !matches!(token, "time" | "duration" | "instant") {
        return Err(OrdinaryCarrierError::Shape);
    }
    let ty = |t: &str| format!("mpk.csharp.value.{t}.v1");
    let own = ty(token);
    let (args, result) = match op {
        "construct" => (vec![ty("i64")], own.clone()),
        "ticks" | "milliseconds"
            if token == "instant" && op == "milliseconds"
                || token != "instant" && op == "ticks" =>
        {
            (vec![own.clone()], ty("i64"))
        }
        "hour" | "minute" | "second" | "millisecond" | "days" | "hours" | "minutes" | "seconds"
        | "milliseconds" => (vec![own.clone()], ty("i32")),
        "add_duration" | "subtract_duration" => (vec![own.clone(), ty("duration")], own.clone()),
        "subtract" if token == "time" => (vec![own.clone(); 2], ty("duration")),
        "difference" => (vec![own.clone(); 2], ty("duration")),
        "add" | "subtract" => (vec![own.clone(); 2], own.clone()),
        "negate" => (vec![own.clone()], own.clone()),
        "compare" => (vec![own.clone(); 2], ty("i32")),
        "equal" | "not_equal" | "less" | "less_equal" | "greater" | "greater_equal" => {
            (vec![own.clone(); 2], ty("bool"))
        }
        _ => return Err(OrdinaryCarrierError::Shape),
    };
    business_signature(id, args, result)
}
pub(super) fn business_signature(
    id: &str,
    args: Vec<String>,
    result: String,
) -> R<ClosedOperationSignature> {
    let recipe =
        BusinessOperation::new(id, &args, &result).map_err(|_| OrdinaryCarrierError::Shape)?;
    let mut checks = vec![];
    for exception in recipe.exception_types() {
        let name = match exception {
            "System.ArgumentOutOfRangeException" => "exception.range",
            "System.OverflowException" => "exception.overflow",
            _ => return Err(OrdinaryCarrierError::Shape),
        };
        checks.push(RequiredCheck {
            id: name.into(),
            tag: RequiredCheckTag::Exception,
            failure_type_id: Some(exception.into()),
        });
    }
    checks.extend(recipe.ordered_errors().into_iter().map(|id| RequiredCheck {
        id: id.into(),
        tag: RequiredCheckTag::ErrorOutcome,
        failure_type_id: None,
    }));
    Ok(ClosedOperationSignature {
        id: id.into(),
        tag: ClosedOperationTag::Data,
        argument_type_ids: args,
        normal_result_type_id: result,
        ordered_checks: checks,
    })
}
fn temporal_circuit(id: &str) -> R<IntegerCircuit> {
    let signature = temporal_signature(id)?;
    let (token, op) = id.split_once('.').unwrap();
    let mut c = Circuit::new(&vec![64; signature.argument_type_ids.len()]);
    let a = c.inputs[0].clone();
    let b = c.inputs.get(1).cloned().unwrap_or_default();
    let mut failures = vec![];
    let output = if matches!(
        op,
        "compare" | "equal" | "not_equal" | "less" | "less_equal" | "greater" | "greater_equal"
    ) {
        let eq = c.equal(&a, &b);
        let lt = c.lt(&a, &b, token != "time");
        let gt = c.lt(&b, &a, token != "time");
        match op {
            "compare" => {
                let positive = c.select(gt, &literal(1, 32), &literal(0, 32));
                c.select(lt, &literal(u32::MAX as u128, 32), &positive)
            }
            "equal" => vec![eq],
            "not_equal" => vec![c.not(eq)],
            "less" => vec![lt],
            "less_equal" => vec![c.or(lt, eq)],
            "greater" => vec![gt],
            _ => vec![c.or(gt, eq)],
        }
    } else if matches!(op, "construct" | "ticks") || token == "instant" && op == "milliseconds" {
        if token == "time" && op == "construct" {
            let in_range = c.lt(&a, &literal(DAY, 64), false);
            failures.push(c.not(in_range));
        }
        a
    } else if token == "time" && matches!(op, "add_duration" | "subtract") {
        let a = Circuit::extend(&a, 65, false);
        let b = Circuit::extend(&b, 65, op == "add_duration");
        let wide = if op == "subtract" {
            c.sub(&a, &b).0
        } else {
            c.add(&a, &b, F).0
        };
        euclidean_day(&mut c, &wide)
    } else if token == "duration" && matches!(op, "add" | "subtract" | "negate") {
        let a = Circuit::extend(&a, 65, true);
        let wide = if op == "negate" {
            c.neg(&a)
        } else {
            let b = Circuit::extend(&b, 65, true);
            if op == "add" {
                c.add(&a, &b, F).0
            } else {
                c.sub(&a, &b).0
            }
        };
        failures.push(overflows_i64(&mut c, &wide));
        wide[..64].to_vec()
    } else if token == "instant" {
        // 79 signed bits contain the full difference of two i64 values times
        // 10,000; preserve those high bits until the range check is constructed.
        let width = if op == "difference" { 79 } else { 65 };
        let a = Circuit::extend(&a, width, true);
        let wide = if op == "difference" {
            let b = Circuit::extend(&b, width, true);
            let delta = c.sub(&a, &b).0;
            c.multiply(&delta, &literal(10000, width), width)
        } else {
            let (q, r) = signed_divide_constant(&mut c, &b, 10000);
            failures.push(c.nonzero(&r));
            let b = Circuit::extend(&q, width, true);
            if op == "add_duration" {
                c.add(&a, &b, F).0
            } else {
                c.sub(&a, &b).0
            }
        };
        failures.push(overflows_i64(&mut c, &wide));
        wide[..64].to_vec()
    } else {
        let (divisor, modulus) = match op {
            "days" => (DAY, None),
            "hour" => (36_000_000_000, None),
            "hours" => (36_000_000_000, Some(24)),
            "minute" | "minutes" => (600_000_000, Some(60)),
            "second" | "seconds" => (10_000_000, Some(60)),
            "millisecond" | "milliseconds" => (10_000, Some(1000)),
            _ => return Err(OrdinaryCarrierError::Shape),
        };
        let signed = token == "duration";
        let q = if signed {
            signed_divide_constant(&mut c, &a, divisor).0
        } else {
            divide_constant(&mut c, &a, divisor).0
        };
        let q = if let Some(m) = modulus {
            if signed {
                signed_divide_constant(&mut c, &q, m).1
            } else {
                divide_constant(&mut c, &q, m).1
            }
        } else {
            q
        };
        Circuit::extend(&q, 32, signed)
    };
    if failures.len() != signature.ordered_checks.len() {
        return Err(OrdinaryCarrierError::Shape);
    }
    Ok(IntegerCircuit {
        signature,
        circuit: c,
        output,
        failures,
    })
}
/// Partial W09 capability: exactly Time, Duration and Instant operations retained
/// by validated source VIR. Domain membership and application proofs are separate.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct OrdinaryTemporalProgram {
    schema: String,
    source_ir_sha256: String,
    foundation_sha256: String,
    definitions: Vec<OrdinaryScalarDefinition>,
    certificate_sha256: String,
    #[serde(skip)]
    certificate: Vec<u8>,
}
impl OrdinaryTemporalProgram {
    pub fn definitions(&self) -> &[OrdinaryScalarDefinition] {
        &self.definitions
    }
    pub fn certificate_bytes(&self) -> &[u8] {
        &self.certificate
    }
    pub fn canonical_bytes(&self) -> Vec<u8> {
        serde_json::to_vec(self).expect("typed temporal program")
    }
}
pub fn generate_csharp_practical_ordinary_temporal(
    vir: &ValidatedPracticalVir,
) -> R<OrdinaryTemporalProgram> {
    let mut b = Builder::new()?;
    let mut definitions = vec![];
    let signatures = vir
        .operation_signatures()
        .iter()
        .filter(|s| {
            ["time.", "duration.", "instant."]
                .iter()
                .any(|p| s.id.starts_with(p))
        })
        .map(|s| (s.id.clone(), s))
        .collect::<BTreeMap<_, _>>();
    for (id, signature) in signatures {
        let p = temporal_circuit(&id)?;
        if &p.signature != signature {
            return Err(OrdinaryCarrierError::Linkage);
        }
        definitions.push(emit_circuit(&mut b, p, "Temporal")?);
    }
    let certificate = b.finish()?;
    let p = OrdinaryTemporalProgram {
        schema: "mpk.csharp.ordinary_temporal.v1".into(),
        source_ir_sha256: vir.hash().into(),
        foundation_sha256: vir.construction_context().0.content_sha256().into(),
        definitions,
        certificate_sha256: mpk_cert::hash_hex(&mpk_cert::certificate_hash(&certificate)),
        certificate,
    };
    if p.canonical_bytes().len() > 16 * 1024 * 1024 {
        return Err(OrdinaryCarrierError::Limit);
    }
    Ok(p)
}
pub fn import_csharp_practical_ordinary_temporal(
    input: &[u8],
    certificate: &[u8],
    vir: &ValidatedPracticalVir,
) -> R<OrdinaryTemporalProgram> {
    if input.len() > 16 * 1024 * 1024 || certificate.len() > 16 * 1024 * 1024 {
        return Err(OrdinaryCarrierError::Limit);
    }
    let p = generate_csharp_practical_ordinary_temporal(vir)?;
    if input != p.canonical_bytes() || certificate != p.certificate {
        return Err(OrdinaryCarrierError::Linkage);
    }
    Ok(p)
}

#[cfg(test)]
mod tests {
    use super::*;
    const OPERATIONS: &[(&str, &[&str])] = &[
        (
            "time",
            &[
                "construct",
                "ticks",
                "hour",
                "minute",
                "second",
                "millisecond",
                "add_duration",
                "subtract",
            ],
        ),
        (
            "duration",
            &[
                "construct",
                "ticks",
                "days",
                "hours",
                "minutes",
                "seconds",
                "milliseconds",
                "add",
                "subtract",
                "negate",
            ],
        ),
        (
            "instant",
            &[
                "milliseconds",
                "add_duration",
                "subtract_duration",
                "difference",
            ],
        ),
    ];
    const COMPARISONS: &[&str] = &[
        "compare",
        "equal",
        "not_equal",
        "less",
        "less_equal",
        "greater",
        "greater_equal",
    ];
    const FIXTURES: &[&str] = &[
        "time.add_duration",
        "duration.milliseconds",
        "instant.difference",
        "instant.add_duration",
    ];
    fn observed(p: &IntegerCircuit, args: &[u128]) -> (u128, Option<usize>) {
        let values = p.circuit.evaluate(args);
        (
            p.output
                .iter()
                .enumerate()
                .fold(0, |v, (i, &bit)| v | ((values[bit] as u128) << i)),
            p.failures.iter().position(|&bit| values[bit]),
        )
    }
    fn reference(token: &str, op: &str, a: i128, b: i128) -> Result<i128, usize> {
        if COMPARISONS.contains(&op) {
            return Ok(match op {
                "compare" => {
                    if a < b {
                        -1
                    } else {
                        i128::from(a > b)
                    }
                }
                "equal" => i128::from(a == b),
                "not_equal" => i128::from(a != b),
                "less" => i128::from(a < b),
                "less_equal" => i128::from(a <= b),
                "greater" => i128::from(a > b),
                _ => i128::from(a >= b),
            });
        }
        if token == "time" && op == "construct" && !(0..DAY as i128).contains(&a) {
            return Err(0);
        }
        let value = match (token, op) {
            (_, "construct" | "ticks") | ("instant", "milliseconds") => a,
            ("time", "subtract") => (a - b).rem_euclid(DAY as i128),
            ("time", "add_duration") => (a + b).rem_euclid(DAY as i128),
            ("duration", "add") => a + b,
            ("duration", "subtract") => a - b,
            ("duration", "negate") => -a,
            (_, "days") => a / DAY as i128,
            (_, "hour") => a / 36_000_000_000,
            (_, "hours") => a / 36_000_000_000 % 24,
            (_, "minute" | "minutes") => a / 600_000_000 % 60,
            (_, "second" | "seconds") => a / 10_000_000 % 60,
            (_, "millisecond" | "milliseconds") => a / 10_000 % 1000,
            ("instant", "difference") => (a - b) * 10000,
            ("instant", "add_duration" | "subtract_duration") => {
                if b % 10000 != 0 {
                    return Err(0);
                }
                if op == "add_duration" {
                    a + b / 10000
                } else {
                    a - b / 10000
                }
            }
            _ => panic!("unknown reference {token}.{op}"),
        };
        if i64::try_from(value).is_err() {
            return Err(usize::from(token == "instant" && op != "difference"));
        }
        Ok(value)
    }
    #[test]
    fn temporal_circuits_match_widened_reference() {
        let mut numbers = vec![
            i64::MIN as i128,
            i64::MIN as i128 + 1,
            -864_000_000_001,
            -36_000_000_001,
            -10001,
            -10000,
            -9999,
            -1,
            0,
            1,
            9999,
            10000,
            10001,
            36_000_000_001,
            DAY as i128 - 1,
            DAY as i128,
            DAY as i128 + 1,
            i64::MAX as i128 - 1,
            i64::MAX as i128,
        ];
        let mut seed = 0xa79de34167u64;
        for _ in 0..12 {
            seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
            numbers.push(seed as i64 as i128);
        }
        for &(token, ops) in OPERATIONS {
            for &op in ops.iter().chain(COMPARISONS.iter()) {
                let id = format!("{token}.{op}");
                let p = temporal_circuit(&id).unwrap();
                for &left in &numbers {
                    for &right in &numbers {
                        let a = if token == "time" && op != "construct" {
                            left.rem_euclid(DAY as i128)
                        } else {
                            left
                        };
                        let b = if token == "time" && op != "add_duration" {
                            right.rem_euclid(DAY as i128)
                        } else {
                            right
                        };
                        let (actual, error) = observed(&p, &[a as u128, b as u128]);
                        match reference(token, op, a, b) {
                            Ok(expected) => {
                                assert_eq!(error, None, "{id}({a},{b})");
                                assert_eq!(
                                    actual,
                                    expected as u128 & ((1u128 << p.output.len()) - 1),
                                    "{id}({a},{b})"
                                );
                            }
                            Err(expected) => assert_eq!(error, Some(expected), "{id}({a},{b})"),
                        }
                    }
                }
            }
        }
        for id in [
            "instant.construct",
            "time.negate",
            "duration.hour",
            "time.add",
            "instant.ticks",
        ] {
            assert!(temporal_circuit(id).is_err(), "{id}");
        }
    }
    #[test]
    fn temporal_circuits_match_t03_business_oracle() {
        let bundle = validate_registered_foundation_bundle(
            registered_foundation_descriptor_transport(),
            registered_foundation_definitions_transport(),
        )
        .unwrap();
        let roots = serde_json::json!(["time", "duration", "instant", "i64", "i32", "bool"].map(|id| serde_json::json!({"origin":"semantic_binding", "provenance_id":format!("temporal.{id}"), "type":{"kind":"primitive", "id":id}})));
        let bytes =
            canonical_closed_root_set_transport(&bundle, &roots, &serde_json::json!({})).unwrap();
        let roots = validate_closed_root_set(&bundle, &bytes).unwrap();
        let closed = derive_closed_instances(&bundle, &roots).unwrap();
        let value = |id: &str, n: i128| match id.strip_prefix("mpk.csharp.value.").unwrap() {
            "time.v1" => MonomorphicValue::Time {
                type_id: id.into(),
                ticks: n.to_string(),
            },
            "duration.v1" => MonomorphicValue::Duration {
                type_id: id.into(),
                ticks: n.to_string(),
            },
            "instant.v1" => MonomorphicValue::Instant {
                type_id: id.into(),
                milliseconds: n.to_string(),
            },
            _ => MonomorphicValue::Signed {
                type_id: id.into(),
                value: n.to_string(),
            },
        };
        for &(token, ops) in OPERATIONS {
            for &op in ops.iter().chain(COMPARISONS.iter()) {
                let id = format!("{token}.{op}");
                let p = temporal_circuit(&id).unwrap();
                let recipe = BusinessOperation::new(
                    &id,
                    &p.signature.argument_type_ids,
                    &p.signature.normal_result_type_id,
                )
                .unwrap();
                for [left, right] in [
                    [i64::MIN as i128, -1],
                    [i64::MAX as i128, 10001],
                    [i64::MAX as i128, 10000],
                    [-10001, 10000],
                    [0, DAY as i128],
                    [DAY as i128 - 1, i64::MIN as i128],
                    [0, 0],
                    [1, -1],
                ] {
                    let a = if token == "time" && op != "construct" {
                        left.rem_euclid(DAY as i128)
                    } else {
                        left
                    };
                    let b = if token == "time" && op != "add_duration" {
                        right.rem_euclid(DAY as i128)
                    } else {
                        right
                    };
                    let args = p
                        .signature
                        .argument_type_ids
                        .iter()
                        .zip([a, b])
                        .map(|(id, n)| value(id, n))
                        .collect::<Vec<_>>();
                    let (actual, error) = observed(&p, &[a as u128, b as u128]);
                    match recipe.evaluate(&bundle, &roots, &closed, &args) {
                        Ok(value) => {
                            let n: i128 = match value {
                                MonomorphicValue::Bool { value, .. } => i128::from(value),
                                MonomorphicValue::Signed { value, .. } => value.parse().unwrap(),
                                MonomorphicValue::Time { ticks, .. }
                                | MonomorphicValue::Duration { ticks, .. } => {
                                    ticks.parse().unwrap()
                                }
                                MonomorphicValue::Instant { milliseconds, .. } => {
                                    milliseconds.parse().unwrap()
                                }
                                _ => panic!("unexpected oracle value"),
                            };
                            assert_eq!(error, None, "{id}({a},{b})");
                            assert_eq!(
                                actual,
                                n as u128 & ((1u128 << p.output.len()) - 1),
                                "{id}({a},{b})"
                            );
                        }
                        Err(e) => {
                            let expected = p
                                .signature
                                .ordered_checks
                                .iter()
                                .position(|check| {
                                    e.exception_type().is_some_and(|t| {
                                        check.failure_type_id.as_deref() == Some(t)
                                    }) || e.error_id().is_some_and(|id| check.id == id)
                                })
                                .unwrap_or_else(|| panic!("unexpected oracle error {id}: {e:?}"));
                            assert_eq!(error, Some(expected), "{id}({a},{b})");
                        }
                    }
                }
            }
        }
    }
    #[test]
    fn temporal_circuits_emit_all_signatures() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../develop/migrations/csharp-03/ordinary-foundation/temporal-circuits");
        let output = std::env::var_os("MPK_W09_TEMPORAL_OUT").map(std::path::PathBuf::from);
        if let Some(dir) = &output {
            std::fs::create_dir_all(dir).unwrap();
        }
        let mut metrics = vec![];
        for &(token, ops) in OPERATIONS {
            for &op in ops.iter().chain(COMPARISONS.iter()) {
                let id = format!("{token}.{op}");
                let mut b = Builder::new().unwrap();
                let definition = emit_circuit(&mut b, temporal_circuit(&id).unwrap(), "Temporal")
                    .unwrap_or_else(|e| panic!("{id}: {e:?}"));
                let bytes = b.finish().unwrap_or_else(|e| panic!("{id}: {e:?}"));
                let cert = decode_canonical_certificate(&bytes).unwrap();
                crate::csharp_practical_vc_model::validate_csharp_practical_certificate_structure(
                    &cert,
                )
                .unwrap();
                metrics.push(serde_json::json!({"id": id, "definition": definition, "terms": cert.term_table.len(), "declarations": cert.declarations.len(), "hash": mpk_cert::hash_hex(&mpk_cert::certificate_hash(&bytes))}));
                if FIXTURES.contains(&id.as_str()) {
                    let hex = bytes.iter().map(|b| format!("{b:02x}")).collect::<String>() + "\n";
                    if let Some(dir) = &output {
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
        }
        if let Some(dir) = &output {
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
    // The test interpreter evaluates bounded static transformer chains with
    // ordinary Rust recursion. Its stack is independent of certificate binder
    // depth and of both production checkers' unchanged limits.
    fn run_core_test(f: fn()) {
        std::thread::Builder::new()
            .stack_size(32 * 1024 * 1024)
            .spawn(f)
            .unwrap()
            .join()
            .unwrap();
    }
    #[test]
    fn temporal_circuits_core_orders_precision_before_range() {
        run_core_test(temporal_circuits_core_orders_precision_before_range_body);
    }
    fn temporal_circuits_core_orders_precision_before_range_body() {
        use super::super::super::tests::{bit, run, V};
        let mut builder = Builder::new().unwrap();
        let def = emit_circuit(
            &mut builder,
            temporal_circuit("instant.add_duration").unwrap(),
            "Temporal",
        )
        .unwrap();
        let cert = decode_canonical_certificate(&builder.finish().unwrap()).unwrap();
        for (ticks, expected) in [(10001u64, Some(0)), (10000, Some(1)), (0, None)] {
            let args = [i64::MAX as u64, ticks]
                .into_iter()
                .map(|raw| V::Cube((0..64).map(|i| raw & (1 << i) != 0).collect()))
                .collect::<Vec<_>>();
            assert_eq!(
                bit(run(&cert, &def.success_definition, args.clone())),
                expected.is_none()
            );
            for (i, name) in def.ordered_failure_definitions.iter().enumerate() {
                assert_eq!(bit(run(&cert, name, args.clone())), expected == Some(i));
            }
        }
    }
    #[test]
    fn temporal_circuits_core_evaluation_matches_network() {
        run_core_test(temporal_circuits_core_evaluation_matches_network_body);
    }
    fn temporal_circuits_core_evaluation_matches_network_body() {
        use super::super::super::tests::{apply, bit, run, V};
        for id in [
            "time.construct",
            "duration.negate",
            "duration.compare",
            "instant.difference",
        ] {
            let p = temporal_circuit(id).unwrap();
            let mut b = Builder::new().unwrap();
            let def = emit_circuit(&mut b, temporal_circuit(id).unwrap(), "Temporal").unwrap();
            let cert = decode_canonical_certificate(&b.finish().unwrap()).unwrap();
            for raw in [
                [0u128, 1],
                [DAY - 1, DAY],
                [u64::MAX as u128, 1],
                [1 << 63, 0],
            ] {
                let args = p
                    .circuit
                    .inputs
                    .iter()
                    .enumerate()
                    .map(|(a, w)| V::Cube((0..w.len()).map(|i| raw[a] & (1 << i) != 0).collect()))
                    .collect::<Vec<_>>();
                let (expected, error) = observed(&p, &raw);
                assert_eq!(
                    bit(run(&cert, &def.success_definition, args.clone())),
                    error.is_none(),
                    "{id}"
                );
                for (i, name) in def.ordered_failure_definitions.iter().enumerate() {
                    assert_eq!(
                        bit(run(&cert, name, args.clone())),
                        error == Some(i),
                        "{id}"
                    );
                }
                let value = run(&cert, &def.result_definition, args);
                for i in 0..p.output.len() {
                    let mut v = value.clone();
                    for j in 0..address_bits(p.output.len() as u32) {
                        v = apply(&cert, v, V::Bit(i & (1 << j) != 0));
                    }
                    assert_eq!(
                        bit(v),
                        error.is_none() && expected & (1 << i) != 0,
                        "{id}, bit {i}"
                    );
                }
            }
        }
    }
}
