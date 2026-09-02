use crate::rustc_driver_adapter::{
    self, HirAnalysis, MirLowering, PrimaryAnalysis, RustcDriverError,
};
use rust2vir_internal::driver_protocol::{seal_request_value, DriverInputIdentity, DriverRequest};
use rust2vir_internal::file_loader::SnapshotFileLoader;
use rust2vir_internal::json::{self, JsonValue};
use rust2vir_internal::sha256::{digest, hex};
use std::collections::BTreeMap;
use std::fs;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

const VECTOR: &[u8] = include_bytes!("../../testdata/rust-driver-v1.json");
static NEXT_TEMP: AtomicU64 = AtomicU64::new(0);

#[allow(dead_code)]
pub fn analyze(source: &[u8], function: &str) -> Result<HirAnalysis, RustcDriverError> {
    let root = std::env::temp_dir().join(format!(
        "rust2vir-hir-check-{}-{}",
        std::process::id(),
        NEXT_TEMP.fetch_add(1, Ordering::Relaxed)
    ));
    let source_path = root.join("src/lib.rs");
    let output_path = root.join("target");
    fs::create_dir_all(source_path.parent().expect("source parent")).expect("create source tree");
    fs::create_dir(&output_path).expect("create output directory");
    fs::write(&source_path, source).expect("write source fixture");
    let source_inventory = [DriverInputIdentity {
        kind: "source".to_owned(),
        normalized_path: "src/lib.rs".to_owned(),
        size_bytes: source.len() as u64,
        sha256: hex(&digest(source)),
    }];
    let loader = match SnapshotFileLoader::open(&root, "src/lib.rs", &source_inventory) {
        Ok(loader) => loader,
        Err(error) => {
            fs::remove_dir_all(root).expect("remove rejected source fixture");
            return Err(RustcDriverError::Source(error));
        }
    };
    let result = rustc_driver_adapter::analyze_hir_primary(
        &compiler_arguments(&source_path, &output_path),
        &request(function, &source_inventory, &[]),
        Arc::new(loader),
    );
    fs::remove_dir_all(root).expect("remove source fixture");
    result
}

#[allow(dead_code)]
pub fn analyze_contracts(
    source: &[u8],
    function: &str,
    contracts: &[(&str, &[u8])],
) -> Result<PrimaryAnalysis, RustcDriverError> {
    let root = std::env::temp_dir().join(format!(
        "rust2vir-contract-check-{}-{}",
        std::process::id(),
        NEXT_TEMP.fetch_add(1, Ordering::Relaxed)
    ));
    let source_path = root.join("src/lib.rs");
    let output_path = root.join("target");
    fs::create_dir_all(source_path.parent().expect("source parent")).expect("create source tree");
    fs::create_dir(&output_path).expect("create output directory");
    fs::write(&source_path, source).expect("write source fixture");

    let source_inventory = [DriverInputIdentity {
        kind: "source".to_owned(),
        normalized_path: "src/lib.rs".to_owned(),
        size_bytes: source.len() as u64,
        sha256: hex(&digest(source)),
    }];
    let mut contract_inventory = Vec::with_capacity(contracts.len());
    for (path, bytes) in contracts {
        let contract_path = root.join(path);
        fs::create_dir_all(contract_path.parent().expect("contract parent"))
            .expect("create contract directory");
        fs::write(&contract_path, bytes).expect("write contract fixture");
        contract_inventory.push(DriverInputIdentity {
            kind: "contract".to_owned(),
            normalized_path: (*path).to_owned(),
            size_bytes: bytes.len() as u64,
            sha256: hex(&digest(bytes)),
        });
    }
    let loader = match SnapshotFileLoader::open_with_contracts(
        &root,
        "src/lib.rs",
        &source_inventory,
        &contract_inventory,
    ) {
        Ok(loader) => loader,
        Err(error) => {
            fs::remove_dir_all(root).expect("remove rejected contract fixture");
            return Err(RustcDriverError::Source(error));
        }
    };
    let result = rustc_driver_adapter::analyze_primary(
        &compiler_arguments(&source_path, &output_path),
        &request(function, &source_inventory, &contract_inventory),
        Arc::new(loader),
    );
    fs::remove_dir_all(root).expect("remove contract fixture");
    result
}

