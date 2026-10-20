use mpk_cert::encode::{Certificate, DeclarationKind};
use mpk_vc::go_practical_ordinary::{
    generate_go_practical_string_foundation, generate_go_practical_string_value,
    generate_go_practical_word_index_foundation, import_go_practical_string_foundation,
    import_go_practical_string_value, GoOrdinaryError, GoStringValueError,
};
#[path = "support/go_word_division_dependencies.rs"]
mod dependencies;

fn retain(name: &str, extension: &str, raw: &[u8]) {
    if let Some(path) = std::env::var_os("MPK_GO_STRING_FOUNDATION_OUTPUT") {
        use std::io::Write;
        let root = std::path::PathBuf::from(path);
        std::fs::create_dir_all(&root).unwrap();
        std::fs::OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(root.join(format!("{name}.{extension}")))
            .unwrap()
            .write_all(raw)
            .unwrap();
    }
}
fn inspect(name: &str, bytes: &[u8], accepted: bool) {
    retain(name, "mpcert", bytes);
    inspect_saved(name, bytes, accepted);
}

fn inspect_saved(name: &str, bytes: &[u8], accepted: bool) {
    let result = mpk_kernel::verify_certificate_bytes_json_output(bytes);
    retain(name, "rust.json", format!("{}\n", result.json).as_bytes());
    if accepted && !result.accepted {
        let certificate = mpk_cert::decode_canonical_certificate(bytes).unwrap();
        let (mut lower, mut upper) = (1, certificate.declarations.len());
        while lower < upper {
            let end = (lower + upper) / 2;
            let mut prefix = certificate.clone();
            prefix.declarations.truncate(end);
            if mpk_kernel::check_declarations(&prefix).is_ok() {
                lower = end + 1;
            } else {
                upper = end;
            }
        }
        panic!(
            "{name}: first declaration type-check failure {}: {}",
            certificate.name_table[certificate.declarations[lower - 1].name as usize],
            result.json
        );
    }
    assert_eq!(result.accepted, accepted, "{name}: {}", result.json);
    let output: serde_json::Value = serde_json::from_str(&result.json).unwrap();
    if accepted {
        assert_eq!(output["axiom_count"], 0);
        assert!(output["axiom_report"]["entries"]
            .as_array()
            .unwrap()
            .is_empty());
    } else {
        assert_eq!(output["error_code"], "KERNEL_CORE_CHECK");
        let detail: serde_json::Value =
            serde_json::from_str(output["error_detail"].as_str().unwrap()).unwrap();
        assert_eq!(detail["code"], "CORE_TYPE_MISMATCH");
    }
}
fn foundation() -> &'static mpk_vc::go_practical_ordinary::GoStringFoundation {
    static VALUE: std::sync::OnceLock<mpk_vc::go_practical_ordinary::GoStringFoundation> =
        std::sync::OnceLock::new();
    VALUE.get_or_init(|| generate_go_practical_string_foundation().unwrap())
}
fn baseline() -> Certificate {
    mpk_cert::decode_canonical_certificate(foundation().certificate_bytes()).unwrap()
}

fn is_constant(c: &Certificate, node: u32, name: &str) -> bool {
    matches!(c.term_table[node as usize],mpk_cert::encode::TermNode::Const{global,..} if c.name_table[c.declarations[global as usize].name as usize]==name)
}
fn cube_body(c: &Certificate, mut node: u32, depth: usize) -> u32 {
    for _ in 0..depth {
        let mpk_cert::encode::TermNode::Pi { ty, body } = c.term_table[node as usize] else {
            panic!("frozen Bool cube lost a complete address")
        };
        assert!(is_constant(c, ty, "Std.Bool"));
        node = body;
    }
    node
}
fn frozen_string_shape(c: &Certificate) {
    let d = c
        .declarations
        .iter()
        .find(|d| c.name_table[d.name as usize] == "Mpk.Go.Practical.String.Type")
        .unwrap();
    let DeclarationKind::Def { value, .. } = d.kind else {
        panic!("string carrier is not an ordinary definition")
    };
    let mpk_cert::encode::TermNode::Pi { ty, body } = c.term_table[value as usize] else {
        panic!("frozen binary dependent product")
    };
    assert!(is_constant(c, ty, "Std.Bool"));
    let mpk_cert::encode::TermNode::App {
        function,
        arguments,
    } = &c.term_table[body as usize]
    else {
        panic!("frozen heterogeneous product")
    };
    assert!(is_constant(c, *function, "Std.Bool.recType"));
    assert_eq!(arguments.len(), 3);
    assert!(matches!(
        c.term_table[arguments[2] as usize],
        mpk_cert::encode::TermNode::Var(0)
    ));
    assert!(is_constant(c, cube_body(c, arguments[0], 6), "Std.Bool"));
    let byte = cube_body(c, arguments[1], 8);
    assert!(is_constant(c, cube_body(c, byte, 3), "Std.Bool"));
}

