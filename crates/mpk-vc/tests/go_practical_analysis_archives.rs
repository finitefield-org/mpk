use mpk_vc::go_practical_type_keys::{
    check_go_analysis_vendor_archive_bytes, parse_go_analysis_vendor_manifest,
    GoAnalysisVendorArchiveError, GoAnalysisVendorArchiveInputs, GoTypeSourceBuffer,
};
use serde_json::Value;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

fn evidence() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../develop/migrations/go-practical/t02-w01/verification/analysis-archive/native")
}

fn source(row: &Value, root: &Path) -> GoTypeSourceBuffer {
    let raw = std::fs::read(
        root.join("original-bytes")
            .join(row["raw_file"].as_str().unwrap()),
    )
    .unwrap();
    GoTypeSourceBuffer::import(&serde_json::to_vec(&row["object"]).unwrap(), &raw).unwrap()
}

fn inputs() -> (
    Value,
    GoTypeSourceBuffer,
    Vec<GoTypeSourceBuffer>,
    Vec<GoTypeSourceBuffer>,
) {
    let root = evidence();
    let packet: Value =
        serde_json::from_slice(&std::fs::read(root.join("native.json")).unwrap()).unwrap();
    let mut sources = BTreeMap::new();
    for row in packet["snapshot_sources"].as_array().unwrap() {
        let original = source(row, &root);
        assert!(sources
            .insert(original.object().id.clone(), original)
            .is_none());
    }
    let manifest = sources[packet["manifest_source_id"].as_str().unwrap()].clone();
    let archives = packet["archives"]
        .as_array()
        .unwrap()
        .iter()
        .map(|row| sources[row["source_id"].as_str().unwrap()].clone())
        .collect();
    let vendor = packet["vendor_source_ids"]
        .as_array()
        .unwrap()
        .iter()
        .map(|id| sources[id.as_str().unwrap()].clone())
        .collect();
    (packet, manifest, archives, vendor)
}

fn check(
    manifest: GoTypeSourceBuffer,
    archives: Vec<GoTypeSourceBuffer>,
    vendor: Vec<GoTypeSourceBuffer>,
) -> Result<GoAnalysisVendorArchiveInputs, GoAnalysisVendorArchiveError> {
    let manifest = parse_go_analysis_vendor_manifest(manifest).unwrap();
    check_go_analysis_vendor_archive_bytes(manifest, archives, vendor)
}

fn replace(vendor: &mut [GoTypeSourceBuffer], path: &str, change: impl FnOnce(&[u8]) -> Vec<u8>) {
    let source = vendor
        .iter_mut()
        .find(|source| source.object().path == path)
        .unwrap();
    *source = GoTypeSourceBuffer::new(
        &source.object().capability,
        &source.object().path,
        &source.object().kind,
        &change(source.bytes()),
    )
    .unwrap();
}

