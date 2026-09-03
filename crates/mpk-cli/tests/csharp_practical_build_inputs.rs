use serde_json::{json, Map, Value};
use sha2::{Digest, Sha256};
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::Command;

const DESCRIPTOR_PATH: &str = "develop/migrations/csharp-03/build-inputs/build-inputs.json";
const INVENTORY_PATH: &str = "develop/migrations/csharp-03/build-inputs/candidate-inventory.json";
const TOOLCHAIN_HASH: &str = "d4af1170b2813a5581bb0f60b65fd4e7509576093045557b88689bf7e0876b4f";
const REFERENCE_HASH: &str = "30623f64b7d85564260e62464e652bfaa89eb56e0e55193989bfb99538ba6cad";
const PROJECT_FILES_HASH: &str = "4193dc64e338730e67128010e0f17160305a51ed4ba2d4b0df13aad65d7fc443";

fn repository_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn bytes(relative: &str) -> Vec<u8> {
    fs::read(repository_root().join(relative))
        .unwrap_or_else(|error| panic!("failed to read {relative}: {error}"))
}

fn load(relative: &str) -> Value {
    serde_json::from_slice(&bytes(relative))
        .unwrap_or_else(|error| panic!("failed to parse {relative}: {error}"))
}

fn object(value: &Value) -> &Map<String, Value> {
    value.as_object().expect("JSON object")
}

fn array(value: &Value) -> &[Value] {
    value.as_array().expect("JSON array")
}

fn text(value: &Value) -> &str {
    value.as_str().expect("JSON string")
}

fn integer(value: &Value) -> u64 {
    value.as_u64().expect("nonnegative JSON integer")
}

fn sha256(input: &[u8]) -> String {
    format!("{:x}", Sha256::digest(input))
}

fn git_output(arguments: &[&str]) -> Vec<u8> {
    let output = Command::new("/usr/bin/git")
        .args(arguments)
        .current_dir(repository_root())
        .output()
        .expect("run read-only Git query");
    assert!(
        output.status.success(),
        "git {:?}: {}",
        arguments,
        String::from_utf8_lossy(&output.stderr)
    );
    output.stdout
}

fn canonical_bytes(value: &Value) -> Vec<u8> {
    serde_json::to_vec(value).expect("serialize canonical JSON")
}

fn canonical_sha256(value: &Value) -> String {
    sha256(&canonical_bytes(value))
}

fn typed_sha256(domain: &[u8], value: &Value) -> String {
    let mut hasher = Sha256::new();
    hasher.update(domain);
    hasher.update(canonical_bytes(value));
    format!("{:x}", hasher.finalize())
}

fn assert_canonical_transport(relative: &str, value: &Value) {
    let mut expected = canonical_bytes(value);
    expected.push(b'\n');
    assert_eq!(bytes(relative), expected, "noncanonical JSON at {relative}");
}

fn assert_exact_keys(value: &Value, expected: &[&str]) {
    assert_eq!(
        object(value).keys().map(String::as_str).collect::<Vec<_>>(),
        expected,
        "closed object shape drift"
    );
}

fn assert_sha256(value: &Value) {
    let value = text(value);
    assert_eq!(value.len(), 64);
    assert!(value
        .bytes()
        .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte)));
}