#[test]
fn frozen_string_foundation_preserves_all_full64_prerequisites() {
    inspect("string-foundation", foundation().certificate_bytes(), true);
    let c = baseline();
    frozen_string_shape(&c);
    let current = dependencies::declarations(&c);
    let prior = generate_go_practical_word_index_foundation().unwrap();
    let file=std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../develop/migrations/go-practical/t02-w01/verification/word-index-foundation/epochs/explicit-controls/selected/candidates/word-index-foundation.mpcert");
    assert_eq!(prior.certificate_bytes(), std::fs::read(file).unwrap());
    let prior = mpk_cert::decode_canonical_certificate(prior.certificate_bytes()).unwrap();
    assert_eq!(prior.declarations.len(), 1005);
    for (name, value) in dependencies::declarations(&prior) {
        assert_eq!(current.get(&name), Some(&value), "{name}");
    }
    assert!(c.imports.is_empty() && c.source_manifest.is_none());
    assert_eq!(c.axiom_report.summary.total_axiom_count, 0);
    assert_eq!(
        foundation(),
        &import_go_practical_string_foundation(foundation().certificate_bytes()).unwrap()
    );
    println!("string foundation: declarations={} terms={} bytes={} all1005 exact prerequisites; no source or nominal authority",c.declarations.len(),c.term_table.len(),foundation().certificate_bytes().len());
}

#[test]
fn complete_native_byte_alphabet_has_every_ordinary_bit_checked() {
    let bytes = (0..=255u8).collect::<Vec<_>>();
    let value = generate_go_practical_string_value(&bytes).unwrap();
    assert_eq!(value.native_bytes(), bytes);
    inspect("string-all-bytes", value.certificate_bytes(), true);
    let c = mpk_cert::decode_canonical_certificate(value.certificate_bytes()).unwrap();
    assert_eq!(
        c.declarations
            .iter()
            .filter(|d| matches!(d.kind, DeclarationKind::Theorem { .. })
                && c.name_table[d.name as usize].starts_with("Mpk.Go.Practical.String.Value."))
            .count(),
        2112
    );
    assert_eq!(
        value,
        import_go_practical_string_value(value.certificate_bytes(), &bytes).unwrap()
    );
    assert_eq!(
        generate_go_practical_string_value(&[0; 257]),
        Err(GoStringValueError::Domain { bytes: 257 })
    );
    assert_eq!(
        import_go_practical_string_value(value.certificate_bytes(), b"different"),
        Err(GoStringValueError::Foundation(GoOrdinaryError::Linkage))
    );
    println!("complete native alphabet: declarations={} terms={} bytes={} all64 length+2048 data bits checked",c.declarations.len(),c.term_table.len(),value.certificate_bytes().len());
}

fn base64(value: &serde_json::Value) -> Vec<u8> {
    if value.is_null() {
        return vec![];
    }
    let text = value.as_str().unwrap().as_bytes();
    assert_eq!(text.len() % 4, 0);
    let mut output = Vec::new();
    let digit = |b| match b {
        b'A'..=b'Z' => (b - b'A') as u32,
        b'a'..=b'z' => (b - b'a' + 26) as u32,
        b'0'..=b'9' => (b - b'0' + 52) as u32,
        b'+' => 62,
        b'/' => 63,
        _ => panic!("invalid original base64"),
    };
    for (index, chunk) in text.chunks_exact(4).enumerate() {
        let padding = if chunk[2] == b'=' {
            assert_eq!(chunk[3], b'=');
            2
        } else if chunk[3] == b'=' {
            1
        } else {
            0
        };
        if padding > 0 {
            assert_eq!((index + 1) * 4, text.len());
        }
        let n = (digit(chunk[0]) << 18)
            | (digit(chunk[1]) << 12)
            | if padding == 2 {
                0
            } else {
                digit(chunk[2]) << 6
            }
            | if padding > 0 { 0 } else { digit(chunk[3]) };
        output.push((n >> 16) as u8);
        if padding < 2 {
            output.push((n >> 8) as u8);
        }
        if padding == 0 {
            output.push(n as u8);
        }
    }
    output
}

