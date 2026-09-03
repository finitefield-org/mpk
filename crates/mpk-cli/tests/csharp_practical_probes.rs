use serde_json::{json, Map, Value};
use sha2::{Digest, Sha256};
use std::cmp::Reverse;
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
const CONTROL_RESULT_PATH: &str =
    "develop/migrations/csharp-03/probes/roslyn-control-exception-pattern.json";
const CONTROL_RESULT_SIZE: usize = 2_331_920;
const CONTROL_RESULT_SHA256: &str =
    "b1215ad7f4a0e08dc269834229d7158158d31c0e9475218fa0791feea5a1629a";
const CONTROL_PROBE_SOURCE_SHA256: &str =
    "f62ff3deb7c0fff2799f99426ab9dbd7e6fd373a5fd9d8ed91bbb118a9808f1f";
const CONTROL_SHAPE_IDS_SHA256: &str =
    "431e5891260b9e3284f6b3646ae25d4643d9d53c8fdede0db69a1d2fd5d2d501";
const CONTROL_ADMITTED_SHAPE_IDS_SHA256: &str =
    "524e05d67fa72c5520176711f06a42739f44d881ad30e2c0b31cfbc83f76864c";
const CONTROL_REJECTED_SHAPE_IDS_SHA256: &str =
    "b510c715a9d915a1217bccfbcc80611877c06a5bd8cf036bd1959db7012e0870";
const DEPENDENCY_RESULT_PATH: &str =
    "develop/migrations/csharp-03/probes/roslyn-dependency-generic-suspension.json";
const DEPENDENCY_RESULT_SIZE: usize = 4_511_101;
const DEPENDENCY_RESULT_SHA256: &str =
    "5dadf10613f95be9b35c108008a33474c55d222bef1be987c2614c6dcc48fe96";
const DEPENDENCY_PROBE_SOURCE_SHA256: &str =
    "7e2114bdb75ef5b78e330c24e04c551c7766740ba037a12419547212026c6db6";
const DEPENDENCY_SHAPE_IDS_SHA256: &str =
    "6f7cb87aa1efae91b220244b5b85cac5d13e9995b8b93539bc04cc1925060446";
const DEPENDENCY_ADMITTED_SHAPE_IDS_SHA256: &str =
    "3529ba40edc421a2a19fe74eceaf825426063c4336da2b72c75cc4c06633d35c";
const DEPENDENCY_REJECTED_SHAPE_IDS_SHA256: &str =
    "4a72c24a0b06bb25e4e8b69dcd17695a253d754ee85b6599c636d9d944415ef4";
const DEPENDENCY_FAMILY_IDS_SHA256: &str =
    "407f67fc75f02b61d555834ade2f192e0db3e249f74f16b505291235bb7e93be";
const RUNTIME_RESULT_PATH: &str =
    "develop/migrations/csharp-03/probes/runtime-primitive-string-numeric-codec.json";
const RUNTIME_RESULT_SIZE: usize = 9_318_258;
const RUNTIME_RESULT_SHA256: &str =
    "0055835ce456fb9c438336332bc0e2a214d900c137eca34f90c3fcddd2688769";
const RUNTIME_PROBE_SOURCE_SHA256: &str =
    "d587acd6b1baab5602c8da8c54a803a9baa797400b70a6328bfd059e6a9f5f42";
const RUNTIME_VECTOR_IDS_SHA256: &str =
    "e4e2f9c55154bec304a66e80c5d574c071307ff91e4bd93b3a0073153905073c";
const RUNTIME_OPERATION_IDS_SHA256: &str =
    "96db56971b3cc908ac618880bf4d1993567d0217ea3a325c14deb4691277b3a5";
const RUNTIME_FAMILY_IDS_SHA256: &str =
    "802e897a25d358fce385ea9390da70a5c2cd5bb9a3d6f4dc5a419e5ee6e9da37";
const RUNTIME_CULTURE_VARIANT_IDS_SHA256: &str =
    "d17191a68f4d0e2e0596e309e4e945765f294f7e0a2a2a397e558fc66ae0c965";

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

fn load_control() -> Value {
    serde_json::from_slice(&bytes(CONTROL_RESULT_PATH)).expect("parse W05 probe result")
}

fn control_case<'a>(document: &'a Value, case_id: &str) -> &'a Value {
    array(&document["observations"]["cases"])
        .iter()
        .find(|case| case["id"] == case_id)
        .unwrap_or_else(|| panic!("missing control case {case_id}"))
}

fn control_target<'a>(document: &'a Value, shape_id: &str) -> &'a Value {
    for case in array(&document["observations"]["cases"]) {
        for candidate in array(&case["targets"]) {
            if candidate["shape_id"] == shape_id {
                return candidate;
            }
        }
    }
    panic!("missing control target {shape_id}");
}

fn control_observation<'a>(document: &'a Value, family: &str, observation_id: &str) -> &'a Value {
    let key = match family {
        "decision_graph" => "decision_graphs",
        "exception_region" => "exception_regions",
        _ => panic!("unknown observation family {family}"),
    };
    for case in array(&document["observations"]["cases"]) {
        for observation in array(&case[key]) {
            if observation["id"] == observation_id {
                return observation;
            }
        }
    }
    panic!("missing control observation {observation_id}");
}

fn control_observation_mut<'a>(
    document: &'a mut Value,
    family: &str,
    observation_id: &str,
) -> &'a mut Value {
    let key = match family {
        "decision_graph" => "decision_graphs",
        "exception_region" => "exception_regions",
        _ => panic!("unknown observation family {family}"),
    };
    for case in document["observations"]["cases"]
        .as_array_mut()
        .expect("control cases")
    {
        for observation in case[key].as_array_mut().expect("control observations") {
            if observation["id"] == observation_id {
                return observation;
            }
        }
    }
    panic!("missing control observation {observation_id}");
}

fn control_upgrade_mutation_field(
    family: &str,
    observation: &Value,
) -> Result<&'static str, String> {
    match family {
        "decision_graph" if !array(&observation["nodes"]).is_empty() => {
            Ok("nodes[0].operation_kind")
        }
        "decision_graph" => Err("empty decision graph mutation".to_owned()),
        "exception_region" if !array(&observation["catches"]).is_empty() => {
            if array(&observation["handler_search_order"]).is_empty() {
                Err("empty handler-search mutation".to_owned())
            } else {
                Ok("handler_search_order[0]")
            }
        }
        "exception_region" => Ok("nesting_depth"),
        _ => Err("unknown control mutation family".to_owned()),
    }
}

fn apply_control_upgrade_mutation(observation: &mut Value, mutation_field: &str) {
    match mutation_field {
        "nodes[0].operation_kind" => {
            let node = &mut observation["nodes"].as_array_mut().expect("decision nodes")[0];
            node["operation_kind"] = json!(format!(
                "{}#upgrade-mutation",
                text(&node["operation_kind"])
            ));
        }
        "handler_search_order[0]" => {
            let search = observation["handler_search_order"]
                .as_array_mut()
                .expect("handler search");
            search[0] = json!(integer(&search[0]) + 1);
        }
        "nesting_depth" => {
            observation["nesting_depth"] = json!(integer(&observation["nesting_depth"]) + 1);
        }
        _ => panic!("unknown control mutation field {mutation_field}"),
    }
}

fn collect_region_kinds(region: &Value, kinds: &mut BTreeSet<String>) {
    kinds.insert(text(&region["kind"]).to_owned());
    for nested in array(&region["nested"]) {
        collect_region_kinds(nested, kinds);
    }
}