// CSHARP-03-T01-W03
#[test]
fn private_descriptor_closes_the_exact_toolchain_build_and_environment() {
    let descriptor = load(DESCRIPTOR_PATH);
    assert_canonical_transport(DESCRIPTOR_PATH, &descriptor);
    assert_exact_keys(
        &descriptor,
        &[
            "baseline",
            "build_recipe",
            "candidate_inventory",
            "environment_closure",
            "forbidden_discovery",
            "notice_sources",
            "offline_extraction",
            "project_files",
            "project_root",
            "schema",
            "toolchain_inputs",
            "toolchain_inputs_sha256",
            "work_item",
        ],
    );
    assert_eq!(
        descriptor["schema"],
        "mpk.csharp_practical.t01_w03.private_build_inputs.v0"
    );
    assert_eq!(descriptor["work_item"], "CSHARP-03-T01-W03");
    assert_eq!(descriptor["toolchain_inputs_sha256"], TOOLCHAIN_HASH);
    assert_eq!(descriptor["project_root"], "csharp-tools/csharp2vir");
    assert_eq!(descriptor["candidate_inventory"], INVENTORY_PATH);
    assert_eq!(
        descriptor["baseline"],
        json!({
            "artifact_consumer_inventory": "develop/migrations/csharp-03/artifact-consumer-inventory.json",
            "raw_sha256": "6b5b7f601f6174d61496424084d264604a5a3325a460a5c0640bfcd71a564c49",
            "source_commit": "f84a5c6ff5122a3a5e64d9305fe999ed1f501f85",
            "source_tree": "c14885505d0eeb6901aa077dd6f497b2fc0a4d5d",
        })
    );
    assert_eq!(
        String::from_utf8(git_output(&[
            "show",
            "-s",
            "--format=%T",
            "f84a5c6ff5122a3a5e64d9305fe999ed1f501f85",
        ]))
        .unwrap()
        .trim(),
        "c14885505d0eeb6901aa077dd6f497b2fc0a4d5d"
    );
    assert_eq!(
        sha256(&git_output(&[
            "show",
            "f84a5c6ff5122a3a5e64d9305fe999ed1f501f85:develop/migrations/csharp-03/artifact-consumer-inventory.json",
        ])),
        "6b5b7f601f6174d61496424084d264604a5a3325a460a5c0640bfcd71a564c49"
    );

    let toolchain = &descriptor["toolchain_inputs"];
    let mut toolchain_preimage = toolchain.clone();
    object(&toolchain_preimage);
    toolchain_preimage
        .as_object_mut()
        .unwrap()
        .remove("toolchain_inputs_sha256");
    assert_eq!(canonical_bytes(&toolchain_preimage).len(), 29_335);
    assert_eq!(
        typed_sha256(b"MPK-CSHARP-TOOLCHAIN-INPUTS-0.1\0", &toolchain_preimage),
        TOOLCHAIN_HASH
    );
    assert_eq!(toolchain["toolchain_inputs_sha256"], TOOLCHAIN_HASH);
    assert_eq!(
        toolchain["host"],
        json!({
            "architecture": "x86_64",
            "execution_host_profile_id": "mpk.host.linux-x86_64-gnu.glibc2_27.cgroup2_tmpfs.v0",
            "interpreter": "/lib64/ld-linux-x86-64.so.2",
            "minimum_kernel_abi": "6.4.0",
            "native_library_roots": ["/mpk/toolchain/dotnet", "/lib/x86_64-linux-gnu"],
            "os": "linux",
            "rid": "linux-x64",
            "runtime_layout_profile_id": "mpk.runtime.linux-x86_64-gnu.glibc2_27.cgroup2_tmpfs.v0",
        })
    );
    assert_eq!(
        toolchain["roslyn_source"],
        json!({
            "commit": "c0573ed0a7dc3e3b4d2e70da47f97cc51a35524f",
            "release_kind": "stable",
            "repository": "https://github.com/dotnet/roslyn",
        })
    );

    let archives = array(&toolchain["archives"]);
    let expected_archives = [
        (
            "dotnet-runtime-linux-x64",
            "tar.gz",
            "10.0.11",
            36_639_275,
            "7d847ecaa123efae40b114c5d45641e456b4cd65e5114b4612095d45d7c71a63",
        ),
        (
            "dotnet-sdk-linux-x64",
            "tar.gz",
            "10.0.400",
            240_133_692,
            "7ad9d2db01512e41fd580a0630321bb70cd062d7fe4c5badfb4ce81ec1eddbb8",
        ),
        (
            "microsoft-codeanalysis-analyzers",
            "nupkg",
            "5.3.0",
            2_555_904,
            "7948d4ecc0de91c6c3431501bad7162bbb221f1e879ab81d0cc2e7a4661154b3",
        ),
        (
            "microsoft-codeanalysis-common",
            "nupkg",
            "5.6.0",
            6_654_563,
            "43edf870a0941cb31476280b4ae0ae9ea7290336a0ca7abb3ba2c9d3d10fbba8",
        ),
        (
            "microsoft-codeanalysis-csharp",
            "nupkg",
            "5.6.0",
            18_005_025,
            "9ebcc6664f682ee084be10d3f884be5c154eb8756bc0bf111a3d2b6453f8491a",
        ),
        (
            "microsoft-netcore-app-ref",
            "nupkg",
            "10.0.11",
            7_188_368,
            "e363ceebe508456156679650ccb88a6b6bc38e89264511c655137fc5c2a1d7a7",
        ),
    ];
    assert_eq!(archives.len(), expected_archives.len());
    for (archive, (id, kind, version, size, digest)) in archives.iter().zip(expected_archives) {
        assert_eq!(archive["id"], id);
        assert_eq!(archive["kind"], kind);
        assert_eq!(archive["version"], version);
        assert_eq!(archive["size_bytes"], size);
        assert_eq!(archive["sha256"], digest);
        assert!(text(&archive["url"]).starts_with("https://"));
    }
    assert_eq!(array(&toolchain["package_graph"]).len(), 4);
    assert_eq!(array(&toolchain["managed_projection"]).len(), 2);
    assert!(array(&toolchain["package_graph"])
        .iter()
        .any(
            |package| package["package_id"] == "Microsoft.CodeAnalysis.Analyzers"
                && package["use"] == "build-metadata-only"
        ));

    let references = &toolchain["reference_projection"];
    assert_eq!(references["count"], 167);
    assert_eq!(references["total_bytes"], 6_046_008);
    assert_eq!(references["canonical_payload_bytes"], 24_670);
    assert_eq!(references["inventory_sha256"], REFERENCE_HASH);
    assert_eq!(array(&references["inventory"]).len(), 167);
    assert_eq!(
        typed_sha256(
            b"MPK-CSHARP-REFERENCE-INVENTORY-0.1\0",
            &references["inventory"],
        ),
        REFERENCE_HASH
    );
    assert_eq!(array(&references["metadata"]).len(), 3);
    assert_eq!(
        array(&references["inventory"])
            .iter()
            .map(|record| integer(&record["size_bytes"]))
            .sum::<u64>(),
        6_046_008
    );

    assert_eq!(
        descriptor["build_recipe"],
        json!({
            "compiler": "sdk/10.0.400/Roslyn/bincore/csc.dll",
            "compiler_arguments": ["/nologo", "/noconfig", "/nostdlib+", "/deterministic+", "/optimize+", "/debug-", "/target:exe", "/platform:anycpu", "/langversion:14.0", "/nullable:enable", "/checked+", "/unsafe-", "/warnaserror+", "/utf8output", "/filealign:512", "/highentropyva+"],
            "id": "mpk.csharp.build_recipe.csc_direct.v0",
            "language_version": "14.0",
            "network_namespace": "required",
            "package_restore": "forbidden",
            "runtime_framework": "Microsoft.NETCore.App",
            "runtime_version": "10.0.11",
            "source_date_epoch": 0,
            "target_framework": "net10.0",
        })
    );
    assert!(!descriptor["build_recipe"].to_string().contains("analyzer"));
    assert_eq!(
        descriptor["forbidden_discovery"],
        json!({
            "ambient_references": "forbidden",
            "analyzers_at_compile": "forbidden",
            "nuget_package_cache_discovery": "forbidden",
            "project_evaluation": "forbidden",
            "reference_selection": "exact_embedded_reference_projection",
            "restore": "forbidden",
            "source_generators": "forbidden",
            "source_selection": "exact_project_files_manifest",
        })
    );

    let environment = &descriptor["environment_closure"];
    assert_eq!(
        environment["build_process"],
        "empty_then_exact_declared_variables"
    );
    assert_eq!(environment["unlisted_variables"], "ignored");
    assert_eq!(object(&environment["variables"]).len(), 20);
    assert_eq!(environment["variables"]["PATH"], "/nonexistent");
    assert_eq!(
        environment["variables"]["DOTNET_ROOT"],
        "$PINNED_DOTNET_ROOT"
    );
    assert_eq!(environment["variables"]["NUGET_PACKAGES"], "$EMPTY_NUGET");
    assert_eq!(environment["variables"]["SOURCE_DATE_EPOCH"], "0");
    assert_eq!(environment["variables"]["TZ"], "UTC");
    assert_eq!(
        environment["ambient_probe"],
        json!({
            "name": "MPK_CSHARP_PRACTICAL_UNLISTED_AMBIENT",
            "values": ["clean", "hostile"],
        })
    );

    assert_eq!(
        descriptor["offline_extraction"],
        json!({
            "archive_cache_file_count": 6,
            "archive_cache_file_mode": "0444",
            "case_collisions": "forbidden",
            "copied_archive_file_mode": "0444",
            "directory_mode": "0755",
            "limits": {
                "max_archive_bytes": 536_870_912,
                "max_archive_entries": 16_384,
                "max_extracted_bytes": 2_147_483_648_u64,
                "max_json_bytes": 67_108_864,
            },
            "network_after_cache_validation": "forbidden",
            "nupkg_regular_file_mode": "0644",
            "path_traversal": "forbidden",
            "symlinks": "forbidden",
            "tar_member_stats": [
                {"archive_id": "dotnet-runtime-linux-x64", "directory_0755": 7, "regular_0644": 177, "regular_0744": 0, "regular_0755": 16},
                {"archive_id": "dotnet-sdk-linux-x64", "directory_0755": 724, "regular_0644": 4075, "regular_0744": 811, "regular_0755": 21},
            ],
        })
    );
    assert_eq!(array(&descriptor["project_files"]).len(), 34);
    assert_eq!(
        canonical_sha256(&descriptor["project_files"]),
        PROJECT_FILES_HASH
    );
    for record in array(&descriptor["project_files"]) {
        let path = text(&record["path"]);
        let object =
            format!("f84a5c6ff5122a3a5e64d9305fe999ed1f501f85:csharp-tools/csharp2vir/{path}");
        let frozen = git_output(&["show", &object]);
        assert_eq!(
            integer(&record["size_bytes"]),
            frozen.len() as u64,
            "{path}"
        );
        assert_eq!(text(&record["sha256"]), sha256(&frozen), "{path}");
    }
    assert_eq!(array(&descriptor["notice_sources"]).len(), 13);
}