#[test]
fn actual_owned_go_constants_keep_every_original_byte_and_bit() {
    use sha2::{Digest, Sha256};
    let root=std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../develop/migrations/go-practical/t02-w01/verification/string-constants/input-review.json");
    let receipt: serde_json::Value = serde_json::from_slice(&std::fs::read(root).unwrap()).unwrap();
    let mut total = 0;
    let mut rejected = 0;
    for row in receipt["original_evidence_files"].as_array().unwrap() {
        let path = std::path::Path::new(row["path"].as_str().unwrap());
        let name = path.file_name().unwrap().to_str().unwrap();
        let full = path.to_str().unwrap();
        if !name.ends_with("-native.json")
            || !(full.contains("-complete-outputs-selected-evidence/")
                || full.contains("-complete-outputs-replay/evidence/"))
        {
            continue;
        }
        let raw = std::fs::read(path).unwrap();
        assert_eq!(
            format!("{:x}", Sha256::digest(&raw)),
            row["sha256"].as_str().unwrap()
        );
        assert_eq!(raw.len() as u64, row["bytes"].as_u64().unwrap());
        let name = format!("native-{}", path.file_stem().unwrap().to_str().unwrap());
        retain(&name, "native.json", &raw);
        let original: serde_json::Value = serde_json::from_slice(&raw).unwrap();
        let bytes = base64(&original["native_bytes"]);
        let source = base64(&original["original_source"]);
        let expression = base64(&original["original_expression"]);
        assert_eq!(
            format!("{:x}", Sha256::digest(&source)),
            original["source_object"]["raw_sha256"].as_str().unwrap()
        );
        assert_eq!(
            source.len() as u64,
            original["source_object"]["bytes"].as_u64().unwrap()
        );
        let start = original["span"]["start"].as_u64().unwrap() as usize;
        let end = original["span"]["end"].as_u64().unwrap() as usize;
        assert_eq!(&source[start..end], expression);
        assert_eq!(
            original["span"]["source_id"],
            original["source_object"]["id"]
        );
        assert_eq!(original["source_object"]["kind"], "go_file");
        assert_eq!(
            original["ordinary_source_evaluation_or_nominal_identity_proof"],
            false
        );
        assert_eq!(original["registered_context_or_program_proof"], false);
        total += 1;
        if bytes.len() > 256 {
            assert_eq!(bytes.len(), 257);
            assert_eq!(
                generate_go_practical_string_value(&bytes),
                Err(GoStringValueError::Domain { bytes: 257 })
            );
            retain(
                &name,
                "domain.json",
                format!(
                    "{{\"bytes\":257,\"modeled_domain_error\":true,\"semantic_falsity\":false}}\n"
                )
                .as_bytes(),
            );
            rejected += 1;
            continue;
        }
        let value = generate_go_practical_string_value(&bytes).unwrap();
        assert_eq!(value.native_bytes(), bytes);
        inspect(&name, value.certificate_bytes(), true);
        let c = mpk_cert::decode_canonical_certificate(value.certificate_bytes()).unwrap();
        let mut bits = 0;
        for d in &c.declarations {
            let name = &c.name_table[d.name as usize];
            let Some(suffix) = name.strip_prefix("Mpk.Go.Practical.String.Value.") else {
                continue;
            };
            let expected = if let Some(position) = suffix.strip_prefix("LengthBit") {
                ((bytes.len() as u64) >> position.parse::<usize>().unwrap()) & 1 != 0
            } else {
                let (slot, bit) = suffix
                    .strip_prefix("Slot")
                    .unwrap()
                    .split_once("Bit")
                    .unwrap();
                (bytes
                    .get(slot.parse::<usize>().unwrap())
                    .copied()
                    .unwrap_or(0)
                    >> bit.parse::<usize>().unwrap())
                    & 1
                    != 0
            };
            let DeclarationKind::Theorem { ty, .. } = d.kind else {
                panic!("bit was not an actual ordinary theorem")
            };
            let mpk_cert::encode::TermNode::App { arguments, .. } = &c.term_table[ty as usize]
            else {
                panic!("missing exact bit equality")
            };
            assert_eq!(arguments.len(), 3);
            let mpk_cert::encode::TermNode::Const { global, .. } =
                c.term_table[arguments[2] as usize]
            else {
                panic!("missing atomic original bit")
            };
            assert_eq!(
                c.name_table[c.declarations[global as usize].name as usize],
                if expected {
                    "Std.Bool.true"
                } else {
                    "Std.Bool.false"
                }
            );
            bits += 1;
        }
        assert_eq!(bits, 2112);
        assert!(c.source_manifest.is_none() && c.imports.is_empty());
    }
    assert_eq!(total, 19);
    assert_eq!(rejected, 1);
    println!("actual original Go data:19 complete source observations,18 full2112-bit ordinary certificates,one distinct257-byte domain error; no source evaluation or nominal/context proof");
}

