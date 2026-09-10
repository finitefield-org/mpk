use super::boundary_document_tests::document_sources;
use super::*;
use core_eval::{apply, bit, sparse_cube};

#[path = "csharp_practical_ordinary_json_calendar_tests.rs"]
mod calendar_tests;

#[path = "csharp_practical_ordinary_json_syntax_tests.rs"]
mod syntax_tests;

#[path = "csharp_practical_ordinary_json_value_tests.rs"]
mod value_tests;

#[path = "csharp_practical_ordinary_json_product_tests.rs"]
mod product_tests;

// Test-only reassociation audit. Every old declaration/dependency must remain
// identical after replacing Scan's tree by its independently checked initial
// state. Its actual tree must expand to the same 16,386 ordered Step calls.
// Guarded composition is associative: after f, both bracketings return f's
// result when More is false; otherwise both run g, then conditionally h.
pub(super) fn same_json_scan(
    old: &mpk_cert::encode::Certificate,
    new: &mpk_cert::encode::Certificate,
) -> Result<(), String> {
    use mpk_cert::encode::{Certificate, DeclarationKind as D, TermNode as N};
    const PREFIX: &str = "Mpk.CSharp.Ordinary.JsonStringParse.";
    fn global(c: &Certificate, id: u32) -> &str {
        &c.name_table[c.declarations[id as usize].name as usize]
    }
    fn normalize(c: &Certificate) -> Result<Certificate, String> {
        let scan = c
            .declarations
            .iter()
            .position(|d| c.name_table[d.name as usize] == format!("{PREFIX}Scan"))
            .ok_or("missing Scan")?;
        let D::Def { value, .. } = c.declarations[scan].kind else {
            return Err("Scan kind".into());
        };
        let N::Lam { ty, body } = c.term_table[value as usize] else {
            return Err("Scan lambda".into());
        };
        let N::App {
            function,
            ref arguments,
        } = c.term_table[body as usize]
        else {
            return Err("Scan application".into());
        };
        if arguments.len() != 1 {
            return Err("Scan initial arity".into());
        }
        let initial = arguments[0];
        let mut pending = vec![function];
        let mut steps = 0;
        let mut visits = 0;
        while let Some(term) = pending.pop() {
            visits += 1;
            if visits > 100_000 {
                return Err("cyclic or excessive scan expansion".into());
            }
            let N::App {
                function,
                arguments,
            } = &c.term_table[term as usize]
            else {
                return Err("non-application scan node".into());
            };
            let N::Const { global: id, levels } = &c.term_table[*function as usize] else {
                return Err("non-global scan function".into());
            };
            if !levels.is_empty() {
                return Err("scan level arguments".into());
            }
            match global(c, *id).strip_prefix(PREFIX) {
                Some("Compose") => {
                    if arguments.len() != 2 {
                        return Err("Compose arity".into());
                    }
                    pending.push(arguments[1]);
                    pending.push(arguments[0]);
                }
                Some(kind @ ("Step" | "StepTwo" | "StepFour")) => {
                    if arguments.len() != 1 || c.term_table[arguments[0] as usize] != N::Var(0) {
                        return Err("scan source argument changed".into());
                    }
                    if kind == "Step" {
                        steps += 1;
                    } else {
                        let D::Def { value, .. } = c.declarations[*id as usize].kind else {
                            return Err("step group kind".into());
                        };
                        let N::Lam { body, .. } = c.term_table[value as usize] else {
                            return Err("step group lambda".into());
                        };
                        pending.push(body);
                    }
                }
                _ => return Err("unexpected scan operation".into()),
            }
        }
        if steps != 16_386 {
            return Err(format!("packet steps changed: {steps}"));
        }
        let mut normalized = c.clone();
        let value = normalized.term_table.len() as u32;
        normalized.term_table.push(N::Lam { ty, body: initial });
        let D::Def { value: slot, .. } = &mut normalized.declarations[scan].kind else {
            unreachable!()
        };
        *slot = value;
        Ok(normalized)
    }
    let left = normalize(old)?;
    let right = normalize(new)?;
    let roots = old
        .declarations
        .iter()
        .map(|d| old.name_table[d.name as usize].clone())
        .collect();
    super::super::structural_equivalence_tests::same_definition_closure(&left, &right, &roots)?;
    Ok(())
}

fn word(value: u32) -> V {
    sparse_cube(5, (0..32).filter(|i| value & (1 << i) != 0).collect())
}
fn document(length: u32, bytes: &[(usize, u8)]) -> V {
    let mut bits: BTreeSet<_> = (0..32)
        .filter(|i| length & (1 << i) != 0)
        .map(|i| i << 19)
        .collect();
    for &(index, byte) in bytes {
        assert!(index < 1_048_576);
        for k in 0..8 {
            if byte & (1 << k) != 0 {
                bits.insert(1 | (index << 1) | (k << 21));
            }
        }
    }
    sparse_cube(24, bits)
}
fn leaf(cert: &mpk_cert::encode::Certificate, mut value: V, depth: usize, index: usize) -> bool {
    for i in 0..depth {
        value = apply(cert, value, V::Bit(index & (1 << i) != 0));
    }
    bit(value)
}
fn with_program(f: impl FnOnce(&mpk_cert::encode::Certificate, &OrdinaryJsonTokenDefinition)) {
    let bundle = b();
    let (_, row, facts) = document_sources()
        .into_iter()
        .find(|(id, _, _)| id.starts_with("document-"))
        .unwrap();
    let (context, captures) = support::replay_context(&bundle, &row);
    let source = ValidatedDataSource::import_captured_facts(
        &bundle,
        &context,
        &captures,
        &serde_json::to_vec(&facts).unwrap(),
    )
    .unwrap();
    let emitted = emit_data_phase(&bundle, &context, &captures, &source).unwrap();
    let p = generate_csharp_practical_ordinary_json_tokens(emitted.vir()).unwrap();
    let cert = mpk_cert::decode_canonical_certificate(p.certificate_bytes()).unwrap();
    validate_csharp_practical_certificate_structure(&cert).unwrap();
    f(&cert, p.definition().unwrap());
}

#[test]
fn csharp_03_t06_w09_json_scan_regroup_audit_mutations() {
    use mpk_cert::encode::{DeclarationKind as D, TermNode as N};
    with_program(|cert, _| {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(
            "../../develop/migrations/csharp-03/ordinary-foundation/json-tokens/previous-pair-pipeline/document-6b96fc4eed95a1d33fa5784b9062f02fa07ac65b6aef2b4100bb91e9c2f1a613.hex");
        let hex = fs::read_to_string(path).unwrap();
        let bytes = (0..hex.trim().len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).unwrap())
            .collect::<Vec<_>>();
        let old = mpk_cert::decode_canonical_certificate(&bytes).unwrap();
        same_json_scan(&old, cert).unwrap();
        let index = |suffix: &str| {
            cert.declarations
                .iter()
                .position(|d| {
                    cert.name_table[d.name as usize]
                        == format!("Mpk.CSharp.Ordinary.JsonStringParse.{suffix}")
                })
                .unwrap()
        };
        // Fewer packet steps hidden behind the same StepFour name must reject.
        let mut shorter = cert.clone();
        let D::Def { value: pair, .. } = cert.declarations[index("StepTwo")].kind else {
            panic!()
        };
        let D::Def { value, .. } = &mut shorter.declarations[index("StepFour")].kind else {
            panic!()
        };
        *value = pair;
        assert!(same_json_scan(&old, &shorter).is_err());
        // A same-length tree cannot smuggle a changed primitive or source cursor.
        let mut changed = cert.clone();
        let D::Def { value, .. } = &mut changed.declarations[index("Step")].kind else {
            panic!()
        };
        *value = pair;
        assert!(same_json_scan(&old, &changed).is_err());
        let mut cursor = cert.clone();
        let D::Def { value, .. } = cursor.declarations[index("StepFour")].kind else {
            panic!()
        };
        let N::Lam { body, .. } = cursor.term_table[value as usize] else {
            panic!()
        };
        let N::App { arguments, .. } = &cursor.term_table[body as usize] else {
            panic!()
        };
        let left = arguments[0];
        let N::App { arguments, .. } = &cursor.term_table[left as usize] else {
            panic!()
        };
        let source = arguments[0];
        cursor.term_table[source as usize] = N::Var(1);
        assert!(same_json_scan(&old, &cursor).is_err());
        eprintln!("JSON scan regroup audit rejects shortened sequence, changed Step and changed source argument");
    });
}

