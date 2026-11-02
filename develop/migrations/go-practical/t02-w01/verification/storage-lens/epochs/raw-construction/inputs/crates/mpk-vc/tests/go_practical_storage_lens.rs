use mpk_cert::encode::{Certificate, DeclarationKind};
use mpk_vc::go_practical_ordinary::{
    generate_go_practical_storage_lens_foundation,
    import_go_practical_storage_lens_foundation,
};
#[path = "support/go_word_division_dependencies.rs"]
mod dependencies;

fn retain(name: &str, extension: &str, raw: &[u8]) {
    if let Some(root) = std::env::var_os("MPK_GO_STORAGE_LENS_OUTPUT") {
        use std::io::Write;
        let root = std::path::PathBuf::from(root);
        std::fs::create_dir_all(&root).unwrap();
        std::fs::OpenOptions::new().create_new(true).write(true)
            .open(root.join(format!("{name}.{extension}"))).unwrap()
            .write_all(raw).unwrap();
    }
}
fn inspect(name: &str, bytes: &[u8], accepted: bool) {
    retain(name, "mpcert", bytes);
    let result = mpk_kernel::verify_certificate_bytes_json_output(bytes);
    retain(name, "rust.json", format!("{}\n", result.json).as_bytes());
    if accepted && !result.accepted {
        let c = mpk_cert::decode_canonical_certificate(bytes).unwrap();
        let (mut lo, mut hi) = (1, c.declarations.len());
        while lo < hi {
            let end = (lo + hi) / 2;
            let mut prefix = c.clone(); prefix.declarations.truncate(end);
            if mpk_kernel::check_declarations(&prefix).is_ok() { lo = end + 1; }
            else { hi = end; }
        }
        panic!("{name}: first declaration type-check failure {}: {}",
            c.name_table[c.declarations[lo - 1].name as usize], result.json);
    }
    assert_eq!(result.accepted, accepted, "{name}: {}", result.json);
    let v: serde_json::Value = serde_json::from_str(&result.json).unwrap();
    if accepted {
        assert_eq!(v["axiom_count"], 0);
        assert!(v["axiom_report"]["entries"].as_array().unwrap().is_empty());
    } else {
        let d: serde_json::Value = serde_json::from_str(v["error_detail"].as_str().unwrap()).unwrap();
        assert_eq!(d["code"], "CORE_TYPE_MISMATCH");
    }
}
fn foundation() -> &'static mpk_vc::go_practical_ordinary::GoStorageLensFoundation {
    static VALUE: std::sync::OnceLock<mpk_vc::go_practical_ordinary::GoStorageLensFoundation> = std::sync::OnceLock::new();
    VALUE.get_or_init(|| generate_go_practical_storage_lens_foundation().unwrap())
}
fn baseline() -> Certificate {
    mpk_cert::decode_canonical_certificate(foundation().certificate_bytes()).unwrap()
}
#[test]
fn lens_retains_complete_parent() {
    let f = foundation();
    inspect("lens-foundation", f.certificate_bytes(), true);
    assert_eq!(*f, import_go_practical_storage_lens_foundation(f.certificate_bytes()).unwrap());
    let old = mpk_vc::go_practical_ordinary::generate_go_practical_array_foundation().unwrap();
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../develop/migrations/go-practical/t02-w01/verification/array-foundation/epochs/harness-context/foundation/candidates/array-foundation.mpcert");
    assert_eq!(old.certificate_bytes(), std::fs::read(path).unwrap());
    let prior = mpk_cert::decode_canonical_certificate(old.certificate_bytes()).unwrap();
    let c = baseline();
    let hashes = dependencies::declarations(&c);
    for (name, v) in dependencies::declarations(&prior) {
        assert_eq!(hashes.get(&name), Some(&v), "{name}");
    }
    let fresh = c.declarations.iter().filter(|d| c.name_table[d.name as usize].starts_with("Mpk.Go.Practical.Storage.Lens.")).collect::<Vec<_>>();
    let laws = fresh.iter().filter(|d| matches!(d.kind, DeclarationKind::Theorem { .. })).count();
    println!("lens declarations={} terms={} bytes={} new_definitions={} new_laws={} parent_identities={} full_parent_bytes={}", c.declarations.len(), c.term_table.len(), f.certificate_bytes().len(), fresh.len()-laws, laws, prior.declarations.len(), old.certificate_bytes().len());
}