use mpk_cert::encode::{Declaration, TermNode};
fn encode(mut c: Certificate) -> Vec<u8> {
    let old = c.name_table.clone();
    c.name_table.sort();
    for d in &mut c.declarations {
        d.name = c.name_table.binary_search(&old[d.name as usize]).unwrap() as u32;
    }
    for level in &mut c.level_table {
        if let mpk_cert::encode::LevelNode::Param(name) = level {
            *name = c.name_table.binary_search(&old[*name as usize]).unwrap() as u32;
        }
    }
    assert!(c.declarations.len() <= 4096 && c.term_table.len() <= 131_072);
    c.export_block = mpk_cert::build_export_block(&c).unwrap();
    c.axiom_report = mpk_cert::build_axiom_report(&c).unwrap();
    assert_eq!(c.axiom_report.summary.total_axiom_count, 0);
    c.hashes.export_hash = mpk_cert::export_block_hash(&c.export_block);
    c.hashes.axiom_report_hash = mpk_cert::axiom_report_hash_for_report(&c.axiom_report);
    mpk_cert::encode_certificate_bounded(&c, 2_097_152).unwrap()
}
fn term(c: &mut Certificate, node: TermNode) -> u32 {
    let id = c.term_table.len() as u32;
    c.term_table.push(node);
    id
}
fn symbol(c: &mut Certificate, name: &str) -> u32 {
    let global = c
        .declarations
        .iter()
        .position(|d| c.name_table[d.name as usize] == name)
        .unwrap() as u32;
    term(
        c,
        TermNode::Const {
            global,
            levels: vec![],
        },
    )
}
fn call(c: &mut Certificate, name: &str, arguments: Vec<u32>) -> u32 {
    let function = symbol(c, name);
    term(
        c,
        TermNode::App {
            function,
            arguments,
        },
    )
}
fn atom(c: &mut Certificate, value: bool) -> u32 {
    symbol(
        c,
        if value {
            "Std.Bool.true"
        } else {
            "Std.Bool.false"
        },
    )
}
fn false_claim(c: &mut Certificate) -> u32 {
    let ty = symbol(c, "Std.Bool");
    let no = atom(c, false);
    let yes = atom(c, true);
    call(c, "Std.Eq", vec![ty, no, yes])
}
fn replace_result(c: &mut Certificate, ty: u32) -> u32 {
    match c.term_table[ty as usize].clone() {
        TermNode::Pi { ty, body } => {
            let body = replace_result(c, body);
            term(c, TermNode::Pi { ty, body })
        }
        TermNode::App {
            function,
            mut arguments,
        } => {
            let TermNode::Const { global, .. } = c.term_table[function as usize] else {
                return false_claim(c);
            };
            if c.name_table[c.declarations[global as usize].name as usize] == "Std.Eq" {
                if let TermNode::Const { global, .. } = c.term_table[arguments[0] as usize] {
                    if c.name_table[c.declarations[global as usize].name as usize] == "Std.Bool" {
                        arguments[2] = call(c, "Mpk.Go.Practical.Bool.Not", vec![arguments[2]]);
                        return term(
                            c,
                            TermNode::App {
                                function,
                                arguments,
                            },
                        );
                    }
                }
            }
            false_claim(c)
        }
        _ => false_claim(c),
    }
}
fn replace_domain(c: &mut Certificate, ty: u32, at: usize, domain: u32) -> u32 {
    let TermNode::Pi { ty: old, body } = c.term_table[ty as usize].clone() else {
        panic!("missing original premise")
    };
    if at == 0 {
        term(c, TermNode::Pi { ty: domain, body })
    } else {
        let body = replace_domain(c, body, at - 1, domain);
        term(c, TermNode::Pi { ty: old, body })
    }
}
fn replace_lambdas(c: &mut Certificate, value: u32, body: u32) -> u32 {
    if let TermNode::Lam { ty, body: old } = c.term_table[value as usize].clone() {
        let body = replace_lambdas(c, old, body);
        term(c, TermNode::Lam { ty, body })
    } else {
        body
    }
}
fn zero_cube(c: &mut Certificate, depth: usize, mut body: u32) -> u32 {
    let boolean = symbol(c, "Std.Bool");
    for _ in 0..depth {
        body = term(c, TermNode::Lam { ty: boolean, body });
    }
    body
}

