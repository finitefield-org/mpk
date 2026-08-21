#![forbid(unsafe_code)]

use std::env;
use std::fs;
use std::path::Path;

fn main() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let registry_path = manifest_dir.join("../../release/bundles/bundle-registry.json");
    println!("cargo:rerun-if-changed={}", registry_path.display());

    let bytes = fs::read(&registry_path).unwrap_or_else(|error| {
        panic!(
            "read fixed release registry {}: {error}",
            registry_path.display()
        )
    });
    let constants = mpk_vc::registry_build_constants(&bytes).unwrap_or_else(|error| {
        panic!(
            "validate fixed release registry {}: {error}",
            registry_path.display()
        )
    });
    let digest = constants
        .registry_sha256
        .iter()
        .map(u8::to_string)
        .collect::<Vec<_>>()
        .join(",");
    let generated = format!(
        "pub const EXPECTED_REGISTRY_ID: &str = {:?};\n\
         pub const EXPECTED_REGISTRY_SHA256: [u8; 32] = [{digest}];\n",
        constants.id
    );
    let output = Path::new(&env::var_os("OUT_DIR").expect("Cargo supplies OUT_DIR"))
        .join("frontend_registry_constants.rs");
    fs::write(&output, generated)
        .unwrap_or_else(|error| panic!("write {}: {error}", output.display()));
}
