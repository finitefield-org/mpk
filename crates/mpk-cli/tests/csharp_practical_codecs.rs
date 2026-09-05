//! CSHARP-03-T03-W10: source strings and the shared scalar codec relation.
use mpk_vc::csharp_practical_vir_model::*;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};
fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}
fn read(p: &str) -> Vec<u8> {
    fs::read(root().join(p)).unwrap()
}
fn json_file(p: &str) -> Value {
    serde_json::from_slice(&read(p)).unwrap()
}
fn ty(s: &str) -> String {
    format!("mpk.csharp.value.{s}.v1")
}
fn fixture() -> (
    ValidatedFoundationBundle,
    ValidatedClosedRootSet,
    ClosedInstanceSet,
) {
    let b = validate_registered_foundation_bundle(
        registered_foundation_descriptor_transport(),
        registered_foundation_definitions_transport(),
    )
    .unwrap();
    let roots:Vec<_>=["i8","u8","i16","u16","i32","u32","i64","u64","decimal","f32","f64","date","time","duration","instant","guid","string","char","bool"].into_iter().map(|p|json!({"origin":"codec_result","provenance_id":format!("codec.{p}"),"type":{"kind":"instance","template":"result","arguments":[{"kind":"primitive","id":p},{"kind":"primitive","id":"parse_error"}]}})).collect();
    let transport = canonical_closed_root_set_transport(&b, &json!(roots), &json!({})).unwrap();
    let r = validate_closed_root_set(&b, &transport).unwrap();
    let c = derive_closed_instances(&b, &r).unwrap();
    (b, r, c)
}
fn utf16hex(s: &str) -> Vec<u16> {
    assert_eq!(s.len() % 4, 0);
    (0..s.len())
        .step_by(4)
        .map(|i| u16::from_str_radix(&s[i..i + 4], 16).unwrap())
        .collect()
}
fn fields(v: &Value) -> Vec<(&str, &str)> {
    v["inputs"]
        .as_array()
        .unwrap()
        .iter()
        .map(|x| x.as_str().unwrap().split_once('=').unwrap())
        .collect()
}
fn token(codec: &str) -> &str {
    match codec {
        "binary32" => "f32",
        "binary64" => "f64",
        "decimal.fixed" | "decimal.normalized" => "decimal",
        "duration_ticks" => "duration",
        "unix_milliseconds" => "instant",
        "guid.n" | "guid.d" => "guid",
        p => p.strip_prefix("integer.").unwrap_or(p),
    }
}
fn raw_value(token: &str, data: &str) -> MonomorphicValue {
    let id = ty(token);
    match token {
        "decimal" => {
            let parts: Vec<_> = data
                .split(';')
                .map(|p| p.split_once('=').unwrap().1)
                .collect();
            MonomorphicValue::DecimalBits {
                type_id: id,
                negative: parts[0] == "1",
                scale: parts[1].parse().unwrap(),
                coefficient: u128::from_str_radix(parts[2], 16).unwrap().to_string(),
            }
        }
        "f32" => MonomorphicValue::F32Bits {
            type_id: id,
            bits: data.into(),
        },
        "f64" => MonomorphicValue::F64Bits {
            type_id: id,
            bits: data.into(),
        },
        "guid" => MonomorphicValue::Guid {
            type_id: id,
            n: data.into(),
        },
        "date" => MonomorphicValue::Date {
            type_id: id,
            day_number: data.parse().unwrap(),
        },
        "time" => MonomorphicValue::Time {
            type_id: id,
            ticks: data.into(),
        },
        "duration" => MonomorphicValue::Duration {
            type_id: id,
            ticks: data.into(),
        },
        "instant" => MonomorphicValue::Instant {
            type_id: id,
            milliseconds: data.into(),
        },
        t if t.starts_with('u') => MonomorphicValue::Unsigned {
            type_id: id,
            value: data.into(),
        },
        _ => MonomorphicValue::Signed {
            type_id: id,
            value: data.into(),
        },
    }
}
fn encoded(v: &MonomorphicValue) -> String {
    match v {
        MonomorphicValue::Signed { value, .. } | MonomorphicValue::Unsigned { value, .. } => {
            value.clone()
        }
        MonomorphicValue::F32Bits { bits, .. } | MonomorphicValue::F64Bits { bits, .. } => {
            bits.clone()
        }
        MonomorphicValue::Guid { n, .. } => n.clone(),
        MonomorphicValue::Date { day_number, .. } => day_number.to_string(),
        MonomorphicValue::Time { ticks, .. } | MonomorphicValue::Duration { ticks, .. } => {
            ticks.clone()
        }
        MonomorphicValue::Instant { milliseconds, .. } => milliseconds.clone(),
        MonomorphicValue::DecimalBits {
            negative,
            scale,
            coefficient,
            ..
        } => format!(
            "sign={};scale={scale:02};coefficient={:024x}",
            u8::from(*negative),
            coefficient.parse::<u128>().unwrap()
        ),
        MonomorphicValue::String { utf16, .. } => {
            utf16.iter().map(|u| format!("{u:04x}")).collect()
        }
        MonomorphicValue::Char { utf16, .. } => format!("{utf16:04x}"),
        MonomorphicValue::Bool { value, .. } => value.to_string(),
        _ => panic!("unhandled {v:?}"),
    }
}
#[test]
fn csharp_03_t03_w10_replays_every_frozen_codec_under_all_hostile_cultures() {
    let (b, r, c) = fixture();
    let record = json_file(
        "develop/migrations/csharp-03/probes/runtime-primitive-string-numeric-codec.json",
    );
    assert_eq!(
        format!(
            "{:x}",
            Sha256::digest(read(
                "develop/migrations/csharp-03/probes/runtime-primitive-string-numeric-codec.json"
            ))
        ),
        "0055835ce456fb9c438336332bc0e2a214d900c137eca34f90c3fcddd2688769"
    );
    let mut counts = Vec::new();
    for run in record["observations"]["culture_runs"].as_array().unwrap() {
        let mut count = 0;
        for v in run["vectors"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|v| v["family"].as_str().unwrap().starts_with("codec."))
        {
            count += 1;
            let op = v["operation"]
                .as_str()
                .unwrap()
                .strip_prefix("codec.")
                .unwrap();
            let (id, action) = op.rsplit_once('.').unwrap();
            let data = fields(v);
            let scale = data
                .iter()
                .find(|(k, _)| *k == "scale")
                .map(|(_, v)| v.parse().unwrap());
            let rounding = data
                .iter()
                .find(|(k, _)| *k == "rounding")
                .map(|(_, v)| *v)
                .or((id == "decimal.fixed").then_some("ToEven"));
            let codec = BoundaryCodec::new(id, &ty(token(id)), scale, rounding).unwrap();
            if action == "parse" {
                let (key, text) = data[0];
                let input = if key == "text_utf16_repeat" {
                    let (hex, n) = text.split_once(";count=").unwrap();
                    utf16hex(hex).repeat(n.parse().unwrap())
                } else {
                    assert_eq!(key, "text_utf16");
                    utf16hex(text)
                };
                let result = codec.parse(&input);
                if v["profile"]["kind"] == "error" {
                    let arm = result.unwrap_err();
                    assert_eq!(
                        format!(
                            "parse_error.{}",
                            serde_json::to_value(arm).unwrap().as_str().unwrap()
                        ),
                        v["profile"]["error_id"],
                        "{}",
                        v["id"]
                    );
                } else {
                    let value = result.unwrap_or_else(|e| panic!("{}: {e:?}", v["id"]));
                    assert_eq!(encoded(&value), v["profile"]["value"], "{}", v["id"]);
                    assert_eq!(
                        codec.format(&b, &r, &c, &value).unwrap(),
                        input,
                        "{}",
                        v["id"]
                    );
                }
                let result_id=csharp_practical_closed_instance_id(&b,&json!({"kind":"instance","template":"result","arguments":[{"kind":"primitive","id":token(id)},{"kind":"primitive","id":"parse_error"}]})).unwrap();
                let typed = codec.parse_typed(&b, &r, &c, &result_id, &input).unwrap();
                assert!(matches!(typed, MonomorphicValue::TaggedSum { .. }));
            } else {
                let value = if data[0].0 == "canonical" {
                    codec
                        .parse(&data[0].1.encode_utf16().collect::<Vec<_>>())
                        .unwrap()
                } else {
                    raw_value(token(id), data[0].1)
                };
                let output = codec.format(&b, &r, &c, &value).unwrap();
                if action == "format" {
                    assert_eq!(
                        String::from_utf16(&output).unwrap(),
                        v["profile"]["value"],
                        "{}",
                        v["id"]
                    );
                } else {
                    assert_eq!(action, "roundtrip");
                    let parsed = codec.parse(&output).unwrap();
                    assert_eq!(codec.format(&b, &r, &c, &parsed).unwrap(), output);
                    if id != "decimal.fixed" {
                        let structural =
                            generate_structural_program(&b, &r, &c, &ty(token(id))).unwrap();
                        if matches!(token(id), "f32" | "f64") {
                            assert_eq!(value, parsed);
                        } else {
                            assert!(structural.structural_equal(&value, &parsed).unwrap());
                        }
                    }
                    assert_eq!(v["profile"]["value"], "true");
                }
            }
        }
        counts.push(count);
    }
    assert_eq!(counts.len(), 3);
    assert_eq!(counts[0], 396);
    assert!(counts.iter().all(|n| *n == counts[0]));
}
#[test]
fn csharp_03_t03_w10_configuration_types_and_bounds_are_closed() {
    let (b, r, c) = fixture();
    assert_eq!(
        BoundaryCodec::new("general", &ty("i32"), None, None),
        Err(CodecError::UnknownCodec)
    );
    assert_eq!(
        BoundaryCodec::new("decimal.fixed", &ty("decimal"), Some(2), Some("locale")),
        Err(CodecError::UnknownRounding)
    );
    for id in ["integer.i32", "binary32", "date", "decimal.normalized"] {
        assert!(BoundaryCodec::new(id, &ty("bool"), None, None).is_err());
    }
    assert!(BoundaryCodec::new("integer.i32", &ty("i32"), Some(2), None).is_err());
    let codec = BoundaryCodec::new("integer.i32", &ty("i32"), None, None).unwrap();
    assert!(codec
        .format(&b, &r, &c, &raw_value("i32", "2147483648"))
        .is_err());
    assert!(codec.format(&b, &r, &c, &raw_value("u32", "1")).is_err());
    assert_eq!(codec.parse(&[0xd800]), Err(ParseErrorArm::Syntax));
    assert_eq!(
        codec.parse(&vec![0xd800; 16385]),
        Err(ParseErrorArm::InputBound)
    );
    let fixed =
        BoundaryCodec::new("decimal.fixed", &ty("decimal"), Some(29), Some("ToEven")).unwrap();
    assert_eq!(fixed.parse(&[b'x' as u16]), Err(ParseErrorArm::Syntax));
    assert_eq!(
        fixed.parse(&[b'0' as u16]),
        Err(ParseErrorArm::ScalePrecision)
    );
    assert!(fixed
        .format(
            &b,
            &r,
            &c,
            &raw_value("decimal", "sign=0;scale=00;coefficient=0")
        )
        .is_err());
}

