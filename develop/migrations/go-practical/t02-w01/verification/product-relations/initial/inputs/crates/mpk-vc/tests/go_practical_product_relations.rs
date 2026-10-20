use mpk_vc::go_practical_ordinary::{
    generate_go_practical_product_foundation, generate_go_practical_product_relation_foundation,
    generate_go_practical_word_product_equality_foundation,
    generate_go_practical_word_signed_equality_foundation,
    import_go_practical_product_relation_foundation,
    import_go_practical_word_product_equality_foundation, GoOrdinaryError,
};
use sha2::{Digest, Sha256};
#[path = "support/go_product_relation_controls.rs"]
mod controls;
#[path = "support/go_word_division_dependencies.rs"]
mod dependencies;

fn inspect(name: &str, bytes: &[u8], accepted: bool) {
    let result = mpk_kernel::verify_certificate_bytes_json_output(bytes);
    if let Some(out) = std::env::var_os("MPK_GO_PRODUCT_RELATIONS_TEST_OUTPUT") {
        use std::io::Write;
        let out = std::path::PathBuf::from(out);
        std::fs::create_dir_all(&out).unwrap();
        for (extension, raw) in [
            ("mpcert", bytes.to_vec()),
            ("rust.json", format!("{}\n", result.json).into_bytes()),
        ] {
            let mut file = std::fs::OpenOptions::new()
                .create_new(true)
                .write(true)
                .open(out.join(format!("{name}.{extension}")))
                .unwrap();
            file.write_all(&raw).unwrap();
        }
    }
    assert_eq!(result.accepted, accepted, "{name}: {}", result.json);
    if accepted {
        let report: serde_json::Value = serde_json::from_str(&result.json).unwrap();
        assert_eq!(report["axiom_count"], 0);
        assert!(report["axiom_report"]["entries"]
            .as_array()
            .unwrap()
            .is_empty());
    } else {
        let report: serde_json::Value = serde_json::from_str(&result.json).unwrap();
        assert_eq!(report["error_code"], "KERNEL_CORE_CHECK");
        let detail: serde_json::Value =
            serde_json::from_str(report["error_detail"].as_str().unwrap()).unwrap();
        assert_eq!(detail["code"], "CORE_TYPE_MISMATCH");
    }
}

fn retained(new: &[u8], old: &[u8], hash: &str, count: usize) {
    assert_eq!(format!("{:x}", Sha256::digest(old)), hash);
    let old = mpk_cert::decode_canonical_certificate(old).unwrap();
    assert_eq!(old.declarations.len(), count);
    let new = mpk_cert::decode_canonical_certificate(new).unwrap();
    assert!(new.declarations.len() <= 4096 && new.term_table.len() <= 131_072);
    assert!(new.imports.is_empty());
    assert_eq!(new.axiom_report.summary.total_axiom_count, 0);
    let names = dependencies::declarations(&new);
    for (name, identity) in dependencies::declarations(&old) {
        assert_eq!(names.get(&name), Some(&identity), "{name}");
    }
}

#[test]
fn actual_relational_product_component_retains_every_original_product_identity() {
    let foundation = generate_go_practical_product_relation_foundation().unwrap();
    let bytes = foundation.certificate_bytes();
    inspect("product-relations", bytes, true);
    let old = generate_go_practical_product_foundation().unwrap();
    retained(
        bytes,
        old.certificate_bytes(),
        "06d60305b4616213d61ca3dae35decf76fb150e729e1f063c8c6266cc9ec709e",
        78,
    );
    let c = mpk_cert::decode_canonical_certificate(bytes).unwrap();
    assert_eq!(c.declarations.len(), 94);
    println!(
        "relations: declarations={} terms={} bytes={} zero axioms",
        c.declarations.len(),
        c.term_table.len(),
        bytes.len()
    );
}

#[test]
fn actual_all_mixed_word_pairs_and_arbitrary_integer_representatives() {
    let foundation = generate_go_practical_word_product_equality_foundation().unwrap();
    let bytes = foundation.certificate_bytes();
    inspect("word-product-equality", bytes, true);
    let old = generate_go_practical_word_signed_equality_foundation().unwrap();
    retained(
        bytes,
        old.certificate_bytes(),
        "aff545ade52f37b0c5dcf215e8fd5f5aff1a06d269193a9827cbc4b350e6c06f",
        3489,
    );
    let product = generate_go_practical_product_foundation().unwrap();
    retained(
        bytes,
        product.certificate_bytes(),
        "06d60305b4616213d61ca3dae35decf76fb150e729e1f063c8c6266cc9ec709e",
        78,
    );
    let c = mpk_cert::decode_canonical_certificate(bytes).unwrap();
    assert_eq!(c.declarations.len(), 3581);
    let names = dependencies::declarations(&c);
    for left in [8, 16, 32, 64] {
        for right in [8, 16, 32, 64] {
            for member in ["EquivalentReflect", "EquivalentComplete"] {
                assert!(names.contains_key(&format!(
                    "Mpk.Go.Practical.Word.ProductEquality.W{left}W{right}.{member}"
                )));
            }
        }
    }
    assert!(names.contains_key("Mpk.Go.Practical.Int.ProductEquality.RepresentationShift"));
    println!(
        "word pairs: declarations={} terms={} bytes={} zero axioms",
        c.declarations.len(),
        c.term_table.len(),
        bytes.len()
    );
}

#[test]
fn exact_component_ownership_remains_stricter_than_core_acceptance() {
    let relations = generate_go_practical_product_relation_foundation().unwrap();
    assert_eq!(
        relations,
        import_go_practical_product_relation_foundation(relations.certificate_bytes()).unwrap()
    );
    let words = generate_go_practical_word_product_equality_foundation().unwrap();
    assert_eq!(
        words,
        import_go_practical_word_product_equality_foundation(words.certificate_bytes()).unwrap()
    );
    let different = controls::honest_different(false);
    inspect("honest-different-relations", &different, true);
    assert_eq!(
        import_go_practical_product_relation_foundation(&different),
        Err(GoOrdinaryError::Linkage)
    );
    let different = controls::honest_different(true);
    inspect("honest-different-word-pairs", &different, true);
    assert_eq!(
        import_go_practical_word_product_equality_foundation(&different),
        Err(GoOrdinaryError::Linkage)
    );
    for original in [relations.certificate_bytes(), words.certificate_bytes()] {
        let mut trailing = original.to_vec();
        trailing.push(0);
        assert_eq!(
            import_go_practical_product_relation_foundation(&trailing),
            Err(GoOrdinaryError::Linkage)
        );
        assert_eq!(
            import_go_practical_word_product_equality_foundation(&trailing),
            Err(GoOrdinaryError::Linkage)
        );
    }
    assert_eq!(
        import_go_practical_product_relation_foundation(&vec![0; 2_097_153]),
        Err(GoOrdinaryError::Limit)
    );
    assert_eq!(
        import_go_practical_word_product_equality_foundation(&vec![0; 2_097_153]),
        Err(GoOrdinaryError::Limit)
    );
}

#[test]
fn reject_falsified_relations_comparators_and_real_field_premises() {
    controls::semantic_controls();
}