fn oracle(bytes: &[u8], width: u32) -> Option<(u64, u32)> {
    let digits = bytes.iter().take_while(|b| b.is_ascii_digit()).count();
    let token = &bytes[..digits];
    if token.is_empty() || (token.len() > 1 && token[0] == b'0') {
        return None;
    }
    let number = std::str::from_utf8(token).unwrap().parse::<u64>().ok()?;
    (u128::from(number) < (1u128 << width)).then_some((number, digits as u32))
}
fn check_unsigned(
    cert: &mpk_cert::encode::Certificate,
    d: &OrdinaryJsonUnsignedDefinition,
    input: V,
    expected: Option<(u64, u32)>,
    whole: bool,
) {
    check_integer_packet(cert, &d.parse_definition, d.width, input, expected, whole);
}
fn check_integer_packet(
    cert: &mpk_cert::encode::Certificate,
    definition: &str,
    width: u32,
    input: V,
    expected: Option<(u64, u32)>,
    whole: bool,
) {
    let output = run(cert, definition, vec![input]);
    let mut bits = [false; 128];
    if let Some((number, count)) = expected {
        bits[0] = true;
        bits[1] = whole;
        for i in 0..64 {
            bits[2 + i] = number & (1 << i) != 0;
        }
        for i in 0..32 {
            bits[66 + i] = count & (1 << i) != 0;
        }
    }
    for (i, expected) in bits.into_iter().enumerate() {
        assert_eq!(
            leaf(cert, output.clone(), 7, i),
            expected,
            "integer {width} packet bit {i}"
        );
    }
}
#[test]
fn csharp_03_t06_w09_json_tokens_unsigned_semantics() {
    with_program(|cert, d| {
        let mut cases = BTreeSet::new();
        for bytes in [
            b"".as_slice(),
            b"0",
            b"1",
            b"9",
            b"10",
            b"100",
            b"00",
            b"01",
            b"000000000000000000000",
            b"-0",
            b"-1",
            b"+1",
            b" 1",
            b"true",
            b"null",
            b"1.0",
            b"1e3",
            b"1,",
            b"0]",
            b"42}",
            b"123:0",
            b"1 ",
            b"1\n",
            b"18446744073709551615",
            b"18446744073709551616",
            b"18446744073709551610",
            b"18446744073709551619",
            b"184467440737095516150",
        ] {
            cases.insert(bytes.to_vec());
        }
        for width in [8, 16, 32, 64] {
            let edge = 1u128 << width;
            for number in [edge / 2 - 1, edge / 2, edge - 1, edge, edge + 1] {
                cases.insert(number.to_string().into_bytes());
            }
        }
        for first in [0, 47, 48, 49, 57, 58, 127, 128, 255] {
            cases.insert(vec![first, b'2']);
        }
        cases.insert(vec![b'1', 0xff]);
        cases.insert(vec![b'0', 0]);
        let mut tested = 0;
        for def in &d.unsigned {
            for bytes in &cases {
                let expected = oracle(bytes, def.width);
                let whole = expected.is_some_and(|(_, n)| n as usize == bytes.len());
                let cells = bytes.iter().copied().enumerate().collect::<Vec<_>>();
                check_unsigned(
                    cert,
                    def,
                    document(bytes.len() as u32, &cells),
                    expected,
                    whole,
                );
                tested += 1;
                if tested % 32 == 0 {
                    eprintln!("JSON unsigned {tested}/{}", cases.len() * 4);
                }
            }
        }
        eprintln!(
            "JSON unsigned: {tested} independent prefix/value/range cases, all 128 result bits"
        );
    });
}
#[test]
fn csharp_03_t06_w09_json_tokens_bounds_and_shared_consumers() {
    with_program(|cert, d| {
        const MAX: u32 = 1_048_576;
        let document_def = &d.strings.document;
        assert_eq!(document_def, &d.keywords.document);
        assert_eq!(d.strings.fragments, d.keywords.fragments);
        assert_eq!(
            cert.declarations
                .iter()
                .filter(|decl| cert.name_table[decl.name as usize] == document_def.type_name)
                .count(),
            1
        );
        for def in &d.unsigned {
            for length in [0, 1, 2, 65_536, MAX, MAX + 1, 1 << 31, u32::MAX] {
                let expected = if (1..=MAX).contains(&length) {
                    Some((7, 1))
                } else {
                    None
                };
                check_unsigned(
                    cert,
                    def,
                    document(length, &[(0, b'7')]),
                    expected,
                    length == 1,
                );
            }
            let input = document(MAX, &[(MAX as usize - 2, b'4'), (MAX as usize - 1, b'2')]);
            let slice = run(
                cert,
                &d.strings.fragments.slice_definition,
                vec![input, word(MAX - 2), word(2)],
            );
            check_unsigned(cert, def, slice, Some((42, 2)), true);
        }
        // A shared document/fragment environment feeds each lexical family.
        for (at, token) in [(0, b"true".as_slice()), (8, b"null".as_slice())] {
            let cells = token
                .iter()
                .enumerate()
                .map(|(i, &b)| (at + i, b))
                .collect::<Vec<_>>();
            let slice = run(
                cert,
                &d.strings.fragments.slice_definition,
                vec![document(12, &cells), word(at as u32), word(4)],
            );
            let packet = run(cert, &d.keywords.parse_definition, vec![slice]);
            assert!(leaf(cert, packet.clone(), 6, 0));
            assert!(leaf(cert, packet.clone(), 6, 1));
            assert_eq!(leaf(cert, packet.clone(), 6, 2), at == 8);
            assert_eq!(leaf(cert, packet, 6, 3), at == 0);
        }
        for bytes in [b"\"\"".as_slice(), b"\"A\"", b"\"A\",", b"\"\\n\""] {
            let cells = bytes.iter().copied().enumerate().collect::<Vec<_>>();
            let input = document(bytes.len() as u32, &cells);
            let prefix = bytes != b"\"\\n\"";
            assert_eq!(
                bit(run(
                    cert,
                    &d.strings.prefix_valid_definition,
                    vec![input.clone()]
                )),
                prefix
            );
            assert_eq!(
                bit(run(cert, &d.strings.valid_definition, vec![input])),
                prefix && !bytes.ends_with(b",")
            );
        }
        eprintln!("JSON tokens: 32 u32-length cases, four 1-MiB-edge slices, shared keyword/string consumers");
    });
}

