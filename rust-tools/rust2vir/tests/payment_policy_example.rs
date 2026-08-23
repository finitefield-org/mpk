#![allow(internal_features)]
#![feature(rustc_private)]

extern crate rustc_abi;
extern crate rustc_ast;
extern crate rustc_driver;
extern crate rustc_hir;
extern crate rustc_index;
extern crate rustc_interface;
extern crate rustc_middle;
extern crate rustc_session;
extern crate rustc_span;
extern crate rustc_target;

#[path = "../src/rustc_driver.rs"]
mod rustc_driver_adapter;

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use rust2vir_internal::cli::{
    LowerRequest, ReleaseArguments, RustSelection, RustTarget, SEMANTIC_PROFILE,
};
use rust2vir_internal::driver_protocol::{
    construct_request, encode_lowered, parse_output_transport, DriverComponentIdentity,
    DriverReleaseIdentity, DriverStatus,
};
use rust2vir_internal::emit::success_envelope;
use rust2vir_internal::file_loader::SnapshotFileLoader;
use rust2vir_internal::json;
use rust2vir_internal::manifest;
use rust2vir_internal::module_closure;
use rust2vir_internal::preflight;
use rust2vir_internal::snapshot::Snapshot;
use rust2vir_internal::{EXPECTED_RUSTC_COMMIT, EXPECTED_RUSTC_RELEASE};

const UPDATE_ENV: &str = "MPK_UPDATE_RUST_PAYMENT_POLICY";
const SHA_FRONTEND: &str = "e25a3f125432b56e00d8c0474f1dc9ddfdb6ed1a48eadc9febea681a74d9444f";
const SHA_DRIVER: &str = "54c026dfc75a82f8aa602857c8acd83e9499b908b42b179c67653fd1b92f6bb8";
const SHA_TOOLCHAIN: &str = "cdaa0ae4d4f56da86f403d58799fd2298f078b043d8392311487315cbcc2c63f";
const REGISTRY_SHA256: &str = "226baa5e744f2966615a5fe03d6bfa0395db4b191e92bc099e63436fa9936aba";
const TARGET: RustTarget = RustTarget::X86_64UnknownLinuxGnu;

static NEXT_TEMP: AtomicU64 = AtomicU64::new(0);

#[derive(Clone, Copy)]
struct Case {
    id: &'static str,
    contracts: &'static [&'static str],
    artifact_names: [&'static str; 4],
}

const CASES: &[Case] = &[
    Case {
        id: "positive",
        contracts: &["contracts/helper.json", "contracts/selected.json"],
        artifact_names: [
            "frontend-envelope.json",
            "vir.json",
            "source-map.json",
            "source-manifest.frontend.json",
        ],
    },
    Case {
        id: "insufficient-precondition",
        contracts: &[
            "contracts/helper.json",
            "contracts/insufficient-precondition.json",
        ],
        artifact_names: [
            "insufficient-precondition.frontend-envelope.json",
            "insufficient-precondition.vir.json",
            "insufficient-precondition.source-map.json",
            "insufficient-precondition.source-manifest.frontend.json",
        ],
    },
];

#[derive(Clone, Debug, Eq, PartialEq)]
struct FrontendArtifacts {
    envelope: Vec<u8>,
    vir: Vec<u8>,
    source_map: Vec<u8>,
    source_manifest: Vec<u8>,
}

impl FrontendArtifacts {
    fn values(&self) -> [&[u8]; 4] {
        [
            &self.envelope,
            &self.vir,
            &self.source_map,
            &self.source_manifest,
        ]
    }
}

struct TempRun {
    root: PathBuf,
}

impl TempRun {
    fn new(case: &Case, run: usize) -> Self {
        let serial = NEXT_TEMP.fetch_add(1, Ordering::Relaxed);
        let root = std::env::temp_dir().join(format!(
            "rust2vir-payment-policy-{}-{run}-{}-{serial}",
            case.id,
            std::process::id()
        ));
        fs::create_dir(&root).expect("create payment-policy run root");
        fs::create_dir(root.join("private")).expect("create snapshot parent");
        fs::create_dir(root.join("target")).expect("create rustc output root");
        Self { root }
    }

    fn private(&self) -> PathBuf {
        self.root.join("private")
    }

    fn target(&self) -> PathBuf {
        self.root.join("target")
    }
}

impl Drop for TempRun {
    fn drop(&mut self) {
        if self.root.exists() {
            fs::remove_dir_all(&self.root).expect("remove payment-policy run root");
        }
    }
}

