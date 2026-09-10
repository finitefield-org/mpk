use super::integer_format_tests::format_sources;
use super::*;
use core_eval::{apply, bit, sparse_cube};

fn codec_sources() -> Vec<(String, Value, Value)> {
    let mut sources = format_sources();
    let replay = read("data-phase/data-stage-replay.json");
    for (name, id) in [
        (
            "date",
            "05e148b20eb9017bdab9a113ab247301fa596ef171961443f1c68727025fe98f",
        ),
        (
            "time",
            "031a55eda554f5e143b296fc6cc5cbbedb31c8039ea5159bef8f71c1bbb8254a",
        ),
    ] {
        let row = replay
            .as_array()
            .unwrap()
            .iter()
            .find(|r| r["id"] == id)
            .unwrap();
        sources.push((
            format!("codec-{name}-source"),
            row.clone(),
            row["outcome"]["facts"].clone(),
        ));
    }
    sources
}

fn observe(cert: &mpk_cert::encode::Certificate, value: &V, depth: u32, address: usize) -> bool {
    let mut v = value.clone();
    for i in 0..depth {
        v = apply(cert, v, V::Bit(address & (1 << i) != 0));
    }
    bit(v)
}
fn text_value(input: &[u16], length: u32) -> V {
    let mut cells = BTreeSet::from([2]); // inactive length-header padding
    for i in 0..32 {
        if length & (1 << i) != 0 {
            cells.insert(i << 14);
        }
    }
    for (i, ch) in input.iter().take(16384).enumerate() {
        for k in 0..16 {
            if ch & (1 << k) != 0 {
                cells.insert(1 | (i << 1) | (k << 15));
            }
        }
    }
    if length < 16384 {
        cells.insert(1 | (16383 << 1) | (15 << 15));
    }
    sparse_cube(19, cells)
}
pub(in crate::ordinary_carriers) fn parse_case(
    cert: &mpk_cert::encode::Certificate,
    d: &OrdinaryCalendarCodecDefinition,
    input: &[u16],
    length: Option<u32>,
) {
    let length = length.unwrap_or(input.len() as u32);
    let oracle = BoundaryCodec::new(&d.codec_id, &d.value_type_id, None, None).unwrap();
    let expected = if length > 16384 {
        Err(ParseErrorArm::InputBound)
    } else {
        oracle.parse(input)
    };
    let result = run(cert, &d.parse_definition, vec![text_value(input, length)]);
    let depth = d.value_depth + 1;
    let mut wanted = BTreeSet::new();
    match expected {
        Ok(value) => {
            let n = match value {
                MonomorphicValue::Date { day_number, .. } => day_number as u64,
                MonomorphicValue::Time { ticks, .. } => ticks.parse().unwrap(),
                _ => panic!("unexpected value"),
            };
            for i in 0..1usize << d.value_depth {
                if n & (1u64 << i) != 0 {
                    wanted.insert(1 | (i << 1));
                }
            }
        }
        Err(error) => {
            wanted.insert(0);
            let tag = match error {
                ParseErrorArm::InputBound => 0,
                ParseErrorArm::Syntax => 1,
                ParseErrorArm::Range => 4,
                _ => panic!("calendar codec has no noncanonical/precision alternative"),
            };
            for i in 0..32 {
                if tag & (1 << i) != 0 {
                    wanted.insert(1 | (i << (depth - 5)));
                }
            }
        }
    }
    for at in 0..1 << depth {
        assert_eq!(
            observe(cert, &result, depth, at),
            wanted.contains(&at),
            "{} input {input:?} length {length} bit {at}",
            d.codec_id
        );
    }
}
#[test]
fn csharp_03_t06_w09_calendar_codecs_semantics() {
    calendar_semantics(false);
}

#[test]
fn csharp_03_t06_w09_calendar_codecs_format_regression() {
    calendar_semantics(true);
}

