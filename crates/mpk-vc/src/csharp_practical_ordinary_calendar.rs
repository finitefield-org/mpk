//! Ordinary Gregorian Date, Guid and DayOfWeek operations on their fixed carriers.
use super::temporal::{business_signature, divide_constant, literal};
use super::*;
fn constant(n: u128) -> Word {
    literal(n, 32)
}
fn add_constant(c: &mut Circuit, a: &[Bit], n: u128) -> Word {
    c.add(a, &literal(n, a.len()), F).0
}
fn times_constant(c: &mut Circuit, a: &[Bit], n: u128) -> Word {
    c.multiply(a, &literal(n, a.len()), a.len())
}
fn remainder(c: &mut Circuit, a: &[Bit], n: u128) -> Word {
    Circuit::extend(&divide_constant(c, a, n).1, a.len(), false)
}
fn leap_year(c: &mut Circuit, y: &[Bit]) -> Bit {
    let r4 = remainder(c, y, 4);
    let nz4 = c.nonzero(&r4);
    let div4 = c.not(nz4);
    let r100 = remainder(c, y, 100);
    let nz100 = c.nonzero(&r100);
    let r400 = remainder(c, y, 400);
    let nz400 = c.nonzero(&r400);
    let div400 = c.not(nz400);
    let century = c.or(nz100, div400);
    c.and(div4, century)
}
fn month_length(c: &mut Circuit, y: &[Bit], m: &[Bit]) -> Word {
    let leap = leap_year(c, y);
    let february = c.equal(m, &constant(2));
    let short = [4, 6, 9, 11].into_iter().fold(F, |acc, n| {
        let eq = c.equal(m, &constant(n));
        c.or(acc, eq)
    });
    let ordinary = c.select(short, &constant(30), &constant(31));
    let feb = c.select(leap, &constant(29), &constant(28));
    c.select(february, &feb, &ordinary)
}
fn month_start(c: &mut Circuit, leap: Bit, month: usize) -> Word {
    let base = [0, 31, 59, 90, 120, 151, 181, 212, 243, 273, 304, 334][month - 1];
    if month > 2 {
        c.select(leap, &constant(base + 1), &constant(base))
    } else {
        constant(base)
    }
}
fn days_before_year(c: &mut Circuit, year: &[Bit]) -> Word {
    let y = c.sub(year, &constant(1)).0;
    let ordinary = times_constant(c, &y, 365);
    let q4 = divide_constant(c, &y, 4).0;
    let q100 = divide_constant(c, &y, 100).0;
    let q400 = divide_constant(c, &y, 400).0;
    let n = c.add(&ordinary, &q4, F).0;
    let n = c.sub(&n, &q100).0;
    c.add(&n, &q400, F).0
}
fn date_number(c: &mut Circuit, y: &[Bit], m: &[Bit], day: &[Bit]) -> Word {
    let leap = leap_year(c, y);
    let mut before = constant(0);
    for month in 1..=12 {
        let selected = c.equal(m, &constant(month as u128));
        let start = month_start(c, leap, month);
        before = c.select(selected, &start, &before);
    }
    let year_start = days_before_year(c, y);
    let n = c.add(&year_start, &before, F).0;
    let n = c.add(&n, day, F).0;
    c.sub(&n, &constant(1)).0
}
fn range(c: &mut Circuit, value: &[Bit], low: u128, high: u128) -> Bit {
    let below = c.lt(value, &literal(low, value.len()), false);
    let above = c.lt(&literal(high, value.len()), value, false);
    c.or(below, above)
}
fn signed_offset_range(c: &mut Circuit, offset: &[Bit], bound: i128) -> Bit {
    let below = c.lt(offset, &literal((-bound) as u128, 32), true);
    let above = c.lt(&constant(bound as u128), offset, true);
    c.or(below, above)
}
fn date_parts(c: &mut Circuit, n: &[Bit]) -> (Word, Word, Word) {
    // Split a Gregorian cycle into 400/100/4/1-year blocks. On the final
    // day of a 400-year or four-year block, cap the corresponding quotient
    // at three so the extra day remains in the preceding leap year.
    let (q400, r) = divide_constant(c, n, 146097);
    let r = Circuit::extend(&r, 32, false);
    let q100 = divide_constant(c, &r, 36524).0;
    let four = c.equal(&q100, &constant(4));
    let q100 = c.select(four, &constant(3), &q100);
    let delta = times_constant(c, &q100, 36524);
    let r = c.sub(&r, &delta).0;
    let (q4, r) = divide_constant(c, &r, 1461);
    let r = Circuit::extend(&r, 32, false);
    let q1 = divide_constant(c, &r, 365).0;
    let four = c.equal(&q1, &constant(4));
    let q1 = c.select(four, &constant(3), &q1);
    let delta = times_constant(c, &q1, 365);
    let day_of_year = c.sub(&r, &delta).0;
    let a = times_constant(c, &q400, 400);
    let b = times_constant(c, &q100, 100);
    let y = c.add(&a, &b, F).0;
    let a = times_constant(c, &q4, 4);
    let y = c.add(&y, &a, F).0;
    let y = c.add(&y, &q1, F).0;
    let y = add_constant(c, &y, 1);
    let leap = leap_year(c, &y);
    let mut month = constant(1);
    let mut before = constant(0);
    for m in 2..=12 {
        let start = month_start(c, leap, m);
        let below = c.lt(&day_of_year, &start, false);
        let selected = c.not(below);
        month = c.select(selected, &constant(m as u128), &month);
        before = c.select(selected, &start, &before);
    }
    let day = c.sub(&day_of_year, &before).0;
    let day = add_constant(c, &day, 1);
    (y, month, day)
}
fn calendar_signature(id: &str) -> R<ClosedOperationSignature> {
    let (token, op) = id.split_once('.').ok_or(OrdinaryCarrierError::Shape)?;
    if !matches!(token, "date" | "guid" | "day_of_week") {
        return Err(OrdinaryCarrierError::Shape);
    }
    let ty = |t: &str| format!("mpk.csharp.value.{t}.v1");
    let own = ty(token);
    let (args, out) = match (token, op) {
        ("date", "construct") => (vec![ty("i32"); 3], own.clone()),
        ("date", "add_days" | "add_months" | "add_years") => {
            (vec![own.clone(), ty("i32")], own.clone())
        }
        ("date", "year" | "month" | "day" | "day_number") => (vec![own.clone()], ty("i32")),
        ("date", "day_of_week") => (vec![own.clone()], ty("day_of_week")),
        ("guid", "empty") => (vec![], own.clone()),
        (_, "compare") => (vec![own.clone(); 2], ty("i32")),
        (_, "equal" | "not_equal" | "less" | "less_equal" | "greater" | "greater_equal") => {
            (vec![own.clone(); 2], ty("bool"))
        }
        _ => return Err(OrdinaryCarrierError::Shape),
    };
    business_signature(id, args, out)
}
fn calendar_circuit(id: &str) -> R<IntegerCircuit> {
    let signature = calendar_signature(id)?;
    let (token, op) = id.split_once('.').unwrap();
    let width = if token == "guid" { 128 } else { 32 };
    let mut c = Circuit::new(&vec![width; signature.argument_type_ids.len()]);
    let a = c.inputs.first().cloned().unwrap_or_default();
    let b = c.inputs.get(1).cloned().unwrap_or_default();
    let mut failures = vec![];
    let output = match op {
        "compare" | "equal" | "not_equal" | "less" | "less_equal" | "greater" | "greater_equal" => {
            // Canonical Guid N fields are most-significant first, so unsigned
            // comparison of the 128-bit N number equals lexicographic fields.
            let eq = c.equal(&a, &b);
            let lt = c.lt(&a, &b, token == "day_of_week");
            let gt = c.lt(&b, &a, token == "day_of_week");
            match op {
                "compare" => {
                    let positive = c.select(gt, &constant(1), &constant(0));
                    c.select(lt, &constant(u32::MAX as u128), &positive)
                }
                "equal" => vec![eq],
                "not_equal" => vec![c.not(eq)],
                "less" => vec![lt],
                "less_equal" => vec![c.or(lt, eq)],
                "greater" => vec![gt],
                _ => vec![c.or(gt, eq)],
            }
        }
        "empty" => vec![F; 128],
        "day_number" => a,
        "day_of_week" => {
            let n = add_constant(&mut c, &a, 1);
            remainder(&mut c, &n, 7)
        }
        "construct" => {
            let d = c.inputs[2].clone();
            let y_bad = range(&mut c, &a, 1, 9999);
            let m_bad = range(&mut c, &b, 1, 12);
            let length = month_length(&mut c, &a, &b);
            let zero = c.equal(&d, &constant(0));
            let above = c.lt(&length, &d, false);
            let d_bad = c.or(zero, above);
            let bad = c.or(y_bad, m_bad);
            failures.push(c.or(bad, d_bad));
            date_number(&mut c, &a, &b, &d)
        }
        "add_days" => {
            let a = Circuit::extend(&a, 33, false);
            let b = Circuit::extend(&b, 33, true);
            let sum = c.add(&a, &b, F).0;
            failures.push(range(&mut c, &sum, 0, 3652058));
            sum[..32].to_vec()
        }
        "year" | "month" | "day" => {
            let (y, m, d) = date_parts(&mut c, &a);
            match op {
                "year" => y,
                "month" => m,
                _ => d,
            }
        }
        "add_months" | "add_years" => {
            let (y, m, d) = date_parts(&mut c, &a);
            let bound = if op == "add_months" { 120000 } else { 10000 };
            let offset_bad = signed_offset_range(&mut c, &b, bound);
            let (y, m, bad) = if op == "add_years" {
                let y = Circuit::extend(&y, 33, false);
                let b = Circuit::extend(&b, 33, true);
                let sum = c.add(&y, &b, F).0;
                let bad = range(&mut c, &sum, 1, 9999);
                (sum[..32].to_vec(), m, bad)
            } else {
                let y = c.sub(&y, &constant(1)).0;
                let months = times_constant(&mut c, &y, 12);
                let months = c.add(&months, &m, F).0;
                let months = c.sub(&months, &constant(1)).0;
                let months = Circuit::extend(&months, 33, false);
                let b = Circuit::extend(&b, 33, true);
                let months = c.add(&months, &b, F).0;
                let bad = range(&mut c, &months, 0, 119987);
                let (q, r) = divide_constant(&mut c, &months[..32], 12);
                let y = add_constant(&mut c, &q, 1);
                let m = Circuit::extend(&r, 32, false);
                let m = add_constant(&mut c, &m, 1);
                (y, m, bad)
            };
            failures.push(c.or(offset_bad, bad));
            let length = month_length(&mut c, &y, &m);
            let too_large = c.lt(&length, &d, false);
            let d = c.select(too_large, &length, &d);
            date_number(&mut c, &y, &m, &d)
        }
        _ => return Err(OrdinaryCarrierError::Shape),
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
/// Partial W09 capability for Date/Guid/DayOfWeek. Value domains and application
/// proofs remain separate; definitions are reconstructed from original VIR.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct OrdinaryCalendarProgram {
    schema: String,
    source_ir_sha256: String,
    foundation_sha256: String,
    definitions: Vec<OrdinaryScalarDefinition>,
    certificate_sha256: String,
    #[serde(skip)]
    certificate: Vec<u8>,
}
impl OrdinaryCalendarProgram {
    pub fn definitions(&self) -> &[OrdinaryScalarDefinition] {
        &self.definitions
    }
    pub fn certificate_bytes(&self) -> &[u8] {
        &self.certificate
    }
    pub fn canonical_bytes(&self) -> Vec<u8> {
        serde_json::to_vec(self).expect("typed calendar program")
    }
}
pub fn generate_csharp_practical_ordinary_calendar(
    vir: &ValidatedPracticalVir,
) -> R<OrdinaryCalendarProgram> {
    let mut b = Builder::new()?;
    let mut definitions = vec![];
    let signatures = vir
        .operation_signatures()
        .iter()
        .filter(|s| {
            ["date.", "guid.", "day_of_week."]
                .iter()
                .any(|p| s.id.starts_with(p))
        })
        .map(|s| (s.id.clone(), s))
        .collect::<BTreeMap<_, _>>();
    for (id, signature) in signatures {
        let p = calendar_circuit(&id)?;
        if &p.signature != signature {
            return Err(OrdinaryCarrierError::Linkage);
        }
        definitions.push(emit_circuit(&mut b, p, "Calendar")?);
    }
    let certificate = b.finish()?;
    let p = OrdinaryCalendarProgram {
        schema: "mpk.csharp.ordinary_calendar.v1".into(),
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
pub fn import_csharp_practical_ordinary_calendar(
    input: &[u8],
    certificate: &[u8],
    vir: &ValidatedPracticalVir,
) -> R<OrdinaryCalendarProgram> {
    if input.len() > 16 * 1024 * 1024 || certificate.len() > 16 * 1024 * 1024 {
        return Err(OrdinaryCarrierError::Limit);
    }
    let p = generate_csharp_practical_ordinary_calendar(vir)?;
    if input != p.canonical_bytes() || certificate != p.certificate {
        return Err(OrdinaryCarrierError::Linkage);
    }
    Ok(p)
}

#[cfg(test)]
mod tests {
    use super::*;
    const COMPARISONS: &[&str] = &[
        "compare",
        "equal",
        "not_equal",
        "less",
        "less_equal",
        "greater",
        "greater_equal",
    ];
    const OPERATIONS: &[(&str, &[&str])] = &[
        (
            "date",
            &[
                "construct",
                "add_days",
                "add_months",
                "add_years",
                "year",
                "month",
                "day",
                "day_number",
                "day_of_week",
                "compare",
                "equal",
                "not_equal",
                "less",
                "less_equal",
                "greater",
                "greater_equal",
            ],
        ),
        ("guid", &["empty", "compare", "equal", "not_equal"]),
        ("day_of_week", COMPARISONS),
    ];
    const FIXTURES: &[&str] = &[
        "date.construct",
        "date.add_months",
        "date.add_years",
        "guid.empty",
        "guid.compare",
    ];
    fn observed(p: &IntegerCircuit, args: &[u128]) -> (u128, Option<usize>) {
        let values = p.circuit.evaluate(args);
        (
            p.output
                .iter()
                .enumerate()
                .fold(0, |n, (i, &bit)| n | ((values[bit] as u128) << i)),
            p.failures.iter().position(|&b| values[b]),
        )
    }
    fn host_day(y: u32, m: u32, d: u32) -> u128 {
        let prev = y - 1;
        let mut n = 365 * prev + prev / 4 - prev / 100 + prev / 400;
        for month in 1..m {
            n += host_month_length(y, month);
        }
        (n + d - 1) as u128
    }
    fn host_month_length(y: u32, m: u32) -> u32 {
        match m {
            2 => {
                if y.is_multiple_of(4) && (!y.is_multiple_of(100) || y.is_multiple_of(400)) {
                    29
                } else {
                    28
                }
            }
            4 | 6 | 9 | 11 => 30,
            _ => 31,
        }
    }
    #[test]
    fn calendar_circuits_match_t03_oracle_at_calendar_and_guid_boundaries() {
        let bundle = validate_registered_foundation_bundle(
            registered_foundation_descriptor_transport(),
            registered_foundation_definitions_transport(),
        )
        .unwrap();
        let roots=serde_json::json!(["date","guid","day_of_week","i32","bool"].map(|id|serde_json::json!({"origin":"semantic_binding","provenance_id":format!("calendar.{id}"),"type":{"kind":"primitive","id":id}})));
        let bytes =
            canonical_closed_root_set_transport(&bundle, &roots, &serde_json::json!({})).unwrap();
        let roots = validate_closed_root_set(&bundle, &bytes).unwrap();
        let closed = derive_closed_instances(&bundle, &roots).unwrap();
        let value = |id: &str, n: u128| match id.strip_prefix("mpk.csharp.value.").unwrap() {
            "date.v1" => MonomorphicValue::Date {
                type_id: id.into(),
                day_number: n as u32,
            },
            "guid.v1" => MonomorphicValue::Guid {
                type_id: id.into(),
                n: format!("{n:032x}"),
            },
            "day_of_week.v1" => MonomorphicValue::Enum {
                type_id: id.into(),
                underlying: "i32".into(),
                carrier: n.to_string(),
            },
            _ => MonomorphicValue::Signed {
                type_id: id.into(),
                value: (n as u32 as i32).to_string(),
            },
        };
        for &(token, ops) in OPERATIONS {
            for &op in ops {
                let id = format!("{token}.{op}");
                let p = calendar_circuit(&id).unwrap();
                let recipe = BusinessOperation::new(
                    &id,
                    &p.signature.argument_type_ids,
                    &p.signature.normal_result_type_id,
                )
                .unwrap();
                let mut cases = vec![];
                if token == "date" && op == "construct" {
                    for y in [
                        0,
                        1,
                        4,
                        100,
                        400,
                        1600,
                        1900,
                        2000,
                        2100,
                        2400,
                        9999,
                        10000,
                        u32::MAX,
                        1 << 31,
                    ] {
                        for m in [0, 1, 2, 4, 12, 13, u32::MAX] {
                            for d in [0, 1, 28, 29, 30, 31, 32, u32::MAX] {
                                cases.push(vec![y as u128, m as u128, d as u128]);
                            }
                        }
                    }
                } else if token == "date" {
                    let mut days = vec![0, 1, 3652057, 3652058];
                    for y in [4, 100, 400, 1600, 1900, 2000, 2100, 2400, 9999] {
                        for m in [1, 2, 3, 12] {
                            days.push(host_day(y, m, 1));
                            days.push(host_day(y, m, host_month_length(y, m)));
                        }
                    }
                    for n in days {
                        if op.starts_with("add_") {
                            for offset in [
                                i32::MIN,
                                -120001,
                                -120000,
                                -10001,
                                -10000,
                                -401,
                                -13,
                                -12,
                                -1,
                                0,
                                1,
                                12,
                                13,
                                401,
                                10000,
                                10001,
                                120000,
                                120001,
                                i32::MAX,
                            ] {
                                cases.push(vec![n, offset as u32 as u128]);
                            }
                        } else if COMPARISONS.contains(&op) {
                            for b in [0, n, 3652058] {
                                cases.push(vec![n, b]);
                            }
                        } else {
                            cases.push(vec![n]);
                        }
                    }
                } else if token == "guid" {
                    let mut raw = vec![0, 1, u128::MAX, 1 << 127, (1 << 127) - 1];
                    // Every canonical N-field boundary, including the unsigned high bit.
                    for bit in [8, 16, 24, 32, 40, 48, 56, 64, 80, 96] {
                        raw.push(1u128 << bit);
                        raw.push((1u128 << bit) - 1);
                    }
                    if op == "empty" {
                        cases.push(vec![]);
                    } else {
                        for &a in &raw {
                            for &b in &raw {
                                cases.push(vec![a, b]);
                            }
                        }
                    }
                } else {
                    for a in 0..=6 {
                        for b in 0..=6 {
                            cases.push(vec![a, b]);
                        }
                    }
                }
                for args in cases {
                    let (actual, error) = observed(&p, &args);
                    let values = p
                        .signature
                        .argument_type_ids
                        .iter()
                        .zip(&args)
                        .map(|(id, &v)| value(id, v))
                        .collect::<Vec<_>>();
                    match recipe.evaluate(&bundle, &roots, &closed, &values) {
                        Ok(v) => {
                            let expected = match v {
                                MonomorphicValue::Date { day_number, .. } => day_number as u128,
                                MonomorphicValue::Guid { n, .. } => {
                                    u128::from_str_radix(&n, 16).unwrap()
                                }
                                MonomorphicValue::Enum { carrier, .. } => carrier.parse().unwrap(),
                                MonomorphicValue::Signed { value, .. } => {
                                    value.parse::<i32>().unwrap() as u32 as u128
                                }
                                MonomorphicValue::Bool { value, .. } => u128::from(value),
                                _ => panic!("unexpected calendar oracle value"),
                            };
                            assert_eq!(error, None, "{id} {args:?}");
                            assert_eq!(actual, expected, "{id} {args:?}");
                        }
                        Err(e) => {
                            assert_eq!(e, BusinessError::ArgumentOutOfRange, "{id} {args:?}");
                            assert_eq!(error, Some(0), "{id} {args:?}");
                        }
                    }
                }
            }
        }
        for id in [
            "guid.less",
            "guid.construct",
            "day_of_week.empty",
            "date.ticks",
        ] {
            assert!(calendar_circuit(id).is_err());
        }
    }
    #[test]
    fn calendar_circuits_decompose_every_day_of_a_gregorian_cycle() {
        let mut c = Circuit::new(&[32]);
        let input = c.inputs[0].clone();
        let (y, m, d) = date_parts(&mut c, &input);
        let outputs = [y, m, d].concat();
        let expected = (1..=400)
            .flat_map(|y| {
                (1..=12).flat_map(move |m| (1..=host_month_length(y, m)).map(move |d| (y, m, d)))
            })
            .collect::<Vec<_>>();
        assert_eq!(expected.len(), 146097);
        // Bit-slicing executes the same Boolean gate network for 64 distinct
        // inputs at once; expected calendar dates use the independent host loop.
        for (batch, rows) in expected.chunks(64).enumerate() {
            let mut values = Vec::<u64>::with_capacity(c.gates.len());
            for &g in &c.gates {
                let v = match g {
                    Gate::False => 0,
                    Gate::True => u64::MAX,
                    Gate::Input(0, bit) => (0..rows.len()).fold(0, |v, lane| {
                        v | ((((batch * 64 + lane) >> bit) & 1) as u64) << lane
                    }),
                    Gate::Input(..) => panic!("unexpected calendar argument"),
                    Gate::Not(a) => !values[a],
                    Gate::And(a, b) => values[a] & values[b],
                    Gate::Xor(a, b) => values[a] ^ values[b],
                    Gate::Mux(s, a, b) => (values[s] & values[a]) | (!values[s] & values[b]),
                };
                values.push(v);
            }
            for (lane, &(y, m, d)) in rows.iter().enumerate() {
                for (field, expected) in outputs.chunks(32).zip([y, m, d]) {
                    let actual = field.iter().enumerate().fold(0u32, |v, (bit, &gate)| {
                        v | ((((values[gate] >> lane) & 1) as u32) << bit)
                    });
                    assert_eq!(actual, expected, "day {}", batch * 64 + lane);
                }
            }
        }
    }
    #[test]
    fn calendar_circuits_emit_all_signatures() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../develop/migrations/csharp-03/ordinary-foundation/calendar-circuits");
        let out = std::env::var_os("MPK_W09_CALENDAR_OUT").map(std::path::PathBuf::from);
        if let Some(dir) = &out {
            std::fs::create_dir_all(dir).unwrap();
        }
        let mut metrics = vec![];
        for &(token, ops) in OPERATIONS {
            for &op in ops {
                let id = format!("{token}.{op}");
                let mut b = Builder::new().unwrap();
                let definition = emit_circuit(&mut b, calendar_circuit(&id).unwrap(), "Calendar")
                    .unwrap_or_else(|e| panic!("{id}: {e:?}"));
                let bytes = b.finish().unwrap_or_else(|e| panic!("{id}: {e:?}"));
                let cert = decode_canonical_certificate(&bytes).unwrap();
                metrics.push(serde_json::json!({"id":id,"definition":definition,"terms":cert.term_table.len(),"declarations":cert.declarations.len(),"hash":mpk_cert::hash_hex(&mpk_cert::certificate_hash(&bytes))}));
                if FIXTURES.contains(&id.as_str()) {
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
    fn calendar_circuits_core_evaluation_matches_network() {
        std::thread::Builder::new()
            .stack_size(32 * 1024 * 1024)
            .spawn(|| {
                use super::super::super::tests::{apply, bit, run, V};
                for (id, cases) in [
                    ("guid.empty", vec![vec![]]),
                    (
                        "guid.compare",
                        vec![vec![1 << 127, (1 << 127) - 1], vec![0, 0], vec![0, 1]],
                    ),
                    (
                        "date.add_days",
                        vec![vec![0, u32::MAX as u128], vec![3652058, 1], vec![146096, 1]],
                    ),
                    ("date.construct", vec![vec![2000, 2, 29], vec![1900, 2, 29]]),
                ] {
                    let p = calendar_circuit(id).unwrap();
                    let mut b = Builder::new().unwrap();
                    let definition =
                        emit_circuit(&mut b, calendar_circuit(id).unwrap(), "Calendar").unwrap();
                    let cert = decode_canonical_certificate(&b.finish().unwrap()).unwrap();
                    for raw in cases {
                        let (out, error) = observed(&p, &raw);
                        let args = p
                            .circuit
                            .inputs
                            .iter()
                            .zip(raw)
                            .map(|(w, raw)| {
                                V::Cube((0..w.len()).map(|i| raw & (1 << i) != 0).collect())
                            })
                            .collect::<Vec<_>>();
                        assert_eq!(
                            bit(run(&cert, &definition.success_definition, args.clone())),
                            error.is_none(),
                            "{id}"
                        );
                        for (i, name) in definition.ordered_failure_definitions.iter().enumerate() {
                            assert_eq!(
                                bit(run(&cert, name, args.clone())),
                                error == Some(i),
                                "{id}"
                            );
                        }
                        let value = run(&cert, &definition.result_definition, args);
                        for i in 0..p.output.len() {
                            let mut v = value.clone();
                            for j in 0..address_bits(p.output.len() as u32) {
                                v = apply(&cert, v, V::Bit(i & (1 << j) != 0));
                            }
                            assert_eq!(
                                bit(v),
                                error.is_none() && out & (1 << i) != 0,
                                "{id} bit {i}"
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
