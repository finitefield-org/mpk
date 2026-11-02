//! Born-ignored full original assembly diagnostic; no source/profile/program authority.
#[path = "support/go_ordinary_assembly_probe.rs"]
mod assembly;
#[path = "support/go_word_division_dependencies.rs"]
mod dependencies;
use mpk_cert::encode::{Certificate, DeclarationKind, TermNode};
use mpk_vc::go_practical_ordinary::{
    generate_go_practical_product_foundation, generate_go_practical_word_shift_foundation,
};
use sha2::{Digest, Sha256};

fn own(name: &str, extension: &str, bytes: &[u8]) {
    use std::io::Write;
    let out = std::path::PathBuf::from(
        std::env::var_os("MPK_GO_ORDINARY_ASSEMBLY_PROBE_OUTPUT").unwrap(),
    );
    std::fs::create_dir_all(&out).unwrap();
    let mut file = std::fs::OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(out.join(format!("{name}.{extension}")))
        .unwrap();
    file.write_all(bytes).unwrap();
}
fn inspect(name: &str, c: Certificate, accepted: bool) {
    let counts = (c.declarations.len(), c.term_table.len());
    let bytes = assembly::encode(c);
    own(name, "mpcert", &bytes);
    assert!(counts.0 <= 4096 && counts.1 <= 131072 && bytes.len() <= 2097152);
    let result = mpk_kernel::verify_certificate_bytes_json_output(&bytes);
    own(name, "rust.json", format!("{}\n", result.json).as_bytes());
    println!(
        "{name}: declarations={} terms={} bytes={} accepted={}",
        counts.0,
        counts.1,
        bytes.len(),
        result.accepted
    );
    assert_eq!(result.accepted, accepted, "{name}: {}", result.json);
    if !accepted {
        let report: serde_json::Value = serde_json::from_str(&result.json).unwrap();
        assert_eq!(report["error_code"], "KERNEL_CORE_CHECK");
        let detail: serde_json::Value =
            serde_json::from_str(report["error_detail"].as_str().unwrap()).unwrap();
        assert_eq!(detail["code"], "CORE_TYPE_MISMATCH");
    }
}
fn global(c: &Certificate, name: &str) -> usize {
    c.declarations
        .iter()
        .position(|d| c.name_table[d.name as usize] == name)
        .unwrap()
}
fn term(c: &mut Certificate, n: TermNode) -> u32 {
    let id = c.term_table.len() as u32;
    c.term_table.push(n);
    id
}

#[test]
#[ignore = "Explicit full4096-declaration closed assembly probe; no full56/source/context authority."]
fn complete_full64_shift_and_product_with_strict_original_identity() {
    let shift = generate_go_practical_word_shift_foundation().unwrap();
    let product = generate_go_practical_product_foundation().unwrap();
    own("original-shift", "mpcert", shift.certificate_bytes());
    own("original-product", "mpcert", product.certificate_bytes());
    assert_eq!(
        format!("{:x}", Sha256::digest(shift.certificate_bytes())),
        "119c384ddefd5afc6c42de2171df608d4b5b6329304912d3e9ce29860751e1b2"
    );
    assert_eq!(
        format!("{:x}", Sha256::digest(product.certificate_bytes())),
        "06d60305b4616213d61ca3dae35decf76fb150e729e1f063c8c6266cc9ec709e"
    );
    let inputs = [shift.certificate_bytes(), product.certificate_bytes()]
        .map(|b| mpk_cert::decode_canonical_certificate(b).unwrap());
    let merged = assembly::merge(&inputs).unwrap();
    assert_eq!(merged.declarations.len(), 4096);
    let actual = dependencies::declarations(&merged);
    for c in &inputs {
        for (name, value) in dependencies::declarations(c) {
            assert_eq!(actual.get(&name), Some(&value), "{name}");
        }
    }
    inspect("full-shift-product", merged.clone(), true);
    // A genuinely valid alternate proof has the same type, but is not the
    // exact original named definition/proof identity allowed by this assembly.
    let mut changed = inputs[1].clone();
    let index = changed
        .declarations
        .iter()
        .enumerate()
        .find_map(|(i, d)| {
            let name = &changed.name_table[d.name as usize];
            (inputs[0]
                .declarations
                .iter()
                .any(|other| inputs[0].name_table[other.name as usize] == *name)
                && matches!(d.kind, DeclarationKind::Theorem { .. }))
            .then_some(i)
        })
        .unwrap();
    let DeclarationKind::Theorem { ty, proof } = changed.declarations[index].kind else {
        unreachable!()
    };
    let body = term(&mut changed, TermNode::Var(0));
    let alternate = term(
        &mut changed,
        TermNode::Let {
            ty,
            value: proof,
            body,
        },
    );
    changed.declarations[index].kind = DeclarationKind::Theorem {
        ty,
        proof: alternate,
    };
    let name = changed.name_table[changed.declarations[index].name as usize].clone();
    let error = assembly::merge(&[inputs[0].clone(), changed.clone()]).unwrap_err();
    assert_eq!(error, format!("nominal identity conflict: {name}"));
    own("nominal-conflict","json",format!("{}\n",serde_json::json!({"original_name":name,"actual_assembly_rejection":error,"scope":"Exact original same-name body/proof identity, independent of type correctness. The alternate complete component is actually kernel-checked separately; no caller extension/profile authority."})).as_bytes());
    inspect("valid-named-proof-variant", changed, true);
    let mut wrong = merged;
    let boolean = global(&wrong, "Std.Bool") as u32;
    let false_id = global(&wrong, "Std.Bool.false") as u32;
    let true_id = global(&wrong, "Std.Bool.true") as u32;
    let equal = global(&wrong, "Std.Eq") as u32;
    let ty = term(
        &mut wrong,
        TermNode::Const {
            global: boolean,
            levels: vec![],
        },
    );
    let left = term(
        &mut wrong,
        TermNode::Const {
            global: false_id,
            levels: vec![],
        },
    );
    let right = term(
        &mut wrong,
        TermNode::Const {
            global: true_id,
            levels: vec![],
        },
    );
    let function = term(
        &mut wrong,
        TermNode::Const {
            global: equal,
            levels: vec![],
        },
    );
    let changed_type = term(
        &mut wrong,
        TermNode::App {
            function,
            arguments: vec![ty, left, right],
        },
    );
    let index = global(&wrong, "Mpk.Go.Practical.Product.UnitZeroValid");
    let DeclarationKind::Theorem { proof, .. } = wrong.declarations[index].kind else {
        unreachable!()
    };
    wrong.declarations[index].kind = DeclarationKind::Theorem {
        ty: changed_type,
        proof,
    };
    inspect("false-unit-in-full-assembly", wrong, false);
}