fn validate_control_document(document: &Value) -> Result<(), String> {
    if object(document)
        .keys()
        .map(String::as_str)
        .collect::<Vec<_>>()
        != [
            "baseline",
            "coverage",
            "measurement",
            "observations",
            "probe_input",
            "schema",
            "shape_index",
            "upgrade_mutations",
            "work_item",
        ]
    {
        return Err("control top-level keys".to_owned());
    }
    if document["schema"] != "mpk.csharp_practical.t01_w05.roslyn_control_probe.v0"
        || document["work_item"] != "CSHARP-03-T01-W05"
    {
        return Err("control identity".to_owned());
    }
    if document["baseline"]
        != json!({
            "build_inputs": "develop/migrations/csharp-03/build-inputs/build-inputs.json",
            "build_inputs_raw_sha256": W03_DESCRIPTOR_SHA256,
            "candidate_inventory": "develop/migrations/csharp-03/build-inputs/candidate-inventory.json",
            "candidate_inventory_raw_sha256": W03_INVENTORY_SHA256,
            "data_probe": RESULT_PATH,
            "data_probe_raw_sha256": RESULT_SHA256,
            "source_commit": "b6680168c2666be503741575c009f0a26dd0da22",
            "source_tree": "0f1e86bbdf986870b60fe335da58290baac26b0f"
        })
    {
        return Err("control baseline".to_owned());
    }
    if document["probe_input"]
        != json!({
            "compiler_arguments": [
                "/nologo", "/noconfig", "/nostdlib+", "/deterministic+",
                "/optimize+", "/debug-", "/target:exe", "/platform:x64",
                "/langversion:14.0", "/nullable:enable", "/checked+", "/unsafe-",
                "/warnaserror+", "/utf8output", "/filealign:512", "/highentropyva+"
            ],
            "path": "develop/probes/csharp-03/ControlExceptionPatternProbe.cs",
            "raw_sha256": CONTROL_PROBE_SOURCE_SHA256,
            "reference_projection_sha256": REFERENCE_SHA256,
            "size_bytes": 70299,
            "toolchain_inputs_sha256": TOOLCHAIN_SHA256
        })
    {
        return Err("control probe input".to_owned());
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
        return Err("control observation measurement".to_owned());
    }
    if document["observations"]["schema"]
        != "mpk.csharp_practical.t01_w05.roslyn_control_probe.raw.v0"
        || document["observations"]["work_item"] != "CSHARP-03-T01-W05"
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
        return Err("control compiler identity".to_owned());
    }

    let cases = array(&document["observations"]["cases"]);
    if cases.len() != 18 {
        return Err("control case count".to_owned());
    }
    let mut case_ids = BTreeSet::new();
    let mut cases_by_id = BTreeMap::new();
    let mut targets = BTreeMap::new();
    let mut observations = BTreeMap::new();
    let mut rejected_clean = false;
    let mut rejected_warning = false;
    let mut rejected_error = false;
    for case in cases {
        if object(case).keys().map(String::as_str).collect::<Vec<_>>()
            != [
                "abrupt_completions",
                "compiler_outcome",
                "control_flow_graphs",
                "decision_graphs",
                "diagnostics",
                "disposition",
                "exception_regions",
                "id",
                "operation_roots",
                "source",
                "source_order",
                "source_utf8_sha256",
                "syntax",
                "targets",
            ]
        {
            return Err("control case keys".to_owned());
        }
        let case_id = text(&case["id"]);
        if !case_ids.insert(case_id) {
            return Err("duplicate control case".to_owned());
        }
        let disposition = text(&case["disposition"]);
        cases_by_id.insert(case_id, disposition);
        let source = text(&case["source"]);
        if source.contains("Mpk.")
            || source.contains("MPK")
            || sha256(source.as_bytes()) != case["source_utf8_sha256"]
        {
            return Err("control source identity".to_owned());
        }
        let source_utf16_length = source.encode_utf16().count() as u64;
        if array(&case["syntax"]).is_empty() || array(&case["operation_roots"]).is_empty() {
            return Err("control observations missing".to_owned());
        }
        let diagnostics = array(&case["diagnostics"]);
        let mut has_error_diagnostic = false;
        for diagnostic in diagnostics {
            if object(diagnostic)
                .keys()
                .map(String::as_str)
                .collect::<Vec<_>>()
                != [
                    "id",
                    "is_suppressed",
                    "location_kind",
                    "severity",
                    "span",
                    "warning_level",
                ]
            {
                return Err("control diagnostic keys".to_owned());
            }
            has_error_diagnostic |= diagnostic["severity"] == "Error";
        }
        if (case["compiler_outcome"] == "error") != has_error_diagnostic {
            return Err("control compiler outcome/diagnostic mismatch".to_owned());
        }
        if case["compiler_outcome"] == "success" {
            let cfg_eligible_roots = array(&case["operation_roots"])
                .iter()
                .filter(|root| {
                    matches!(
                        text(&root["kind"]),
                        "ConstructorBodyOperation" | "MethodBodyOperation"
                    )
                })
                .count();
            if array(&case["control_flow_graphs"]).len() != cfg_eligible_roots {
                return Err("successful control CFG closure".to_owned());
            }
        }
        match disposition {
            "admitted_shape" => {
                if case["compiler_outcome"] != "success"
                    || !diagnostics.is_empty()
                    || array(&case["control_flow_graphs"]).is_empty()
                {
                    return Err("admitted control diagnostics or CFG".to_owned());
                }
            }
            "rejected_near_miss" => match text(&case["compiler_outcome"]) {
                "success" if diagnostics.is_empty() => rejected_clean = true,
                "success" => rejected_warning = true,
                "error" => rejected_error = true,
                _ => return Err("control compiler outcome".to_owned()),
            },
            _ => return Err("control disposition".to_owned()),
        }
        for syntax in array(&case["syntax"]) {
            validate_span(&syntax["span"], source_utf16_length)?;
            validate_span(&syntax["full_span"], source_utf16_length)?;
        }

        let mut expected_source_order: Vec<(u64, String, String)> = Vec::new();
        let marker_count = source.matches("/*@shape:").count();
        if marker_count != array(&case["targets"]).len() {
            return Err("control marker/target mismatch".to_owned());
        }
        for target_value in array(&case["targets"]) {
            if object(target_value)
                .keys()
                .map(String::as_str)
                .collect::<Vec<_>>()
                != [
                    "candidate_reason",
                    "candidate_symbols",
                    "marker_span",
                    "operation",
                    "shape_id",
                    "source_ordinal",
                    "symbol",
                    "syntax",
                    "type",
                ]
            {
                return Err("control target keys".to_owned());
            }
            validate_span(&target_value["marker_span"], source_utf16_length)?;
            validate_span(&target_value["syntax"]["span"], source_utf16_length)?;
            let shape_id = text(&target_value["shape_id"]);
            if targets
                .insert(
                    shape_id,
                    (
                        case_id,
                        disposition,
                        integer(&target_value["source_ordinal"]),
                        canonical_sha256(target_value),
                    ),
                )
                .is_some()
            {
                return Err("duplicate control target".to_owned());
            }
            expected_source_order.push((
                integer(&target_value["marker_span"]["start"]),
                "target".to_owned(),
                shape_id.to_owned(),
            ));
        }

        for (family, key) in [
            ("decision_graph", "decision_graphs"),
            ("exception_region", "exception_regions"),
        ] {
            let mut previous_observation_start = None;
            for (observation_ordinal, observation) in array(&case[key]).iter().enumerate() {
                if integer(&observation["source_ordinal"]) != observation_ordinal as u64 {
                    return Err("control observation source order".to_owned());
                }
                let observation_id = text(&observation["id"]);
                let mutation_field = control_upgrade_mutation_field(family, observation)?;
                if observations
                    .insert(
                        observation_id,
                        (
                            case_id,
                            disposition,
                            family,
                            canonical_sha256(observation),
                            mutation_field,
                        ),
                    )
                    .is_some()
                {
                    return Err("duplicate control observation".to_owned());
                }
                let start = if family == "decision_graph" {
                    integer(&observation["root"]["span"]["start"])
                } else {
                    integer(&observation["span"]["start"])
                };
                if previous_observation_start.is_some_and(|previous| previous > start) {
                    return Err("control observation lexical order".to_owned());
                }
                previous_observation_start = Some(start);
                expected_source_order.push((start, family.to_owned(), observation_id.to_owned()));
                if family == "decision_graph" {
                    let nodes = array(&observation["nodes"]);
                    if nodes.is_empty() {
                        return Err("empty decision graph".to_owned());
                    }
                    let node_ids = nodes
                        .iter()
                        .map(|node| text(&node["id"]))
                        .collect::<BTreeSet<_>>();
                    let mut previous_node_order = None;
                    for (ordinal, node) in nodes.iter().enumerate() {
                        if integer(&node["source_ordinal"]) != ordinal as u64 {
                            return Err("decision source order".to_owned());
                        }
                        validate_span(&node["span"], source_utf16_length)?;
                        let node_order = (
                            integer(&node["span"]["start"]),
                            Reverse(integer(&node["span"]["length"])),
                            text(&node["operation_kind"]).to_owned(),
                        );
                        if previous_node_order
                            .as_ref()
                            .is_some_and(|previous| previous > &node_order)
                        {
                            return Err("decision lexical order".to_owned());
                        }
                        previous_node_order = Some(node_order);
                        if !node["parent_id"].is_null()
                            && !node_ids.contains(text(&node["parent_id"]))
                        {
                            return Err("decision parent".to_owned());
                        }
                    }
                    for edge in array(&observation["edges"]) {
                        if edge["kind"] != "operation_parent"
                            || !node_ids.contains(text(&edge["from"]))
                            || !node_ids.contains(text(&edge["to"]))
                        {
                            return Err("decision edge".to_owned());
                        }
                    }
                } else {
                    validate_span(&observation["span"], source_utf16_length)?;
                    let catches = array(&observation["catches"]);
                    let search = array(&observation["handler_search_order"]);
                    if search.len() != catches.len() {
                        return Err("handler search count".to_owned());
                    }
                    let mut previous_catch_start = None;
                    for (ordinal, catch_clause) in catches.iter().enumerate() {
                        if integer(&catch_clause["source_ordinal"]) != ordinal as u64
                            || integer(&search[ordinal]) != ordinal as u64
                        {
                            return Err("handler search order".to_owned());
                        }
                        validate_span(&catch_clause["span"], source_utf16_length)?;
                        let catch_start = integer(&catch_clause["span"]["start"]);
                        if previous_catch_start.is_some_and(|previous| previous > catch_start) {
                            return Err("catch lexical order".to_owned());
                        }
                        previous_catch_start = Some(catch_start);
                    }
                    let mut previous_throw_start = None;
                    for (ordinal, throw_operation) in
                        array(&observation["throws"]).iter().enumerate()
                    {
                        if integer(&throw_operation["source_ordinal"]) != ordinal as u64 {
                            return Err("throw source order".to_owned());
                        }
                        validate_span(&throw_operation["span"], source_utf16_length)?;
                        let throw_start = integer(&throw_operation["span"]["start"]);
                        if previous_throw_start.is_some_and(|previous| previous > throw_start) {
                            return Err("throw lexical order".to_owned());
                        }
                        previous_throw_start = Some(throw_start);
                    }
                }
            }
        }
        for abrupt in array(&case["abrupt_completions"]) {
            validate_span(&abrupt["span"], source_utf16_length)?;
            expected_source_order.push((
                integer(&abrupt["span"]["start"]),
                "abrupt_completion".to_owned(),
                text(&abrupt["id"]).to_owned(),
            ));
        }
        expected_source_order.sort();
        let actual_source_order = array(&case["source_order"])
            .iter()
            .enumerate()
            .map(|(ordinal, row)| {
                if integer(&row["source_ordinal"]) != ordinal as u64 {
                    return Err("case source ordinal".to_owned());
                }
                Ok((
                    integer(&row["start"]),
                    text(&row["category"]).to_owned(),
                    text(&row["id"]).to_owned(),
                ))
            })
            .collect::<Result<Vec<_>, String>>()?;
        if actual_source_order != expected_source_order {
            return Err("case source order".to_owned());
        }
    }
    if !rejected_clean || !rejected_warning || !rejected_error {
        return Err("rejected compiler outcome coverage".to_owned());
    }

    let shape_index = array(&document["shape_index"]);
    let mut shape_ids = Vec::new();
    let mut admitted = Vec::new();
    let mut rejected = Vec::new();
    for row in shape_index {
        let shape_id = text(&row["shape_id"]);
        let Some(expected) = targets.get(shape_id) else {
            return Err("control shape target link".to_owned());
        };
        if (
            text(&row["case_id"]),
            text(&row["disposition"]),
            integer(&row["source_ordinal"]),
            text(&row["observation_sha256"]),
        ) != (expected.0, expected.1, expected.2, expected.3.as_str())
        {
            return Err("control shape index".to_owned());
        }
        shape_ids.push(shape_id);
        match expected.1 {
            "admitted_shape" => admitted.push(shape_id),
            "rejected_near_miss" => rejected.push(shape_id),
            _ => return Err("control shape disposition".to_owned()),
        }
    }
    if shape_ids.len() != 103
        || admitted.len() != 62
        || rejected.len() != 41
        || shape_ids.len() != targets.len()
        || !shape_ids.windows(2).all(|pair| pair[0] < pair[1])
    {
        return Err("control shape catalog".to_owned());
    }
    if canonical_sha256(&Value::Array(
        shape_ids.iter().map(|id| json!(id)).collect(),
    )) != CONTROL_SHAPE_IDS_SHA256
        || canonical_sha256(&Value::Array(admitted.iter().map(|id| json!(id)).collect()))
            != CONTROL_ADMITTED_SHAPE_IDS_SHA256
        || canonical_sha256(&Value::Array(rejected.iter().map(|id| json!(id)).collect()))
            != CONTROL_REJECTED_SHAPE_IDS_SHA256
    {
        return Err("control shape fingerprint".to_owned());
    }
    let coverage = array(&document["coverage"]);
    if coverage
        .iter()
        .map(|row| text(&row["requirement"]))
        .collect::<Vec<_>>()
        != [
            "abrupt_completion",
            "catch_and_throw",
            "filters_finally_and_regions",
            "guards",
            "loops_and_structured_branches",
            "patterns",
            "switch_statements_and_expressions",
        ]
        || coverage
            .iter()
            .any(|row| array(&row["shape_ids"]).is_empty())
    {
        return Err("control coverage".to_owned());
    }
    let covered = coverage
        .iter()
        .flat_map(|row| array(&row["shape_ids"]).iter().map(text))
        .collect::<BTreeSet<_>>();
    if covered != shape_ids.iter().copied().collect() {
        return Err("unowned control shape".to_owned());
    }

    let mutations = array(&document["upgrade_mutations"]);
    let mut mutation_ids = BTreeSet::new();
    let mut linked_observations = BTreeSet::new();
    let mut family_dispositions = BTreeMap::new();
    let mut mutation_fields = BTreeSet::new();
    for row in mutations {
        if object(row).keys().map(String::as_str).collect::<Vec<_>>()
            != [
                "case_id",
                "disposition",
                "family",
                "mutation_field",
                "mutation_id",
                "observation_id",
                "observation_sha256",
            ]
        {
            return Err("control upgrade mutation keys".to_owned());
        }
        let observation_id = text(&row["observation_id"]);
        let Some(expected) = observations.get(observation_id) else {
            return Err("upgrade observation link".to_owned());
        };
        if row["case_id"] != expected.0
            || row["disposition"] != expected.1
            || row["family"] != expected.2
            || row["observation_sha256"] != expected.3
            || row["mutation_field"] != expected.4
            || !mutation_ids.insert(text(&row["mutation_id"]))
            || !linked_observations.insert(observation_id)
        {
            return Err("control upgrade mutation".to_owned());
        }
        *family_dispositions
            .entry((expected.2, expected.1))
            .or_insert(0usize) += 1;
        mutation_fields.insert(text(&row["mutation_field"]));
    }
    if mutations.len() != 65
        || linked_observations.len() != observations.len()
        || mutation_fields
            != BTreeSet::from([
                "handler_search_order[0]",
                "nesting_depth",
                "nodes[0].operation_kind",
            ])
        || family_dispositions
            != BTreeMap::from([
                (("decision_graph", "admitted_shape"), 22),
                (("decision_graph", "rejected_near_miss"), 18),
                (("exception_region", "admitted_shape"), 11),
                (("exception_region", "rejected_near_miss"), 14),
            ])
    {
        return Err("control upgrade catalog".to_owned());
    }
    Ok(())
}

