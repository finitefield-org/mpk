use mpk_vc::go_practical_type_keys::{
    parse_go_analysis_vendor_manifest, GoAnalysisVendorError, GoTypeSourceBuffer,
};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

fn evidence() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../develop/migrations/go-practical/t02-w01/verification/analysis-vendor/native")
}

#[test]
fn complete_original_vendor_and_native_boundary_controls_match_go() {
    let root = evidence();
    let rows: Value =
        serde_json::from_slice(&std::fs::read(root.join("native.json")).unwrap()).unwrap();
    let rows = rows.as_array().unwrap();
    assert!(rows.len() >= 36);
    let mut names = BTreeSet::new();
    for row in rows {
        let name = row["name"].as_str().unwrap();
        assert!(names.insert(name));
        let hash = row["sha256"].as_str().unwrap();
        assert_eq!(hash.len(), 64);
        let raw = std::fs::read(root.join("original-bytes").join(format!("{hash}.raw"))).unwrap();
        assert_eq!(raw.len(), row["bytes"].as_u64().unwrap() as usize, "{name}");
        assert_eq!(format!("{:x}", Sha256::digest(&raw)), hash, "{name}");
        let source = GoTypeSourceBuffer::new(
            "analysis-frontend",
            "vendor/modules.txt",
            row["kind"].as_str().unwrap(),
            &raw,
        )
        .unwrap();
        assert_eq!(
            source.object().id,
            row["source_id"].as_str().unwrap(),
            "{name}"
        );
        let expected = row["accepted"].as_bool().unwrap();
        let actual = parse_go_analysis_vendor_manifest(source.clone());
        assert_eq!(actual.is_ok(), expected, "{name}: {actual:?}");
        if let Ok(actual) = actual {
            assert_eq!(actual.original_source(), &source, "{name}");
            assert_eq!(actual.original_source().bytes(), raw, "{name}");
            let expected_paths = row["packages"]
                .as_array()
                .unwrap()
                .iter()
                .map(|path| path.as_str().unwrap().to_owned())
                .collect::<BTreeSet<_>>();
            assert_eq!(actual.package_paths(), &expected_paths, "{name}");
            if name == "complete-original-plus-maximum-packages" {
                assert_eq!(actual.package_paths().len(), 32_768);
            }
        } else if name == "complete-original-plus-one-over-maximum" {
            assert_eq!(actual.unwrap_err(), GoAnalysisVendorError::Limit);
        } else if name == "wrong-source-kind" {
            assert_eq!(actual.unwrap_err(), GoAnalysisVendorError::Source);
        }
    }
    for required in [
        "original-complete-vendor-manifest",
        "valid-path-a..b",
        "invalid-utf8",
        "complete-original-plus-maximum-packages",
        "complete-original-plus-one-over-maximum",
    ] {
        assert!(names.contains(required), "missing {required}");
    }
}