fn string_operands(v: &Value) -> Vec<StringOperand> {
    fields(v)
        .into_iter()
        .map(|(k, s)| match k {
            "index" | "start" | "length" => StringOperand::Index {
                value: s.parse().unwrap(),
            },
            "char" | "left_char" | "right_char" => StringOperand::Char {
                utf16: u16::from_str_radix(s, 16).unwrap(),
            },
            "literal" => StringOperand::Text {
                utf16: Some(if s == "empty" {
                    vec![]
                } else {
                    utf16hex(&s.replace(',', ""))
                }),
            },
            _ => StringOperand::Text {
                utf16: if s == "null" { None } else { Some(utf16hex(s)) },
            },
        })
        .collect()
}
#[test]
fn csharp_03_t03_w10_string_source_fixture_matches_frozen_runtime_and_rust() {
    let frozen = json_file(
        "develop/migrations/csharp-03/probes/runtime-primitive-string-numeric-codec.json",
    );
    let source = json_file("develop/migrations/csharp-03/codecs/source-strings.json");
    let selected: Vec<_> = frozen["observations"]["culture_runs"][0]["vectors"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|v| {
            v["family"].as_str().unwrap().starts_with("string.")
                && v["profile_outcome"] == "candidate_admitted"
                && v["operation"] != "string.switch.constant"
        })
        .cloned()
        .collect();
    assert_eq!(source, json!(selected));
    assert_eq!(selected.len(), 63);
    for v in selected {
        let operation = v["operation"].as_str().unwrap();
        let result = evaluate_string_operation(operation, &string_operands(&v), false);
        if v["profile"]["kind"] == "error" {
            let error = result.unwrap_err();
            let id = match error {
                StringError::NullReceiver => "exception.null_receiver",
                StringError::NullArgument => "exception.null_argument",
                StringError::IndexOutOfRange | StringError::ArgumentOutOfRange => "exception.range",
                _ => panic!("{error:?}"),
            };
            assert_eq!(id, v["profile"]["error_id"], "{}", v["id"]);
            assert!(error.exception_type().is_some());
        } else {
            let value = result.unwrap_or_else(|e| panic!("{}: {e:?}", v["id"]));
            let output = if operation == "string.compare.ordinal" {
                encoded(&value).parse::<i32>().unwrap().signum().to_string()
            } else {
                encoded(&value)
            };
            assert_eq!(output, v["profile"]["value"], "{}", v["id"]);
        }
    }
}
#[test]
fn csharp_03_t03_w10_bound_obligations_nulls_and_restricted_concat() {
    let profile = json_file("develop/specs/vectors/csharp-practical-profile-v1.json");
    let rows: Vec<_> = profile["vectors"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|v| v["implementation_owner"] == "CSHARP-03-T03-W10")
        .collect();
    assert_eq!(rows.len(), 3);
    for v in rows {
        let n = v["inputs"]["value"].as_u64().unwrap() as usize;
        let value = StringOperand::Text {
            utf16: Some(vec![0xd800; n]),
        };
        assert_eq!(
            evaluate_string_operation("string.literal.decode", &[value], false).is_ok(),
            v["expected"]["accept"] == true
        );
    }
    let null = StringOperand::Text { utf16: None };
    let empty = StringOperand::Text {
        utf16: Some(vec![]),
    };
    assert_eq!(
        evaluate_string_operation(
            "string.equals.ordinal",
            &[null.clone(), empty.clone()],
            true
        ),
        Err(StringError::NullReceiver)
    );
    assert_eq!(
        evaluate_string_operation("string.contains.ordinal", &[null.clone(), null], false),
        Err(StringError::NullReceiver)
    );
    assert!(evaluate_string_operation(
        "string.concat.operator.char_char",
        &[
            StringOperand::Char { utf16: 1 },
            StringOperand::Char { utf16: 2 }
        ],
        false
    )
    .is_err());
    assert!(evaluate_string_operation(
        "string.interpolation.restricted",
        &[StringOperand::Index { value: 1 }],
        false
    )
    .is_err());
    let large = StringOperand::Text {
        utf16: Some(vec![65; 16384]),
    };
    assert_eq!(
        evaluate_string_operation(
            "string.concat.operator.string_char",
            &[large, StringOperand::Char { utf16: 0 }],
            false
        ),
        Err(StringError::OutputBound)
    );
    assert_eq!(StringError::OutputBound.exception_type(), None);
    let pieces = [empty, StringOperand::Char { utf16: 0xd800 }];
    assert_eq!(
        evaluate_string_operation("string.interpolation.restricted", &pieces, false),
        evaluate_string_operation("string.concat.operator.string_char", &pieces, false)
    );
}