// CSHARP-03-T01-W05
#[test]
fn canonical_control_probe_closes_decisions_regions_and_source_order() {
    let raw = bytes(CONTROL_RESULT_PATH);
    assert_eq!(raw.len(), CONTROL_RESULT_SIZE);
    assert_eq!(sha256(&raw), CONTROL_RESULT_SHA256);
    let document: Value = serde_json::from_slice(&raw).expect("parse control probe");
    let mut canonical = canonical_bytes(&document);
    canonical.push(b'\n');
    assert_eq!(raw, canonical);
    validate_control_document(&document).unwrap();

    for (shape_id, syntax_kind, operation_kind) in [
        ("loop.while.statement", "WhileStatement", "Loop"),
        ("loop.do.statement", "DoStatement", "Loop"),
        ("loop.for.statement", "ForStatement", "Loop"),
        ("loop.foreach.array_var", "ForEachStatement", "Loop"),
        ("loop.foreach.string", "ForEachStatement", "Loop"),
        ("switch.statement.integer", "SwitchStatement", "Switch"),
        (
            "switch.expression.logical",
            "SwitchExpression",
            "SwitchExpression",
        ),
        (
            "pattern.constant.integer",
            "ConstantPattern",
            "ConstantPattern",
        ),
        ("pattern.and", "AndPattern", "BinaryPattern"),
        (
            "pattern.relational.greater",
            "RelationalPattern",
            "RelationalPattern",
        ),
        ("pattern.list.empty", "ListPattern", "ListPattern"),
        (
            "exception.propagation.call_to_handler",
            "InvocationExpression",
            "Invocation",
        ),
        (
            "exception.source.implicit_parameterless_base",
            "ConstructorDeclaration",
            "ConstructorBodyOperation",
        ),
        ("exception.throw.rethrow", "ThrowStatement", "Throw"),
    ] {
        let probe_target = control_target(&document, shape_id);
        assert_eq!(probe_target["syntax"]["kind"], syntax_kind, "{shape_id}");
        assert_eq!(
            probe_target["operation"]["kind"], operation_kind,
            "{shape_id}"
        );
    }
    assert_eq!(
        control_target(&document, "pattern.parenthesized")["syntax"]["kind"],
        "ParenthesizedPattern"
    );
    assert!(control_target(&document, "pattern.parenthesized")["operation"].is_null());

    let implicit_base = control_target(&document, "exception.source.implicit_parameterless_base");
    let implicit_base_call = &implicit_base["operation"]["children"][0]["children"][0];
    assert_eq!(implicit_base_call["kind"], "Invocation");
    assert_eq!(implicit_base_call["is_implicit"], true);
    assert_eq!(
        implicit_base_call["details"]["target"]["display"],
        "System.Exception.Exception()"
    );

    for shape_id in [
        "pattern.guard.statement",
        "pattern.guard.expression",
        "exception.filter.pure_boolean",
        "exception.filter.failure",
        "exception.search.outer_filter_before_inner_finally",
        "near_miss.exception.filter_side_effect",
    ] {
        let target = control_target(&document, shape_id);
        assert!(
            array(&document["observations"]["cases"])
                .iter()
                .flat_map(|case| array(&case["decision_graphs"]))
                .flat_map(|graph| array(&graph["nodes"]))
                .any(|node| {
                    node["operation_kind"] == target["operation"]["kind"]
                        && node["span"] == target["operation"]["span"]
                }),
            "decision graph is missing {shape_id}"
        );
    }

    let probe_source = String::from_utf8(bytes(
        "develop/probes/csharp-03/ControlExceptionPatternProbe.cs",
    ))
    .expect("UTF-8 control probe source");
    assert!(probe_source.contains("MetadataImportOptions.Public"));
    for forbidden in [
        "BindingFlags",
        "InternalsVisibleTo",
        "Microsoft.CodeAnalysis.CSharp.Binder",
        "Microsoft.CodeAnalysis.CSharp.Symbols",
        "NonPublic",
        ".GetField(",
        ".GetMethod(",
        ".GetMethods(",
        ".GetProperties(",
        ".GetProperty(",
        ".Invoke(",
    ] {
        assert!(
            !probe_source.contains(forbidden),
            "private/compiler-internal API escape: {forbidden}"
        );
    }

    let decision_kinds = array(&document["observations"]["cases"])
        .iter()
        .flat_map(|case| array(&case["decision_graphs"]))
        .flat_map(|graph| array(&graph["nodes"]))
        .map(|node| text(&node["operation_kind"]))
        .collect::<BTreeSet<_>>();
    for required in [
        "BinaryPattern",
        "Branch",
        "ConstantPattern",
        "DeclarationPattern",
        "DiscardPattern",
        "ListPattern",
        "Loop",
        "NegatedPattern",
        "RecursivePattern",
        "RelationalPattern",
        "SlicePattern",
        "Switch",
        "SwitchExpression",
        "TypePattern",
    ] {
        assert!(decision_kinds.contains(required), "missing {required}");
    }
}

// CSHARP-03-T01-W05
#[test]
fn exception_regions_freeze_lexical_search_filter_failure_and_unwind() {
    let document = load_control();
    validate_control_document(&document).unwrap();
    let mut region_kinds = BTreeSet::new();
    for case in array(&document["observations"]["cases"]) {
        for cfg in array(&case["control_flow_graphs"]) {
            collect_region_kinds(&cfg["regions"], &mut region_kinds);
        }
    }
    for required in [
        "Catch",
        "Filter",
        "FilterAndHandler",
        "Finally",
        "Try",
        "TryAndCatch",
        "TryAndFinally",
    ] {
        assert!(region_kinds.contains(required), "missing region {required}");
    }

    let nested = control_case(&document, "admitted-filters-and-finally");
    assert!(array(&nested["exception_regions"])
        .iter()
        .any(|region| region["nesting_depth"] == 1 && !region["finally"].is_null()));
    assert_eq!(
        control_target(
            &document,
            "exception.search.outer_filter_before_inner_finally"
        )["operation"]["kind"],
        "Invocation"
    );

    let failure = control_case(&document, "admitted-filter-failure-and-abrupt-finally");
    let filtered = array(&failure["exception_regions"])
        .iter()
        .find(|region| array(&region["catches"]).len() == 2)
        .expect("filter failure region");
    assert!(!filtered["catches"][0]["filter"].is_null());
    assert!(filtered["catches"][1]["filter"].is_null());
    assert_eq!(filtered["handler_search_order"], json!([0, 1]));

    let abrupt_kinds = array(&document["observations"]["cases"])
        .iter()
        .flat_map(|case| array(&case["abrupt_completions"]))
        .map(|abrupt| text(&abrupt["completion_kind"]))
        .collect::<BTreeSet<_>>();
    assert_eq!(
        abrupt_kinds,
        BTreeSet::from(["Break", "Continue", "GoTo", "Rethrow", "Return", "Throw"])
    );
}

// CSHARP-03-T01-W05
#[test]
fn every_decision_graph_and_exception_region_has_an_upgrade_mutation() {
    let document = load_control();
    validate_control_document(&document).unwrap();
    let mut mutation_ids = BTreeSet::new();
    for row in array(&document["upgrade_mutations"]) {
        assert!(mutation_ids.insert(text(&row["mutation_id"])));
        let family = text(&row["family"]);
        let observation_id = text(&row["observation_id"]);
        let mut observation = control_observation(&document, family, observation_id).clone();
        apply_control_upgrade_mutation(&mut observation, text(&row["mutation_field"]));
        assert_ne!(canonical_sha256(&observation), row["observation_sha256"]);
    }
    assert_eq!(mutation_ids.len(), 65);
    for family in ["decision_graph", "exception_region"] {
        let row = array(&document["upgrade_mutations"])
            .iter()
            .find(|row| row["family"] == family)
            .expect("upgrade family");
        let observation_id = text(&row["observation_id"]).to_owned();
        let mut changed = document.clone();
        let observation = control_observation_mut(&mut changed, family, &observation_id);
        apply_control_upgrade_mutation(observation, text(&row["mutation_field"]));
        assert!(validate_control_document(&changed).is_err());
    }
}

// CSHARP-03-T01-W05
#[test]
fn control_probe_preserves_w04_and_the_active_release_boundary() {
    assert_eq!(sha256(&bytes(RESULT_PATH)), RESULT_SHA256);
    assert_eq!(
        sha256(&bytes("develop/probes/csharp-03/DataConstructionProbe.cs")),
        PROBE_SOURCE_SHA256
    );
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
        assert!(!content.contains("roslyn_control_probe"));
    }
}

// CSHARP-03-T01-W05
#[test]
fn pinned_control_probe_rerun_is_byte_identical_when_the_linux_cache_is_available() {
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
        repository_root().join("develop/probes/csharp-03/run-control-exception-pattern-probe.sh"),
    )
    .arg("--check")
    .env_clear()
    .env("PATH", "/usr/bin:/bin")
    .output()
    .expect("execute pinned W05 Roslyn probe");
    assert!(
        output.status.success(),
        "stdout={} stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output.stdout.is_empty());
    assert!(output.stderr.is_empty());
}

fn load_dependency() -> Value {
    serde_json::from_slice(&bytes(DEPENDENCY_RESULT_PATH)).expect("parse W06 probe result")
}

fn dependency_case<'a>(document: &'a Value, case_id: &str) -> &'a Value {
    array(&document["observations"]["cases"])
        .iter()
        .find(|case| case["id"] == case_id)
        .unwrap_or_else(|| panic!("missing dependency case {case_id}"))
}

fn dependency_target<'a>(document: &'a Value, shape_id: &str) -> &'a Value {
    for case in array(&document["observations"]["cases"]) {
        for candidate in array(&case["targets"]) {
            if candidate["shape_id"] == shape_id {
                return candidate;
            }
        }
    }
    panic!("missing dependency target {shape_id}");
}

fn dependency_target_mut<'a>(document: &'a mut Value, shape_id: &str) -> &'a mut Value {
    for case in document["observations"]["cases"]
        .as_array_mut()
        .expect("dependency cases")
    {
        for candidate in case["targets"].as_array_mut().expect("dependency targets") {
            if candidate["shape_id"] == shape_id {
                return candidate;
            }
        }
    }
    panic!("missing mutable dependency target {shape_id}");
}

fn dependency_family(shape_id: &str) -> Option<&'static str> {
    [
        (
            "exception.compiler_metadata.",
            "exception.compiler_metadata",
        ),
        ("exception.incidental.", "exception.incidental_metadata"),
        ("exception.nullable.", "exception.nullable_shorthand"),
        ("exception.array.", "exception.array_non_generic"),
        (
            "near_miss.dependency.generated_source.",
            "dependency.generated_source",
        ),
        ("near_miss.dependency.namespace.", "dependency.namespace"),
        ("near_miss.dependency.package.", "dependency.package"),
        ("near_miss.dependency.assembly.", "dependency.assembly"),
        ("near_miss.dependency.attribute.", "dependency.attribute"),
        ("near_miss.dependency.interface.", "dependency.interface"),
        ("near_miss.dependency.base_type.", "dependency.base_type"),
        ("near_miss.dependency.project.", "dependency.project"),
        ("near_miss.dependency.ambient.", "dependency.ambient"),
        (
            "near_miss.attribute.compiler_marker.",
            "attribute.compiler_marker_spelling",
        ),
        ("near_miss.attribute.source.", "attribute.source_written"),
        ("near_miss.generic.declaration.", "generic.declaration"),
        ("near_miss.generic.method.", "generic.method"),
        (
            "near_miss.generic.type_parameter.",
            "generic.type_parameter",
        ),
        ("near_miss.generic.constraint.", "generic.constraint"),
        ("near_miss.generic.variance.", "generic.variance"),
        ("near_miss.generic.explicit_call.", "generic.explicit_call"),
        ("near_miss.generic.inferred_call.", "generic.inferred_call"),
        ("near_miss.generic.closed_use.", "generic.closed_use"),
        (
            "near_miss.generic.framework_type.",
            "generic.framework_type",
        ),
        ("near_miss.generic.open_type.", "generic.open_type"),
        (
            "near_miss.generic.explicit_nullable.",
            "generic.explicit_nullable",
        ),
        (
            "near_miss.generic.unsupported_nullable.",
            "generic.unsupported_nullable",
        ),
        (
            "near_miss.generic.transitive_metadata.",
            "generic.transitive_metadata",
        ),
        ("near_miss.iterator.async.", "iterator.async"),
        ("near_miss.iterator.declaration.", "iterator.declaration"),
        ("near_miss.iterator.yield.", "iterator.yield"),
        ("near_miss.iterator.protocol.", "iterator.protocol"),
        ("near_miss.iterator.state_machine", "iterator.state_machine"),
        ("near_miss.async.declaration.", "async.declaration"),
        ("near_miss.async.await.", "async.await"),
        ("near_miss.async.task.", "async.task"),
        ("near_miss.async.value_task.", "async.value_task"),
        ("near_miss.async.awaiter.", "async.awaiter"),
        ("near_miss.async.cancellation.", "async.cancellation"),
        ("near_miss.async.parallel.", "async.parallel"),
        ("near_miss.async.state_machine", "async.state_machine"),
    ]
    .into_iter()
    .find_map(|(prefix, family)| shape_id.starts_with(prefix).then_some(family))
}

