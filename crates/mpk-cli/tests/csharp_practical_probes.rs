use serde_json::{json, Map, Value};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

const RESULT_PATH: &str = "develop/migrations/csharp-03/probes/roslyn-data-construction.json";
const RESULT_SIZE: usize = 5_925_271;
const RESULT_SHA256: &str = "c5de8bc209331c2295497210a570ba0be32e0871b3dd2576980d6c109222142e";
const SHAPE_IDS_SHA256: &str = "727b7203815631d83cdb8475a2ce8360061205318763ed36a09fce76628a57b2";
const ADMITTED_SHAPE_IDS_SHA256: &str =
    "fe3a7b166ac51e184249debc491532b71fa30a9d1a5723cc830da67a8792ff6e";
const REJECTED_SHAPE_IDS_SHA256: &str =
    "506ba206622d81aa61b5ee8973958fc2c68a4155cf64d047e0daec4bcc9fd346";
const PROBE_SOURCE_SHA256: &str =
    "e49a96c63ef1dc8548d54b5ad5cb6dd81ebb90b56fa7a27d54adfcb99c1d4657";
const W03_DESCRIPTOR_SHA256: &str =
    "83bf64dcbedce89f79613fe7aab3d95a92179122df54f9b5407273a245738015";
const W03_INVENTORY_SHA256: &str =
    "ff4b48790c67135144419c816149f8edfbd7b40ade231d6ab44c8433efef0cce";
const TOOLCHAIN_SHA256: &str = "d4af1170b2813a5581bb0f60b65fd4e7509576093045557b88689bf7e0876b4f";
const REFERENCE_SHA256: &str = "30623f64b7d85564260e62464e652bfaa89eb56e0e55193989bfb99538ba6cad";

fn repository_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn bytes(relative: &str) -> Vec<u8> {
    fs::read(repository_root().join(relative))
        .unwrap_or_else(|error| panic!("failed to read {relative}: {error}"))
}

fn load() -> Value {
    serde_json::from_slice(&bytes(RESULT_PATH)).expect("parse W04 probe result")
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

fn canonical_bytes(value: &Value) -> Vec<u8> {
    serde_json::to_vec(value).expect("serialize canonical JSON")
}

fn canonical_sha256(value: &Value) -> String {
    sha256(&canonical_bytes(value))
}

fn assert_exact_keys(value: &Value, expected: &[&str]) {
    assert_eq!(
        object(value).keys().map(String::as_str).collect::<Vec<_>>(),
        expected,
        "closed object shape drift"
    );
}

fn assert_sha256(value: &Value) {
    let digest = text(value);
    assert_eq!(digest.len(), 64);
    assert!(digest
        .bytes()
        .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte)));
}

fn target<'a>(document: &'a Value, shape_id: &str) -> &'a Value {
    for case in array(&document["observations"]["cases"]) {
        for candidate in array(&case["targets"]) {
            if candidate["shape_id"] == shape_id {
                return candidate;
            }
        }
    }
    panic!("missing target {shape_id}");
}

fn target_mut<'a>(document: &'a mut Value, shape_id: &str) -> &'a mut Value {
    for case in document["observations"]["cases"]
        .as_array_mut()
        .expect("cases")
    {
        for candidate in case["targets"].as_array_mut().expect("targets") {
            if candidate["shape_id"] == shape_id {
                return candidate;
            }
        }
    }
    panic!("missing mutable target {shape_id}");
}

fn validate_span(span: &Value, source_utf16_length: u64) -> Result<(), String> {
    let fields = object(span);
    if fields.keys().map(String::as_str).collect::<Vec<_>>() != ["end", "length", "start"] {
        return Err("span keys".to_owned());
    }
    let start = integer(&span["start"]);
    let end = integer(&span["end"]);
    let length = integer(&span["length"]);
    if end < start || end - start != length || end > source_utf16_length {
        return Err("span bounds".to_owned());
    }
    Ok(())
}