fn signed_oracle(bytes: &[u8], width: u32) -> Option<(u64, u32)> {
    let sign = usize::from(bytes.first() == Some(&b'-'));
    let digits = bytes[sign..]
        .iter()
        .take_while(|b| b.is_ascii_digit())
        .count();
    if digits == 0 || (digits > 1 && bytes[sign] == b'0') {
        return None;
    }
    let consumed = sign + digits;
    let token = std::str::from_utf8(&bytes[..consumed]).unwrap();
    let value = token.parse::<i128>().ok()?;
    let limit = 1i128 << (width - 1);
    if value < -limit || value >= limit || (sign == 1 && value == 0) {
        return None;
    }
    Some(((value as u64) & ((1u64 << width) - 1), consumed as u32))
}
#[test]
fn csharp_03_t06_w09_json_tokens_signed_semantics() {
    with_program(|cert, d| {
        let mut cases = BTreeSet::new();
        for text in [
            "",
            "-",
            "0",
            "1",
            "-1",
            "+1",
            "-0",
            "-00",
            "00",
            "01",
            "-01",
            "--1",
            "-+1",
            " -1",
            "-1 ",
            "-1,",
            "-0]",
            "-1.5",
            "1e2",
            "-1e2",
            "2147483648",
            "-2147483649",
            "18446744073709551615",
            "-18446744073709551616",
        ] {
            cases.insert(text.as_bytes().to_vec());
        }
        for width in [8, 16, 32] {
            let limit = 1i128 << (width - 1);
            for n in [-limit - 1, -limit, -limit + 1, limit - 1, limit, limit + 1] {
                cases.insert(n.to_string().into_bytes());
            }
        }
        cases.insert(vec![b'-', b'1', 255]);
        cases.insert(vec![b'-', 0, b'1']);
        let mut tested = 0;
        for def in &d.signed {
            for bytes in &cases {
                let expected = signed_oracle(bytes, def.width);
                let whole = expected.is_some_and(|(_, n)| n as usize == bytes.len());
                let cells = bytes.iter().copied().enumerate().collect::<Vec<_>>();
                check_integer_packet(
                    cert,
                    &def.parse_definition,
                    def.width,
                    document(bytes.len() as u32, &cells),
                    expected,
                    whole,
                );
                tested += 1;
                if tested % 16 == 0 {
                    eprintln!("JSON signed {tested}/{}", cases.len() * 3);
                }
            }
        }
        eprintln!(
            "JSON signed: {tested} independent prefix/value/range cases, all 128 packet bits"
        );
    });
}
#[test]
fn csharp_03_t06_w09_json_tokens_signed_bounds() {
    with_program(|cert, d| {
        const MAX: u32 = 1_048_576;
        for def in &d.signed {
            let value = (-7i64 as u64) & ((1u64 << def.width) - 1);
            for length in [0, 1, 2, 3, MAX, MAX + 1, 1 << 31, u32::MAX] {
                let expected = if (2..=MAX).contains(&length) {
                    Some((value, 2))
                } else {
                    None
                };
                check_integer_packet(
                    cert,
                    &def.parse_definition,
                    def.width,
                    document(length, &[(0, b'-'), (1, b'7')]),
                    expected,
                    length == 2,
                );
            }
            let slice = run(
                cert,
                &d.strings.fragments.slice_definition,
                vec![
                    document(MAX, &[(MAX as usize - 2, b'-'), (MAX as usize - 1, b'7')]),
                    word(MAX - 2),
                    word(2),
                ],
            );
            check_integer_packet(
                cert,
                &def.parse_definition,
                def.width,
                slice,
                Some((value, 2)),
                true,
            );
        }
        eprintln!("JSON signed: 24 full-u32 length cases and three high-offset ordinary slices");
    });
}

#[test]
fn csharp_03_t06_w09_json_tokens_quoted_decimal_settings() {
    with_program(|cert, d| {
        assert_eq!(d.quoted_decimals.len(), 146);
        let doc = |s: &str| document(s.len() as u32, &s.bytes().enumerate().collect::<Vec<_>>());
        // Every closed setting has an ordinary wrapper, including all five
        // rounding identities. Parsing fixed text is exact and does not round.
        for def in &d.quoted_decimals {
            let text = match def.codec.scale {
                Some(scale) if scale != 0 => format!("\"1.{}\"", "0".repeat(scale as usize)),
                _ => "\"1\"".into(),
            };
            let out = run(
                cert,
                &def.parse_definition,
                vec![doc(&text), word(0), word_tag(0)],
            );
            let scale = usize::from(def.codec.scale.unwrap_or(0));
            // Exact fixed text retains scale and coefficient (1.0 -> 10, 1).
            // 10^scale has its first set coefficient bit at `scale`.
            let mut samples = vec![
                (0, true),
                (1, true),
                (2, false),
                (4 + (scale << 2), true),
                (384, false),
            ];
            samples.extend((0..5).map(|i| (3 + (i << 6), scale & (1 << i) != 0)));
            for (at, expected) in samples {
                assert_eq!(
                    leaf(cert, out.clone(), 10, at),
                    expected,
                    "decimal setting {:?}/{:?} bit {at}",
                    def.codec.scale,
                    def.codec.rounding
                );
            }
        }
        eprintln!("Quoted decimals: all 146 closed setting samples passed");
    });
}