fn dependency_mutation_field(target: &Value) -> Result<&'static str, String> {
    if !array(&target["emitted_evidence"]).is_empty() {
        return Ok("emitted_evidence[0]");
    }
    if target["operation"]["kind"].is_string() {
        return Ok("operation.kind");
    }
    if target["symbol"]["display"].is_string() {
        return Ok("symbol.display");
    }
    if target["declared_symbol"]["display"].is_string() {
        return Ok("declared_symbol.display");
    }
    if target["syntax"]["kind"].is_string() {
        return Ok("syntax.kind");
    }
    Err("dependency mutation field".to_owned())
}

fn apply_dependency_mutation(target: &mut Value, field: &str) {
    match field {
        "emitted_evidence[0]" => {
            target["emitted_evidence"].as_array_mut().expect("evidence")[0] =
                json!("#upgrade-mutation");
        }
        "operation.kind" => {
            target["operation"]["kind"] = json!(format!(
                "{}#upgrade-mutation",
                text(&target["operation"]["kind"])
            ));
        }
        "symbol.display" => {
            target["symbol"]["display"] = json!(format!(
                "{}#upgrade-mutation",
                text(&target["symbol"]["display"])
            ));
        }
        "declared_symbol.display" => {
            target["declared_symbol"]["display"] = json!(format!(
                "{}#upgrade-mutation",
                text(&target["declared_symbol"]["display"])
            ));
        }
        "syntax.kind" => {
            target["syntax"]["kind"] = json!(format!(
                "{}#upgrade-mutation",
                text(&target["syntax"]["kind"])
            ));
        }
        _ => panic!("unknown dependency mutation field {field}"),
    }
}

fn valid_sha256(value: &Value) -> bool {
    value.as_str().is_some_and(|digest| {
        digest.len() == 64
            && digest
                .bytes()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    })
}