#[test]
fn csharp_03_t03_w10_pinned_source_harness_when_available() {
    let package = json_file("develop/migrations/csharp-03/build-inputs/build-inputs.json");
    let archives = package["toolchain_inputs"]["archives"].as_array().unwrap();
    if !cfg!(target_os = "linux") {
        return;
    }
    let cache=root().join("release/build-input-cache/csharp/d4af1170b2813a5581bb0f60b65fd4e7509576093045557b88689bf7e0876b4f/archives");
    let count = archives
        .iter()
        .filter(|archive| {
            cache
                .join(format!(
                    "{}.{}",
                    archive["id"].as_str().unwrap(),
                    archive["kind"].as_str().unwrap()
                ))
                .is_file()
        })
        .count();
    assert!(
        count == 0 || count == archives.len(),
        "partial pinned cache"
    );
    if count == 0 {
        return;
    }
    let output = Command::new(root().join("scripts/build-csharp-practical-frontend.sh"))
        .arg("--test-codecs")
        .env_clear()
        .env("PATH", "/usr/bin:/bin")
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output.stdout.is_empty());
    assert!(output.stderr.is_empty());
}

#[test]
fn csharp_03_t03_w10_contract_codec_signatures_reject_crossed_types() {
    let (b, r, c) = fixture();
    let result_id = |token| {
        csharp_practical_closed_instance_id(&b,&json!({"kind":"instance","template":"result","arguments":[{"kind":"primitive","id":token},{"kind":"primitive","id":"parse_error"}]})).unwrap()
    };
    let check = |id: &str| RequiredCheck {
        id: id.into(),
        tag: RequiredCheckTag::ParseError,
        failure_type_id: Some(ty("parse_error")),
    };
    let parse = ClosedOperationSignature {
        id: "codec.integer.i32.parse".into(),
        tag: ClosedOperationTag::BoundaryParse,
        argument_type_ids: vec![ty("string")],
        normal_result_type_id: result_id("i32"),
        ordered_checks: ["input_bound", "syntax", "noncanonical", "range"]
            .iter()
            .map(|suffix| check(&format!("parse_error.{suffix}")))
            .collect(),
    };
    validate_closed_operation_signature(&r, &c, &parse).unwrap();
    let format = ClosedOperationSignature {
        id: "codec.integer.i32.format".into(),
        tag: ClosedOperationTag::BoundaryFormat,
        argument_type_ids: vec![ty("i32")],
        normal_result_type_id: ty("string"),
        ordered_checks: vec![RequiredCheck {
            id: "obligation.output_bound".into(),
            tag: RequiredCheckTag::StaticObligation,
            failure_type_id: None,
        }],
    };
    validate_closed_operation_signature(&r, &c, &format).unwrap();
    for mutation in 0..3 {
        let mut bad = parse.clone();
        match mutation {
            0 => bad.normal_result_type_id = result_id("string"),
            1 => bad.argument_type_ids.push(ty("i32")),
            _ => bad.id = "codec.general.parse".into(),
        };
        assert!(validate_closed_operation_signature(&r, &c, &bad).is_err());
    }
    let mut bad = format.clone();
    bad.argument_type_ids[0] = ty("u32");
    assert!(validate_closed_operation_signature(&r, &c, &bad).is_err());
    bad = format;
    bad.argument_type_ids.push(ty("i32"));
    assert!(validate_closed_operation_signature(&r, &c, &bad).is_err());
}
#[test]
fn csharp_03_t03_w10_exact_private_inputs() {
    let path = "develop/migrations/csharp-03/codecs/codecs-inputs.json";
    let manifest = json_file(path);
    assert_eq!(
        manifest["schema"],
        "mpk.csharp_practical.t03_w10.codecs_inputs.v1"
    );
    assert_eq!(manifest["work_item"], "CSHARP-03-T03-W10");
    let expected = [
        "crates/mpk-cli/tests/csharp_practical_codecs_harness.cs",
        "csharp-tools/csharp2vir/PracticalArrays.cs",
        "csharp-tools/csharp2vir/PracticalCapture.cs",
        "csharp-tools/csharp2vir/PracticalConstruction.cs",
        "csharp-tools/csharp2vir/PracticalDataTypes.cs",
        "csharp-tools/csharp2vir/PracticalStrings.cs",
        "csharp-tools/csharp2vir/PracticalStructural.cs",
        "csharp-tools/csharp2vir/PracticalSyntaxNormalization.cs",
        "develop/migrations/csharp-03/codecs/source-strings.json",
    ];
    let files = manifest["files"].as_array().unwrap();
    assert_eq!(files.len(), expected.len());
    let mut canonical = serde_json::to_vec(&manifest).unwrap();
    canonical.push(b'\n');
    assert_eq!(canonical, read(path));
    for (file, path) in files.iter().zip(expected) {
        let data = read(path);
        assert_eq!(file["path"], path);
        assert_eq!(file["size_bytes"], data.len());
        assert_eq!(file["sha256"], format!("{:x}", Sha256::digest(data)));
    }
    for p in [
        "csharp-tools/csharp2vir/csharp2vir.csproj",
        "csharp-tools/csharp2vir/Program.cs",
        "develop/migrations/csharp-03/build-inputs/build-inputs.json",
    ] {
        assert!(!String::from_utf8(read(p))
            .unwrap()
            .contains("PracticalStrings"));
    }
}