#[allow(dead_code)]
pub fn lower(
    source: &[u8],
    function: &str,
    contracts: &[(&str, &[u8])],
) -> Result<MirLowering, RustcDriverError> {
    lower_for_target(source, function, contracts, "x86_64-unknown-linux-gnu", 64)
}

#[allow(dead_code)]
pub fn lower_for_target(
    source: &[u8],
    function: &str,
    contracts: &[(&str, &[u8])],
    target: &str,
    pointer_width: u8,
) -> Result<MirLowering, RustcDriverError> {
    lower_with_session_target(source, function, contracts, target, target, pointer_width)
}

#[allow(dead_code)]
pub fn lower_with_session_target(
    source: &[u8],
    function: &str,
    contracts: &[(&str, &[u8])],
    compiler_target: &str,
    request_target: &str,
    pointer_width: u8,
) -> Result<MirLowering, RustcDriverError> {
    let root = std::env::temp_dir().join(format!(
        "rust2vir-mir-lower-{}-{}",
        std::process::id(),
        NEXT_TEMP.fetch_add(1, Ordering::Relaxed)
    ));
    let source_path = root.join("src/lib.rs");
    let output_path = root.join("target");
    fs::create_dir_all(source_path.parent().expect("source parent")).expect("create source tree");
    fs::create_dir(&output_path).expect("create output directory");
    fs::write(&source_path, source).expect("write source fixture");

    let source_inventory = [DriverInputIdentity {
        kind: "source".to_owned(),
        normalized_path: "src/lib.rs".to_owned(),
        size_bytes: source.len() as u64,
        sha256: hex(&digest(source)),
    }];
    let mut contract_inventory = Vec::with_capacity(contracts.len());
    for (path, bytes) in contracts {
        let contract_path = root.join(path);
        fs::create_dir_all(contract_path.parent().expect("contract parent"))
            .expect("create contract directory");
        fs::write(&contract_path, bytes).expect("write contract fixture");
        contract_inventory.push(DriverInputIdentity {
            kind: "contract".to_owned(),
            normalized_path: (*path).to_owned(),
            size_bytes: bytes.len() as u64,
            sha256: hex(&digest(bytes)),
        });
    }
    let loader = match SnapshotFileLoader::open_with_contracts(
        &root,
        "src/lib.rs",
        &source_inventory,
        &contract_inventory,
    ) {
        Ok(loader) => loader,
        Err(error) => {
            fs::remove_dir_all(root).expect("remove rejected lowering fixture");
            return Err(RustcDriverError::Source(error));
        }
    };
    let result = rustc_driver_adapter::lower_primary(
        &compiler_arguments_for_target(&source_path, &output_path, compiler_target),
        &request_for_target(
            function,
            request_target,
            pointer_width,
            &source_inventory,
            &contract_inventory,
        ),
        Arc::new(loader),
    );
    fs::remove_dir_all(root).expect("remove lowering fixture");
    result
}

fn compiler_arguments(source: &std::path::Path, output: &std::path::Path) -> Vec<String> {
    compiler_arguments_for_target(source, output, "x86_64-unknown-linux-gnu")
}

fn compiler_arguments_for_target(
    source: &std::path::Path,
    output: &std::path::Path,
    target: &str,
) -> Vec<String> {
    [
        "/mpk/toolchain/bin/rustc".to_owned(),
        "--crate-name".to_owned(),
        "vector".to_owned(),
        "--edition=2021".to_owned(),
        source.to_str().expect("UTF-8 source path").to_owned(),
        "--crate-type".to_owned(),
        "lib".to_owned(),
        "--emit=metadata".to_owned(),
        "--out-dir".to_owned(),
        output.to_str().expect("UTF-8 output path").to_owned(),
        "--target".to_owned(),
        target.to_owned(),
        "-C".to_owned(),
        "overflow-checks=yes".to_owned(),
        "-C".to_owned(),
        "panic=abort".to_owned(),
        "-C".to_owned(),
        "debug-assertions=no".to_owned(),
        "-C".to_owned(),
        "opt-level=0".to_owned(),
        "-Z".to_owned(),
        "mir-opt-level=0".to_owned(),
        "--remap-path-prefix=/mpk/input=.".to_owned(),
    ]
    .into_iter()
    .collect()
}