fn validate_document(document: &Value) -> Result<(), String> {
    let top_keys = object(document)
        .keys()
        .map(String::as_str)
        .collect::<Vec<_>>();
    if top_keys
        != [
            "baseline",
            "coverage",
            "measurement",
            "observations",
            "probe_input",
            "schema",
            "shape_index",
            "work_item",
        ]
    {
        return Err("top-level keys".to_owned());
    }
    if document["schema"] != "mpk.csharp_practical.t01_w04.roslyn_data_probe.v0"
        || document["work_item"] != "CSHARP-03-T01-W04"
    {
        return Err("document identity".to_owned());
    }
    if document["baseline"]
        != json!({
            "build_inputs": "develop/migrations/csharp-03/build-inputs/build-inputs.json",
            "build_inputs_raw_sha256": W03_DESCRIPTOR_SHA256,
            "candidate_inventory": "develop/migrations/csharp-03/build-inputs/candidate-inventory.json",
            "candidate_inventory_raw_sha256": W03_INVENTORY_SHA256,
            "source_commit": "4ad2cd480792d8e7cac71eb798e6b55b66bd97fb",
            "source_tree": "3ab99588482bfb3666088fa88dede679c748c17c"
        })
    {
        return Err("baseline".to_owned());
    }
    if document["probe_input"]
        != json!({
            "compiler_arguments": [
                "/nologo", "/noconfig", "/nostdlib+", "/deterministic+",
                "/optimize+", "/debug-", "/target:exe", "/platform:x64",
                "/langversion:14.0", "/nullable:enable", "/checked+", "/unsafe-",
                "/warnaserror+", "/utf8output", "/filealign:512", "/highentropyva+"
            ],
            "path": "develop/probes/csharp-03/DataConstructionProbe.cs",
            "raw_sha256": PROBE_SOURCE_SHA256,
            "reference_projection_sha256": REFERENCE_SHA256,
            "size_bytes": 80645,
            "toolchain_inputs_sha256": TOOLCHAIN_SHA256
        })
    {
        return Err("probe input".to_owned());
    }

    assert_exact_keys(
        &document["measurement"],
        &[
            "probe_binary_sha256",
            "raw_observation_sha256",
            "raw_observation_size_bytes",
        ],
    );
    assert_sha256(&document["measurement"]["probe_binary_sha256"]);
    assert_sha256(&document["measurement"]["raw_observation_sha256"]);
    let mut observation_bytes = canonical_bytes(&document["observations"]);
    observation_bytes.push(b'\n');
    if sha256(&observation_bytes) != document["measurement"]["raw_observation_sha256"]
        || observation_bytes.len() as u64
            != integer(&document["measurement"]["raw_observation_size_bytes"])
    {
        return Err("observation measurement".to_owned());
    }
    if document["observations"]["schema"] != "mpk.csharp_practical.t01_w04.roslyn_data_probe.raw.v0"
        || document["observations"]["work_item"] != "CSHARP-03-T01-W04"
        || document["observations"]["compiler"]
            != json!({
                "architecture": "X64",
                "language": "C#",
                "language_version": "14.0",
                "nullable_context": "Enable",
                "reference_count": 167,
                "roslyn_common": {
                    "culture": "neutral", "name": "Microsoft.CodeAnalysis",
                    "public_key_token": "31bf3856ad364e35", "version": "5.6.0.0"
                },
                "roslyn_csharp": {
                    "culture": "neutral", "name": "Microsoft.CodeAnalysis.CSharp",
                    "public_key_token": "31bf3856ad364e35", "version": "5.6.0.0"
                },
                "runtime_version": "10.0.11"
            })
    {
        return Err("compiler identity".to_owned());
    }

    let cases = array(&document["observations"]["cases"]);
    if cases.len() != 14 {
        return Err("case count".to_owned());
    }
    let mut case_ids = BTreeSet::new();
    let mut observed_targets = BTreeMap::new();
    let mut admitted_cases = 0usize;
    let mut rejected_success = false;
    let mut rejected_error = false;
    for case in cases {
        let expected_case_keys = [
            "compiler_outcome",
            "control_flow_graphs",
            "diagnostics",
            "disposition",
            "emitted_metadata",
            "id",
            "operation_roots",
            "semantic_nodes",
            "source",
            "source_types",
            "source_utf8_sha256",
            "syntax",
            "targets",
        ];
        if object(case).keys().map(String::as_str).collect::<Vec<_>>() != expected_case_keys {
            return Err("case keys".to_owned());
        }
        let case_id = text(&case["id"]);
        if !case_ids.insert(case_id) {
            return Err("duplicate case".to_owned());
        }
        let source = text(&case["source"]);
        if source.contains("Mpk.") || source.contains("MPK") {
            return Err("source dependency".to_owned());
        }
        if sha256(source.as_bytes()) != case["source_utf8_sha256"] {
            return Err("source hash".to_owned());
        }
        let source_utf16_length = source.encode_utf16().count() as u64;
        if array(&case["syntax"]).is_empty()
            || array(&case["semantic_nodes"]).is_empty()
            || array(&case["operation_roots"]).is_empty()
            || array(&case["control_flow_graphs"]).is_empty()
        {
            return Err("missing syntax/symbol/operation/cfg observation".to_owned());
        }
        for syntax in array(&case["syntax"]) {
            validate_span(&syntax["span"], source_utf16_length)?;
            validate_span(&syntax["full_span"], source_utf16_length)?;
        }
        let disposition = text(&case["disposition"]);
        match disposition {
            "admitted_shape" => {
                admitted_cases += 1;
                if case["compiler_outcome"] != "success" || !array(&case["diagnostics"]).is_empty()
                {
                    return Err("admitted diagnostic".to_owned());
                }
            }
            "rejected_near_miss" => match text(&case["compiler_outcome"]) {
                "success" => rejected_success = true,
                "error" => rejected_error = true,
                _ => return Err("compiler outcome".to_owned()),
            },
            _ => return Err("case disposition".to_owned()),
        }
        let marker_count = source.matches("/*@shape:").count();
        if marker_count != array(&case["targets"]).len() {
            return Err("marker/target mismatch".to_owned());
        }
        for probe_target in array(&case["targets"]) {
            let expected_target_keys = [
                "candidate_reason",
                "candidate_symbols",
                "conversion",
                "converted_type",
                "declared_symbol",
                "emitted_type",
                "enclosing_flow_root",
                "marker_span",
                "operation",
                "related_type_members",
                "shape_id",
                "symbol",
                "syntax",
                "type",
            ];
            if object(probe_target)
                .keys()
                .map(String::as_str)
                .collect::<Vec<_>>()
                != expected_target_keys
            {
                return Err("target keys".to_owned());
            }
            validate_span(&probe_target["marker_span"], source_utf16_length)?;
            validate_span(&probe_target["syntax"]["span"], source_utf16_length)?;
            let shape_id = text(&probe_target["shape_id"]);
            if observed_targets
                .insert(
                    shape_id,
                    (case_id, disposition, canonical_sha256(probe_target)),
                )
                .is_some()
            {
                return Err("duplicate target".to_owned());
            }
        }
    }
    if admitted_cases != 8 || !rejected_success || !rejected_error {
        return Err("case outcome coverage".to_owned());
    }

    let shape_index = array(&document["shape_index"]);
    if shape_index.len() != 181 {
        return Err("shape index count".to_owned());
    }
    let mut shape_ids = Vec::new();
    let mut admitted = Vec::new();
    let mut rejected = Vec::new();
    let mut mutation_ids = BTreeSet::new();
    for row in shape_index {
        let expected_index_keys = [
            "case_id",
            "disposition",
            "observation_sha256",
            "shape_id",
            "upgrade_mutation_id",
        ];
        if object(row).keys().map(String::as_str).collect::<Vec<_>>() != expected_index_keys {
            return Err("shape index keys".to_owned());
        }
        let shape_id = text(&row["shape_id"]);
        let Some((case_id, disposition, observation_hash)) = observed_targets.get(shape_id) else {
            return Err("shape target link".to_owned());
        };
        if row["case_id"] != *case_id
            || row["disposition"] != *disposition
            || row["observation_sha256"] != *observation_hash
        {
            return Err("observation hash or link".to_owned());
        }
        assert_sha256(&row["observation_sha256"]);
        shape_ids.push(shape_id);
        match *disposition {
            "admitted_shape" => {
                admitted.push(shape_id);
                let mutation = text(&row["upgrade_mutation_id"]);
                if !mutation_ids.insert(mutation) {
                    return Err("duplicate upgrade mutation".to_owned());
                }
            }
            "rejected_near_miss" => {
                rejected.push(shape_id);
                if !row["upgrade_mutation_id"].is_null() {
                    return Err("near miss upgrade mutation".to_owned());
                }
            }
            _ => return Err("shape disposition".to_owned()),
        }
    }
    if !shape_ids.windows(2).all(|pair| pair[0] < pair[1])
        || shape_ids.len() != observed_targets.len()
        || admitted.len() != 129
        || rejected.len() != 52
        || mutation_ids.len() != admitted.len()
    {
        return Err("shape catalog structure".to_owned());
    }
    let ids_json = Value::Array(shape_ids.iter().map(|id| json!(id)).collect());
    let admitted_json = Value::Array(admitted.iter().map(|id| json!(id)).collect());
    let rejected_json = Value::Array(rejected.iter().map(|id| json!(id)).collect());
    if canonical_sha256(&ids_json) != SHAPE_IDS_SHA256
        || canonical_sha256(&admitted_json) != ADMITTED_SHAPE_IDS_SHA256
        || canonical_sha256(&rejected_json) != REJECTED_SHAPE_IDS_SHA256
    {
        return Err("shape catalog fingerprint".to_owned());
    }

    let coverage = array(&document["coverage"]);
    let expected_requirements = [
        "arrays",
        "collection_calls",
        "compiler_owned_markers",
        "constructors_and_initializers",
        "conversions",
        "data_intrinsics",
        "declarations",
        "expression_bodies",
        "generic_metadata_boundary",
        "instance_calls_and_overloads",
        "nullable",
        "ordinary_using_and_directives",
        "strings",
        "var",
    ];
    if coverage
        .iter()
        .map(|row| text(&row["requirement"]))
        .collect::<Vec<_>>()
        != expected_requirements
        || coverage
            .iter()
            .any(|row| array(&row["shape_ids"]).is_empty())
    {
        return Err("requirement coverage".to_owned());
    }
    let covered = coverage
        .iter()
        .flat_map(|row| array(&row["shape_ids"]).iter().map(text))
        .collect::<BTreeSet<_>>();
    if covered != shape_ids.iter().copied().collect() {
        return Err("unowned shape".to_owned());
    }
    Ok(())
}