fn validate_dependency_document(document: &Value) -> Result<(), String> {
    if object(document)
        .keys()
        .map(String::as_str)
        .collect::<Vec<_>>()
        != [
            "baseline",
            "family_index",
            "measurement",
            "observations",
            "probe_input",
            "schema",
            "shape_index",
            "upgrade_mutations",
            "work_item",
        ]
    {
        return Err("dependency top-level keys".to_owned());
    }
    if document["schema"] != "mpk.csharp_practical.t01_w06.roslyn_exclusion_probe.v0"
        || document["work_item"] != "CSHARP-03-T01-W06"
    {
        return Err("dependency identity".to_owned());
    }
    if document["baseline"]
        != json!({
            "build_inputs": "develop/migrations/csharp-03/build-inputs/build-inputs.json",
            "build_inputs_raw_sha256": W03_DESCRIPTOR_SHA256,
            "candidate_inventory": "develop/migrations/csharp-03/build-inputs/candidate-inventory.json",
            "candidate_inventory_raw_sha256": W03_INVENTORY_SHA256,
            "control_probe": CONTROL_RESULT_PATH,
            "control_probe_raw_sha256": CONTROL_RESULT_SHA256,
            "source_commit": "13415911853c0368c103bd9d5feeb8374596d724",
            "source_tree": "5d9000f11b2c3cab35ad08dc61a66fb14894d249"
        })
    {
        return Err("dependency baseline".to_owned());
    }
    if document["probe_input"]
        != json!({
            "compiler_arguments": [
                "/nologo", "/noconfig", "/nostdlib+", "/deterministic+",
                "/optimize+", "/debug-", "/target:exe", "/platform:x64",
                "/langversion:14.0", "/nullable:enable", "/checked+", "/unsafe-",
                "/warnaserror+", "/utf8output", "/filealign:512", "/highentropyva+"
            ],
            "path": "develop/probes/csharp-03/DependencyGenericSuspensionProbe.cs",
            "raw_sha256": DEPENDENCY_PROBE_SOURCE_SHA256,
            "reference_projection_sha256": REFERENCE_SHA256,
            "size_bytes": 89065,
            "toolchain_inputs_sha256": TOOLCHAIN_SHA256
        })
    {
        return Err("dependency probe input".to_owned());
    }
    if object(&document["measurement"])
        .keys()
        .map(String::as_str)
        .collect::<Vec<_>>()
        != [
            "probe_binary_sha256",
            "raw_observation_sha256",
            "raw_observation_size_bytes",
        ]
        || !valid_sha256(&document["measurement"]["probe_binary_sha256"])
        || !valid_sha256(&document["measurement"]["raw_observation_sha256"])
    {
        return Err("dependency measurement".to_owned());
    }
    let mut observation_bytes = canonical_bytes(&document["observations"]);
    observation_bytes.push(b'\n');
    if sha256(&observation_bytes) != document["measurement"]["raw_observation_sha256"]
        || observation_bytes.len() as u64
            != integer(&document["measurement"]["raw_observation_size_bytes"])
    {
        return Err("dependency observation measurement".to_owned());
    }

    let observations = &document["observations"];
    if object(observations)
        .keys()
        .map(String::as_str)
        .collect::<Vec<_>>()
        != [
            "cases",
            "compiler",
            "schema",
            "synthetic_references",
            "work_item",
        ]
        || observations["schema"] != "mpk.csharp_practical.t01_w06.roslyn_exclusion_probe.raw.v0"
        || observations["work_item"] != "CSHARP-03-T01-W06"
        || observations["compiler"]
            != json!({
                "architecture": "X64",
                "base_reference_count": 167,
                "language": "C#",
                "language_version": "14.0",
                "nullable_context": "Enable",
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
        return Err("dependency raw identity".to_owned());
    }

    let synthetic = array(&observations["synthetic_references"]);
    let mut synthetic_ids = BTreeSet::new();
    let mut synthetic_origins = BTreeSet::new();
    for reference in synthetic {
        if object(reference)
            .keys()
            .map(String::as_str)
            .collect::<Vec<_>>()
            != [
                "assembly_name",
                "id",
                "origin",
                "pe_sha256",
                "pe_size_bytes",
                "source_sha256",
                "virtual_path",
            ]
            || !valid_sha256(&reference["pe_sha256"])
            || !valid_sha256(&reference["source_sha256"])
            || integer(&reference["pe_size_bytes"]) == 0
            || !text(&reference["virtual_path"]).starts_with("/virtual/")
        {
            return Err("synthetic reference".to_owned());
        }
        synthetic_ids.insert(text(&reference["id"]));
        synthetic_origins.insert(text(&reference["origin"]));
    }
    if synthetic.len() != 3
        || synthetic_ids != BTreeSet::from(["ambient-project", "mpk-package", "mpk-project"])
        || synthetic_origins != BTreeSet::from(["ambient", "package", "project"])
        || synthetic
            .iter()
            .map(|reference| {
                format!(
                    "{}|{}|{}|{}",
                    text(&reference["id"]),
                    text(&reference["origin"]),
                    text(&reference["assembly_name"]),
                    text(&reference["virtual_path"])
                )
            })
            .collect::<Vec<_>>()
            != [
                "ambient-project|ambient|Vendor.Ambient.Dependency|/virtual/ambient/Vendor.Ambient.Dependency.dll",
                "mpk-package|package|Mpk.Package.Dependency|/virtual/packages/Mpk.Package/1.0.0/lib/net10.0/Mpk.Package.Dependency.dll",
                "mpk-project|project|Mpk.Project.Dependency|/virtual/projects/Mpk.Project.Dependency/bin/Mpk.Project.Dependency.dll",
            ]
    {
        return Err("synthetic reference set".to_owned());
    }

    let cases = array(&observations["cases"]);
    let mut case_ids = BTreeSet::new();
    let mut admitted_cases = 0usize;
    let mut rejected_outcomes = BTreeSet::new();
    let mut shape_records = Vec::new();
    let mut family_shapes: BTreeMap<String, Vec<String>> = BTreeMap::new();
    let mut family_outcomes: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    let mut targets_by_shape = BTreeMap::new();
    for case in cases {
        if object(case).keys().map(String::as_str).collect::<Vec<_>>()
            != [
                "compiler_outcome",
                "diagnostics",
                "disposition",
                "emitted_metadata",
                "extra_references",
                "generated_sources",
                "id",
                "operation_roots",
                "source",
                "source_order",
                "source_utf8_sha256",
                "syntax",
                "targets",
            ]
        {
            return Err("dependency case keys".to_owned());
        }
        let case_id = text(&case["id"]);
        if !case_ids.insert(case_id) {
            return Err("duplicate dependency case".to_owned());
        }
        let disposition = text(&case["disposition"]);
        let has_error = array(&case["diagnostics"])
            .iter()
            .any(|diagnostic| diagnostic["severity"] == "Error");
        let expected_compiler_outcome = if has_error { "error" } else { "success" };
        if case["compiler_outcome"] != expected_compiler_outcome {
            return Err("dependency compiler outcome".to_owned());
        }
        if disposition == "admitted_exception_observation" {
            admitted_cases += 1;
            if has_error || !array(&case["diagnostics"]).is_empty() {
                return Err("admitted dependency diagnostics".to_owned());
            }
        } else if disposition == "rejected_profile_form" {
            rejected_outcomes.insert(if has_error {
                "error"
            } else if array(&case["diagnostics"]).is_empty() {
                "clean"
            } else {
                "warning"
            });
        } else {
            return Err("dependency disposition".to_owned());
        }
        let source = text(&case["source"]);
        if sha256(source.as_bytes()) != case["source_utf8_sha256"]
            || array(&case["syntax"]).is_empty()
            || array(&case["operation_roots"]).is_empty()
        {
            return Err("dependency case observation".to_owned());
        }
        for generated in array(&case["generated_sources"]) {
            if object(generated)
                .keys()
                .map(String::as_str)
                .collect::<Vec<_>>()
                != ["hint_name", "path", "source", "source_utf8_sha256"]
                || sha256(text(&generated["source"]).as_bytes()) != generated["source_utf8_sha256"]
            {
                return Err("generated source observation".to_owned());
            }
        }
        let targets = array(&case["targets"]);
        let source_order = array(&case["source_order"]);
        if targets.len() != source_order.len() {
            return Err("dependency source order count".to_owned());
        }
        let source_utf16_length = source.encode_utf16().count() as u64;
        let mut previous_start = 0u64;
        for (ordinal, (target, order)) in targets.iter().zip(source_order).enumerate() {
            if object(target)
                .keys()
                .map(String::as_str)
                .collect::<Vec<_>>()
                != [
                    "candidate_reason",
                    "candidate_symbols",
                    "converted_type",
                    "declared_symbol",
                    "emitted_evidence",
                    "enclosing_symbol",
                    "family",
                    "generic_facts",
                    "marker_span",
                    "operation",
                    "profile_outcome",
                    "shape_id",
                    "source_ordinal",
                    "symbol",
                    "syntax",
                    "type",
                ]
                || object(order).keys().map(String::as_str).collect::<Vec<_>>()
                    != ["shape_id", "source_ordinal", "start"]
            {
                return Err("dependency target keys".to_owned());
            }
            let shape_id = text(&target["shape_id"]);
            let Some(family) = dependency_family(shape_id) else {
                return Err("dependency family".to_owned());
            };
            let expected_profile = if shape_id.starts_with("exception.") {
                "admitted_exception"
            } else {
                "rejected"
            };
            if target["family"] != family
                || target["profile_outcome"] != expected_profile
                || integer(&target["source_ordinal"]) != ordinal as u64
                || order["shape_id"] != shape_id
                || integer(&order["source_ordinal"]) != ordinal as u64
            {
                return Err("dependency target identity".to_owned());
            }
            validate_span(&target["marker_span"], source_utf16_length)?;
            validate_span(&target["syntax"]["span"], source_utf16_length)?;
            let start = integer(&target["marker_span"]["start"]);
            if integer(&order["start"]) != start || (ordinal > 0 && start < previous_start) {
                return Err("dependency source order".to_owned());
            }
            previous_start = start;
            let facts = &target["generic_facts"];
            if object(facts).keys().map(String::as_str).collect::<Vec<_>>()
                != [
                    "constructed_nullable_value_type",
                    "immediate_specialization",
                    "source_contains_generic_name",
                    "source_contains_nullable_shorthand",
                    "source_contains_type_parameter",
                    "symbol_arity",
                    "symbol_is_generic",
                    "type_arguments",
                    "type_parameters",
                ]
                || facts["constructed_nullable_value_type"].as_bool().is_none()
                || facts["source_contains_generic_name"].as_bool().is_none()
                || facts["source_contains_nullable_shorthand"]
                    .as_bool()
                    .is_none()
                || facts["source_contains_type_parameter"].as_bool().is_none()
                || facts["symbol_arity"].as_u64().is_none()
                || facts["symbol_is_generic"].as_bool().is_none()
                || !facts["type_arguments"].is_array()
                || !facts["type_parameters"].is_array()
                || !target["syntax"].is_object()
            {
                return Err("dependency target observation".to_owned());
            }
            if targets_by_shape.insert(shape_id, target).is_some() {
                return Err("duplicate dependency shape".to_owned());
            }
            family_shapes
                .entry(family.to_owned())
                .or_default()
                .push(shape_id.to_owned());
            family_outcomes
                .entry(family.to_owned())
                .or_default()
                .insert(expected_profile.to_owned());
            shape_records.push(json!({
                "case_id": case_id,
                "disposition": disposition,
                "family": family,
                "observation_sha256": canonical_sha256(target),
                "profile_outcome": expected_profile,
                "shape_id": shape_id,
                "source_ordinal": ordinal
            }));
        }
    }
    if cases.len() != 16
        || admitted_cases != 1
        || rejected_outcomes != BTreeSet::from(["clean", "error", "warning"])
    {
        return Err("dependency case catalog".to_owned());
    }

    shape_records.sort_by(|left, right| text(&left["shape_id"]).cmp(text(&right["shape_id"])));
    if document["shape_index"] != json!(shape_records) {
        return Err("dependency shape index".to_owned());
    }
    let shape_ids = shape_records
        .iter()
        .map(|row| text(&row["shape_id"]).to_owned())
        .collect::<Vec<_>>();
    let admitted_shape_ids = shape_ids
        .iter()
        .filter(|shape_id| shape_id.starts_with("exception."))
        .cloned()
        .collect::<Vec<_>>();
    let rejected_shape_ids = shape_ids
        .iter()
        .filter(|shape_id| !shape_id.starts_with("exception."))
        .cloned()
        .collect::<Vec<_>>();
    if shape_ids.len() != 144
        || admitted_shape_ids.len() != 12
        || rejected_shape_ids.len() != 132
        || canonical_sha256(&json!(shape_ids)) != DEPENDENCY_SHAPE_IDS_SHA256
        || canonical_sha256(&json!(admitted_shape_ids)) != DEPENDENCY_ADMITTED_SHAPE_IDS_SHA256
        || canonical_sha256(&json!(rejected_shape_ids)) != DEPENDENCY_REJECTED_SHAPE_IDS_SHA256
    {
        return Err("dependency shape catalog".to_owned());
    }

    for ids in family_shapes.values_mut() {
        ids.sort();
    }
    let family_index = family_shapes
        .iter()
        .map(|(family, ids)| {
            json!({
                "family": family,
                "profile_outcomes": family_outcomes[family].iter().collect::<Vec<_>>(),
                "shape_ids": ids,
                "shape_ids_sha256": canonical_sha256(&json!(ids))
            })
        })
        .collect::<Vec<_>>();
    let family_ids = family_shapes.keys().cloned().collect::<Vec<_>>();
    if family_index.len() != 41
        || document["family_index"] != json!(family_index)
        || canonical_sha256(&json!(family_ids)) != DEPENDENCY_FAMILY_IDS_SHA256
    {
        return Err("dependency family index".to_owned());
    }

    let mut mutations = Vec::new();
    let mut mutation_ids = BTreeSet::new();
    for row in &shape_records {
        let shape_id = text(&row["shape_id"]);
        let target = targets_by_shape[shape_id];
        let field = dependency_mutation_field(target)?;
        let mutation_id = format!(
            "CSHARP-03-T01-W06-UPGRADE-{}",
            shape_id.to_ascii_uppercase().replace('.', "-")
        );
        if !mutation_ids.insert(mutation_id.clone()) {
            return Err("duplicate dependency mutation".to_owned());
        }
        mutations.push(json!({
            "case_id": row["case_id"],
            "family": row["family"],
            "mutation_field": field,
            "mutation_id": mutation_id,
            "observation_sha256": row["observation_sha256"],
            "profile_outcome": row["profile_outcome"],
            "shape_id": shape_id
        }));
    }
    mutations.sort_by(|left, right| text(&left["mutation_id"]).cmp(text(&right["mutation_id"])));
    if mutations.len() != 144 || document["upgrade_mutations"] != json!(mutations) {
        return Err("dependency mutation index".to_owned());
    }
    Ok(())
}

// CSHARP-03-T01-W06
#[test]
fn canonical_dependency_probe_closes_symbols_families_and_source_order() {
    let raw = bytes(DEPENDENCY_RESULT_PATH);
    assert_eq!(raw.len(), DEPENDENCY_RESULT_SIZE);
    assert_eq!(sha256(&raw), DEPENDENCY_RESULT_SHA256);
    let document: Value = serde_json::from_slice(&raw).expect("parse dependency probe");
    let mut canonical = canonical_bytes(&document);
    canonical.push(b'\n');
    assert_eq!(raw, canonical);
    validate_dependency_document(&document).unwrap();

    let probe_source = String::from_utf8(bytes(
        "develop/probes/csharp-03/DependencyGenericSuspensionProbe.cs",
    ))
    .expect("UTF-8 dependency probe source");
    assert!(probe_source.contains("MetadataImportOptions.Public"));
    for forbidden in [
        "BindingFlags",
        "InternalsVisibleTo",
        "Microsoft.CodeAnalysis.CSharp.Binder",
        "Microsoft.CodeAnalysis.CSharp.Symbols",
        "NonPublic",
        "System.Reflection.MethodInfo",
        "System.Reflection.PropertyInfo",
        "Activator.CreateInstance",
        "GetType().Get",
    ] {
        assert!(
            !probe_source.contains(forbidden),
            "private/compiler-internal API escape: {forbidden}"
        );
    }
}

// CSHARP-03-T01-W06
#[test]
fn dependency_and_attribute_forms_are_observed_without_becoming_capabilities() {
    let document = load_dependency();
    validate_dependency_document(&document).unwrap();
    let dependency_families = array(&document["family_index"])
        .iter()
        .filter(|row| text(&row["family"]).starts_with("dependency."))
        .map(|row| text(&row["family"]))
        .collect::<BTreeSet<_>>();
    assert_eq!(
        dependency_families,
        BTreeSet::from([
            "dependency.ambient",
            "dependency.assembly",
            "dependency.attribute",
            "dependency.base_type",
            "dependency.generated_source",
            "dependency.interface",
            "dependency.namespace",
            "dependency.package",
            "dependency.project",
        ])
    );

    let package = dependency_target(&document, "near_miss.dependency.package.reference_origin");
    assert_eq!(package["operation"]["kind"], "PropertyReference");
    assert_eq!(package["symbol"]["display"], "Mpk.Package.Api.Value");
    assert!(text(&package["symbol"]["containing_assembly"]).starts_with("Mpk.Package.Dependency,"));
    assert!(array(&package["emitted_evidence"])
        .iter()
        .any(|reference| reference["name"] == "Mpk.Package.Dependency"));

    let project = dependency_target(&document, "near_miss.dependency.project.reference_origin");
    let ambient = dependency_target(&document, "near_miss.dependency.ambient.reference");
    assert!(text(&project["symbol"]["containing_assembly"]).starts_with("Mpk.Project.Dependency,"));
    assert!(
        text(&ambient["symbol"]["containing_assembly"]).starts_with("Vendor.Ambient.Dependency,")
    );
    assert_eq!(
        dependency_case(&document, "dependency-namespace-spoof")["diagnostics"][0]["id"],
        "CS1030"
    );

    let generated = dependency_target(&document, "near_miss.dependency.generated_source.member");
    assert_eq!(generated["operation"]["kind"], "Invocation");
    assert!(array(&generated["symbol"]["locations"])
        .iter()
        .any(|location| location["origin"] == "generated"));
    assert_eq!(
        generated["emitted_evidence"][0]["hint_name"],
        "GeneratedDependency.g.cs"
    );

    let attributes = array(&document["shape_index"])
        .iter()
        .filter(|row| row["family"] == "attribute.source_written")
        .collect::<Vec<_>>();
    assert_eq!(attributes.len(), 23);
    let source_attribute_case = dependency_case(&document, "source-written-attributes");
    assert_eq!(
        array(&source_attribute_case["syntax"])
            .iter()
            .filter(|item| item["item"] == "node" && item["kind"] == "Attribute")
            .count(),
        attributes.len(),
        "every source-written attribute syntax needs an owned target"
    );
    for row in attributes {
        let target = dependency_target(&document, text(&row["shape_id"]));
        assert_eq!(target["syntax"]["kind"], "Attribute");
        assert_eq!(target["operation"]["kind"], "Attribute");
        assert_eq!(
            target["operation"]["source_path"],
            "src/source-written-attributes.cs"
        );
        if row["shape_id"] == "near_miss.attribute.source.attribute_usage" {
            assert!(text(&target["symbol"]["display"])
                .starts_with("System.AttributeUsageAttribute.AttributeUsageAttribute("));
            assert!(array(&target["symbol"]["locations"])
                .iter()
                .all(|location| location["origin"] == "metadata"));
        } else {
            assert!(text(&target["symbol"]["containing_assembly"])
                .starts_with("probe_source_written_attributes,"));
            assert!(array(&target["symbol"]["locations"])
                .iter()
                .all(|location| location["origin"] == "selected"));
        }
    }

    let required = dependency_target(&document, "exception.compiler_metadata.required_attributes");
    let required_text = serde_json::to_string(&required["emitted_evidence"]).unwrap();
    assert!(required_text.contains("RequiredMemberAttribute"));
    assert!(required_text.contains("CompilerFeatureRequiredAttribute"));
    let init = dependency_target(&document, "exception.compiler_metadata.init_modreq");
    assert!(serde_json::to_string(&init["emitted_evidence"])
        .unwrap()
        .contains("IsExternalInit"));
    assert_eq!(
        array(&document["shape_index"])
            .iter()
            .filter(|row| row["family"] == "attribute.compiler_marker_spelling")
            .count(),
        3
    );
}

// CSHARP-03-T01-W06
#[test]
fn exact_nullable_shorthand_is_the_only_constructed_generic_exception() {
    let document = load_dependency();
    validate_dependency_document(&document).unwrap();
    let admitted_constructed = array(&document["shape_index"])
        .iter()
        .filter(|row| row["profile_outcome"] == "admitted_exception")
        .filter_map(|row| {
            let target = dependency_target(&document, text(&row["shape_id"]));
            target["generic_facts"]["constructed_nullable_value_type"]
                .as_bool()
                .unwrap()
                .then_some(target)
        })
        .collect::<Vec<_>>();
    assert_eq!(admitted_constructed.len(), 4);
    for target in admitted_constructed {
        assert!(text(&target["shape_id"]).starts_with("exception.nullable.shorthand."));
        assert_eq!(
            target["generic_facts"]["source_contains_generic_name"],
            false
        );
        if target["shape_id"] == "exception.nullable.shorthand.implicit_conversion" {
            assert_eq!(
                target["converted_type"]["original_definition"],
                "System.Nullable<T>"
            );
        } else {
            assert_eq!(
                target["generic_facts"]["source_contains_nullable_shorthand"],
                true
            );
        }
        assert_eq!(
            target["generic_facts"]["immediate_specialization"]["shape"],
            "option"
        );
        assert_eq!(
            target["generic_facts"]["immediate_specialization"]["residual_type_parameter"],
            false
        );
        assert_eq!(
            target["generic_facts"]["immediate_specialization"]["payload"]["display"],
            "int"
        );
    }

    let reference = dependency_target(&document, "exception.nullable.reference_annotation");
    let array_target = dependency_target(&document, "exception.array.not_constructed_generic");
    assert_eq!(
        reference["generic_facts"]["constructed_nullable_value_type"],
        false
    );
    assert_eq!(
        array_target["generic_facts"]["constructed_nullable_value_type"],
        false
    );
    assert!(reference["generic_facts"]["immediate_specialization"].is_null());
    assert!(array_target["generic_facts"]["immediate_specialization"].is_null());

    for shape_id in [
        "near_miss.generic.explicit_nullable.system",
        "near_miss.generic.explicit_nullable.alias",
        "near_miss.generic.explicit_nullable.construction",
        "near_miss.generic.explicit_nullable.cast",
    ] {
        let target = dependency_target(&document, shape_id);
        assert_eq!(target["profile_outcome"], "rejected");
        assert_eq!(
            target["generic_facts"]["constructed_nullable_value_type"],
            true
        );
    }
    let explicit = dependency_target(&document, "near_miss.generic.explicit_nullable.system");
    assert_eq!(
        explicit["type"]["original_definition"],
        "System.Nullable<T>"
    );
    assert_eq!(
        explicit["generic_facts"]["source_contains_generic_name"],
        true
    );

    let generic_families = array(&document["family_index"])
        .iter()
        .filter(|row| text(&row["family"]).starts_with("generic."))
        .map(|row| text(&row["family"]))
        .collect::<BTreeSet<_>>();
    assert_eq!(generic_families.len(), 13);
    for required in [
        "generic.closed_use",
        "generic.constraint",
        "generic.declaration",
        "generic.explicit_call",
        "generic.explicit_nullable",
        "generic.framework_type",
        "generic.inferred_call",
        "generic.method",
        "generic.open_type",
        "generic.transitive_metadata",
        "generic.type_parameter",
        "generic.unsupported_nullable",
        "generic.variance",
    ] {
        assert!(generic_families.contains(required), "missing {required}");
    }
}

// CSHARP-03-T01-W06
#[test]
fn incidental_generic_metadata_does_not_expand_the_source_surface() {
    let document = load_dependency();
    validate_dependency_document(&document).unwrap();
    for (shape_id, operation_kind, symbol_display) in [
        (
            "exception.incidental.string_length",
            "PropertyReference",
            "string.Length",
        ),
        (
            "exception.incidental.array_length",
            "PropertyReference",
            "System.Array.Length",
        ),
        (
            "exception.incidental.decimal_round",
            "Invocation",
            "decimal.Round(decimal, int, System.MidpointRounding)",
        ),
        (
            "exception.incidental.date_only_constructor",
            "ObjectCreation",
            "System.DateOnly.DateOnly(int, int, int)",
        ),
    ] {
        let target = dependency_target(&document, shape_id);
        assert_eq!(target["operation"]["kind"], operation_kind, "{shape_id}");
        assert_eq!(target["symbol"]["display"], symbol_display, "{shape_id}");
        assert_eq!(target["profile_outcome"], "admitted_exception");
        assert_eq!(
            target["generic_facts"]["source_contains_generic_name"],
            false
        );
        let type_text = serde_json::to_string(&target["type"]).unwrap();
        assert!(
            type_text.contains('<'),
            "missing incidental metadata: {shape_id}"
        );
    }
    for row in array(&document["shape_index"])
        .iter()
        .filter(|row| row["family"] == "generic.transitive_metadata")
    {
        assert_eq!(row["profile_outcome"], "rejected");
    }
    assert_eq!(
        array(&document["family_index"])
            .iter()
            .find(|row| row["family"] == "generic.transitive_metadata")
            .unwrap()["shape_ids"]
            .as_array()
            .unwrap()
            .len(),
        8
    );
}

// CSHARP-03-T01-W06
#[test]
fn iterator_and_async_shapes_are_rejection_and_upgrade_only() {
    let document = load_dependency();
    validate_dependency_document(&document).unwrap();
    let suspension_rows = array(&document["shape_index"])
        .iter()
        .filter(|row| {
            let family = text(&row["family"]);
            family.starts_with("iterator.") || family.starts_with("async.")
        })
        .collect::<Vec<_>>();
    assert_eq!(suspension_rows.len(), 50);
    assert!(suspension_rows
        .iter()
        .all(|row| row["profile_outcome"] == "rejected"));
    let suspension_families = suspension_rows
        .iter()
        .map(|row| text(&row["family"]))
        .collect::<BTreeSet<_>>();
    assert_eq!(
        suspension_families,
        BTreeSet::from([
            "async.await",
            "async.awaiter",
            "async.cancellation",
            "async.declaration",
            "async.parallel",
            "async.state_machine",
            "async.task",
            "async.value_task",
            "iterator.async",
            "iterator.declaration",
            "iterator.protocol",
            "iterator.state_machine",
            "iterator.yield",
        ])
    );
    for (shape_id, symbol) in [
        (
            "near_miss.async.task.factory_run",
            "System.Threading.Tasks.Task.Run(System.Action)",
        ),
        (
            "near_miss.async.task.race_when_any",
            "System.Threading.Tasks.Task.WhenAny(System.Threading.Tasks.Task, System.Threading.Tasks.Task)",
        ),
        (
            "near_miss.async.parallel.for",
            "System.Threading.Tasks.Parallel.For(int, int, System.Action<int>)",
        ),
    ] {
        let target = dependency_target(&document, shape_id);
        assert_eq!(target["profile_outcome"], "rejected");
        assert_eq!(target["operation"]["kind"], "Invocation");
        assert_eq!(target["symbol"]["display"], symbol);
    }
    for (shape_id, observed_symbol) in [
        (
            "near_miss.iterator.protocol.ienumerable_generic",
            "System.Collections.Generic.IEnumerable<int>",
        ),
        (
            "near_miss.iterator.protocol.ienumerable_non_generic",
            "System.Collections.IEnumerable",
        ),
        (
            "near_miss.iterator.async.iasyncenumerator",
            "System.Collections.Generic.IAsyncEnumerator<int>",
        ),
    ] {
        let target = dependency_target(&document, shape_id);
        assert_eq!(target["profile_outcome"], "rejected");
        assert_eq!(target["symbol"]["display"], observed_symbol);
    }
    let async_enumerable =
        dependency_target(&document, "near_miss.iterator.async.iasyncenumerable");
    assert_eq!(async_enumerable["profile_outcome"], "rejected");
    assert_eq!(
        async_enumerable["declared_symbol"]["display"],
        "System.Collections.Generic.IAsyncEnumerable<int> values"
    );
    for shape_id in [
        "near_miss.iterator.state_machine",
        "near_miss.iterator.async.state_machine",
        "near_miss.async.state_machine",
        "near_miss.async.state_machine.custom_awaiter",
        "near_miss.async.state_machine.lambda",
    ] {
        let evidence =
            serde_json::to_string(&dependency_target(&document, shape_id)["emitted_evidence"])
                .unwrap();
        assert!(
            evidence.contains("d__"),
            "missing state-machine type: {shape_id}"
        );
        assert!(
            evidence.contains("StateMachineAttribute"),
            "missing state-machine attribute: {shape_id}"
        );
    }

    let mut seen_families = BTreeSet::new();
    for row in array(&document["upgrade_mutations"]) {
        let shape_id = text(&row["shape_id"]);
        let target = dependency_target(&document, shape_id);
        assert_eq!(canonical_sha256(target), row["observation_sha256"]);
        let mut changed_target = target.clone();
        apply_dependency_mutation(&mut changed_target, text(&row["mutation_field"]));
        assert_ne!(canonical_sha256(&changed_target), row["observation_sha256"]);

        let family = text(&row["family"]);
        if seen_families.insert(family) {
            let mut changed_document = document.clone();
            let changed = dependency_target_mut(&mut changed_document, shape_id);
            apply_dependency_mutation(changed, text(&row["mutation_field"]));
            assert!(
                validate_dependency_document(&changed_document).is_err(),
                "mutation accepted for {family}"
            );
        }
    }
    assert_eq!(seen_families.len(), 41);
}

// CSHARP-03-T01-W06
#[test]
fn dependency_probe_preserves_w05_and_the_active_release_boundary() {
    assert_eq!(sha256(&bytes(CONTROL_RESULT_PATH)), CONTROL_RESULT_SHA256);
    assert_eq!(
        sha256(&bytes(
            "develop/probes/csharp-03/ControlExceptionPatternProbe.cs"
        )),
        CONTROL_PROBE_SOURCE_SHA256
    );
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
        assert!(!content.contains("roslyn_exclusion_probe"));
    }
}