// CSHARP-03-T01-W03
#[test]
fn private_inventory_freezes_two_identical_builds_and_stays_unregistered() {
    let descriptor = load(DESCRIPTOR_PATH);
    let inventory = load(INVENTORY_PATH);
    assert_canonical_transport(INVENTORY_PATH, &inventory);
    assert_exact_keys(
        &inventory,
        &[
            "archive_layout",
            "archive_sha256",
            "archive_size_bytes",
            "build_count",
            "build_recipe_sha256",
            "candidate_file_count",
            "candidate_files_sha256",
            "descriptor_raw_sha256",
            "frontend_files",
            "notice_files",
            "project_files_sha256",
            "registration",
            "schema",
            "toolchain_inputs_sha256",
            "work_item",
        ],
    );
    assert_eq!(
        inventory["schema"],
        "mpk.csharp_practical.t01_w03.private_candidate_inventory.v0"
    );
    assert_eq!(inventory["work_item"], "CSHARP-03-T01-W03");
    assert_eq!(inventory["toolchain_inputs_sha256"], TOOLCHAIN_HASH);
    assert_eq!(inventory["build_count"], 2);
    assert_eq!(inventory["candidate_file_count"], 18);
    assert_eq!(inventory["archive_size_bytes"], 10_516_480);
    assert_eq!(
        inventory["archive_sha256"],
        "a26bc0ad42ed424812caf25b5b8d73df95e2ccefaa0442282ecb8399c440a302"
    );
    assert_eq!(
        inventory["archive_layout"],
        json!({
            "directory_mode": "0755",
            "file_mode": "0644",
            "format": "ustar",
            "gid": 0,
            "gname": "",
            "mtime": 0,
            "uid": 0,
            "uname": "",
        })
    );
    assert_eq!(
        inventory["registration"],
        json!({
            "active_registry_memberships": 0,
            "release_descriptor": "absent",
            "state": "private_unregistered",
        })
    );

    let descriptor_bytes = bytes(DESCRIPTOR_PATH);
    assert_eq!(
        sha256(&descriptor_bytes),
        text(&inventory["descriptor_raw_sha256"])
    );
    assert_eq!(
        canonical_sha256(&descriptor["build_recipe"]),
        text(&inventory["build_recipe_sha256"])
    );
    assert_eq!(
        canonical_sha256(&descriptor["project_files"]),
        text(&inventory["project_files_sha256"])
    );
    let candidate_records = array(&inventory["frontend_files"])
        .iter()
        .chain(array(&inventory["notice_files"]))
        .cloned()
        .collect::<Vec<_>>();
    assert_eq!(candidate_records.len(), 18);
    assert_eq!(
        canonical_sha256(&Value::Array(candidate_records.clone())),
        text(&inventory["candidate_files_sha256"])
    );
    let mut previous = "";
    for record in &candidate_records {
        assert_exact_keys(record, &["mode", "path", "sha256", "size_bytes"]);
        assert_eq!(record["mode"], "0644");
        assert!(integer(&record["size_bytes"]) > 0);
        assert_sha256(&record["sha256"]);
        let path = text(&record["path"]);
        assert!(path > previous, "candidate inventory is not path-sorted");
        previous = path;
    }
    assert_eq!(array(&inventory["frontend_files"]).len(), 5);
    assert_eq!(array(&inventory["notice_files"]).len(), 13);

    for projection in array(&descriptor["toolchain_inputs"]["managed_projection"]) {
        let runtime_path = text(&projection["runtime_path"]);
        let emitted = array(&inventory["frontend_files"])
            .iter()
            .find(|record| record["path"] == runtime_path)
            .expect("managed assembly is present in the candidate");
        assert_eq!(emitted["size_bytes"], projection["size_bytes"]);
        assert_eq!(emitted["sha256"], projection["sha256"]);
    }

    for active_file in [
        "release/bundles/semantic-profile-registry.json",
        "release/bundles/bundle-registry.json",
        "release/bundles/candidates/csharp.json",
        "release/build-inputs/csharp/build-inputs.json",
        "release/build-inputs/csharp/candidate-inventory.json",
    ] {
        let active = String::from_utf8(bytes(active_file)).expect("active UTF-8 document");
        assert!(!active.contains("mpk.csharp_practical"), "{active_file}");
        assert!(!active.contains("CSHARP-03-T01-W03"), "{active_file}");
    }
    assert_eq!(
        sha256(&bytes("release/build-inputs/csharp/build-inputs.json")),
        "0345044d16d4efb3568c32a3d7bc67fec508fe9359eff423a7f09c7f69b348dc"
    );
    assert_eq!(
        sha256(&bytes(
            "release/build-inputs/csharp/candidate-inventory.json"
        )),
        "4ff3ba6fdc2eb2857c32563b959f11194075a4264164cd7aebc808858e500e9b"
    );
    assert_eq!(
        sha256(&bytes("develop/specs/vectors/csharp-profile-v0.json")),
        "8109f781ca1f2b90ba02f786da09ba97602f4cd484b8835b561d5ecf4e7781c8"
    );
}