#[test]
fn csharp_03_t06_w09_json_tokens_quoted_decimals() {
    with_program(|cert, d| {
        let doc = |s: &str| document(s.len() as u32, &s.bytes().enumerate().collect::<Vec<_>>());
        let mut count = 0;
        let mut check = |scale: Option<u8>,
                         doc: V,
                         start: u32,
                         ending: u8,
                         expected: Option<(bool, u8, u128, u32, bool)>| {
            let def = d
                .quoted_decimals
                .iter()
                .find(|v| {
                    v.codec.scale == scale
                        && (scale.is_none() || v.codec.rounding.as_deref() == Some("ToEven"))
                })
                .unwrap();
            let out = run(
                cert,
                &def.parse_definition,
                vec![doc, word(start), word_tag(ending)],
            );
            let mut wanted = [false; 1024];
            if let Some((negative, decimal_scale, coefficient, end, whole)) = expected {
                wanted[0] = true;
                wanted[1] = whole;
                wanted[2] = negative;
                for i in 0..8 {
                    wanted[2 + 1 + (i << 6)] = decimal_scale & (1 << i) != 0;
                }
                for i in 0..96 {
                    wanted[2 + 2 + (i << 2)] = coefficient & (1 << i) != 0;
                }
                for i in 0..32 {
                    wanted[514 + i] = end & (1 << i) != 0;
                }
            }
            for (at, expected) in wanted.into_iter().enumerate() {
                assert_eq!(
                    leaf(cert, out.clone(), 10, at),
                    expected,
                    "decimal case {count} bit {at}"
                );
            }
            count += 1;
            eprintln!("quoted decimal full packet {count}");
        };
        for (scale, text, negative, value_scale, coefficient) in [
            (None, "0", false, 0, 0),
            (None, "-12.34", true, 2, 1234),
            (
                None,
                "79228162514264337593543950335",
                false,
                0,
                (1u128 << 96) - 1,
            ),
            (None, "0.0000000000000000000000000001", false, 28, 1),
            (Some(4), "1.2300", false, 4, 12300),
            (Some(2), "0.00", false, 2, 0),
        ] {
            let text = format!("\"{text}\"");
            check(
                scale,
                doc(&text),
                0,
                0,
                Some((negative, value_scale, coefficient, text.len() as u32, true)),
            );
        }
        for (ending, suffix) in [(1, ","), (2, "]"), (3, "}")] {
            check(
                None,
                doc(&format!("xx\"-1\"{suffix}false")),
                2,
                ending,
                Some((true, 0, 1, 6, false)),
            );
        }
        for (scale, input) in [
            (None, "\"-0\""),
            (None, "\"1.0\""),
            (None, "\"+1\""),
            (None, "\"01\""),
            (None, "\"1e2\""),
            (None, "\"1 \""),
            (None, "\"\\u0031\""),
            (None, "\"１\""),
            (None, "1"),
            (Some(2), "\"1.2\""),
            (None, "\"79228162514264337593543950336\""),
            (None, "\"0.00000000000000000000000000001\""),
        ] {
            check(scale, doc(input), 0, 0, None);
        }
        for ending in [4, 0x80] {
            check(None, doc("\"1\":0"), 0, ending, None);
        }
        check(None, doc("\"1\"x"), 0, 0, None);
        // Sparse observation uses the full original u32 bound and absolute
        // offset, including the last permitted document byte.
        let start = 1_048_572;
        let bytes = b"\"1\"}"
            .iter()
            .copied()
            .enumerate()
            .map(|(i, b)| (start as usize + i, b))
            .collect::<Vec<_>>();
        check(
            None,
            document(1_048_576, &bytes),
            start,
            3,
            Some((false, 0, 1, start + 3, false)),
        );
        check(None, document(1_048_577, &bytes), start, 3, None);
        check(
            None,
            document(u32::MAX, &[(0, b'"'), (1, b'1'), (2, b'"')]),
            0,
            0,
            None,
        );
        check(None, doc("\"1\""), u32::MAX, 0, None);
        eprintln!("Quoted decimals: {count} all-1024-bit packets");
    });
}

fn word_tag(value: u8) -> V {
    sparse_cube(3, (0..8).filter(|i| value & (1 << i) != 0).collect())
}

#[test]
fn csharp_03_t06_w09_json_tokens_quoted_hex_codecs() {
    with_program(|cert, d| {
        assert_eq!(
            d.quoted_codecs
                .iter()
                .map(|v| v.codec.codec_id.as_str())
                .collect::<Vec<_>>(),
            ["binary32", "binary64", "guid.n", "guid.d"]
        );
        let mut cases = 0;
        let mut check =
            |id: &str, doc: V, start: u32, ending: u8, expected: Option<(u128, u32, bool)>| {
                let def = d
                    .quoted_codecs
                    .iter()
                    .find(|v| v.codec.codec_id == id)
                    .unwrap();
                let tag = sparse_cube(3, (0..8).filter(|i| ending & (1 << i) != 0).collect());
                let out = run(cert, &def.parse_definition, vec![doc, word(start), tag]);
                let mut bits = [false; 256];
                if let Some((value, end, whole)) = expected {
                    bits[0] = true;
                    bits[1] = whole;
                    for i in 0..128 {
                        bits[2 + i] = value & (1u128 << i) != 0;
                    }
                    for i in 0..32 {
                        bits[130 + i] = end & (1 << i) != 0;
                    }
                }
                for (i, bit) in bits.into_iter().enumerate() {
                    assert_eq!(
                        leaf(cert, out.clone(), 8, i),
                        bit,
                        "quoted codec case {cases}, {id}, bit {i}"
                    );
                }
                cases += 1;
            };
        let doc = |s: &str| document(s.len() as u32, &s.bytes().enumerate().collect::<Vec<_>>());
        // Exact bit patterns, not host floating-point equality. In particular
        // preserve NaN payloads, negative zero and both ends of a GUID.
        for (id, text, value) in [
            ("binary32", "80000000", 0x80000000),
            ("binary32", "ffc12345", 0xffc12345),
            ("binary64", "8000000000000000", 0x8000000000000000),
            ("binary64", "7ff8123456789abc", 0x7ff8123456789abc),
            (
                "guid.n",
                "fedcba98765432100123456789abcdef",
                0xfedcba98765432100123456789abcdef,
            ),
            (
                "guid.d",
                "fedcba98-7654-3210-0123-456789abcdef",
                0xfedcba98765432100123456789abcdef,
            ),
        ] {
            let input = format!("\"{text}\"");
            check(
                id,
                doc(&input),
                0,
                0,
                Some((value, input.len() as u32, true)),
            );
        }
        for (id, text) in [
            ("binary32", "00000000"),
            ("binary64", "0000000000000000"),
            ("guid.n", "00000000000000000000000000000000"),
            ("guid.d", "00000000-0000-0000-0000-000000000000"),
        ] {
            for (ending, suffix) in [(1, ","), (2, "]"), (3, "}")] {
                let input = format!("xx\"{text}\"{suffix}false");
                check(
                    id,
                    doc(&input),
                    2,
                    ending,
                    Some((0, text.len() as u32 + 4, false)),
                );
            }
            for suffix in ["x", " "] {
                let input = format!("\"{text}{suffix}\"");
                check(id, doc(&input), 0, 0, None);
            }
            let input = format!("\"{text}\":0");
            check(id, doc(&input), 0, 4, None);
            check(id, doc(&input), 0, 0x84, None);
        }
        for (id, input) in [
            ("binary32", "\"FFc12345\""),
            ("binary32", "\"0000000\""),
            ("binary32", "\"\\u00300000000\""),
            ("binary32", "\"０0000000\""),
            ("binary32", "00000000"),
            ("binary64", "null"),
            ("guid.n", "\"fedcba98-7654-3210-0123-456789abcdef\""),
            ("guid.d", "\"fedcba98765432100123456789abcdef\""),
            ("guid.d", "\"fedcba98_7654-3210-0123-456789abcdef\""),
        ] {
            check(id, doc(input), 0, 0, None);
        }
        let token = b"\"80000000\"}";
        let start = 1_048_576 - token.len() as u32;
        let bytes = token
            .iter()
            .copied()
            .enumerate()
            .map(|(i, b)| (start as usize + i, b))
            .collect::<Vec<_>>();
        check(
            "binary32",
            document(1_048_576, &bytes),
            start,
            3,
            Some((0x80000000, 1_048_575, false)),
        );
        check("binary32", document(1_048_575, &bytes), start, 3, None);
        for length in [1_048_577, 1 << 31, u32::MAX] {
            check(
                "binary32",
                document(
                    length,
                    &b"\"00000000\""
                        .iter()
                        .copied()
                        .enumerate()
                        .collect::<Vec<_>>(),
                ),
                0,
                0,
                None,
            );
        }
        eprintln!("Quoted hex codecs: {cases} all-256-bit cases; exact f32/f64/Guid payloads, code-unit/length/canonical checks, end framing and full document bounds");
    });
}