fn calendar_semantics(format_only: bool) {
    let bundle = b();
    let mut cases = 0;
    for (id, row, facts) in codec_sources()
        .into_iter()
        .filter(|(id, _, _)| id.starts_with("codec-"))
    {
        let (context, captures) = support::replay_context(&bundle, &row);
        let source = ValidatedDataSource::import_captured_facts(
            &bundle,
            &context,
            &captures,
            &serde_json::to_vec(&facts).unwrap(),
        )
        .unwrap();
        let emitted = emit_data_phase(&bundle, &context, &captures, &source).unwrap();
        let p = generate_csharp_practical_ordinary_calendar_codecs(emitted.vir()).unwrap();
        let cert = mpk_cert::decode_canonical_certificate(p.certificate_bytes()).unwrap();
        for d in p.definitions() {
            let oracle = BoundaryCodec::new(&d.codec_id, &d.value_type_id, None, None).unwrap();
            let is_date = d.codec_id == "date";
            let mut spellings: BTreeSet<String> = if is_date {
                [
                    "0001-01-01",
                    "0004-02-29",
                    "0100-02-28",
                    "0400-02-29",
                    "1600-02-29",
                    "1900-02-28",
                    "1999-12-31",
                    "2000-02-29",
                    "2000-12-31",
                    "2001-01-01",
                    "2024-02-29",
                    "9999-12-31",
                    "0000-01-01",
                    "0001-00-01",
                    "0001-13-01",
                    "2000-02-00",
                    "2000-02-30",
                    "1900-02-29",
                    "2100-02-29",
                    "2023-02-29",
                    "2024-04-31",
                    "10000-01-01",
                    "1-01-01",
                    "2000/01/01",
                    "2000-1-01",
                    "2000-01-1",
                    "-001-01-01",
                    "+001-01-01",
                    "0000-00-0x",
                ]
                .into_iter()
                .map(Into::into)
                .collect()
            } else {
                [
                    "00:00:00.0000000",
                    "00:00:00.0000001",
                    "00:00:00.9999999",
                    "00:00:01.0000000",
                    "00:01:00.0000000",
                    "01:00:00.0000000",
                    "12:34:56.1234567",
                    "23:59:59.9999999",
                    "24:00:00.0000000",
                    "00:60:00.0000000",
                    "00:00:60.0000000",
                    "99:99:99.9999999",
                    "1:00:00.0000000",
                    "01:0:00.0000000",
                    "01:00:0.0000000",
                    "00:00:00.000000",
                    "00:00:00.00000000",
                    "00:00:00",
                    "00-00-00.0000000",
                    "00:00:00,0000000",
                    "-0:00:00.0000000",
                    "+0:00:00.0000000",
                    "99:99:99.999999x",
                ]
                .into_iter()
                .map(Into::into)
                .collect()
            };
            spellings.extend(["".into(), " ".into(), "0".into()]);
            // Every month edge in leap/common/century years uses the independent
            // codec as its oracle; formatting round-trips successful values.
            if is_date {
                for year in [1, 4, 100, 400, 1900, 2000, 2023, 2024, 9999] {
                    for month in 1..=12 {
                        for day in [1, 28, 29, 30, 31] {
                            spellings.insert(format!("{year:04}-{month:02}-{day:02}"));
                        }
                    }
                }
            }
            if format_only {
                spellings.clear();
            }
            for text in spellings {
                eprintln!("calendar parse {id}: {text:?}");
                let input = text.encode_utf16().collect::<Vec<_>>();
                parse_case(&cert, d, &input, None);
                cases += 1;
            }
            let values: BTreeSet<u64> = if is_date {
                // Includes Gregorian cycle ends and adjacent day numbers.
                [
                    0, 1, 30, 31, 58, 59, 60, 364, 365, 366, 1094, 1095, 1154, 1155, 1460, 1461,
                    1462, 36523, 36524, 36525, 146095, 146096, 146097, 146098, 693653, 693654,
                    730178, 730179, 3652057, 3652058,
                ]
                .into_iter()
                .collect()
            } else {
                let mut values = BTreeSet::from([
                    0,
                    1,
                    9_999,
                    10_000,
                    9_999_999,
                    10_000_000,
                    10_000_001,
                    599_999_999,
                    600_000_000,
                    600_000_001,
                    35_999_999_999,
                    36_000_000_000,
                    36_000_000_001,
                    452_961_234_567,
                    863_999_999_998,
                    863_999_999_999,
                ]);
                for hour in 0..24 {
                    values.insert(hour * 36_000_000_000);
                }
                values
            };
            for n in values {
                let value = if is_date {
                    MonomorphicValue::Date {
                        type_id: d.value_type_id.clone(),
                        day_number: n as u32,
                    }
                } else {
                    MonomorphicValue::Time {
                        type_id: d.value_type_id.clone(),
                        ticks: n.to_string(),
                    }
                };
                let expected = oracle
                    .format(
                        &bundle,
                        emitted.closure().roots(),
                        emitted.closure().closed(),
                        &value,
                    )
                    .unwrap();
                assert_eq!(expected.len(), d.text_length as usize);
                let ones = (0..1usize << d.value_depth)
                    .filter(|i| n & (1u64 << i) != 0)
                    .collect();
                let formatted = run(
                    &cert,
                    &d.format_definition,
                    vec![sparse_cube(d.value_depth, ones)],
                );
                eprintln!("calendar format {id}: {n}");
                for bit in 0..32 {
                    assert_eq!(
                        observe(&cert, &formatted, 19, bit << 14),
                        expected.len() & (1 << bit) != 0
                    );
                }
                for index in 0..16 {
                    for k in 0..16 {
                        let wanted = expected.get(index).is_some_and(|ch| ch & (1 << k) != 0);
                        assert_eq!(
                            observe(&cert, &formatted, 19, 1 | (index << 1) | (k << 15)),
                            wanted,
                            "{} value {n} char {index} bit {k}",
                            d.codec_id
                        );
                    }
                }
                for index in [
                    16, 17, 31, 32, 63, 127, 255, 511, 1023, 2047, 4095, 8191, 16383,
                ] {
                    for k in 0..16 {
                        assert!(!observe(
                            &cert,
                            &formatted,
                            19,
                            1 | (index << 1) | (k << 15)
                        ));
                    }
                }
                for pad in 0..13 {
                    assert!(!observe(&cert, &formatted, 19, 1 << (pad + 1)));
                }
                let parsed = run(&cert, &d.parse_definition, vec![formatted]);
                for at in 0..1usize << (d.value_depth + 1) {
                    let wanted = at & 1 != 0 && n & (1u64 << (at >> 1)) != 0;
                    assert_eq!(
                        observe(&cert, &parsed, d.value_depth + 1, at),
                        wanted,
                        "ordinary parse(format) {} {n} bit {at}",
                        d.codec_id
                    );
                }
                cases += 1;
            }
            if format_only {
                continue;
            }
            let canonical = if is_date {
                "2000-02-29"
            } else {
                "12:34:56.1234567"
            }
            .encode_utf16()
            .collect::<Vec<_>>();
            for i in 0..canonical.len() {
                for bad in [0x80, 0xd800, 0xffff, 0xff11, 0x0131] {
                    let mut corrupted = canonical.clone();
                    corrupted[i] = bad;
                    parse_case(&cert, d, &corrupted, None);
                    cases += 1;
                }
            }
            for length in [16385, 65536, 1 << 31, u32::MAX] {
                parse_case(&cert, d, &canonical, Some(length));
                cases += 1;
            }
            for length in [16383, 16384] {
                parse_case(&cert, d, &vec![b'0' as u16; length], None);
                cases += 1;
            }
        }
    }
    eprintln!("calendar codec semantic cases: {cases}");
}