// CSHARP-03-T01-W06
#[test]
fn pinned_dependency_probe_rerun_is_byte_identical_when_the_linux_cache_is_available() {
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
        repository_root()
            .join("develop/probes/csharp-03/run-dependency-generic-suspension-probe.sh"),
    )
    .arg("--check")
    .env_clear()
    .env("PATH", "/usr/bin:/bin")
    .output()
    .expect("execute pinned W06 Roslyn probe");
    assert!(
        output.status.success(),
        "stdout={} stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output.stdout.is_empty());
    assert!(output.stderr.is_empty());
}

fn load_runtime_probe() -> Value {
    serde_json::from_slice(&bytes(RUNTIME_RESULT_PATH)).expect("parse W07 runtime probe result")
}

fn runtime_vector_in_run<'a>(run: &'a Value, vector_id: &str) -> &'a Value {
    array(&run["vectors"])
        .iter()
        .find(|vector| vector["id"] == vector_id)
        .unwrap_or_else(|| panic!("missing runtime vector {vector_id}"))
}

fn runtime_vector<'a>(document: &'a Value, vector_id: &str) -> &'a Value {
    runtime_vector_in_run(
        &array(&document["observations"]["culture_runs"])[0],
        vector_id,
    )
}

fn runtime_semantic_projection(vector: &Value) -> Value {
    let mut result = vector.clone();
    result
        .as_object_mut()
        .expect("runtime vector object")
        .remove("differential")
        .expect("runtime differential");
    result
}

fn runtime_string_set(value: &Value) -> BTreeSet<String> {
    array(value)
        .iter()
        .map(|item| text(item).to_owned())
        .collect()
}

fn validate_runtime_result(value: &Value, profile: bool) -> Result<(), String> {
    let expected = if profile {
        &["error_id", "kind", "result_encoding", "value"][..]
    } else {
        &["exception", "kind", "result_encoding", "value"][..]
    };
    if object(value).keys().map(String::as_str).collect::<Vec<_>>() != expected {
        return Err("runtime result keys".to_owned());
    }
    let kind = text(&value["kind"]);
    let encoding = text(&value["result_encoding"]);
    let result = text(&value["value"]);
    if !result.is_ascii() || result.len() > 1024 {
        return Err("runtime result bound".to_owned());
    }
    if profile {
        match kind {
            "value" if value["error_id"].is_null() && encoding != "none" => {}
            "error" | "rejected"
                if value["error_id"].as_str().is_some()
                    && encoding == "none"
                    && result.is_empty() => {}
            _ => return Err("runtime profile result".to_owned()),
        }
    } else {
        match kind {
            "value" if value["exception"].is_null() && encoding != "none" => {}
            "exception"
                if value["exception"].as_str().is_some()
                    && encoding == "none"
                    && result.is_empty() => {}
            "not_applicable"
                if value["exception"].is_null() && encoding == "none" && result.is_empty() => {}
            _ => return Err("runtime differential result".to_owned()),
        }
    }
    Ok(())
}

