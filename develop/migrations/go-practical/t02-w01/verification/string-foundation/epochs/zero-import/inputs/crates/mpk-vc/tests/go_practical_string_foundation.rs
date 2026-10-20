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

#[test]
fn frozen_string_foundation_preserves_all_full64_prerequisites() {
    inspect("string-foundation", foundation().certificate_bytes(), true);
    let c = baseline();
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
