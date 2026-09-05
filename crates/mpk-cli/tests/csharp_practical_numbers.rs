//! CSHARP-03-T03-W11: exact integer numeric models, frozen corpus and source lowering.
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
fn fields(v: &Value) -> Vec<(&str, &str)> {
    v["inputs"]
        .as_array()
        .unwrap()
        .iter()
        .map(|x| x.as_str().unwrap().split_once('=').unwrap())
        .collect()
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
fn admitted(v: &Value) -> bool {
    v["family"]
        .as_str()
        .is_some_and(|s| s.starts_with("floating") || s.starts_with("decimal"))
        && v["profile_outcome"] == "candidate_admitted"
}
fn integer_token(name: &str) -> &str {
    match name {
        "int32" => "i32",
        "int64" => "i64",
        "uint64" => "u64",
        "single" => "f32",
        "double" => "f64",
        p => p,
    }
}
fn dec(negative: bool, coefficient: u128, scale: u8) -> MonomorphicValue {
    MonomorphicValue::DecimalBits {
        type_id: ty("decimal"),
        negative,
        coefficient: coefficient.to_string(),
        scale,
    }
}
fn inputs(v: &Value) -> Vec<MonomorphicValue> {
    let op = v["operation"].as_str().unwrap();
    let fields = fields(v);
    if let Some((_, case)) = fields.iter().find(|(k, _)| *k == "case") {
        let max = dec(false, (1u128 << 96) - 1, 0);
        let min = dec(true, (1u128 << 96) - 1, 0);
        return match *case {
            "max_divide_fraction" => vec![max, dec(false, 1, 1)],
            "max_divide_zero" | "max_remainder_zero" => vec![max, dec(false, 0, 0)],
            "max_plus_one" => vec![max, dec(false, 1, 0)],
            "min_minus_one" => vec![min, dec(false, 1, 0)],
            "max_times_two" => vec![max, dec(false, 2, 0)],
            "negate_min" => vec![min],
            _ => panic!("unknown case"),
        };
    }
    let token = if op.contains("conversion.") {
        integer_token(
            op.split("conversion.")
                .nth(1)
                .unwrap()
                .split("_to_")
                .next()
                .unwrap(),
        )
    } else if op.starts_with("floating.single.") {
        "f32"
    } else if op.starts_with("floating.double.") {
        "f64"
    } else {
        "decimal"
    };
    fields
        .iter()
        .filter(|(key, _)| *key != "rounding")
        .map(|(key, value)| raw_value(if *key == "digits" { "i32" } else { token }, value))
        .collect()
}
fn result_token(v: &Value) -> &str {
    match v["profile"]["result_encoding"].as_str().unwrap() {
        "bool" => "bool",
        "decimal_bits" => "decimal",
        "ieee_binary32_bits" => "f32",
        "ieee_binary64_bits" => "f64",
        "signed_decimal" => {
            if v["operation"].as_str().unwrap().contains("int64") {
                "i64"
            } else {
                "i32"
            }
        }
        "none" => {
            if v["operation"].as_str().unwrap().starts_with("numeric.") {
                if v["operation"].as_str().unwrap().contains("int64") {
                    "i64"
                } else {
                    "i32"
                }
            } else if v["operation"]
                .as_str()
                .unwrap()
                .contains("decimal_to_int32")
            {
                "i32"
            } else {
                "decimal"
            }
        }
        _ => panic!("encoding {v}"),
    }
}
#[test]
fn csharp_03_t03_w11_every_frozen_numeric_vector_in_all_cultures() {
    let (b, r, c) = fixture();
    let record = json_file(
        "develop/migrations/csharp-03/probes/runtime-primitive-string-numeric-codec.json",
    );
    let mut failures = Vec::new();
    let mut count = 0;
    for run in record["observations"]["culture_runs"].as_array().unwrap() {
        for v in run["vectors"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|v| admitted(v))
        {
            count += 1;
            let operands = inputs(v);
            let out = result_token(v);
            let args: Vec<_> = operands.iter().map(|v| v.type_id().to_owned()).collect();
            let fields = fields(v);
            let mode = fields
                .iter()
                .find(|(k, _)| *k == "rounding")
                .map(|(_, v)| *v);
            let plan =
                NumericOperation::new(v["operation"].as_str().unwrap(), &args, &ty(out), mode)
                    .unwrap_or_else(|e| panic!("{}: {e:?}", v["id"]));
            let result = plan.evaluate(&b, &r, &c, &operands);
            let correct = if v["profile"]["kind"] == "error" {
                let expected = match v["profile"]["error_id"].as_str().unwrap() {
                    "exception.range" => NumericError::Range,
                    "exception.overflow" => NumericError::Overflow,
                    "exception.division_by_zero" => NumericError::DivideByZero,
                    _ => panic!("error id"),
                };
                result == Err(expected)
            } else if let Ok(value) = &result {
                if out == "decimal" {
                    generate_structural_program(&b, &r, &c, &ty("decimal"))
                        .unwrap()
                        .structural_equal(
                            value,
                            &raw_value("decimal", v["profile"]["value"].as_str().unwrap()),
                        )
                        .unwrap()
                } else {
                    encoded(value) == v["profile"]["value"]
                }
            } else {
                false
            };
            if !correct && failures.len() < 25 {
                failures.push(format!(
                    "{} expected {} got {result:?}",
                    v["id"], v["profile"]
                ));
            }
        }
    }
    assert_eq!(count, 2989 * 3);
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
#[test]
fn csharp_03_t03_w11_small_integer_domains_and_codec_roundtrips() {
    let (b, r, c) = fixture();
    for token in ["f32", "f64", "decimal"] {
        let input = ty(token);
        let codec = NumericOperation::boundary_codec(&input).unwrap();
        for a in -8i32..=8 {
            for bv in -8i32..=8 {
                let value = |n: i32| {
                    if token == "decimal" {
                        dec(n < 0, u128::from(n.unsigned_abs()), 0)
                    } else {
                        let from = if token == "f32" { "i32" } else { "i64" };
                        let id = if token == "f32" {
                            "numeric.conversion.int32_to_single"
                        } else {
                            "numeric.conversion.int64_to_double"
                        };
                        NumericOperation::new(id, &[ty(from)], &input, None)
                            .unwrap()
                            .evaluate(&b, &r, &c, &[raw_value(from, &n.to_string())])
                            .unwrap()
                    }
                };
                let left = value(a);
                let right = value(bv);
                let prefix = if token == "f32" {
                    "floating.single"
                } else if token == "f64" {
                    "floating.double"
                } else {
                    "decimal"
                };
                for (op, n) in [("add", a + bv), ("subtract", a - bv), ("multiply", a * bv)] {
                    let result = NumericOperation::new(
                        &format!("{prefix}.{op}"),
                        &[input.clone(), input.clone()],
                        &input,
                        None,
                    )
                    .unwrap()
                    .evaluate(&b, &r, &c, &[left.clone(), right.clone()])
                    .unwrap();
                    assert!(generate_structural_program(&b, &r, &c, &input)
                        .unwrap()
                        .structural_equal(&result, &value(n))
                        .unwrap());
                    let formatted = codec.format(&b, &r, &c, &result).unwrap();
                    let parsed = codec.parse(&formatted).unwrap();
                    assert!(generate_structural_program(&b, &r, &c, &input)
                        .unwrap()
                        .structural_equal(&parsed, &result)
                        .unwrap());
                }
            }
        }
    }
}
#[test]
fn csharp_03_t03_w11_closed_signatures_and_value_mutations() {
    let (b, r, c) = fixture();
    for id in [
        "floating.single.sqrt",
        "floating.double.fma",
        "floating.single.value_equality",
        "decimal.parse",
        "numeric.conversion.double_to_int32.checked",
        "numeric.conversion.single_to_int32",
        "numeric.conversion.decimal_to_single",
    ] {
        assert!(NumericOperation::new(id, &[ty("f32")], &ty("f32"), None).is_err());
    }
    assert!(NumericOperation::new(
        "decimal.round",
        &[ty("decimal")],
        &ty("decimal"),
        Some("Unknown")
    )
    .is_err());
    assert!(NumericOperation::new(
        "floating.single.add",
        &[ty("f32"), ty("f64")],
        &ty("f32"),
        None
    )
    .is_err());
    let plus =
        NumericOperation::new("floating.single.plus", &[ty("f32")], &ty("f32"), None).unwrap();
    for bits in ["7FC12345", "000000000", "xyz00000"] {
        let v = MonomorphicValue::F32Bits {
            type_id: ty("f32"),
            bits: bits.into(),
        };
        assert_eq!(
            plus.evaluate(&b, &r, &c, &[v]),
            Err(NumericError::OperandType)
        );
    }
    let round = NumericOperation::new(
        "decimal.round",
        &[ty("decimal"), ty("i32")],
        &ty("decimal"),
        Some("ToEven"),
    )
    .unwrap();
    assert_eq!(
        round.evaluate(&b, &r, &c, &[dec(false, 1, 29), raw_value("i32", "29")]),
        Err(NumericError::OperandType)
    );
    assert_eq!(
        round.evaluate(&b, &r, &c, &[dec(false, 1, 0), raw_value("i32", "29")]),
        Err(NumericError::Range)
    );
    for token in ["f32", "f64"] {
        assert!(!generate_structural_program(&b, &r, &c, &ty(token))
            .unwrap()
            .is_total());
    }
}
#[test]
fn csharp_03_t03_w11_finite_edge_bits_match_independent_ieee_hardware() {
    let (b, r, c) = fixture();
    let edges = [
        0,
        1,
        0x000fffffffffffff,
        0x0010000000000000,
        0x3fefffffffffffff,
        0x3ff0000000000000,
        0x3ff0000000000001,
        0x7fefffffffffffff,
        0x8000000000000000,
        0x8000000000000001,
        0xbff0000000000000,
    ];
    for a in edges {
        for bv in edges {
            for (op, expected) in [
                ("add", f64::from_bits(a) + f64::from_bits(bv)),
                ("subtract", f64::from_bits(a) - f64::from_bits(bv)),
                ("multiply", f64::from_bits(a) * f64::from_bits(bv)),
                ("divide", f64::from_bits(a) / f64::from_bits(bv)),
                ("remainder", f64::from_bits(a) % f64::from_bits(bv)),
            ] {
                if expected.is_nan() {
                    continue;
                }
                let plan = NumericOperation::new(
                    &format!("floating.double.{op}"),
                    &[ty("f64"), ty("f64")],
                    &ty("f64"),
                    None,
                )
                .unwrap();
                let result = plan
                    .evaluate(
                        &b,
                        &r,
                        &c,
                        &[
                            raw_value("f64", &format!("{a:016x}")),
                            raw_value("f64", &format!("{bv:016x}")),
                        ],
                    )
                    .unwrap();
                assert_eq!(
                    encoded(&result),
                    format!("{:016x}", expected.to_bits()),
                    "{op} {a:x} {bv:x}"
                );
            }
        }
    }
    let mut seed = 17u64;
    for _ in 0..1024 {
        seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
        let a = seed;
        seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
        let bv = seed;
        let x = f32::from_bits(a as u32);
        let y = f32::from_bits(bv as u32);
        if !x.is_finite() || !y.is_finite() {
            continue;
        }
        for (op, expected) in [
            ("add", x + y),
            ("subtract", x - y),
            ("multiply", x * y),
            ("divide", x / y),
            ("remainder", x % y),
        ] {
            if expected.is_nan() {
                continue;
            }
            let plan = NumericOperation::new(
                &format!("floating.single.{op}"),
                &[ty("f32"), ty("f32")],
                &ty("f32"),
                None,
            )
            .unwrap();
            let result = plan
                .evaluate(
                    &b,
                    &r,
                    &c,
                    &[
                        raw_value("f32", &format!("{:08x}", a as u32)),
                        raw_value("f32", &format!("{:08x}", bv as u32)),
                    ],
                )
                .unwrap();
            assert_eq!(
                encoded(&result),
                format!("{:08x}", expected.to_bits()),
                "{op} {a:x} {bv:x}"
            );
        }
        let x = f64::from_bits(a);
        if x.is_finite() {
            let plan = NumericOperation::new(
                "numeric.conversion.double_to_single",
                &[ty("f64")],
                &ty("f32"),
                None,
            )
            .unwrap();
            assert_eq!(
                encoded(
                    &plan
                        .evaluate(&b, &r, &c, &[raw_value("f64", &format!("{a:016x}"))])
                        .unwrap()
                ),
                format!("{:08x}", (x as f32).to_bits())
            );
        }
    }
}
#[test]
fn csharp_03_t03_w11_exact_private_inputs_and_frozen_projection() {
    let bytes =
        read("develop/migrations/csharp-03/probes/runtime-primitive-string-numeric-codec.json");
    assert_eq!(
        format!("{:x}", Sha256::digest(&bytes)),
        "0055835ce456fb9c438336332bc0e2a214d900c137eca34f90c3fcddd2688769"
    );
    let record: Value = serde_json::from_slice(&bytes).unwrap();
    let rows:Vec<_>=record["observations"]["culture_runs"][0]["vectors"].as_array().unwrap().iter().filter(|v|admitted(v)).map(|v|json!({"id":v["id"],"operation":v["operation"],"inputs":v["inputs"],"profile":v["profile"]})).collect();
    assert_eq!(
        json!(rows),
        json_file("develop/migrations/csharp-03/numeric/numeric-runtime.json")
    );
    let path = "develop/migrations/csharp-03/numeric/numeric-inputs.json";
    let manifest = json_file(path);
    assert_eq!(
        manifest["schema"],
        "mpk.csharp_practical.t03_w11.numeric_inputs.v1"
    );
    assert_eq!(manifest["work_item"], "CSHARP-03-T03-W11");
    let mut canonical = serde_json::to_vec(&manifest).unwrap();
    canonical.push(b'\n');
    assert_eq!(canonical, read(path));
    let expected = [
        "crates/mpk-cli/tests/csharp_practical_numeric_harness.cs",
        "csharp-tools/csharp2vir/PracticalArrays.cs",
        "csharp-tools/csharp2vir/PracticalCapture.cs",
        "csharp-tools/csharp2vir/PracticalConstruction.cs",
        "csharp-tools/csharp2vir/PracticalDataTypes.cs",
        "csharp-tools/csharp2vir/PracticalNumeric.cs",
        "csharp-tools/csharp2vir/PracticalStrings.cs",
        "csharp-tools/csharp2vir/PracticalStructural.cs",
        "csharp-tools/csharp2vir/PracticalSyntaxNormalization.cs",
        "develop/migrations/csharp-03/numeric/numeric-runtime.json",
    ];
    let files = manifest["files"].as_array().unwrap();
    assert_eq!(files.len(), expected.len());
    for (file, path) in files.iter().zip(expected) {
        assert_eq!(file["path"], path);
        let bytes = read(path);
        assert_eq!(file["size_bytes"], bytes.len());
        assert_eq!(file["sha256"], format!("{:x}", Sha256::digest(&bytes)));
    }
    let installed = json_file("develop/migrations/csharp-03/build-inputs/build-inputs.json");
    assert!(!installed.to_string().contains("PracticalNumeric.cs"));
}
#[test]
fn csharp_03_t03_w11_pinned_source_harness_when_available() {
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
        .arg("--test-numeric")
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
fn csharp_03_t03_w11_decimal_rounding_scale_table_and_literal_bits() {
    let (b, r, c) = fixture();
    let modes = [
        "ToEven",
        "AwayFromZero",
        "ToZero",
        "ToNegativeInfinity",
        "ToPositiveInfinity",
    ];
    for negative in [false, true] {
        for scale in 1..=28 {
            for coefficient in 0..=35u128 {
                for (m, mode) in modes.iter().enumerate() {
                    let quotient = coefficient / 10;
                    let rem = coefficient % 10;
                    let up = match m {
                        0 => rem > 5 || rem == 5 && quotient % 2 == 1,
                        1 => rem >= 5,
                        2 => false,
                        3 => negative && rem != 0,
                        _ => !negative && rem != 0,
                    };
                    let plan = NumericOperation::new(
                        "decimal.round",
                        &[ty("decimal"), ty("i32")],
                        &ty("decimal"),
                        Some(mode),
                    )
                    .unwrap();
                    let value = plan
                        .evaluate(
                            &b,
                            &r,
                            &c,
                            &[
                                dec(negative, coefficient, scale),
                                raw_value("i32", &(scale - 1).to_string()),
                            ],
                        )
                        .unwrap();
                    assert_eq!(value, dec(negative, quotient + u128::from(up), scale - 1));
                }
            }
        }
    }
    let round =
        NumericOperation::new("decimal.round", &[ty("decimal")], &ty("decimal"), None).unwrap();
    assert!(round.exception_types().is_empty());
    for (prefix, token, bits) in [
        ("floating.single", "f32", "7fa12345"),
        ("floating.double", "f64", "8000000000000000"),
    ] {
        let value = raw_value(token, bits);
        let plan =
            NumericOperation::new(&format!("{prefix}.literal"), &[ty(token)], &ty(token), None)
                .unwrap();
        assert_eq!(
            plan.evaluate(&b, &r, &c, std::slice::from_ref(&value))
                .unwrap(),
            value
        );
        let codec = NumericOperation::boundary_codec(&ty(token)).unwrap();
        assert_eq!(
            codec
                .parse(&codec.format(&b, &r, &c, &value).unwrap())
                .unwrap(),
            value
        );
    }
}