fn validate_runtime_document(document: &Value) -> Result<(), String> {
    if object(document)
        .keys()
        .map(String::as_str)
        .collect::<Vec<_>>()
        != [
            "baseline",
            "coverage",
            "culture_variants",
            "family_index",
            "measurement",
            "observations",
            "operation_index",
            "probe_input",
            "schema",
            "upgrade_mutations",
            "vector_index",
            "work_item",
        ]
    {
        return Err("runtime top-level keys".to_owned());
    }
    if document["schema"] != "mpk.csharp_practical.t01_w07.runtime_semantics_probe.v0"
        || document["work_item"] != "CSHARP-03-T01-W07"
    {
        return Err("runtime identity".to_owned());
    }
    if document["baseline"]
        != json!({
            "build_inputs": "develop/migrations/csharp-03/build-inputs/build-inputs.json",
            "build_inputs_raw_sha256": W03_DESCRIPTOR_SHA256,
            "candidate_inventory": "develop/migrations/csharp-03/build-inputs/candidate-inventory.json",
            "candidate_inventory_raw_sha256": W03_INVENTORY_SHA256,
            "dependency_probe": DEPENDENCY_RESULT_PATH,
            "dependency_probe_raw_sha256": DEPENDENCY_RESULT_SHA256,
            "source_commit": "22673dbc96d8ba4f0d9a4cb97c3f2490c00d1804",
            "source_tree": "687631b3799ba385ccde29de9d72286c48d3f8fc"
        })
    {
        return Err("runtime baseline".to_owned());
    }
    if document["probe_input"]
        != json!({
            "compiler_arguments": [
                "/nologo", "/noconfig", "/nostdlib+", "/deterministic+",
                "/optimize+", "/debug-", "/target:exe", "/platform:x64",
                "/langversion:14.0", "/nullable:enable", "/checked+", "/unsafe-",
                "/warnaserror+", "/utf8output", "/filealign:512", "/highentropyva+"
            ],
            "culture_profiles": ["hostile-arabic", "hostile-comma", "hostile-swap"],
            "path": "develop/probes/csharp-03/PrimitiveStringNumericCodecProbe.cs",
            "raw_sha256": RUNTIME_PROBE_SOURCE_SHA256,
            "reference_projection_sha256": REFERENCE_SHA256,
            "size_bytes": 126717,
            "toolchain_inputs_sha256": TOOLCHAIN_SHA256
        })
    {
        return Err("runtime probe input".to_owned());
    }
    if document["measurement"]
        != json!({
            "culture_run_count_per_build": 6,
            "probe_binary_sha256": "7b61263a2847340902b5692dd397c458a72cdd24a7b9158a8f4b3ea2279d85ed",
            "raw_observation_sha256": "872e6150d17476c52ee01db3530f9e710afc8c6252592daa5393f3c705e46967",
            "raw_observation_size_bytes": 6641752,
            "runtime_input_mutations": [
                "unlisted_environment.clean", "unlisted_environment.hostile"
            ]
        })
    {
        return Err("runtime measurement".to_owned());
    }
    let observations = &document["observations"];
    if object(observations)
        .keys()
        .map(String::as_str)
        .collect::<Vec<_>>()
        != ["culture_runs", "schema", "work_item"]
        || observations["schema"]
            != "mpk.csharp_practical.t01_w07.runtime_semantics_observations.v0"
        || observations["work_item"] != "CSHARP-03-T01-W07"
    {
        return Err("runtime observation identity".to_owned());
    }
    let mut observation_bytes = canonical_bytes(observations);
    observation_bytes.push(b'\n');
    if sha256(&observation_bytes) != document["measurement"]["raw_observation_sha256"]
        || observation_bytes.len() as u64
            != integer(&document["measurement"]["raw_observation_size_bytes"])
    {
        return Err("runtime observation measurement".to_owned());
    }

    let runs = array(&observations["culture_runs"]);
    if runs.len() != 3 {
        return Err("runtime culture count".to_owned());
    }
    let expected_cultures = [
        json!({
            "date_separator_utf16": "002a", "decimal_separator_utf16": "066b",
            "group_separator_utf16": "066c", "negative_sign_utf16": "2212",
            "profile": "hostile-arabic",
            "short_date_pattern_utf16": "00640064002a004d004d002a0079007900790079",
            "time_separator_utf16": "0021"
        }),
        json!({
            "date_separator_utf16": "002e", "decimal_separator_utf16": "002c",
            "group_separator_utf16": "002e", "negative_sign_utf16": "007e",
            "profile": "hostile-comma",
            "short_date_pattern_utf16": "0079007900790079002e004d004d002e00640064",
            "time_separator_utf16": "002d"
        }),
        json!({
            "date_separator_utf16": "005f", "decimal_separator_utf16": "003b",
            "group_separator_utf16": "002c", "negative_sign_utf16": "004e00450047",
            "profile": "hostile-swap",
            "short_date_pattern_utf16": "004d004d005f00640064005f0079007900790079",
            "time_separator_utf16": "002e"
        }),
    ];
    let mut run_vectors: Vec<BTreeMap<&str, &Value>> = Vec::new();
    for (run, culture) in runs.iter().zip(expected_cultures) {
        if object(run).keys().map(String::as_str).collect::<Vec<_>>()
            != ["culture", "runtime", "schema", "vectors", "work_item"]
            || run["schema"] != "mpk.csharp_practical.t01_w07.runtime_semantics_probe.raw.v0"
            || run["work_item"] != "CSHARP-03-T01-W07"
            || run["culture"] != culture
            || run["runtime"]
                != json!({
                    "architecture": "X64",
                    "framework_description": ".NET 10.0.11",
                    "runtime_version": "10.0.11"
                })
        {
            return Err("runtime culture run".to_owned());
        }
        let vectors = array(&run["vectors"]);
        if vectors.len() != 3468 {
            return Err("runtime vector count".to_owned());
        }
        let mut by_id = BTreeMap::new();
        let mut previous = None;
        for vector in vectors {
            if object(vector)
                .keys()
                .map(String::as_str)
                .collect::<Vec<_>>()
                != [
                    "accepted_domain",
                    "differential",
                    "error_precedence",
                    "family",
                    "id",
                    "inputs",
                    "operation",
                    "profile",
                    "profile_outcome",
                    "runtime_culture_sensitive",
                ]
            {
                return Err("runtime vector keys".to_owned());
            }
            let vector_id = text(&vector["id"]);
            if previous.is_some_and(|value| value >= vector_id)
                || !vector_id.is_ascii()
                || !text(&vector["family"]).is_ascii()
                || !text(&vector["operation"]).is_ascii()
                || !text(&vector["accepted_domain"]).is_ascii()
                || array(&vector["inputs"]).is_empty()
                || array(&vector["inputs"])
                    .iter()
                    .any(|input| !text(input).is_ascii())
                || vector["runtime_culture_sensitive"].as_bool().is_none()
            {
                return Err("runtime vector scalar".to_owned());
            }
            previous = Some(vector_id);
            validate_runtime_result(&vector["profile"], true)?;
            validate_runtime_result(&vector["differential"], false)?;
            let error = vector["profile"]["error_id"].as_str();
            let precedence = runtime_string_set(&vector["error_precedence"]);
            if error.is_some_and(|value| !precedence.contains(value)) {
                return Err("runtime error precedence link".to_owned());
            }
            if by_id.insert(vector_id, vector).is_some() {
                return Err("duplicate runtime vector".to_owned());
            }
        }
        run_vectors.push(by_id);
    }
    let vector_ids = run_vectors[0].keys().copied().collect::<Vec<_>>();
    if canonical_sha256(&Value::Array(
        vector_ids.iter().map(|id| json!(id)).collect(),
    )) != RUNTIME_VECTOR_IDS_SHA256
        || run_vectors[1].keys().copied().collect::<Vec<_>>() != vector_ids
        || run_vectors[2].keys().copied().collect::<Vec<_>>() != vector_ids
    {
        return Err("runtime vector catalog".to_owned());
    }

    let mut actual_variant_ids = Vec::new();
    for vector_id in &vector_ids {
        let first = run_vectors[0][vector_id];
        for other in [&run_vectors[1][vector_id], &run_vectors[2][vector_id]] {
            if runtime_semantic_projection(first) != runtime_semantic_projection(other) {
                return Err("runtime candidate culture drift".to_owned());
            }
            if first["runtime_culture_sensitive"] == false && first != *other {
                return Err("runtime undeclared culture drift".to_owned());
            }
        }
        if run_vectors[1][vector_id]["differential"] != first["differential"]
            || run_vectors[2][vector_id]["differential"] != first["differential"]
        {
            actual_variant_ids.push(*vector_id);
        }
    }
    if actual_variant_ids.len() != 83
        || canonical_sha256(&Value::Array(
            actual_variant_ids.iter().map(|id| json!(id)).collect(),
        )) != RUNTIME_CULTURE_VARIANT_IDS_SHA256
    {
        return Err("runtime culture variant catalog".to_owned());
    }

    let vector_index = array(&document["vector_index"]);
    if vector_index.len() != vector_ids.len() {
        return Err("runtime vector index count".to_owned());
    }
    for (row, vector_id) in vector_index.iter().zip(&vector_ids) {
        if object(row).keys().map(String::as_str).collect::<Vec<_>>()
            != [
                "candidate_observation_sha256",
                "culture_invariant_runtime",
                "family",
                "operation",
                "profile_outcome",
                "runtime_observations",
                "vector_id",
            ]
            || row["vector_id"] != **vector_id
        {
            return Err("runtime vector index".to_owned());
        }
        let vector = run_vectors[0][vector_id];
        if row["candidate_observation_sha256"]
            != canonical_sha256(&runtime_semantic_projection(vector))
            || row["family"] != vector["family"]
            || row["operation"] != vector["operation"]
            || row["profile_outcome"] != vector["profile_outcome"]
            || row["culture_invariant_runtime"] != !actual_variant_ids.contains(vector_id)
        {
            return Err("runtime vector index link".to_owned());
        }
        let runtime_rows = array(&row["runtime_observations"]);
        if runtime_rows.len() != 3 {
            return Err("runtime vector observations".to_owned());
        }
        for (ordinal, profile) in ["hostile-arabic", "hostile-comma", "hostile-swap"]
            .iter()
            .enumerate()
        {
            let runtime_row = &runtime_rows[ordinal];
            if object(runtime_row)
                .keys()
                .map(String::as_str)
                .collect::<Vec<_>>()
                != ["culture", "observation_sha256"]
                || runtime_row["culture"] != *profile
                || runtime_row["observation_sha256"]
                    != canonical_sha256(&run_vectors[ordinal][vector_id]["differential"])
            {
                return Err("runtime differential link".to_owned());
            }
        }
    }

    let mut operations: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
    let mut families: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
    for vector_id in &vector_ids {
        let vector = run_vectors[0][vector_id];
        operations
            .entry(text(&vector["operation"]))
            .or_default()
            .push(vector_id);
        families
            .entry(text(&vector["family"]))
            .or_default()
            .push(vector_id);
    }
    let operation_index = array(&document["operation_index"]);
    let operation_ids = operation_index
        .iter()
        .map(|row| text(&row["operation"]))
        .collect::<Vec<_>>();
    if operation_index.len() != 154
        || canonical_sha256(&Value::Array(
            operation_ids.iter().map(|id| json!(id)).collect(),
        )) != RUNTIME_OPERATION_IDS_SHA256
        || operation_ids != operations.keys().copied().collect::<Vec<_>>()
    {
        return Err("runtime operation catalog".to_owned());
    }
    for row in operation_index {
        if object(row).keys().map(String::as_str).collect::<Vec<_>>()
            != [
                "accepted_domain",
                "error_precedence",
                "families",
                "observed_error_ids",
                "operation",
                "possible_failures",
                "profile_outcomes",
                "result_encodings",
                "vector_ids",
                "vector_ids_sha256",
            ]
        {
            return Err("runtime operation index keys".to_owned());
        }
        let operation = text(&row["operation"]);
        let expected_ids = &operations[operation];
        let vectors = expected_ids
            .iter()
            .map(|id| run_vectors[0][id])
            .collect::<Vec<_>>();
        let first = vectors[0];
        if vectors.iter().any(|vector| {
            vector["accepted_domain"] != first["accepted_domain"]
                || vector["error_precedence"] != first["error_precedence"]
        }) {
            return Err("runtime operation contract drift".to_owned());
        }
        let expected_families = vectors
            .iter()
            .map(|vector| text(&vector["family"]))
            .collect::<BTreeSet<_>>();
        let expected_errors = vectors
            .iter()
            .filter_map(|vector| vector["profile"]["error_id"].as_str())
            .collect::<BTreeSet<_>>();
        let expected_outcomes = vectors
            .iter()
            .map(|vector| text(&vector["profile_outcome"]))
            .collect::<BTreeSet<_>>();
        let expected_encodings = vectors
            .iter()
            .map(|vector| text(&vector["profile"]["result_encoding"]))
            .collect::<BTreeSet<_>>();
        if *row
            != json!({
                "accepted_domain": first["accepted_domain"],
                "error_precedence": first["error_precedence"],
                "families": expected_families,
                "observed_error_ids": expected_errors,
                "operation": operation,
                "possible_failures": first["error_precedence"],
                "profile_outcomes": expected_outcomes,
                "result_encodings": expected_encodings,
                "vector_ids": expected_ids,
                "vector_ids_sha256": canonical_sha256(&json!(expected_ids)),
            })
        {
            return Err("runtime operation index link".to_owned());
        }
        let possible = runtime_string_set(&row["possible_failures"]);
        let observed = runtime_string_set(&row["observed_error_ids"]);
        let required = possible
            .iter()
            .filter(|value| {
                value.starts_with("exception.")
                    || value.starts_with("parse_error.")
                    || value.starts_with("source_rejection.")
            })
            .cloned()
            .collect::<BTreeSet<_>>();
        if !operation.starts_with("precedence.") && observed != required {
            return Err("runtime operation error closure".to_owned());
        }
    }

    let family_index = array(&document["family_index"]);
    let family_ids = family_index
        .iter()
        .map(|row| text(&row["family"]))
        .collect::<Vec<_>>();
    if family_index.len() != 26
        || canonical_sha256(&Value::Array(
            family_ids.iter().map(|id| json!(id)).collect(),
        )) != RUNTIME_FAMILY_IDS_SHA256
        || family_ids != families.keys().copied().collect::<Vec<_>>()
    {
        return Err("runtime family catalog".to_owned());
    }
    for row in family_index {
        if object(row).keys().map(String::as_str).collect::<Vec<_>>()
            != ["family", "operation_ids", "vector_ids", "vector_ids_sha256"]
        {
            return Err("runtime family index keys".to_owned());
        }
        let family = text(&row["family"]);
        let expected_ids = &families[family];
        let expected_operations = expected_ids
            .iter()
            .map(|id| text(&run_vectors[0][id]["operation"]))
            .collect::<BTreeSet<_>>();
        if *row
            != json!({
                "family": family,
                "operation_ids": expected_operations,
                "vector_ids": expected_ids,
                "vector_ids_sha256": canonical_sha256(&json!(expected_ids)),
            })
        {
            return Err("runtime family index link".to_owned());
        }
    }

    if array(&document["culture_variants"]).len() != actual_variant_ids.len()
        || array(&document["culture_variants"])
            .iter()
            .map(|row| text(&row["vector_id"]))
            .collect::<Vec<_>>()
            != actual_variant_ids
    {
        return Err("runtime culture variant links".to_owned());
    }
    for row in array(&document["culture_variants"]) {
        let vector_id = text(&row["vector_id"]);
        let vector = run_vectors[0][vector_id];
        let expected_observations = ["hostile-arabic", "hostile-comma", "hostile-swap"]
            .iter()
            .enumerate()
            .map(|(ordinal, profile)| {
                json!({
                    "culture": profile,
                    "observation_sha256": canonical_sha256(
                        &run_vectors[ordinal][vector_id]["differential"]
                    ),
                })
            })
            .collect::<Vec<_>>();
        if *row
            != json!({
                "family": vector["family"],
                "operation": vector["operation"],
                "runtime_observations": expected_observations,
                "vector_id": vector_id,
            })
        {
            return Err("runtime culture variant payload".to_owned());
        }
    }
    let mutations = array(&document["upgrade_mutations"]);
    if mutations.len() != operations.len() {
        return Err("runtime mutation count".to_owned());
    }
    let mut mutation_operations = BTreeSet::new();
    for row in mutations {
        if object(row).keys().map(String::as_str).collect::<Vec<_>>()
            != [
                "candidate_observation_sha256",
                "families",
                "mutation_field",
                "mutation_id",
                "operation",
                "vector_id",
            ]
        {
            return Err("runtime mutation keys".to_owned());
        }
        let vector_id = text(&row["vector_id"]);
        let vector = run_vectors[0]
            .get(vector_id)
            .ok_or_else(|| "runtime mutation vector".to_owned())?;
        let operation = text(&vector["operation"]);
        let expected_families = operations[operation]
            .iter()
            .map(|id| text(&run_vectors[0][id]["family"]))
            .collect::<BTreeSet<_>>();
        if row["mutation_field"] != "inputs[0]"
            || row["candidate_observation_sha256"]
                != canonical_sha256(&runtime_semantic_projection(vector))
            || row["operation"] != vector["operation"]
            || row["families"] != json!(expected_families)
            || row["mutation_id"]
                != format!(
                    "CSHARP-03-T01-W07-RUNTIME-INPUT-{}",
                    operation.to_uppercase().replace(['.', '_'], "-")
                )
            || !mutation_operations.insert(text(&row["operation"]))
        {
            return Err("runtime mutation link".to_owned());
        }
    }
    if mutation_operations != operations.keys().copied().collect() {
        return Err("runtime mutation closure".to_owned());
    }

    let covered = array(&document["coverage"])
        .iter()
        .flat_map(|row| array(&row["families"]).iter().map(text))
        .collect::<BTreeSet<_>>();
    if covered != families.keys().copied().collect() {
        return Err("runtime coverage closure".to_owned());
    }
    Ok(())
}