// CSHARP-03-T01-W04
#[test]
fn canonical_probe_closes_the_measured_data_and_construction_shapes() {
    let raw = bytes(RESULT_PATH);
    assert_eq!(raw.len(), RESULT_SIZE);
    assert_eq!(sha256(&raw), RESULT_SHA256);
    let document: Value = serde_json::from_slice(&raw).expect("strict-enough JSON parse");
    let mut canonical = canonical_bytes(&document);
    canonical.push(b'\n');
    assert_eq!(
        raw, canonical,
        "probe result must be canonical JSON plus LF"
    );
    validate_document(&document).unwrap();

    assert_eq!(
        target(&document, "nullable.directive.file_wide_enable")["syntax"]["kind"],
        "NullableDirectiveTrivia"
    );
    assert_eq!(
        target(&document, "using.namespace.compilation_unit")["syntax"]["kind"],
        "UsingDirective"
    );
    assert_eq!(
        target(&document, "nullable.value_shorthand")["type"]["display"],
        "int?"
    );
    assert_eq!(
        target(&document, "intrinsic.single.is_nan")["symbol"]["display"],
        "float.IsNaN(float)"
    );
    assert_eq!(
        target(&document, "string.concat.operator.string_char")["operation"]["kind"],
        "Binary"
    );
    assert!(array(
        &target(&document, "intrinsic.string.incidental_generic_metadata")["type"]["interfaces"]
    )
    .iter()
    .any(|interface| text(&interface["display"]).contains("IEnumerable<char>")));
    assert!(array(
        &target(&document, "intrinsic.array.incidental_generic_metadata")["type"]["interfaces"]
    )
    .iter()
    .any(|interface| text(&interface["display"]).contains("IList<int>")));

    let init = target(&document, "compiler_marker.init_modreq");
    let init_text = serde_json::to_string(init).unwrap();
    assert!(init_text.contains("System.Runtime.CompilerServices.IsExternalInit"));
    let required = target(&document, "compiler_marker.required_attributes");
    let required_text = serde_json::to_string(required).unwrap();
    assert!(required_text.contains("System.Runtime.CompilerServices.RequiredMemberAttribute"));
    assert!(
        required_text.contains("System.Runtime.CompilerServices.CompilerFeatureRequiredAttribute")
    );
    let backing = target(&document, "synthesized.auto_backing_field");
    assert!(array(&backing["related_type_members"])
        .iter()
        .any(|member| {
            member["kind"] == "Field"
                && member["is_implicit"] == true
                && text(&member["metadata_name"]).contains("k__BackingField")
        }));
}