fn request(
    function: &str,
    source_inventory: &[DriverInputIdentity],
    contract_inventory: &[DriverInputIdentity],
) -> DriverRequest {
    request_for_target(
        function,
        "x86_64-unknown-linux-gnu",
        64,
        source_inventory,
        contract_inventory,
    )
}

fn request_for_target(
    function: &str,
    target: &str,
    pointer_width: u8,
    source_inventory: &[DriverInputIdentity],
    contract_inventory: &[DriverInputIdentity],
) -> DriverRequest {
    let vector = json::parse(VECTOR, VECTOR.len()).expect("parse driver vector");
    let fixture = vector.as_object().expect("vector object")["valid_request"]
        .as_object()
        .expect("valid request object");
    let mut value = fixture["value"].clone();
    let root = value.as_object_mut().expect("request object");
    let context = root
        .get_mut("semantic_context")
        .expect("semantic context")
        .as_object_mut()
        .expect("semantic context object");
    let parameters = context
        .get_mut("semantic_parameters")
        .expect("semantic parameters envelope")
        .as_object_mut()
        .expect("semantic parameters envelope object")
        .get_mut("value")
        .expect("semantic parameters value")
        .as_object_mut()
        .expect("semantic parameters object");
    parameters.insert("target_id".to_owned(), JsonValue::String(target.to_owned()));
    parameters.insert(
        "pointer_width".to_owned(),
        JsonValue::Number(pointer_width.to_string()),
    );
    root.get_mut("compiler")
        .expect("compiler")
        .as_object_mut()
        .expect("compiler object")
        .insert("target".to_owned(), JsonValue::String(target.to_owned()));
    root.get_mut("selection")
        .expect("selection envelope")
        .as_object_mut()
        .expect("selection envelope object")
        .get_mut("value")
        .expect("selection value")
        .as_object_mut()
        .expect("selection object")
        .insert(
            "function".to_owned(),
            JsonValue::String(function.to_owned()),
        );
    let retained = root["inputs"]
        .as_array()
        .expect("request inputs")
        .iter()
        .filter(|input| {
            matches!(
                input
                    .as_object()
                    .and_then(|object| object.get("kind"))
                    .and_then(JsonValue::as_str),
                Some("lockfile" | "build_manifest")
            )
        })
        .cloned();
    let mut inputs = retained
        .chain(contract_inventory.iter().map(input_value))
        .chain(source_inventory.iter().map(input_value))
        .collect::<Vec<_>>();
    inputs.sort_by(|left, right| {
        input_path(left)
            .as_bytes()
            .cmp(input_path(right).as_bytes())
    });
    let mut sources = source_inventory
        .iter()
        .map(source_value)
        .collect::<Vec<_>>();
    sources.sort_by(|left, right| {
        input_path(left)
            .as_bytes()
            .cmp(input_path(right).as_bytes())
    });
    root.insert("inputs".to_owned(), JsonValue::Array(inputs));
    root.insert("source_inventory".to_owned(), JsonValue::Array(sources));
    seal_request_value(value).expect("seal request transport")
}

fn input_path(value: &JsonValue) -> &str {
    value
        .as_object()
        .and_then(|object| object.get("normalized_path"))
        .and_then(JsonValue::as_str)
        .expect("constructed input path")
}

fn input_value(input: &DriverInputIdentity) -> JsonValue {
    JsonValue::Object(BTreeMap::from([
        ("kind".to_owned(), JsonValue::String(input.kind.clone())),
        (
            "normalized_path".to_owned(),
            JsonValue::String(input.normalized_path.clone()),
        ),
        ("sha256".to_owned(), JsonValue::String(input.sha256.clone())),
        (
            "size_bytes".to_owned(),
            JsonValue::Number(input.size_bytes.to_string()),
        ),
    ]))
}

fn source_value(input: &DriverInputIdentity) -> JsonValue {
    JsonValue::Object(BTreeMap::from([
        (
            "normalized_path".to_owned(),
            JsonValue::String(input.normalized_path.clone()),
        ),
        ("sha256".to_owned(), JsonValue::String(input.sha256.clone())),
        (
            "size_bytes".to_owned(),
            JsonValue::Number(input.size_bytes.to_string()),
        ),
    ]))
}
