use mpk_cert::encode::{Certificate, DeclarationKind};
use mpk_vc::go_practical_ordinary::{
    generate_go_practical_string_concat_foundation, import_go_practical_string_concat_foundation,
    GoOrdinaryError,
};
#[path = "support/go_word_division_dependencies.rs"]
mod dependencies;
fn retain(name: &str, extension: &str, raw: &[u8]) {
    if let Some(root) = std::env::var_os("MPK_GO_STRING_CONCAT_OUTPUT") {
        use std::io::Write;
        let root = std::path::PathBuf::from(root);
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
        let c = mpk_cert::decode_canonical_certificate(bytes).unwrap();
        let (mut lo, mut hi) = (1, c.declarations.len());
        while lo < hi {
            let end = (lo + hi) / 2;
            let mut prefix = c.clone();
            prefix.declarations.truncate(end);
            if mpk_kernel::check_declarations(&prefix).is_ok() {
                lo = end + 1;
            } else {
                hi = end;
            }
        }
        panic!(
            "{name}: first declaration type-check failure {}: {}",
            c.name_table[c.declarations[lo - 1].name as usize],
            result.json
        );
    }
    assert_eq!(result.accepted, accepted, "{name}: {}", result.json);
    if accepted {
        let v: serde_json::Value = serde_json::from_str(&result.json).unwrap();
        assert_eq!(v["axiom_count"], 0);
        assert!(v["axiom_report"]["entries"].as_array().unwrap().is_empty());
    } else {
        let v: serde_json::Value = serde_json::from_str(&result.json).unwrap();
        let d: serde_json::Value =
            serde_json::from_str(v["error_detail"].as_str().unwrap()).unwrap();
        assert_eq!(d["code"], "CORE_TYPE_MISMATCH");
    }
}