#[test]
fn csharp_03_t06_w09_json_tokens_quoted_scalars() {
    with_program(|cert, d| {
        assert_eq!(
            d.quoted_scalars
                .iter()
                .map(|v| v.kind.as_str())
                .collect::<Vec<_>>(),
            ["char", "i64", "u64", "duration", "instant"]
        );
        // Raw numeric entry points remain exactly the admitted small kinds.
        assert_eq!(
            d.scalars
                .iter()
                .map(|v| v.kind.as_str())
                .collect::<Vec<_>>(),
            ["bool", "null", "u8", "u16", "u32", "i8", "i16", "i32"]
        );
        let mut cases = 0;
        let mut check =
            |kind: &str, doc: V, start: u32, ending: u8, expected: Option<(u64, u32, bool)>| {
                let def = d.quoted_scalars.iter().find(|v| v.kind == kind).unwrap();
                let tag = sparse_cube(3, (0..8).filter(|i| ending & (1 << i) != 0).collect());
                let out = run(cert, &def.parse_definition, vec![doc, word(start), tag]);
                let mut bits = [false; 128];
                if let Some((value, end, whole)) = expected {
                    bits[0] = true;
                    bits[1] = whole;
                    for i in 0..64 {
                        bits[2 + i] = value & (1 << i) != 0;
                    }
                    for i in 0..32 {
                        bits[66 + i] = end & (1 << i) != 0;
                    }
                }
                for (i, bit) in bits.into_iter().enumerate() {
                    assert_eq!(
                        leaf(cert, out.clone(), 7, i),
                        bit,
                        "quoted case {cases}, kind {kind}, bit {i}"
                    );
                }
                cases += 1;
            };
        let doc = |s: &str| document(s.len() as u32, &s.bytes().enumerate().collect::<Vec<_>>());
        for (kind, token, value) in [
            ("char", "\"A\"", 65),
            ("char", "\"\\u0000\"", 0),
            ("char", "\"\\ud800\"", 0xd800),
            ("char", "\"\\udfff\"", 0xdfff),
            ("char", "\"é\"", 0xe9),
            ("i64", "\"0\"", 0),
            ("i64", "\"-1\"", u64::MAX),
            ("i64", "\"-9223372036854775808\"", 1 << 63),
            ("i64", "\"9223372036854775807\"", (1 << 63) - 1),
            ("u64", "\"0\"", 0),
            ("u64", "\"18446744073709551615\"", u64::MAX),
            ("duration", "\"-9223372036854775808\"", 1 << 63),
            ("instant", "\"9223372036854775807\"", (1 << 63) - 1),
        ] {
            check(
                kind,
                doc(token),
                0,
                0,
                Some((value, token.len() as u32, true)),
            );
        }
        for (kind, token, value) in [
            ("char", "\"x\"", 120),
            ("i64", "\"-7\"", (-7i64) as u64),
            ("u64", "\"18446744073709551615\"", u64::MAX),
        ] {
            for (ending, suffix) in [(1, ","), (2, "]"), (3, "}")] {
                let input = format!("xx{token}{suffix}false");
                check(
                    kind,
                    doc(&input),
                    2,
                    ending,
                    Some((value, token.len() as u32 + 2, false)),
                );
            }
        }
        for (kind, token) in [
            ("char", "\"\""),
            ("char", "\"ab\""),
            ("char", "\"😀\""),
            ("char", "null"),
            ("char", "65"),
            ("i64", "0"),
            ("u64", "1"),
            ("duration", "-1"),
            ("instant", "0"),
            ("i64", "\"-9223372036854775809\""),
            ("i64", "\"9223372036854775808\""),
            ("u64", "\"18446744073709551616\""),
            ("u64", "\"-1\""),
            ("i64", "\"-0\""),
            ("i64", "\"+1\""),
            ("u64", "\"01\""),
            ("i64", "\"\""),
            ("i64", "\"-\""),
            ("i64", "\"1x\""),
            ("i64", "\"1 \""),
            ("u64", "\" 1\""),
            ("u64", "\"1e0\""),
            ("i64", "\"\\u0031\""),
            ("u64", "\"１\""),
            ("u64", "\"\\u0000\""),
        ] {
            check(kind, doc(token), 0, 0, None);
        }
        for kind in ["char", "i64", "u64", "duration", "instant"] {
            check(kind, doc("\"1\":0"), 0, 4, None);
            check(kind, doc("\"1\",0"), 0, 0x81, None);
            check(kind, doc("\"1\",0"), 0, 0, None);
        }
        let bytes = [
            (1_048_572, b'"'),
            (1_048_573, b'1'),
            (1_048_574, b'"'),
            (1_048_575, b'}'),
        ];
        check(
            "u64",
            document(1_048_576, &bytes),
            1_048_572,
            3,
            Some((1, 1_048_575, false)),
        );
        check("u64", document(1_048_575, &bytes), 1_048_572, 3, None);
        for kind in ["char", "i64", "u64"] {
            check(
                kind,
                document(1 << 31, &[(0, b'"'), (1, b'1'), (2, b'"')]),
                0,
                0,
                None,
            );
            check(kind, doc("\"1\""), u32::MAX, 0, None);
        }
        eprintln!("Quoted semantic scalars: {cases} all-128-bit cases; char UTF-16 count, full integer ranges, exact inner consumption, framing and full-u32 bounds");
    });
}