// CSHARP-03-T01-W03
#[test]
fn private_harness_rejects_mutations_and_strips_ambient_state() {
    let root = repository_root();
    let wrapper = root.join("scripts/build-csharp-practical-frontend.sh");
    let harness = root.join("scripts/csharp_practical_build_inputs.py");
    assert_eq!(
        fs::metadata(&wrapper).unwrap().permissions().mode() & 0o777,
        0o755
    );
    assert_eq!(
        fs::metadata(&harness).unwrap().permissions().mode() & 0o777,
        0o755
    );

    let wrapper_source = fs::read_to_string(&wrapper).expect("read wrapper");
    let harness_source = fs::read_to_string(&harness).expect("read harness");
    assert!(wrapper_source.contains("/usr/bin/env -i"));
    assert!(wrapper_source.contains("PYTHONDONTWRITEBYTECODE=1"));
    assert!(harness_source.contains(DESCRIPTOR_PATH));
    assert!(harness_source.contains(INVENTORY_PATH));
    assert!(!harness_source.contains("release/build-inputs/csharp/"));
    for mutation in [
        "one-byte",
        "file-count",
        "mode",
        "flag",
        "reference",
        "declared-environment",
        "notice-count",
        "restore-policy",
        "candidate-byte",
        "candidate-file-count",
        "candidate-mode",
        "archive-byte",
    ] {
        assert!(
            harness_source.contains(mutation),
            "missing mutation {mutation}"
        );
    }

    let output = Command::new(&wrapper)
        .arg("--self-test")
        .env_clear()
        .env("PATH", "/hostile/bin")
        .env("HOME", "/ambient-home-must-not-be-used")
        .env("DOTNET_ROOT", "/ambient-dotnet-must-not-be-used")
        .env("NUGET_PACKAGES", "/ambient-nuget-must-not-be-used")
        .env("HTTP_PROXY", "http://127.0.0.1:1")
        .env("HTTPS_PROXY", "http://127.0.0.1:1")
        .env("MPK_CSHARP_PRACTICAL_UNLISTED_AMBIENT", "hostile")
        .output()
        .expect("run private build-input mutation tests");
    assert!(
        output.status.success(),
        "stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output.stdout.is_empty());
    assert!(output.stderr.is_empty());

    let usage = Command::new(&wrapper)
        .arg("--unknown")
        .output()
        .expect("run closed wrapper usage path");
    assert_eq!(usage.status.code(), Some(64));
    assert!(usage.stdout.is_empty());
    assert_eq!(usage.stderr, b"CSHARP_PRACTICAL_BUILD_USAGE\n");
}