#[test]
fn payment_policy_frontend_fixtures_are_real_deterministic_and_path_independent() {
    let update = std::env::var_os(UPDATE_ENV).is_some();
    let artifact_root = example_root().join("artifacts");
    let generated = CASES
        .iter()
        .map(|case| {
            let first = compile_case(case, 0);
            let second = compile_case(case, 1);
            assert_eq!(first, second, "{} clean compiler runs", case.id);
            for bytes in first.values() {
                assert_no_path_leakage(case, bytes);
            }
            first
        })
        .collect::<Vec<_>>();

    assert_ne!(
        generated[0].vir, generated[1].vir,
        "the insufficient-precondition contract must change the lowered VIR"
    );
    assert_ne!(
        generated[0].source_manifest, generated[1].source_manifest,
        "the exact contract set must change the frontend source manifest"
    );

    for (case, artifacts) in CASES.iter().zip(&generated) {
        for (name, bytes) in case.artifact_names.into_iter().zip(artifacts.values()) {
            assert_fixture(&artifact_root.join(name), bytes, update);
        }
    }
}

fn compile_case(case: &Case, run: usize) -> FrontendArtifacts {
    let temporary = TempRun::new(case, run);
    let request = lower_request(case);
    let validated = manifest::validate(
        &request,
        preflight::run(&request).unwrap_or_else(|error| panic!("{} preflight: {error:?}", case.id)),
    )
    .unwrap_or_else(|error| panic!("{} manifest: {error:?}", case.id));
    let (closure, _) = module_closure::discover(validated)
        .unwrap_or_else(|error| panic!("{} module closure: {error:?}", case.id));
    let snapshot = Snapshot::create(&temporary.private(), &closure)
        .unwrap_or_else(|error| panic!("{} snapshot: {error:?}", case.id));
    let private_request = construct_request(&request, snapshot.inputs(), &release_identity())
        .unwrap_or_else(|error| panic!("{} private request: {error:?}", case.id));
    let loader = SnapshotFileLoader::from_request(
        snapshot.path(),
        closure.library_root.as_str(),
        &private_request,
    )
    .unwrap_or_else(|error| panic!("{} source loader: {error:?}", case.id));
    let lowering = rustc_driver_adapter::lower_primary(
        &compiler_arguments(&snapshot, &closure.library_root, &temporary.target()),
        &private_request,
        Arc::new(loader),
    )
    .unwrap_or_else(|error| panic!("{} compiler lowering: {error:?}", case.id));
    let driver_transport = encode_lowered(
        &private_request,
        lowering.raw_lowering,
        lowering.raw_source_map,
    )
    .unwrap_or_else(|error| panic!("{} private output: {error:?}", case.id));
    let output = parse_output_transport(
        &driver_transport,
        &private_request,
        DriverStatus::Lowered.exit_code(),
        false,
    )
    .unwrap_or_else(|error| panic!("{} validate private output: {error:?}", case.id));
    let envelope = success_envelope(&request, &private_request, &output, false)
        .unwrap_or_else(|_| panic!("{} public envelope", case.id));
    let public = json::parse(&envelope[..envelope.len() - 1], envelope.len())
        .unwrap_or_else(|error| panic!("{} parse envelope: {error:?}", case.id));
    let root = public.as_object().expect("frontend envelope object");
    let ir = &root["ir"].as_object().expect("IR envelope")["value"];

    FrontendArtifacts {
        envelope,
        vir: json::canonical(ir).expect("canonical VIR"),
        source_map: json::canonical(&root["source_map"]).expect("canonical source map"),
        source_manifest: json::canonical(&root["source_manifest"])
            .expect("canonical source manifest"),
    }
}

fn lower_request(case: &Case) -> LowerRequest {
    LowerRequest {
        source_root: example_root(),
        selection: RustSelection {
            package: "payment-policy".to_owned(),
            crate_name: "payment_policy".to_owned(),
            kind: "lib",
            function: "payment_policy::approved_reserve_cents".to_owned(),
        },
        semantic_profile: SEMANTIC_PROFILE,
        target: TARGET,
        release: ReleaseArguments {
            frontend_bundle_id: "frontend.rust.rust2vir.candidate.v0".to_owned(),
            frontend_sha256: SHA_FRONTEND.to_owned(),
            release_registry_id: "mpk.release.registry.v0".to_owned(),
            release_registry_sha256: REGISTRY_SHA256.to_owned(),
            toolchain_bundle_id: "toolchain.rust.nightly-2025-06-01.candidate.v0".to_owned(),
            toolchain_root: PathBuf::from("/not-emitted/toolchain"),
            toolchain_distribution_sha256: SHA_TOOLCHAIN.to_owned(),
            driver: PathBuf::from("/not-emitted/rust2vir-driver"),
            driver_sha256: SHA_DRIVER.to_owned(),
        },
        contracts: case
            .contracts
            .iter()
            .map(|path| (*path).to_owned())
            .collect(),
    }
}