#[test]
fn csharp_03_t06_w09_json_tokens_string_framing() {
    with_program(|cert, d| {
        let mut cases = 0;
        let mut check =
            |input: V, start: u32, ending: u8, expected: Option<(u32, bool, Vec<u16>)>| {
                let tag = sparse_cube(3, (0..8).filter(|i| ending & (1 << i) != 0).collect());
                let result = run(
                    cert,
                    &d.string_frame_definition,
                    vec![input, word(start), tag],
                );
                let header = run(
                    cert,
                    &d.string_frame_header_definition,
                    vec![result.clone()],
                );
                let value = run(cert, &d.string_frame_value_definition, vec![result.clone()]);
                let mut bits = [false; 128];
                let units = if let Some((end, whole, units)) = expected {
                    bits[0] = true;
                    bits[1] = whole;
                    for i in 0..32 {
                        bits[2 + i] = end & (1 << i) != 0;
                        bits[34 + i] = units.len() & (1 << i) != 0;
                    }
                    units
                } else {
                    vec![]
                };
                for (i, expected) in bits.into_iter().enumerate() {
                    assert_eq!(
                        leaf(cert, header.clone(), 7, i),
                        expected,
                        "frame {cases} header {i}"
                    );
                    assert_eq!(
                        leaf(cert, result.clone(), 20, i << 13),
                        expected,
                        "frame {cases} packed header {i}"
                    );
                }
                for i in 0..32 {
                    assert_eq!(
                        leaf(cert, value.clone(), 19, i << 14),
                        units.len() & (1 << i) != 0,
                        "frame {cases} length {i}"
                    );
                }
                for (index, unit) in units.iter().copied().enumerate().chain([
                    (units.len(), 0),
                    (8192, 0),
                    (16383, 0),
                ]) {
                    for i in 0..16 {
                        assert_eq!(
                            leaf(cert, value.clone(), 19, 1 | (index << 1) | (i << 15)),
                            unit & (1 << i) != 0,
                            "frame {cases} unit {index} bit {i}"
                        );
                    }
                }
                for i in 1..13 {
                    assert!(
                        !leaf(cert, result.clone(), 20, 1 << i),
                        "frame header padding"
                    );
                }
                for i in 1..14 {
                    assert!(
                        !leaf(cert, value.clone(), 19, 1 << i),
                        "frame value padding"
                    );
                }
                cases += 1;
            };
        let doc = |bytes: &[u8]| {
            document(
                bytes.len() as u32,
                &bytes.iter().copied().enumerate().collect::<Vec<_>>(),
            )
        };
        // All legal enclosing endings, empty/nonempty values, absolute cursor,
        // and a multi-byte, two-unit Unicode scalar. Preceding bytes are owned
        // by the enclosing grammar and deliberately need not form a JSON value.
        for (token, units) in [
            ("\"\"", vec![]),
            ("\"a😀\"", vec![97, 0xd83d, 0xde00]),
            ("\"\\u0000\"", vec![0]),
        ] {
            for (ending, suffix) in [(0, ""), (1, ","), (2, "]"), (3, "}"), (4, ":")] {
                let bytes = format!("xx{token}{suffix}false");
                let bytes = if ending == 0 {
                    format!("xx{token}")
                } else {
                    bytes
                };
                check(
                    doc(bytes.as_bytes()),
                    2,
                    ending,
                    Some((2 + token.len() as u32, ending == 0, units.clone())),
                );
            }
        }
        for (bytes, start, ending) in [
            (b"\"x\":0".as_slice(), 0, 1),
            (b"\"x\",0", 0, 4),
            (b"\"x\"", 0, 1),
            (b"\"x\" ", 0, 0),
            (b"\"x\" :0", 0, 4),
            (b"\"x\",0", 0, 0),
            (b"\"x\",0", 0, 5),
            (b"\"x\",0", 0, 0x81),
            (b"\"x\":0", 0, 0x84),
            (b"\"\\u0041\":0", 0, 4),
            (b"\"\xff\":0", 0, 4),
            (b"\"x\":0", 1, 4),
            (b"\"x\":0", 6, 4),
            (b"\"x\":0", u32::MAX, 4),
            (b"null", 0, 0),
            (b"\"", 0, 0),
            (b"", 0, 0),
        ] {
            check(doc(bytes), start, ending, None);
        }
        // Exact live delimiter and full-u32 document/cursor checks, including
        // high offsets that cannot be tested by truncating to the small input.
        for start in [0, 65_535, 524_285, 1_048_572] {
            let bytes = [
                (start as usize, b'"'),
                (start as usize + 1, b'x'),
                (start as usize + 2, b'"'),
                (start as usize + 3, b':'),
            ];
            check(
                document(start + 4, &bytes),
                start,
                4,
                Some((start + 3, false, vec![120])),
            );
            check(document(start + 3, &bytes), start, 4, None);
        }
        for length in [1_048_577, 1 << 24, 1 << 31, u32::MAX] {
            check(
                document(length, &[(0, b'"'), (1, b'"'), (2, b':')]),
                0,
                4,
                None,
            );
        }
        eprintln!("JSON framed strings: {cases} cursor/delimiter/UTF-16/full-u32 cases with header/value/padding checks");
    });
}

#[test]
fn csharp_03_t06_w09_json_tokens_scalar_grammar() {
    with_program(|cert, d| {
        assert_eq!(
            d.scalars
                .iter()
                .map(|s| s.kind.as_str())
                .collect::<Vec<_>>(),
            ["bool", "null", "u8", "u16", "u32", "i8", "i16", "i32"]
        );
        let mut cases = 0;
        let mut check = |id: &str,
                         input: V,
                         start: u32,
                         ending: u8,
                         expected: Option<(u64, u32)>,
                         whole: bool| {
            let def = d.scalars.iter().find(|s| s.kind == id).unwrap();
            let ending = sparse_cube(3, (0..8).filter(|i| ending & (1 << i) != 0).collect());
            let output = run(
                cert,
                &def.parse_definition,
                vec![input, word(start), ending],
            );
            let mut bits = [false; 128];
            if let Some((value, end)) = expected {
                bits[0] = true;
                bits[1] = whole;
                for i in 0..64 {
                    bits[2 + i] = value & (1 << i) != 0;
                }
                for i in 0..32 {
                    bits[66 + i] = end & (1 << i) != 0;
                }
            }
            for (i, expected) in bits.into_iter().enumerate() {
                assert_eq!(
                    leaf(cert, output.clone(), 7, i),
                    expected,
                    "scalar {id}, start {start}, packet bit {i}"
                );
            }
            cases += 1;
        };
        // Every exposed type, all four enclosing delimiter choices, nonzero
        // cursors, negative minima and nonzero high payload bits.
        for (id, token, value) in [
            ("bool", "true", 1),
            ("null", "null", 0),
            ("u8", "255", 255),
            ("u16", "65535", 65535),
            ("u32", "4294967295", u32::MAX as u64),
            ("i8", "-128", 128),
            ("i16", "-32768", 32768),
            ("i32", "-2147483648", 1 << 31),
        ] {
            for (ending, suffix) in [(0, ""), (1, ","), (2, "]"), (3, "}")] {
                let bytes = format!("xx{token}{suffix}").into_bytes();
                let cells = bytes.iter().copied().enumerate().collect::<Vec<_>>();
                check(
                    id,
                    document(bytes.len() as u32, &cells),
                    2,
                    ending,
                    Some((value, token.len() as u32 + 2)),
                    ending == 0,
                );
            }
        }
        for (id, token) in [
            ("bool", "null"),
            ("null", "false"),
            ("u8", "256"),
            ("u16", "65536"),
            ("u32", "4294967296"),
            ("i8", "-129"),
            ("i16", "32768"),
            ("i32", "2147483648"),
        ] {
            let cells = token.bytes().enumerate().collect::<Vec<_>>();
            check(id, document(token.len() as u32, &cells), 0, 0, None, false);
        }
        for (id, token, value) in [("bool", "false", 0), ("i8", "-7", 249)] {
            // Lexically valid prefixes cannot escape the enclosing grammar.
            for suffix in [
                "x", "true", "false", ":", "[", "{", "\"", " ", "\n", ".0", "e0",
            ] {
                let bytes = format!("{token}{suffix}").into_bytes();
                let cells = bytes.iter().copied().enumerate().collect::<Vec<_>>();
                check(id, document(bytes.len() as u32, &cells), 0, 0, None, false);
            }
            for (suffix, actual) in [("", 0), (",", 1), ("]", 2), ("}", 3)] {
                let bytes = format!("{token}{suffix}").into_bytes();
                let cells = bytes.iter().copied().enumerate().collect::<Vec<_>>();
                for ending in 0..4 {
                    if ending != actual {
                        check(
                            id,
                            document(bytes.len() as u32, &cells),
                            0,
                            ending,
                            None,
                            false,
                        );
                    }
                }
            }
            // Rejected ending tags retain all eight bits, not just the low two.
            let cells = token.bytes().enumerate().collect::<Vec<_>>();
            for ending in [4, 5, 8, 16, 32, 64, 128, 255] {
                check(
                    id,
                    document(token.len() as u32, &cells),
                    0,
                    ending,
                    None,
                    false,
                );
            }
            check(
                id,
                document(token.len() as u32, &cells),
                0,
                0,
                Some((value, token.len() as u32)),
                true,
            );
        }
        const MAX: u32 = 1_048_576;
        // Sparse documents exercise high offset/length bits without an allocated
        // prefix; inactive delimiter bytes must not create a successful frame.
        for length in [0, 3, 4, MAX, MAX + 1, 1 << 31, u32::MAX] {
            let cells = b"true,".iter().copied().enumerate().collect::<Vec<_>>();
            check(
                "bool",
                document(length, &cells),
                0,
                1,
                (length == MAX).then_some((1, 4)),
                false,
            );
        }
        for start in [4, 5, MAX, MAX + 1, 1 << 31, u32::MAX] {
            let cells = b"true".iter().copied().enumerate().collect::<Vec<_>>();
            check("bool", document(4, &cells), start, 0, None, false);
        }
        for (id, token, value) in [("bool", "true", 1), ("i32", "-2147483648", 1 << 31)] {
            let start = MAX - token.len() as u32;
            let cells = token
                .bytes()
                .enumerate()
                .map(|(i, b)| (start as usize + i, b))
                .collect::<Vec<_>>();
            check(
                id,
                document(MAX, &cells),
                start,
                0,
                Some((value, MAX)),
                true,
            );
        }
        eprintln!(
            "JSON scalar grammar: {cases} cursor/type/delimiter/bound cases, all 128 packet bits"
        );
    });
}