#[test]
fn csharp_03_t06_w09_calendar_codecs_original_sources() {
    let bundle = b();
    let out = std::env::var_os("MPK_W09_CALENDAR_CODECS_OUT").map(std::path::PathBuf::from);
    if let Some(p) = &out {
        fs::create_dir_all(p).unwrap();
    }
    let fixture = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../develop/migrations/csharp-03/ordinary-foundation/calendar-codecs");
    let mut rows = vec![];
    let mut definitions = 0;
    let mut kinds = BTreeSet::new();
    let mut previous: Option<(Vec<u8>, Vec<u8>)> = None;
    let mut examined = 0;
    for (id, row, facts) in codec_sources() {
        examined += 1;
        let (context, captures) = support::replay_context(&bundle, &row);
        let source = ValidatedDataSource::import_captured_facts(
            &bundle,
            &context,
            &captures,
            &serde_json::to_vec(&facts).unwrap(),
        )
        .unwrap();
        let emitted = emit_data_phase(&bundle, &context, &captures, &source).unwrap();
        let vir = emitted.vir();
        let p = generate_csharp_practical_ordinary_calendar_codecs(vir)
            .unwrap_or_else(|e| panic!("{id}: {e:?}"));
        let cert = mpk_cert::decode_canonical_certificate(p.certificate_bytes()).unwrap();
        validate_csharp_practical_certificate_structure(&cert).unwrap();
        assert_eq!(
            import_csharp_practical_ordinary_calendar_codecs(
                &p.canonical_bytes(),
                p.certificate_bytes(),
                vir
            )
            .unwrap(),
            p
        );
        let meta: Value = serde_json::from_slice(&p.canonical_bytes()).unwrap();
        for field in [
            "schema",
            "source_ir_sha256",
            "foundation_sha256",
            "definitions",
            "certificate_sha256",
        ] {
            let mut m = meta.clone();
            m[field] = json!("forged");
            assert!(import_csharp_practical_ordinary_calendar_codecs(
                &serde_json::to_vec(&m).unwrap(),
                p.certificate_bytes(),
                vir
            )
            .is_err());
        }
        let mut corrupt = p.certificate_bytes().to_vec();
        *corrupt.last_mut().unwrap() ^= 1;
        assert!(import_csharp_practical_ordinary_calendar_codecs(
            &p.canonical_bytes(),
            &corrupt,
            vir
        )
        .is_err());
        if let Some((m, c)) = &previous {
            assert!(import_csharp_practical_ordinary_calendar_codecs(m, c, vir).is_err());
        }
        previous = Some((p.canonical_bytes(), p.certificate_bytes().to_vec()));
        if p.definitions().is_empty() {
            continue;
        }
        let old_hex = fs::read_to_string(
            fixture
                .join("previous-json-format")
                .join(format!("{id}.hex")),
        )
        .unwrap();
        let old_bytes = (0..old_hex.trim().len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&old_hex[i..i + 2], 16).unwrap())
            .collect::<Vec<_>>();
        let old = mpk_cert::decode_canonical_certificate(&old_bytes).unwrap();
        let roots = p
            .definitions()
            .iter()
            .map(|d| d.parse_definition.clone())
            .collect();
        super::super::structural_equivalence_tests::same_definition_closure(&old, &cert, &roots)
            .unwrap();
        let hex = p
            .certificate_bytes()
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect::<String>()
            + "\n";
        if let Some(dir) = &out {
            fs::write(dir.join(format!("{id}.hex")), &hex).unwrap();
        } else {
            assert_eq!(
                fs::read_to_string(fixture.join(format!("{id}.hex"))).unwrap(),
                hex
            );
        }
        for d in p.definitions() {
            assert!(matches!(d.codec_id.as_str(), "date" | "time"));
            assert_eq!(
                d.value_type_id,
                format!("mpk.csharp.value.{}.v1", d.codec_id)
            );
            assert_eq!(d.value_depth, if d.codec_id == "date" { 5 } else { 6 });
            assert_eq!(d.text_length, if d.codec_id == "date" { 10 } else { 16 });
            kinds.insert(d.codec_id.clone());
        }
        definitions += p.definitions().len();
        rows.push(json!({"id":id,"metadata":meta,"terms":cert.term_table.len(),"declarations":cert.declarations.len()}));
        eprintln!(
            "calendar codec source {id}: {} codecs",
            p.definitions().len()
        );
    }
    assert_eq!(examined, 67);
    assert_eq!(kinds, BTreeSet::from(["date".into(), "time".into()]));
    assert!(rows.iter().any(|r| r["id"] == "codec-date-source"));
    assert!(rows.iter().any(|r| r["id"] == "codec-time-source"));
    eprintln!(
        "calendar codec sources {}, definition occurrences {definitions}",
        rows.len()
    );
    let manifest = json!({"contexts_examined":examined,"sources":rows});
    if let Some(dir) = &out {
        fs::write(
            dir.join("certificates.json"),
            serde_json::to_vec_pretty(&manifest).unwrap(),
        )
        .unwrap();
    } else {
        assert_eq!(
            read("ordinary-foundation/calendar-codecs/certificates.json"),
            manifest
        );
    }
}