// CSHARP-03-T01-W04
#[test]
fn every_admitted_shape_has_a_distinct_upgrade_mutation() {
    let document = load();
    validate_document(&document).unwrap();
    let mut mutation_ids = BTreeSet::new();
    for row in array(&document["shape_index"])
        .iter()
        .filter(|row| row["disposition"] == "admitted_shape")
    {
        let shape_id = text(&row["shape_id"]);
        assert!(mutation_ids.insert(text(&row["upgrade_mutation_id"])));
        let mut changed = target(&document, shape_id).clone();
        let raw_kind = integer(&changed["syntax"]["raw_kind"]);
        changed["syntax"]["raw_kind"] = json!(raw_kind + 1);
        assert_ne!(
            canonical_sha256(&changed),
            row["observation_sha256"],
            "upgrade mutation did not cross {shape_id}"
        );
    }
    assert_eq!(mutation_ids.len(), 129);
}

// CSHARP-03-T01-W04
#[test]
fn one_changed_observation_fails_the_probe_schema() {
    let mut changed = load();
    validate_document(&changed).unwrap();
    let shape_id = text(&changed["shape_index"][0]["shape_id"]).to_owned();
    let probe_target = target_mut(&mut changed, &shape_id);
    let raw_kind = integer(&probe_target["syntax"]["raw_kind"]);
    probe_target["syntax"]["raw_kind"] = json!(raw_kind + 1);
    assert!(validate_document(&changed).is_err());
}