#[test]
fn csharp_03_t06_w09_structural_boundary_original_sources() {
    use super::super::structural_equivalence_tests::same_definition_closure;
    fn without_counts(v: &mut Value) {
        match v {
            Value::Object(fields) => {
                fields.remove("static_transformers");
                fields.values_mut().for_each(without_counts);
            }
            Value::Array(values) => values.iter_mut().for_each(without_counts),
            _ => (),
        }
    }
    let bundle = b();
    let mut sources = document_sources()
        .into_iter()
        .filter(|(id, _, _)| id.starts_with("document-"))
        .collect::<Vec<_>>();
    sources.push(
        super::super::structural_foundation_tests::sources()
            .into_iter()
            .find(|(id, _, _)| id == "unit-void-source")
            .unwrap(),
    );
    let out = std::env::var_os("MPK_W09_STRUCTURAL_BOUNDARY_OUT").map(std::path::PathBuf::from);
    if let Some(dir) = &out {
        fs::create_dir_all(dir).unwrap();
    }
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../develop/migrations/csharp-03/ordinary-foundation/structural-boundary");
    let mut rows = vec![];
    let mut previous: Option<(Vec<u8>, Vec<u8>)> = None;
    for (id, row, facts) in sources {
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
        let structural = generate_csharp_practical_ordinary_structural_foundations(vir).unwrap();
        assert!(structural.boundary_json().is_none());
        assert!(structural.boundary_program_sha256().is_none());
        let json = generate_csharp_practical_ordinary_json_tokens(vir).unwrap();
        let combined = generate_csharp_practical_ordinary_structural_boundary(vir)
            .unwrap_or_else(|e| panic!("{id}: {e:?}"));
        let meta: Value = serde_json::from_slice(&combined.canonical_bytes()).unwrap();
        let structural_meta: Value = serde_json::from_slice(&structural.canonical_bytes()).unwrap();
        let json_meta: Value = serde_json::from_slice(&json.canonical_bytes()).unwrap();
        assert_eq!(
            combined.boundary_program_sha256(),
            json_meta["boundary_program_sha256"].as_str()
        );
        assert_eq!(
            combined.boundary_json().is_some(),
            json.definition().is_some()
        );
        assert_eq!(combined.boundary_json().is_some(), id != "unit-void-source");
        for (key, value) in structural_meta.as_object().unwrap() {
            if !["schema", "static_transformers", "certificate_sha256"].contains(&key.as_str()) {
                assert_eq!(&meta[key], value, "{id}: structural metadata {key}");
            }
        }
        assert_eq!(meta["schema"], "mpk.csharp.ordinary_structural_boundary.v1");
        let cert = mpk_cert::decode_canonical_certificate(combined.certificate_bytes()).unwrap();
        validate_csharp_practical_certificate_structure(&cert).unwrap();
        let mut checked = 0;
        for component in [structural.certificate_bytes(), json.certificate_bytes()] {
            let old = mpk_cert::decode_canonical_certificate(component).unwrap();
            let roots = old
                .declarations
                .iter()
                .map(|d| old.name_table[d.name as usize].clone())
                .collect();
            checked += same_definition_closure(&old, &cert, &roots).unwrap().len();
        }
        if let Some(def) = combined.boundary_json() {
            let mut actual = serde_json::to_value(def).unwrap();
            let mut expected = json_meta["definition"].clone();
            without_counts(&mut actual);
            without_counts(&mut expected);
            assert_eq!(actual, expected, "{id}: boundary definitions");
            assert_eq!(def.static_transformers, combined.static_transformers());
            assert!(combined.static_transformers() >= structural.static_transformers());
        } else {
            assert_eq!(structural.certificate_bytes(), combined.certificate_bytes());
        }
        assert!(combined.static_transformers() <= 16_384);
        assert_eq!(
            import_csharp_practical_ordinary_structural_boundary(
                &combined.canonical_bytes(),
                combined.certificate_bytes(),
                vir
            )
            .unwrap(),
            combined
        );
        assert!(import_csharp_practical_ordinary_structural_foundations(
            &combined.canonical_bytes(),
            combined.certificate_bytes(),
            vir
        )
        .is_err());
        assert!(import_csharp_practical_ordinary_structural_boundary(
            &structural.canonical_bytes(),
            structural.certificate_bytes(),
            vir
        )
        .is_err());
        for field in [
            "schema",
            "source_ir_sha256",
            "foundation_sha256",
            "boundary_program_sha256",
            "boundary_json",
            "static_transformers",
            "certificate_sha256",
            "deferred_instances",
        ] {
            let mut forged = meta.clone();
            forged[field] = json!("forged");
            assert!(import_csharp_practical_ordinary_structural_boundary(
                &serde_json::to_vec(&forged).unwrap(),
                combined.certificate_bytes(),
                vir
            )
            .is_err());
        }
        let mut corrupt = combined.certificate_bytes().to_vec();
        *corrupt.last_mut().unwrap() ^= 1;
        assert!(import_csharp_practical_ordinary_structural_boundary(
            &combined.canonical_bytes(),
            &corrupt,
            vir
        )
        .is_err());
        if let Some((m, c)) = &previous {
            assert!(import_csharp_practical_ordinary_structural_boundary(m, c, vir).is_err());
        }
        previous = Some((
            combined.canonical_bytes(),
            combined.certificate_bytes().to_vec(),
        ));
        let hex = combined
            .certificate_bytes()
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect::<String>()
            + "\n";
        if let Some(dir) = &out {
            fs::write(dir.join(format!("{id}.hex")), hex).unwrap();
        } else {
            assert_eq!(
                fs::read_to_string(root.join(format!("{id}.hex"))).unwrap(),
                hex
            );
        }
        eprintln!("Structural boundary {id}: {} terms, {} declarations, {} transformers, {checked} checked dependency declarations", cert.term_table.len(), cert.declarations.len(), combined.static_transformers());
        rows.push(json!({"id":id,"metadata":meta,"terms":cert.term_table.len(),"declarations":cert.declarations.len(),"checked_component_declarations":checked}));
    }
    assert_eq!(rows.len(), 4);
    let manifest = json!({"sources":rows});
    if let Some(dir) = &out {
        fs::write(
            dir.join("certificates.json"),
            serde_json::to_vec_pretty(&manifest).unwrap(),
        )
        .unwrap();
    } else {
        assert_eq!(
            read("ordinary-foundation/structural-boundary/certificates.json"),
            manifest
        );
    }
}