fn release_identity() -> DriverReleaseIdentity {
    DriverReleaseIdentity {
        frontend_bundle_id: "frontend.rust.rust2vir.candidate.v0".to_owned(),
        frontend_binary_sha256: SHA_FRONTEND.to_owned(),
        driver_binary_sha256: SHA_DRIVER.to_owned(),
        toolchain_bundle_id: "toolchain.rust.nightly-2025-06-01.candidate.v0".to_owned(),
        toolchain_distribution_sha256: SHA_TOOLCHAIN.to_owned(),
        toolchain_components: vec![
            component(
                "executable",
                "cargo",
                EXPECTED_RUSTC_RELEASE,
                "4ab49080934031ce3b87b1a8792e685f99819e8a3f537f110a339d7331f1dcea",
                None,
            ),
            component(
                "content",
                "native-runtime",
                "nightly-2025-06-01",
                "0f448df12a3bb58ca6ab51fcee4c470b117ce7072a02b489ab214454f302a479",
                None,
            ),
            component(
                "content",
                "rust-compiler-runtime",
                "nightly-2025-06-01",
                "3f61be824744b3ad52281dbebaba6718c10ed6af9a82b936a02419b7f43f5693",
                None,
            ),
            component(
                "content",
                "rust-target-i686",
                "nightly-2025-06-01",
                "a1c72b8bdb5dd4d589f386fc0142adee3274ebcb104d69203ad1f4ce5600c5c9",
                None,
            ),
            component(
                "content",
                "rust-target-x86_64",
                "nightly-2025-06-01",
                "73019eb46832161dad2e55a17cc044ff4523441643e5bc1b1ab1c68408961956",
                None,
            ),
            component(
                "executable",
                "rustc",
                EXPECTED_RUSTC_RELEASE,
                "a7c2179d845e8f40305bace1657b903f10d149cc6d72b0c08ecef75487418922",
                Some(EXPECTED_RUSTC_COMMIT),
            ),
        ],
    }
}

fn component(
    kind: &str,
    name: &str,
    release: &str,
    sha256: &str,
    commit_hash: Option<&str>,
) -> DriverComponentIdentity {
    DriverComponentIdentity {
        kind: kind.to_owned(),
        name: name.to_owned(),
        release: release.to_owned(),
        sha256: sha256.to_owned(),
        commit_hash: commit_hash.map(str::to_owned),
    }
}

fn compiler_arguments(
    snapshot: &Snapshot,
    library_root: &rust2vir_internal::path::PortablePath,
    output: &Path,
) -> Vec<String> {
    let source = snapshot.input_path(library_root);
    [
        "/mpk/toolchain/bin/rustc".to_owned(),
        "--crate-name".to_owned(),
        "payment_policy".to_owned(),
        "--edition=2021".to_owned(),
        source.to_str().expect("UTF-8 snapshot source").to_owned(),
        "--crate-type".to_owned(),
        "lib".to_owned(),
        "--emit=metadata".to_owned(),
        "--out-dir".to_owned(),
        output.to_str().expect("UTF-8 output path").to_owned(),
        "--target".to_owned(),
        TARGET.id().to_owned(),
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

fn example_root() -> PathBuf {
    fs::canonicalize(
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../examples/rust-payment-policy"),
    )
    .expect("canonical payment-policy example root")
}

fn assert_fixture(path: &Path, actual: &[u8], update: bool) {
    if update {
        fs::create_dir_all(path.parent().expect("fixture parent"))
            .expect("create payment-policy artifact directory");
        fs::write(path, actual).unwrap_or_else(|error| panic!("write {}: {error}", path.display()));
    } else {
        let expected = fs::read(path).unwrap_or_else(|error| {
            panic!(
                "read {}: {error}; regenerate with {UPDATE_ENV}=1",
                path.display()
            )
        });
        assert_eq!(expected, actual, "{}", path.display());
    }
}

fn assert_no_path_leakage(case: &Case, bytes: &[u8]) {
    let text = std::str::from_utf8(bytes).expect("frontend artifact UTF-8");
    for forbidden in [
        "/root/",
        "/tmp/",
        "/mpk/input",
        "/mpk/toolchain",
        "/not-emitted/",
        "rust2vir-payment-policy-",
    ] {
        assert!(!text.contains(forbidden), "{} leaked {forbidden}", case.id);
    }
    for path in [
        example_root(),
        PathBuf::from(env!("CARGO_MANIFEST_DIR")),
        std::env::temp_dir(),
    ] {
        let forbidden = path.to_string_lossy();
        assert!(
            !text.contains(forbidden.as_ref()),
            "{} leaked host path {forbidden}",
            case.id
        );
    }
}