// CSHARP-03-T01-W04
#[test]
fn private_probe_preserves_the_active_scalar_and_release_boundary() {
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
    for relative in [
        "release/bundles/semantic-profile-registry.json",
        "release/bundles/bundle-registry.json",
    ] {
        let content = String::from_utf8(bytes(relative)).unwrap();
        assert!(!content.contains("CSHARP-03"));
        assert!(!content.contains("mpk.csharp.practical"));
        assert!(!content.contains("roslyn_data_probe"));
    }
}

// CSHARP-03-T01-W04
#[test]
fn pinned_compiler_rerun_is_byte_identical_when_the_linux_cache_is_available() {
    if !cfg!(all(target_os = "linux", target_arch = "x86_64")) {
        return;
    }
    let archives = repository_root()
        .join("release/build-input-cache/csharp")
        .join(TOOLCHAIN_SHA256)
        .join("archives");
    let present = fs::read_dir(&archives)
        .map(|entries| entries.filter_map(Result::ok).count())
        .unwrap_or(0);
    assert!(present == 0 || present == 6, "partial C# archive cache");
    if present == 0 {
        return;
    }
    let output = Command::new(
        repository_root().join("develop/probes/csharp-03/run-data-construction-probe.sh"),
    )
    .arg("--check")
    .env_clear()
    .env("PATH", "/usr/bin:/bin")
    .output()
    .expect("execute pinned W04 Roslyn probe");
    assert!(
        output.status.success(),
        "stdout={} stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output.stdout.is_empty());
    assert!(output.stderr.is_empty());
}