#[test]
fn csharp_03_t06_w09_json_tokens_original_sources() {
    let bundle = b();
    let out = std::env::var_os("MPK_W09_JSON_TOKENS_OUT").map(std::path::PathBuf::from);
    if let Some(dir) = &out {
        fs::create_dir_all(dir).unwrap();
    }
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../develop/migrations/csharp-03/ordinary-foundation/json-tokens");
    let mut rows = vec![];
    let mut observed = false;
    let mut previous: Option<(Vec<u8>, Vec<u8>)> = None;
    let mut examined = 0;
    for (id, row, facts) in document_sources() {
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
        let p = generate_csharp_practical_ordinary_json_tokens(vir)
            .unwrap_or_else(|e| panic!("{id}: {e:?}"));
        let vc = generate_csharp_practical_vc(PracticalVcSource {
            artifact_context: &context,
            captured_inputs: &captures,
            vir,
        })
        .unwrap();
        let boundary = vc.boundary_vcs();
        assert_eq!(p.definition().is_none(), boundary.contracts().is_empty());
        let meta: Value = serde_json::from_slice(&p.canonical_bytes()).unwrap();
        let cert = mpk_cert::decode_canonical_certificate(p.certificate_bytes()).unwrap();
        validate_csharp_practical_certificate_structure(&cert).unwrap();
        assert_eq!(
            import_csharp_practical_ordinary_json_tokens(
                &p.canonical_bytes(),
                p.certificate_bytes(),
                vir
            )
            .unwrap(),
            p
        );
        for field in [
            "schema",
            "source_ir_sha256",
            "foundation_sha256",
            "boundary_program_sha256",
            "definition",
            "certificate_sha256",
        ] {
            let mut forged = meta.clone();
            forged[field] = json!("forged");
            assert!(import_csharp_practical_ordinary_json_tokens(
                &serde_json::to_vec(&forged).unwrap(),
                p.certificate_bytes(),
                vir
            )
            .is_err());
        }
        let mut bad = p.certificate_bytes().to_vec();
        *bad.last_mut().unwrap() ^= 1;
        assert!(
            import_csharp_practical_ordinary_json_tokens(&p.canonical_bytes(), &bad, vir).is_err()
        );
        if let Some((m, c)) = previous.as_ref() {
            assert!(import_csharp_practical_ordinary_json_tokens(m, c, vir).is_err());
        }
        previous = Some((p.canonical_bytes(), p.certificate_bytes().to_vec()));
        let Some(d) = p.definition() else { continue };
        assert_eq!(d.strings.document.depth, 24);
        assert!(d.static_transformers > 0 && d.static_transformers <= 16_384);
        assert_eq!(
            d.unsigned.iter().map(|n| n.width).collect::<Vec<_>>(),
            [8, 16, 32, 64]
        );
        assert_eq!(
            d.signed.iter().map(|n| n.width).collect::<Vec<_>>(),
            [8, 16, 32]
        );
        // Reassociation changes only Scan's composition tree. Compare every
        // archived definition and the exact expanded ordered packet sequence.
        for archive in [
            "previous-unsigned",
            "previous-signed",
            "previous-scalars",
            "previous-string-framing",
            "previous-quoted-scalars",
            "previous-quoted-codecs",
            "previous-quoted-decimal",
        ] {
            let hex = fs::read_to_string(root.join(archive).join(format!("{id}.hex"))).unwrap();
            let bytes = (0..hex.trim().len())
                .step_by(2)
                .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).unwrap())
                .collect::<Vec<_>>();
            let old = mpk_cert::decode_canonical_certificate(&bytes).unwrap();
            same_json_scan(&old, &cert).unwrap();
        }
        // Compare every dependency of each reused codec against its previously
        // checked standalone certificate. Two sources cover all four codecs;
        // no standalone codec semantic matrix needs to be executed again.
        let codec_root = root.parent().unwrap().join("hex-codecs");
        let decimal = super::super::structural_equivalence_tests::certificate(
            "decimal-parsers",
            "binding-vc-money.hex",
        );
        let decimal_meta = read("ordinary-foundation/decimal-parsers/certificates.json");
        let decimal_defs = &decimal_meta["sources"]
            .as_array()
            .unwrap()
            .iter()
            .find(|r| r["id"] == "binding-vc-money")
            .unwrap()["metadata"]["definitions"];
        assert_eq!(
            serde_json::to_value(
                d.quoted_decimals
                    .iter()
                    .map(|v| &v.codec)
                    .collect::<Vec<_>>()
            )
            .unwrap(),
            *decimal_defs
        );
        let decimal_roots = decimal
            .declarations
            .iter()
            .map(|v| decimal.name_table[v.name as usize].clone())
            .collect();
        super::super::structural_equivalence_tests::same_definition_closure(
            &decimal,
            &cert,
            &decimal_roots,
        )
        .unwrap();
        let codec_manifest: Value =
            serde_json::from_slice(&fs::read(codec_root.join("certificates.json")).unwrap())
                .unwrap();
        for source in ["floats", "guid"] {
            let row = codec_manifest["sources"]
                .as_array()
                .unwrap()
                .iter()
                .find(|r| r["id"] == source)
                .unwrap();
            let hex = fs::read_to_string(codec_root.join(format!("{source}.hex"))).unwrap();
            let bytes = (0..hex.trim().len())
                .step_by(2)
                .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).unwrap())
                .collect::<Vec<_>>();
            let old = mpk_cert::decode_canonical_certificate(&bytes).unwrap();
            let mut roots = BTreeSet::new();
            for old_def in row["metadata"]["definitions"].as_array().unwrap() {
                let def = d
                    .quoted_codecs
                    .iter()
                    .find(|v| v.codec.codec_id == old_def["codec_id"].as_str().unwrap())
                    .unwrap();
                assert_eq!(serde_json::to_value(&def.codec).unwrap(), *old_def);
                roots.extend([
                    def.codec.parse_definition.clone(),
                    def.codec.format_definition.clone(),
                ]);
            }
            super::super::structural_equivalence_tests::same_definition_closure(
                &old, &cert, &roots,
            )
            .unwrap();
        }

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
                fs::read_to_string(root.join(format!("{id}.hex"))).unwrap(),
                hex
            );
        }
        rows.push(json!({"id":id,"metadata":meta,"terms":cert.term_table.len(),"declarations":cert.declarations.len()}));
        if !observed {
            check_unsigned(&cert, &d.unsigned[0], document(0, &[]), None, false);
            observed = true;
        }
    }
    assert_eq!(examined, 68);
    assert_eq!(rows.len(), 3);
    assert!(observed);
    eprintln!("JSON tokens contexts {}, nonempty {}", examined, rows.len());
    let manifest = json!({"contexts_examined":examined,"sources":rows});
    if let Some(dir) = &out {
        fs::write(
            dir.join("certificates.json"),
            serde_json::to_vec_pretty(&manifest).unwrap(),
        )
        .unwrap();
    } else {
        assert_eq!(
            read("ordinary-foundation/json-tokens/certificates.json"),
            manifest
        );
    }
}