#[test]
fn exact_string_ownership_rejects_other_honest_data_and_valid_extensions() {
    let mut c = baseline();
    let ty = symbol(&mut c, "Std.Bool");
    let value = atom(&mut c, false);
    let name = c.name_table.len() as u32;
    c.name_table
        .push("Mpk.Go.Practical.String.HonestExtra".into());
    c.declarations.push(Declaration {
        name,
        kind: DeclarationKind::Def {
            ty,
            value,
            reducibility: mpk_cert::encode::DefinitionReducibility::Reducible,
        },
    });
    let bytes = encode(c);
    inspect("string-foundation-honest-extra", &bytes, true);
    assert_eq!(
        import_go_practical_string_foundation(&bytes),
        Err(GoOrdinaryError::Linkage)
    );
    let original = generate_go_practical_string_value(&[0, 255, 128]).unwrap();
    let mut c = mpk_cert::decode_canonical_certificate(original.certificate_bytes()).unwrap();
    let ty = symbol(&mut c, "Std.Bool");
    let value = atom(&mut c, true);
    let name = c.name_table.len() as u32;
    c.name_table
        .push("Mpk.Go.Practical.String.ValueHonestExtra".into());
    c.declarations.push(Declaration {
        name,
        kind: DeclarationKind::Def {
            ty,
            value,
            reducibility: mpk_cert::encode::DefinitionReducibility::Reducible,
        },
    });
    let bytes = encode(c);
    inspect("string-value-honest-extra", &bytes, true);
    assert_eq!(
        import_go_practical_string_value(&bytes, &[0, 255, 128]),
        Err(GoStringValueError::Foundation(GoOrdinaryError::Linkage))
    );
    assert_eq!(
        import_go_practical_string_value(original.certificate_bytes(), &[0, 255, 129]),
        Err(GoStringValueError::Foundation(GoOrdinaryError::Linkage))
    );
}

