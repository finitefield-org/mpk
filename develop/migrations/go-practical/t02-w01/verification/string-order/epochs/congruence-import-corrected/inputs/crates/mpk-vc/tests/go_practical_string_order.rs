use mpk_cert::encode::{Certificate, DeclarationKind};
use mpk_vc::go_practical_ordinary::{
    generate_go_practical_string_equality_foundation,
    generate_go_practical_string_order_foundation, import_go_practical_string_order_foundation,
    GoOrdinaryError,
};
#[path = "support/go_word_division_dependencies.rs"]
mod dependencies;
fn retain(name: &str, extension: &str, raw: &[u8]) {
    if let Some(root) = std::env::var_os("MPK_GO_STRING_ORDER_OUTPUT") {
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

fn foundation() -> &'static mpk_vc::go_practical_ordinary::GoStringOrderFoundation {
    static VALUE: std::sync::OnceLock<mpk_vc::go_practical_ordinary::GoStringOrderFoundation> =
        std::sync::OnceLock::new();
    VALUE.get_or_init(|| generate_go_practical_string_order_foundation().unwrap())
}
fn baseline() -> Certificate {
    mpk_cert::decode_canonical_certificate(foundation().certificate_bytes()).unwrap()
}
#[test]
fn generic_order_preserves_all1215_original_identities() {
    let f = foundation();
    inspect("order-foundation", f.certificate_bytes(), true);
    assert_eq!(
        *f,
        import_go_practical_string_order_foundation(f.certificate_bytes()).unwrap()
    );
    let c = baseline();
    let all = dependencies::declarations(&c);
    let old = generate_go_practical_string_equality_foundation().unwrap();
    let golden = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../develop/migrations/go-practical/t02-w01/verification/string-equality/epochs/semantic-controls/selected/candidates/equality-foundation.mpcert");
    assert_eq!(old.certificate_bytes(), std::fs::read(golden).unwrap());
    let old = mpk_cert::decode_canonical_certificate(old.certificate_bytes()).unwrap();
    assert_eq!(old.declarations.len(), 1215);
    for (name, value) in dependencies::declarations(&old) {
        assert_eq!(all.get(&name), Some(&value), "{name}");
    }
    let old = mpk_vc::go_practical_ordinary::generate_go_practical_word_order_foundation().unwrap();
    let golden = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../develop/migrations/go-practical/t02-w01/verification/word-order/corrected/rust-tests/certificates/word-order-foundation.mpcert");
    assert_eq!(old.certificate_bytes(), std::fs::read(golden).unwrap());
    for (name, value) in dependencies::declarations(
        &mpk_cert::decode_canonical_certificate(old.certificate_bytes()).unwrap(),
    ) {
        if let Some(current) = all.get(&name) {
            assert_eq!(current, &value, "{name}");
        }
    }
    let (defs, laws) = c
        .declarations
        .iter()
        .filter(|d| c.name_table[d.name as usize].starts_with("Mpk.Go.Practical.String.Order."))
        .fold((0, 0), |(defs, laws), d| match d.kind {
            DeclarationKind::Def { .. } => (defs + 1, laws),
            DeclarationKind::Theorem { .. } => (defs, laws + 1),
            _ => panic!("ordinary declarations only"),
        });
    println!("string order:declarations={} terms={} bytes={} all1215 prior identities exact;{} definitions{} ordinary laws",c.declarations.len(),c.term_table.len(),f.certificate_bytes().len(),defs,laws);
}
#[test]
fn exact_ownership_is_distinct_from_core_accepted_extensions() {
    let mut c = baseline();
    let ty = sym(&mut c, "Std.Bool");
    let value = flag(&mut c, true);
    let name = c.name_table.len() as u32;
    c.name_table
        .push("Mpk.Go.Practical.String.Order.HonestExtra".into());
    c.declarations.push(Declaration {
        name,
        kind: DeclarationKind::Def {
            ty,
            value,
            reducibility: mpk_cert::encode::DefinitionReducibility::Reducible,
        },
    });
    let raw = encode(c);
    inspect("order-honest-extra", &raw, true);
    assert_eq!(
        import_go_practical_string_order_foundation(&raw),
        Err(GoOrdinaryError::Linkage)
    );
    assert_eq!(
        import_go_practical_string_order_foundation(&vec![0; 2097153]),
        Err(GoOrdinaryError::Limit)
    );
}

#[path = "support/go_string_equality_eval.rs"]
mod order_eval;

#[test]
fn diagnose_boolean_total_original() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../develop/migrations/go-practical/t02-w01/verification/string-order/epochs/factored-boolean-total/foundation/candidates/order-foundation.mpcert");
    let raw = std::fs::read(path).unwrap();
    retain("original-total-candidate", "mpcert", &raw);
    let mut c = mpk_cert::decode_canonical_certificate(&raw).unwrap();
    let mut observations = vec![];
    for case in 0..16 {
        let name = format!("Mpk.Go.Practical.String.Order.TotalCase{case}");
        let DeclarationKind::Theorem { ty, .. } = c
            .declarations
            .iter()
            .find(|d| c.name_table[d.name as usize] == name)
            .unwrap()
            .kind
        else {
            unreachable!()
        };
        let TermNode::App { ref arguments, .. } = c.term_table[ty as usize] else {
            unreachable!()
        };
        let values = [arguments[1], arguments[2]];
        let mut names = vec![];
        for (side, value) in values.into_iter().enumerate() {
            let name = format!("Mpk.Go.Practical.String.Order.Diagnostic{case}Side{side}");
            let idx = c.name_table.len() as u32;
            c.name_table.push(name.clone());
            let ty = sym(&mut c, "Std.Bool");
            c.declarations.push(Declaration {
                name: idx,
                kind: DeclarationKind::Def {
                    ty,
                    value,
                    reducibility: mpk_cert::encode::DefinitionReducibility::Reducible,
                },
            });
            names.push(name);
        }
        let mut model = order_eval::Interpreter::new(&c);
        let result = names
            .iter()
            .map(|name| {
                let value = model.invoke(name, &[]);
                model.boolean(value)
            })
            .collect::<Vec<_>>();
        println!("original TotalCase{case}:{result:?}");
        observations.push(serde_json::json!({"case":case,"actual_endpoints":result,"expected_endpoints":[case==0,case==0],"candidate_is_rejected":true,"theorem_bodies_executed":false}));
    }
    retain(
        "original-total-algebra",
        "observation.json",
        format!("{}\n", serde_json::to_string_pretty(&observations).unwrap()).as_bytes(),
    );
}
