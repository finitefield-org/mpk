use crate::decl_driver::check_declarations;

fn fixture(name: &str) -> mpk_cert::encode::Certificate {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/core-bool-cases")
        .join(format!("{name}.hex"));
    let text = std::fs::read_to_string(path).unwrap();
    let text = text.trim();
    let bytes = (0..text.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&text[i..i + 2], 16).unwrap())
        .collect::<Vec<_>>();
    mpk_cert::decode_canonical_certificate(&bytes).unwrap()
}

#[test]
fn bool_cases_accepts_universal_proofs_and_constructor_equations() {
    for name in [
        "right-identity",
        "constructor-false",
        "constructor-true",
        "open-motive",
        "conjunction-left",
        "conjunction-right",
    ] {
        let certificate = fixture(name);
        assert!(certificate.proof_node_table.is_empty());
        assert!(certificate.theory_certificates.is_empty());
        assert!(!certificate
            .declarations
            .iter()
            .any(|d| matches!(d.kind, mpk_cert::encode::DeclarationKind::Axiom { .. })));
        check_declarations(&certificate).unwrap_or_else(|error| panic!("{name}: {error:?}"));
    }
}

#[test]
fn bool_cases_rejects_mutated_interfaces_proofs_and_universe_arguments() {
    for name in [
        "wrong-branch",
        "wrong-conjunction-left",
        "wrong-conjunction-right",
        "wrong-motive-universe",
        "wrong-interface",
        "nongenerated",
        "extra-constructor",
        "extra-constructor-after-cases",
        "renamed",
        "wrong-case-levels",
        "wrong-major-levels",
        "wrong-case-levels-open",
        "wrong-major-levels-open",
    ] {
        assert!(check_declarations(&fixture(name)).is_err(), "{name}");
    }
}