#[test]
fn complete_original_archives_and_vendor_bytes_match_native_go() {
    let (packet, manifest, archives, vendor) = inputs();
    assert_eq!(vendor.len(), 133);
    assert_eq!(archives.len(), 3);
    let result = check(manifest.clone(), archives.clone(), vendor.clone()).unwrap();
    assert_eq!(result.manifest().package_paths().len(), 29);
    assert_eq!(result.supplied_files().len(), 133);
    assert_eq!(result.differences().len(), 1);
    assert!(result.differences()[0].path.ends_with("/go/ssa/TODO"));
    let mut total = 0;
    for native in packet["archives"].as_array().unwrap() {
        let archive = &result.archives()[native["module"].as_str().unwrap()];
        assert_eq!(archive.checksum(), native["checksum"].as_str().unwrap());
        assert_eq!(
            archive.namespace_entries(),
            native["namespace_entries"].as_u64().unwrap() as usize
        );
        let members = native["members"].as_array().unwrap();
        assert_eq!(archive.members().len(), members.len());
        for member in members {
            let observed = &archive.members()[member["Path"].as_str().unwrap()];
            assert_eq!(observed.bytes, member["Bytes"].as_u64().unwrap());
            assert_eq!(observed.raw_sha256, member["RawSHA256"].as_str().unwrap());
        }
        total += members.len();
    }
    assert_eq!(total, 1751);

    let changed = source(&packet["changed_vendor_source"], &evidence());
    let changed_zip = source(&packet["changed_archive_source"], &evidence());
    for control in packet["controls"].as_array().unwrap() {
        let name = control["name"].as_str().unwrap();
        let mut files = vendor.clone();
        let mut zips = archives.clone();
        match name {
            "changed-vendor-go" => {
                let index = files
                    .iter()
                    .position(|source| source.object().path == changed.object().path)
                    .unwrap();
                files[index] = changed.clone();
            }
            "missing-only-package-file" => files.retain(|source| {
                source.object().path != "vendor/golang.org/x/sync/errgroup/errgroup.go"
            }),
            "unknown-asset" => files.push(
                GoTypeSourceBuffer::new(
                    "analysis-frontend",
                    "vendor/golang.org/x/tools/go/packages/caller.data",
                    "auxiliary",
                    b"caller",
                )
                .unwrap(),
            ),
            "omitted-one-of-several-go-files" => files.retain(|source| {
                source.object().path != "vendor/golang.org/x/tools/go/packages/doc.go"
            }),
            "changed-todo" => replace(&mut files, "vendor/golang.org/x/tools/go/ssa/TODO", |raw| {
                [raw, b"\nextra\n"].concat()
            }),
            "changed-archive" => {
                let index = zips
                    .iter()
                    .position(|source| source.object().path == changed_zip.object().path)
                    .unwrap();
                zips[index] = changed_zip.clone();
            }
            _ => panic!("unexpected native control: {name}"),
        }
        let result = check(manifest.clone(), zips, files);
        assert_eq!(
            result.is_ok(),
            control["accepted"].as_bool().unwrap(),
            "{name}: {result:?}"
        );
        if name == "omitted-one-of-several-go-files" {
            assert_eq!(
                result.as_ref().unwrap().supplied_files().len(),
                132,
                "an untrusted supplied list does not prove a complete namespace"
            );
        }
        let error = match name {
            "changed-vendor-go" => Some(GoAnalysisVendorArchiveError::Content),
            "missing-only-package-file" => Some(GoAnalysisVendorArchiveError::MissingPackage),
            "unknown-asset" => Some(GoAnalysisVendorArchiveError::Source),
            "changed-archive" => Some(GoAnalysisVendorArchiveError::Archive),
            _ => None,
        };
        if let Some(error) = error {
            assert_eq!(result.as_ref().unwrap_err(), &error, "{name}");
        }
    }
}

#[test]
fn original_owners_reject_duplicates_and_accept_original_nested_roots() {
    let (_, manifest, archives, vendor) = inputs();
    assert_eq!(
        check(manifest.clone(), archives[..2].to_vec(), vendor.clone()).unwrap_err(),
        GoAnalysisVendorArchiveError::Archive,
        "a missing original archive is not a resource-limit failure"
    );
    let mut duplicate = vendor.clone();
    duplicate.push(vendor[0].clone());
    assert_eq!(
        check(manifest.clone(), archives.clone(), duplicate).unwrap_err(),
        GoAnalysisVendorArchiveError::Ownership
    );
    let nested = |source: &GoTypeSourceBuffer, root: &str| {
        GoTypeSourceBuffer::new(
            &source.object().capability,
            &format!("{root}/{}", source.object().path),
            &source.object().kind,
            source.bytes(),
        )
        .unwrap()
    };
    let nested_manifest = nested(&manifest, "frontend/subdir");
    let nested_vendor = vendor
        .iter()
        .map(|source| nested(source, "frontend/subdir"))
        .collect();
    let nested_archives = archives
        .iter()
        .map(|source| nested(source, "cache/subdir"))
        .collect();
    let nested = check(nested_manifest, nested_archives, nested_vendor).unwrap();
    assert_eq!(nested.supplied_files().len(), 133);
    assert_eq!(nested.differences().len(), 1);
}