#[test]
fn csharp_03_t06_w09_calendar_codecs_pinned_bytes() {
    let bundle = b();
    let manifest = read("ordinary-foundation/calendar-codecs/certificates.json");
    let rows = manifest["sources"].as_array().unwrap();
    let fixture = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../develop/migrations/csharp-03/ordinary-foundation/calendar-codecs");
    let mut matched = BTreeSet::new();
    for (id, row, facts) in codec_sources() {
        let Some(record) = rows.iter().find(|r| r["id"] == id) else {
            continue;
        };
        let (context, captures) = support::replay_context(&bundle, &row);
        let source = ValidatedDataSource::import_captured_facts(
            &bundle,
            &context,
            &captures,
            &serde_json::to_vec(&facts).unwrap(),
        )
        .unwrap();
        let emitted = emit_data_phase(&bundle, &context, &captures, &source).unwrap();
        let p = generate_csharp_practical_ordinary_calendar_codecs(emitted.vir()).unwrap();
        assert_eq!(
            record["metadata"],
            serde_json::from_slice::<Value>(&p.canonical_bytes()).unwrap()
        );
        assert_eq!(
            fs::read_to_string(fixture.join(format!("{id}.hex"))).unwrap(),
            p.certificate_bytes()
                .iter()
                .map(|b| format!("{b:02x}"))
                .collect::<String>()
                + "\n"
        );
        assert!(matched.insert(id));
    }
    assert_eq!((matched.len(), rows.len()), (3, 3));
}
