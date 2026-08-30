#![forbid(unsafe_code)]

use std::env;
use std::fs;
use std::path::Path;

fn main() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let registry_path = manifest_dir.join("../../release/bundles/bundle-registry.json");
    let semantic_registry_path =
        manifest_dir.join("../../release/bundles/semantic-profile-registry.json");
    println!("cargo:rerun-if-changed={}", registry_path.display());
    println!(
        "cargo:rerun-if-changed={}",
        semantic_registry_path.display()
    );

    let semantic_bytes = fs::read(&semantic_registry_path).unwrap_or_else(|error| {
        panic!(
            "read fixed semantic registry {}: {error}",
            semantic_registry_path.display()
        )
    });
    let semantic = mpk_vc::semantic_profile_registry::validate_semantic_profile_registry(
        &semantic_bytes,
        mpk_vc::semantic_profile_registry::RegistryRevision::Revision2,
    )
    .unwrap_or_else(|error| {
        panic!(
            "validate fixed semantic registry {}: {error}",
            semantic_registry_path.display()
        )
    });

    let bytes = fs::read(&registry_path).unwrap_or_else(|error| {
        panic!(
            "read fixed release registry {}: {error}",
            registry_path.display()
        )
    });
    let registry =
        mpk_vc::release_bundle_v1::validate_successor_release_registry(&bytes, &semantic)
            .unwrap_or_else(|error| {
                panic!(
                    "validate fixed release registry {}: {error}",
                    registry_path.display()
                )
            });
    let digest = decode_sha256(registry.registry_sha256())
        .iter()
        .map(u8::to_string)
        .collect::<Vec<_>>()
        .join(",");
    let generated = format!(
        "pub const EXPECTED_REGISTRY_ID: &str = {:?};\n\
         pub const EXPECTED_REGISTRY_SHA256_HEX: &str = {:?};\n\
         pub const EXPECTED_REGISTRY_SHA256: [u8; 32] = [{digest}];\n\
         pub const EXPECTED_SEMANTIC_REGISTRY_SHA256: &str = {:?};\n",
        registry.registry().id,
        registry.registry_sha256(),
        semantic.identity().registry_sha256(),
    );
    let output = Path::new(&env::var_os("OUT_DIR").expect("Cargo supplies OUT_DIR"))
        .join("frontend_registry_constants.rs");
    fs::write(&output, generated)
        .unwrap_or_else(|error| panic!("write {}: {error}", output.display()));
}

fn decode_sha256(value: &str) -> [u8; 32] {
    assert_eq!(value.len(), 64, "validated SHA-256 width");
    let mut output = [0_u8; 32];
    for (index, pair) in value.as_bytes().chunks_exact(2).enumerate() {
        output[index] = (nibble(pair[0]) << 4) | nibble(pair[1]);
    }
    output
}

fn nibble(byte: u8) -> u8 {
    match byte {
        b'0'..=b'9' => byte - b'0',
        b'a'..=b'f' => byte - b'a' + 10,
        _ => panic!("validated SHA-256 is lowercase hexadecimal"),
    }
}