#[test]
fn actual_string_laws_require_real_ranges_padding_and_operations() {
    let base = baseline();
    let names = base
        .declarations
        .iter()
        .filter_map(|d| {
            let name = &base.name_table[d.name as usize];
            (name.starts_with("Mpk.Go.Practical.String.")
                && matches!(d.kind, DeclarationKind::Theorem { .. }))
            .then_some(name.clone())
        })
        .collect::<Vec<_>>();
    for name in &names {
        let mut c = base.clone();
        let index = c
            .declarations
            .iter()
            .position(|d| c.name_table[d.name as usize] == *name)
            .unwrap();
        let DeclarationKind::Theorem { ty, proof } = c.declarations[index].kind else {
            unreachable!()
        };
        let ty = replace_result(&mut c, ty);
        c.declarations[index].kind = DeclarationKind::Theorem { ty, proof };
        inspect(
            &format!("false-{}", name.rsplit('.').next().unwrap()),
            &encode(c),
            false,
        );
    }
    let mut premises = 0;
    for (name, positions) in [
        ("MakeValid", vec![2, 3]),
        ("ValidUpper", vec![1]),
        ("ValidPadding", vec![1]),
        ("OutsidePadding", vec![1, 10]),
        ("WithinCapacity", vec![2, 3]),
        ("ByteWithinSlot", vec![2, 3]),
        ("SequenceWithinSlot", vec![2, 3]),
        ("ValidWithinCapacity", vec![2, 3]),
        ("ValidByteWithinSlot", vec![2, 3]),
        ("ValidSequenceWithinSlot", vec![2, 3]),
    ] {
        for position in positions {
            let mut c = base.clone();
            let full = format!("Mpk.Go.Practical.String.{name}");
            let index = c
                .declarations
                .iter()
                .position(|d| c.name_table[d.name as usize] == full)
                .unwrap();
            let DeclarationKind::Theorem { ty, proof } = c.declarations[index].kind else {
                unreachable!()
            };
            let boolean = symbol(&mut c, "Std.Bool");
            let ty = replace_domain(&mut c, ty, position, boolean);
            c.declarations[index].kind = DeclarationKind::Theorem { ty, proof };
            inspect(&format!("premise-{name}-{position}"), &encode(c), false);
            premises += 1;
        }
    }
    let mut prefixes = vec![];
    for name in ["Length", "Data", "Byte", "LengthBit", "SlotBit"] {
        let mut c = base.clone();
        let full = format!("Mpk.Go.Practical.String.{name}");
        let at = c
            .declarations
            .iter()
            .position(|d| c.name_table[d.name as usize] == full)
            .unwrap();
        let DeclarationKind::Def {
            ty,
            value,
            reducibility,
        } = c.declarations[at].kind.clone()
        else {
            unreachable!()
        };
        let body = match name {
            "Length" => {
                let no = atom(&mut c, false);
                zero_cube(&mut c, 6, no)
            }
            "Data" => {
                let no = atom(&mut c, false);
                let byte = zero_cube(&mut c, 3, no);
                zero_cube(&mut c, 8, byte)
            }
            "Byte" => {
                let s = term(&mut c, TermNode::Var(1));
                let data = call(&mut c, "Mpk.Go.Practical.String.Data", vec![s]);
                let no = atom(&mut c, false);
                term(
                    &mut c,
                    TermNode::App {
                        function: data,
                        arguments: vec![no; 8],
                    },
                )
            }
            "LengthBit" => {
                let s = term(&mut c, TermNode::Var(6));
                let data = call(&mut c, "Mpk.Go.Practical.String.Length", vec![s]);
                let no = atom(&mut c, false);
                term(
                    &mut c,
                    TermNode::App {
                        function: data,
                        arguments: vec![no; 6],
                    },
                )
            }
            "SlotBit" => {
                let s = term(&mut c, TermNode::Var(11));
                let data = call(&mut c, "Mpk.Go.Practical.String.Data", vec![s]);
                let address = (3..11).map(|i| term(&mut c, TermNode::Var(i))).collect();
                let byte = term(
                    &mut c,
                    TermNode::App {
                        function: data,
                        arguments: address,
                    },
                );
                let address = (0..3)
                    .rev()
                    .map(|i| term(&mut c, TermNode::Var(i)))
                    .collect();
                term(
                    &mut c,
                    TermNode::App {
                        function: byte,
                        arguments: address,
                    },
                )
            }
            _ => unreachable!(),
        };
        let value = replace_lambdas(&mut c, value, body);
        c.declarations[at].kind = DeclarationKind::Def {
            ty,
            value,
            reducibility,
        };
        let bytes = encode(c.clone());
        let filename = format!("body-{name}");
        retain(&filename, "mpcert", &bytes);
        let mut prefix = c;
        prefix.declarations.truncate(at + 1);
        let typed = mpk_kernel::check_declarations(&prefix).is_ok();
        prefixes.push(serde_json::json!({"name":name,"prefix_declarations":at+1,"definition_prefix_typechecks":typed,"full_certificate_acceptance_claimed":false}));
        assert!(typed, "well-typed changed {name} definition prefix failed");
        inspect_saved(&filename, &bytes, false);
    }
    retain(
        "wrong-operations",
        "prefix.json",
        format!("{}\n", serde_json::to_string_pretty(&prefixes).unwrap()).as_bytes(),
    );
    let value = generate_go_practical_string_value(&(0..=255u8).collect::<Vec<_>>()).unwrap();
    for suffix in [
        "LengthBit8",
        "LengthBit63",
        "Slot0Bit0",
        "Slot127Bit7",
        "Slot128Bit0",
        "Slot255Bit7",
    ] {
        let mut c = mpk_cert::decode_canonical_certificate(value.certificate_bytes()).unwrap();
        let full = format!("Mpk.Go.Practical.String.Value.{suffix}");
        let at = c
            .declarations
            .iter()
            .position(|d| c.name_table[d.name as usize] == full)
            .unwrap();
        let DeclarationKind::Theorem { ty, proof } = c.declarations[at].kind else {
            unreachable!()
        };
        let ty = replace_result(&mut c, ty);
        c.declarations[at].kind = DeclarationKind::Theorem { ty, proof };
        inspect(&format!("literal-false-{suffix}"), &encode(c), false);
    }
    println!("full string controls:{} altered conclusions,{premises} erased real premises,5 well-typed wrong operations,6 wrong actual bits",names.len());
}