// CSHARP-03-T01-W07
#[test]
fn canonical_runtime_probe_closes_operations_errors_and_culture_differentials() {
    let raw = bytes(RUNTIME_RESULT_PATH);
    assert_eq!(raw.len(), RUNTIME_RESULT_SIZE);
    assert_eq!(sha256(&raw), RUNTIME_RESULT_SHA256);
    let document: Value = serde_json::from_slice(&raw).expect("parse runtime probe");
    let mut canonical = canonical_bytes(&document);
    canonical.push(b'\n');
    assert_eq!(
        raw, canonical,
        "runtime probe must be canonical JSON plus LF"
    );
    validate_runtime_document(&document).unwrap();
    assert_eq!(
        sha256(&bytes(
            "develop/probes/csharp-03/PrimitiveStringNumericCodecProbe.cs"
        )),
        RUNTIME_PROBE_SOURCE_SHA256
    );
}

// CSHARP-03-T01-W07
#[test]
fn runtime_probe_freezes_utf16_codecs_float_decimal_and_precedence() {
    let document = load_runtime_probe();
    validate_runtime_document(&document).unwrap();

    assert_eq!(
        runtime_vector(&document, "string.literal.lone_high")["profile"]["value"],
        "d800"
    );
    assert_eq!(
        runtime_vector(&document, "string.length.pair")["profile"]["value"],
        "2"
    );
    assert_eq!(
        runtime_vector(&document, "string.concat.operator.string_string.both_null")["profile"]
            ["value"],
        ""
    );
    let char_char = runtime_vector(&document, "string.concat.char_char");
    assert_eq!(char_char["profile"]["kind"], "rejected");
    assert_eq!(
        char_char["profile"]["error_id"],
        "source_rejection.concat_char_char"
    );
    assert_eq!(char_char["differential"]["value"], "195");

    assert_eq!(
        runtime_vector(&document, "codec.integer.i32.parse.plus")["profile"]["error_id"],
        "parse_error.noncanonical"
    );
    for vector_id in [
        "codec.integer.i32.parse.plus_malformed",
        "codec.decimal.normalized.parse.plus_malformed",
        "codec.decimal.fixed.parse.syntax_before_scale",
    ] {
        assert_eq!(
            runtime_vector(&document, vector_id)["profile"]["error_id"],
            "parse_error.syntax",
            "syntax must win for {vector_id}"
        );
    }
    assert_eq!(
        runtime_vector(
            &document,
            "codec.decimal.fixed.parse.noncanonical_before_scale"
        )["profile"]["error_id"],
        "parse_error.noncanonical"
    );
    assert_eq!(
        runtime_vector(&document, "codec.decimal.normalized.parse.scale_29")["profile"]["error_id"],
        "parse_error.scale_precision"
    );
    assert_eq!(
        runtime_vector(&document, "codec.binary32.parse.signaling_nan")["profile"]["value"],
        "7fa12345"
    );
    assert_eq!(
        runtime_vector(&document, "codec.binary64.parse.negative_zero")["profile"]["value"],
        "8000000000000000"
    );
    assert_eq!(
        runtime_vector(&document, "codec.guid.d.parse.uppercase")["profile"]["error_id"],
        "parse_error.noncanonical"
    );
    assert_eq!(
        runtime_vector(
            &document,
            "codec.decimal.fixed.roundtrip.n1_255.awayfromzero"
        )["profile"]["value"],
        "true"
    );
    assert_eq!(
        runtime_vector(&document, "codec.decimal.fixed.parse.maximum_scale28")["profile"]["value"],
        "sign=0;scale=00;coefficient=ffffffffffffffffffffffff"
    );
    assert_eq!(
        runtime_vector(
            &document,
            "codec.decimal.fixed.parse.unrepresentable_coefficient"
        )["profile"]["error_id"],
        "parse_error.range"
    );
    for endpoint in [
        "integer_scale2",
        "maximum_scale2",
        "maximum_scale28",
        "minimum_scale28",
        "negative_zero_scale28",
        "least_fraction_scale28",
        "round_to_zero_scale0",
    ] {
        for mode in [
            "toeven",
            "awayfromzero",
            "tozero",
            "tonegativeinfinity",
            "topositiveinfinity",
        ] {
            let vector_id = format!("codec.decimal.fixed.roundtrip.{endpoint}.{mode}");
            assert_eq!(
                runtime_vector(&document, &vector_id)["profile"]["value"],
                "true",
                "fixed-scale endpoint round trip: {vector_id}"
            );
        }
    }

    assert_eq!(
        runtime_vector(&document, "floating.single.add.v08.v05")["profile"]["value"],
        "7fe12345"
    );
    assert_eq!(
        runtime_vector(&document, "floating.single.min.v02.v03")["profile"]["value"],
        "80000000"
    );
    assert_eq!(
        runtime_vector(&document, "floating.single.max.v02.v03")["profile"]["value"],
        "00000000"
    );
    assert_eq!(
        runtime_vector(&document, "decimal.edge.max_plus_one")["operation"],
        "decimal.add"
    );
    assert_eq!(
        runtime_vector(&document, "decimal.edge.max_plus_one")["profile"]["error_id"],
        "exception.overflow"
    );
    assert_eq!(
        runtime_vector(&document, "decimal.edge.max_divide_fraction")["profile"]["error_id"],
        "exception.overflow"
    );
    assert_eq!(
        runtime_vector(&document, "decimal.equivalence.negative_zero")["profile"]["value"],
        "true"
    );

    let sidecar = runtime_vector(&document, "precedence.sidecar.codec_before_rounding");
    assert_eq!(sidecar["profile"]["error_id"], "sidecar.unknown_codec");
    assert_eq!(
        array(&sidecar["error_precedence"])
            .iter()
            .map(text)
            .take(2)
            .collect::<Vec<_>>(),
        ["sidecar.unknown_codec", "sidecar.unknown_rounding_mode"]
    );

    let runs = array(&document["observations"]["culture_runs"]);
    let interpolation = runs
        .iter()
        .map(|run| {
            text(
                &runtime_vector_in_run(run, "string.interpolation.numeric")["differential"]
                    ["value"],
            )
        })
        .collect::<BTreeSet<_>>();
    assert_eq!(interpolation.len(), 3, "hostile cultures must disagree");
    for run in runs {
        let vector = runtime_vector_in_run(run, "string.interpolation.numeric");
        assert_eq!(vector["profile"]["kind"], "rejected");
        assert_eq!(
            vector["profile"]["error_id"],
            "source_rejection.interpolation_hole_type"
        );
    }
}

// CSHARP-03-T01-W07
#[test]
fn runtime_probe_mutations_fail_closed() {
    let document = load_runtime_probe();
    validate_runtime_document(&document).unwrap();
    for (vector_id, field) in [
        ("floating.single.add.v00.v00", "value"),
        ("codec.integer.i32.parse.plus", "error_id"),
        ("string.concat.char_char", "error_id"),
    ] {
        let mut changed = document.clone();
        let run = &mut changed["observations"]["culture_runs"][0];
        let vector = run["vectors"]
            .as_array_mut()
            .expect("runtime vectors")
            .iter_mut()
            .find(|vector| vector["id"] == vector_id)
            .expect("mutation vector");
        let original = text(&vector["profile"][field]).to_owned();
        vector["profile"][field] = json!(original + "#mutation");
        assert!(
            validate_runtime_document(&changed).is_err(),
            "runtime mutation accepted for {vector_id}"
        );
    }
    for (index, field) in [
        ("operation_index", "accepted_domain"),
        ("family_index", "operation_ids"),
        ("culture_variants", "runtime_observations"),
        ("upgrade_mutations", "mutation_id"),
        ("vector_index", "culture_invariant_runtime"),
    ] {
        let mut changed = document.clone();
        changed[index][0][field] = json!("index mutation");
        assert!(
            validate_runtime_document(&changed).is_err(),
            "runtime index mutation accepted for {index}.{field}"
        );
    }
}

// CSHARP-03-T01-W07
#[test]
fn runtime_probe_preserves_w06_and_the_active_release_boundary() {
    assert_eq!(
        sha256(&bytes(DEPENDENCY_RESULT_PATH)),
        DEPENDENCY_RESULT_SHA256
    );
    assert_eq!(
        sha256(&bytes(
            "develop/probes/csharp-03/DependencyGenericSuspensionProbe.cs"
        )),
        DEPENDENCY_PROBE_SOURCE_SHA256
    );
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
        assert!(!content.contains("runtime_semantics_probe"));
    }
}

// CSHARP-03-T01-W07
#[test]
fn pinned_runtime_probe_rerun_is_byte_identical_when_the_linux_cache_is_available() {
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
        repository_root()
            .join("develop/probes/csharp-03/run-primitive-string-numeric-codec-probe.sh"),
    )
    .arg("--check")
    .env_clear()
    .env("PATH", "/usr/bin:/bin")
    .output()
    .expect("execute pinned W07 runtime probe");
    assert!(
        output.status.success(),
        "stdout={} stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output.stdout.is_empty());
    assert!(output.stderr.is_empty());
}