use mpk_cert::encode::{Declaration, TermNode};
fn term(c: &mut Certificate, node: TermNode) -> u32 {
    let at = c.term_table.len() as u32;
    c.term_table.push(node);
    at
}
fn sym(c: &mut Certificate, name: &str) -> u32 {
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
    let function = sym(c, name);
    term(
        c,
        TermNode::App {
            function,
            arguments,
        },
    )
}
fn flag(c: &mut Certificate, yes: bool) -> u32 {
    sym(
        c,
        if yes {
            "Std.Bool.true"
        } else {
            "Std.Bool.false"
        },
    )
}
fn encode(mut c: Certificate) -> Vec<u8> {
    let old = c.name_table.clone();
    c.name_table.sort();
    for d in &mut c.declarations {
        d.name = c.name_table.binary_search(&old[d.name as usize]).unwrap() as u32;
    }
    for l in &mut c.level_table {
        if let mpk_cert::encode::LevelNode::Param(name) = l {
            *name = c.name_table.binary_search(&old[*name as usize]).unwrap() as u32;
        }
    }
    assert!(c.declarations.len() <= 4096 && c.term_table.len() <= 131072);
    c.export_block = mpk_cert::build_export_block(&c).unwrap();
    c.axiom_report = mpk_cert::build_axiom_report(&c).unwrap();
    assert_eq!(c.axiom_report.summary.total_axiom_count, 0);
    c.hashes.export_hash = mpk_cert::export_block_hash(&c.export_block);
    c.hashes.axiom_report_hash = mpk_cert::axiom_report_hash_for_report(&c.axiom_report);
    mpk_cert::encode_certificate_bounded(&c, 2097152).unwrap()
}
fn bad_result(c: &mut Certificate, ty: u32) -> u32 {
    match c.term_table[ty as usize].clone() {
        TermNode::Pi { ty, body } => {
            let body = bad_result(c, body);
            term(c, TermNode::Pi { ty, body })
        }
        TermNode::App {
            function,
            mut arguments,
        } => {
            if let TermNode::Const { global, .. } = c.term_table[function as usize] {
                if c.name_table[c.declarations[global as usize].name as usize] == "Std.Eq"
                    && arguments.len() == 3
                {
                    if let TermNode::Const { global, .. } = c.term_table[arguments[0] as usize] {
                        if c.name_table[c.declarations[global as usize].name as usize] == "Std.Bool"
                        {
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
            }
            let boolean = sym(c, "Std.Bool");
            let no = flag(c, false);
            let yes = flag(c, true);
            call(c, "Std.Eq", vec![boolean, no, yes])
        }
        _ => {
            let boolean = sym(c, "Std.Bool");
            let no = flag(c, false);
            let yes = flag(c, true);
            call(c, "Std.Eq", vec![boolean, no, yes])
        }
    }
}
fn erase(c: &mut Certificate, ty: u32, at: usize, domain: u32) -> u32 {
    let TermNode::Pi { ty: old, body } = c.term_table[ty as usize].clone() else {
        panic!("missing actual witness")
    };
    if at == 0 {
        term(c, TermNode::Pi { ty: domain, body })
    } else {
        let body = erase(c, body, at - 1, domain);
        term(c, TermNode::Pi { ty: old, body })
    }
}

fn foundation() -> &'static mpk_vc::go_practical_ordinary::GoStringConcatFoundation {
    static VALUE: std::sync::OnceLock<mpk_vc::go_practical_ordinary::GoStringConcatFoundation> =
        std::sync::OnceLock::new();
    VALUE.get_or_init(|| generate_go_practical_string_concat_foundation().unwrap())
}
fn baseline() -> Certificate {
    mpk_cert::decode_canonical_certificate(foundation().certificate_bytes()).unwrap()
}
#[test]
fn concat_preserves_all2142_prior_identities() {
    let f = foundation();
    inspect("concat-foundation", f.certificate_bytes(), true);
    assert_eq!(
        *f,
        import_go_practical_string_concat_foundation(f.certificate_bytes()).unwrap()
    );
    let c = baseline();
    let all = dependencies::declarations(&c);
    let old =
        mpk_vc::go_practical_ordinary::generate_go_practical_string_slice_foundation().unwrap();
    let path=std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../develop/migrations/go-practical/t02-w01/verification/string-slice/epochs/final-controls/selected/candidates/slice-foundation.mpcert");
    assert_eq!(old.certificate_bytes(), std::fs::read(path).unwrap());
    let old = mpk_cert::decode_canonical_certificate(old.certificate_bytes()).unwrap();
    assert_eq!(old.declarations.len(), 2142);
    for (name, value) in dependencies::declarations(&old) {
        assert_eq!(all.get(&name), Some(&value), "{name}");
    }
    let laws = c
        .declarations
        .iter()
        .filter(|d| c.name_table[d.name as usize].starts_with("Mpk.Go.Practical.String.Concat."))
        .count();
    println!("string concat:declarations={} terms={} bytes={} all2142 prior identities and full old bytes exact;{} new declarations",c.declarations.len(),c.term_table.len(),f.certificate_bytes().len(),laws);
}
#[test]
fn exact_ownership_is_distinct_from_core_accepted_extensions() {
    let mut c = baseline();
    let ty = sym(&mut c, "Std.Bool");
    let value = flag(&mut c, true);
    let name = c.name_table.len() as u32;
    c.name_table
        .push("Mpk.Go.Practical.String.Concat.HonestExtra".into());
    c.declarations.push(Declaration {
        name,
        kind: DeclarationKind::Def {
            ty,
            value,
            reducibility: mpk_cert::encode::DefinitionReducibility::Reducible,
        },
    });
    let raw = encode(c);
    inspect("concat-honest-extra", &raw, true);
    assert_eq!(
        import_go_practical_string_concat_foundation(&raw),
        Err(GoOrdinaryError::Linkage)
    );
    assert_eq!(
        import_go_practical_string_concat_foundation(&vec![0; 2097153]),
        Err(GoOrdinaryError::Limit)
    );
}

#[test]
fn concat_laws_bind_conclusions_and_actual_typed_witnesses() {
    let base = baseline();
    let prefix = "Mpk.Go.Practical.String.Concat.";
    let mut laws = 0;
    let mut witnesses = 0;
    for (at, d) in base.declarations.iter().enumerate() {
        let name = &base.name_table[d.name as usize];
        if !name.starts_with(prefix) {
            continue;
        }
        let DeclarationKind::Theorem { ty, proof } = d.kind else {
            continue;
        };
        let short = name.strip_prefix(prefix).unwrap();
        let mut c = base.clone();
        let changed = bad_result(&mut c, ty);
        c.declarations[at].kind = DeclarationKind::Theorem { ty: changed, proof };
        inspect(&format!("false-{short}"), &encode(c), false);
        laws += 1;
        let mut cursor = ty;
        let mut binder = 0;
        while let TermNode::Pi { ty: domain, body } = base.term_table[cursor as usize] {
            let is_witness = if let TermNode::App { function, .. } =
                base.term_table[domain as usize]
            {
                if let TermNode::Const { global, .. } = base.term_table[function as usize] {
                    matches!(
                        base.name_table[base.declarations[global as usize].name as usize].as_str(),
                        "Std.Eq" | "Mpk.Go.Practical.String.Equality.Equivalent"
                    )
                } else {
                    false
                }
            } else {
                false
            };
            if is_witness {
                let mut c = base.clone();
                let boolean = sym(&mut c, "Std.Bool");
                let changed = erase(&mut c, ty, binder, boolean);
                c.declarations[at].kind = DeclarationKind::Theorem { ty: changed, proof };
                inspect(&format!("premise-{short}-{binder}"), &encode(c), false);
                witnesses += 1;
            }
            cursor = body;
            binder += 1;
        }
    }
    assert_eq!(laws, 46);
    assert_eq!(witnesses, 62);
    retain("control-inventory","json",format!("{}\n",serde_json::json!({"altered_conclusions":laws,"actual_typed_witness_domains":witnesses,"typed_wrong_definitions":7,"zero_axiom_acceptances":2,"complete_candidates":laws+witnesses+9})).as_bytes());
    println!("concat altered conclusions={laws} actual typed witness domains={witnesses}");
}

fn constant_body(c: &mut Certificate, value: u32, replacement: u32, arguments: usize) -> u32 {
    if arguments == 0 {
        return replacement;
    }
    let TermNode::Lam { ty, body } = c.term_table[value as usize] else {
        panic!("missing actual argument")
    };
    let body = constant_body(c, body, replacement, arguments - 1);
    term(c, TermNode::Lam { ty, body })
}

fn byte_word(c: &mut Certificate, n: u8) -> u32 {
    let bits = (0..8).rev().map(|i| flag(c, n & (1 << i) != 0)).collect();
    call(c, "Mpk.Go.Practical.String.Slice.IndexWord", bits)
}
#[test]
fn concat_definitions_are_bound_to_actual_carriers_and_capacity() {
    for short in [
        "CapacityWord",
        "Fits",
        "Carry",
        "LengthWord",
        "RightWord",
        "Data",
        "Value",
    ] {
        let mut c = baseline();
        let name = format!("Mpk.Go.Practical.String.Concat.{short}");
        let at = c
            .declarations
            .iter()
            .position(|d| c.name_table[d.name as usize] == name)
            .unwrap();
        let DeclarationKind::Def {
            ty,
            value,
            reducibility,
        } = c.declarations[at].kind
        else {
            unreachable!()
        };
        let zero = sym(&mut c, "Mpk.Go.Practical.String.Zero");
        let replacement = match short {
            "CapacityWord" => {
                let a = byte_word(&mut c, 255);
                let b = byte_word(&mut c, 2);
                call(&mut c, "Mpk.Go.Practical.Word.W64.Add", vec![a, b])
            }
            "Fits" => flag(&mut c, true),
            "Carry" => flag(&mut c, false),
            "LengthWord" | "RightWord" => {
                call(&mut c, "Mpk.Go.Practical.String.Length", vec![zero])
            }
            "Data" => call(&mut c, "Mpk.Go.Practical.String.Data", vec![zero]),
            _ => zero,
        };
        let arity = match short {
            "CapacityWord" => 0,
            "RightWord" => 9,
            _ => 2,
        };
        let value = constant_body(&mut c, value, replacement, arity);
        c.declarations[at].kind = DeclarationKind::Def {
            ty,
            value,
            reducibility,
        };
        let name = format!("definition-{short}");
        let raw = encode(c);
        retain(&name, "mpcert", &raw);
        observations::definition_countermodel(
            &mpk_cert::decode_canonical_certificate(&raw).unwrap(),
            short,
            retain,
        );
        inspect_saved(&name, &raw, false);
    }
}

#[path = "support/go_string_equality_eval.rs"]
mod actual;
#[path = "support/go_string_concat_observations.rs"]
mod observations;
#[test]
fn observations_cover_full_carriers_capacity_and_normalization() {
    observations::run(&baseline(), retain);
}
