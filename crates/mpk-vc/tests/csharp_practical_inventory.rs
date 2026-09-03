use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

const BASELINE_PATH: &str = "develop/migrations/csharp-03/baseline.json";
const LEDGER_PATH: &str = "develop/docs/csharp-03-implementation-traceability-ledger.md";
const PLAN_PATH: &str = "develop/docs/08_csharp_practical_subset_design-todo.md";

#[test]
fn csharp_03_t01_w01_baseline_recomputes_the_active_release() {
    let baseline = read_json(BASELINE_PATH);
    assert_exact_keys(
        &baseline,
        &[
            "schema",
            "work_item",
            "captured_on",
            "entry_snapshot",
            "java_t10",
            "active_release",
            "checkers",
            "axiom_inventory",
            "test_corpus",
            "absence_assertions",
        ],
    );
    assert_eq!(
        text(&baseline["schema"]),
        "mpk.csharp_practical.baseline.v1"
    );
    assert_eq!(text(&baseline["work_item"]), "CSHARP-03-T01-W01");
    assert_eq!(text(&baseline["captured_on"]), "2026-09-03");

    let entry = &baseline["entry_snapshot"];
    assert_eq!(text(&entry["branch"]), "main");
    assert_eq!(entry["commit"], "4d593f56f8750d151d9fe42627a84e9e4842d1cc");
    assert_eq!(entry["tree"], "43164f9a70793b32743df90a39d99f289c481504");
    assert_eq!(entry["worktree_clean"], true);
    assert_eq!(
        baseline["checkers"]["source_free"]["source_commit"],
        entry["commit"]
    );

    let hashed_paths = assert_all_recorded_raw_hashes(&baseline);
    assert_eq!(
        hashed_paths, 21,
        "the W01 baseline must bind every recorded registry, report, receipt, checker, gate, and corpus file"
    );
    assert_sha256_lengths(&baseline, "baseline");

    let semantic_registry = read_json(text(
        &baseline["active_release"]["semantic_registry"]["path"],
    ));
    let bundle_registry = read_json(text(&baseline["active_release"]["bundle_registry"]["path"]));
    let release_report = read_json(text(&baseline["active_release"]["release_report"]["path"]));
    let successor = &release_report["successor_release"];

    assert_eq!(baseline["active_release"]["status"], successor["status"]);
    assert_eq!(
        baseline["active_release"]["proof_authority"],
        successor["proof_authority"]
    );
    assert_eq!(successor["status"], "active_successor");
    assert_eq!(successor["proof_authority"], "certificate_only");
    assert_eq!(release_report["release_gates"]["passed"], true);

    let semantic_baseline = &baseline["active_release"]["semantic_registry"];
    for key in ["schema", "id", "revision", "registry_sha256"] {
        assert_eq!(semantic_baseline[key], semantic_registry[key], "{key}");
        assert_eq!(
            semantic_baseline[key], successor["semantic_registry"][key],
            "{key}"
        );
    }
    assert_eq!(
        baseline["active_release"]["profiles"],
        project_profiles(&semantic_registry)
    );

    let bundle_baseline = &baseline["active_release"]["bundle_registry"];
    for key in ["schema", "id", "registry_sha256"] {
        assert_eq!(bundle_baseline[key], bundle_registry[key], "{key}");
        assert_eq!(
            bundle_baseline[key], successor["bundle_registry"][key],
            "{key}"
        );
    }
    assert_eq!(
        baseline["active_release"]["frontend_bundles"],
        project_frontend_bundles(&bundle_registry)
    );
    assert_eq!(
        baseline["active_release"]["toolchain_bundles"],
        project_toolchain_bundles(&bundle_registry)
    );
    assert_eq!(
        baseline["active_release"]["tuples"],
        project_tuples(&bundle_registry)
    );
    assert_eq!(
        baseline["active_release"]["candidate_projections"],
        project_candidate_projections(successor)
    );

    assert_java_t10_link(&baseline, successor);
    assert_checker_and_axiom_baseline(&baseline, &release_report);
    assert_test_corpus(&baseline);

    let active_registry_bytes = format!("{semantic_registry}{bundle_registry}");
    let forbidden = array(&baseline["absence_assertions"]["forbidden_active_identifiers"]);
    assert_eq!(
        baseline["absence_assertions"]["result"],
        "no_csharp_practical_identity"
    );
    for identifier in forbidden {
        let identifier = text(identifier);
        assert!(
            !active_registry_bytes.contains(identifier),
            "active registry unexpectedly contains {identifier}"
        );
    }
}

#[test]
fn csharp_03_t01_w01_ledger_has_one_owner_and_status_per_work_item() {
    let plan = read_text(PLAN_PATH);
    let ledger = read_text(LEDGER_PATH);
    let planned_items = planned_work_items(&plan);
    let routed_owners = planned_primary_owners(&plan);
    let rows = ledger_rows(&ledger);

    assert_eq!(
        planned_items.len(),
        73,
        "the reviewed plan must retain 73 W items"
    );
    assert_eq!(
        routed_owners.len(),
        73,
        "every planned W item needs one routed owner"
    );
    assert_eq!(
        rows.len(),
        73,
        "the ledger needs exactly one row per W item"
    );
    assert_eq!(
        rows.keys().cloned().collect::<BTreeSet<_>>(),
        planned_items,
        "ledger and plan work-item sets differ"
    );

    for (work_item, row) in &rows {
        let expected_owner = routed_owners
            .get(work_item)
            .unwrap_or_else(|| panic!("missing routed owner for {work_item}"));
        assert_eq!(
            &row.owner, expected_owner,
            "primary owner drift for {work_item}"
        );

        let expected_status = match work_item.as_str() {
            "CSHARP-03-T01-W01" => "Complete",
            "CSHARP-03-T01-W02" => "Ready",
            _ => "Blocked",
        };
        assert_eq!(row.status, expected_status, "status drift for {work_item}");
        let expected_commit = if work_item == "CSHARP-03-T01-W01" {
            "SELF"
        } else {
            "—"
        };
        assert_eq!(
            row.commit, expected_commit,
            "commit marker drift for {work_item}"
        );
    }

    for required in [
        "## 3. CSHARP-03-T01-W01 completion record",
        "develop/migrations/csharp-03/baseline.json",
        "develop/migrations/archive/java-03-t10-native-receipt.json",
        "JAVA-03-T10",
        "./scripts/check-fast.sh",
        "Native x86-64 Linux gate",
        "Final review findings: `0`",
    ] {
        assert!(ledger.contains(required), "ledger is missing {required}");
    }
}

fn assert_java_t10_link(baseline: &Value, successor: &Value) {
    let java = &baseline["java_t10"];
    assert_eq!(java["commit"], "b7102c1acfcacdbf45b3d5a3ef21aac1ccf56f64");
    assert_eq!(java["tree"], "e139c6f9793929d68997fd40909f74f25e3ace53");
    assert_eq!(
        java["receipt"]["git_blob_sha1"],
        "de38839dd599d57425f23234ba512660c6b160b9"
    );

    let receipt = read_json(text(&java["receipt"]["path"]));
    assert_eq!(java["receipt"]["schema"], receipt["schema"]);
    assert_eq!(java["receipt"]["status"], receipt["status"]);

    let native = &successor["native_acceptance"];
    assert_eq!(native["path"], java["receipt"]["path"]);
    assert_eq!(native["sha256"], java["receipt"]["raw_sha256"]);
    let mut recorded_receipt = native.clone();
    let object = recorded_receipt
        .as_object_mut()
        .expect("native acceptance must be an object");
    object.remove("path");
    object.remove("sha256");
    assert_eq!(
        recorded_receipt, receipt,
        "release report must reproduce the receipt"
    );
    assert_eq!(receipt["status"], "accepted");
    assert_eq!(receipt["architecture"], "x86_64");
    assert_eq!(receipt["installed_release_passes"], 2);
    assert_eq!(receipt["exit_code"], 0);
}

fn assert_checker_and_axiom_baseline(baseline: &Value, report: &Value) {
    let certificate = &report["certificates"][0];
    let source_free = &baseline["checkers"]["source_free"];
    let reference = &baseline["checkers"]["reference"];
    let fixture = &baseline["checkers"]["agreement_fixture"];
    assert_eq!(source_free["id"], "mpk-cli/check");
    assert_eq!(source_free["workspace_version"], "0.1.0");
    assert_eq!(
        source_free["command"],
        "cargo run --quiet -p mpk-cli -- check"
    );
    assert_eq!(
        reference["id"],
        "github.com/finitefield-org/mpk/go-tools/mpk-checker-ref"
    );
    assert_eq!(reference["language_version"], "1.23");
    assert_eq!(reference["format"], "elf64-x86_64-static-stripped");
    assert_eq!(reference["command"], "./scripts/check-reference.sh");
    assert_eq!(
        reference["command_raw_sha256"],
        hex_sha256(&fs::read(repo_path("scripts/check-reference.sh")).expect("reference gate"))
    );
    let go_module = read_text("go-tools/mpk-checker-ref/go.mod");
    assert!(go_module.contains(&format!("module {}", text(&reference["id"]))));
    assert!(go_module.contains(&format!("go {}", text(&reference["language_version"]))));
    assert_eq!(fixture["path"], certificate["path"]);
    assert_eq!(fixture["module"], certificate["module"]);
    assert_eq!(
        fixture["export_sha256"],
        certificate["expected_hashes"]["export"]
    );
    assert_eq!(
        fixture["axiom_report_sha256"],
        certificate["expected_hashes"]["axiom_report"]
    );
    assert_eq!(
        fixture["certificate_sha256"],
        certificate["expected_hashes"]["certificate"]
    );
    assert_eq!(
        baseline["checkers"]["source_free"]["verdict"],
        certificate["source_free_checker"]["verdict"]
    );
    assert_eq!(
        baseline["checkers"]["reference"]["verdict"],
        certificate["reference_checker"]["verdict"]
    );
    assert_eq!(
        certificate["hash_agreement"]["manifest_matches_source_free"],
        true
    );
    assert_eq!(
        certificate["hash_agreement"]["source_free_matches_axiom_report"],
        true
    );
    assert_eq!(
        certificate["hash_agreement"]["source_free_matches_reference"],
        true
    );

    let summary = &certificate["axiom_report"]["report"]["summary"];
    let inventory = &baseline["axiom_inventory"];
    assert_eq!(
        inventory["source"],
        "release-report.json#/certificates/0/axiom_report/report/summary"
    );
    for key in [
        "core_axiom_count",
        "builtin_theory_axiom_count",
        "go_semantics_axiom_count",
        "external_axiom_count",
        "total_axiom_count",
    ] {
        assert_eq!(inventory[key], summary[key], "axiom drift for {key}");
        assert_eq!(inventory[key], 0, "W01 must inherit a zero-axiom release");
    }
}

fn assert_test_corpus(baseline: &Value) {
    let corpus = &baseline["test_corpus"];
    assert_eq!(
        corpus["release_gate"]["command"],
        "./scripts/check-java-frontend.sh"
    );
    assert_eq!(
        corpus["release_gate"]["documented_privileged_invocation"],
        "sudo ./scripts/check-java-frontend.sh"
    );
    assert_eq!(corpus["release_gate"]["installed_release_passes"], 2);
    assert_eq!(
        corpus["aggregate_gate"]["command"],
        "./scripts/check-all.sh"
    );
    assert_eq!(
        corpus["aggregate_gate"]["delegates_to"],
        "./scripts/check-java-frontend.sh"
    );

    let release_gate = read_text(text(&corpus["release_gate"]["path"]));
    let aggregate_gate = read_text(text(&corpus["aggregate_gate"]["path"]));
    assert!(aggregate_gate.contains("scripts/check-java-frontend.sh"));
    for language in ["go", "rust", "csharp_scalar", "java_scalar"] {
        let targets = array(&corpus["per_language_test_targets"][language]);
        assert!(!targets.is_empty(), "{language} corpus is empty");
        for target in targets {
            let target = text(target);
            assert!(
                release_gate.contains(target),
                "release gate no longer executes {language} target {target}"
            );
        }
    }
    assert_eq!(
        corpus["common_checks"],
        json!([
            "cargo fmt --all -- --check",
            "cargo clippy --workspace --all-targets -- -D warnings",
            "cargo test --workspace",
            "cargo test --locked -p mpk-api v2_tests",
            "./scripts/check-release-bundles.sh --fixture successor",
            "./scripts/check-reference.sh",
            "scripts/check-artifact-paths.py",
            "scripts/check-spec-vectors.py",
            "./scripts/check-fuzz-smoke.sh",
            "./scripts/check-no-active-gir.sh",
            "scripts/generate-release-report.py"
        ])
    );
}

fn project_profiles(registry: &Value) -> Value {
    Value::Array(
        array(&registry["profiles"])
            .iter()
            .map(|profile| {
                json!({
                    "source_language": profile["source_language"],
                    "semantic_profile": profile["semantic_profile"],
                    "entry_sha256": profile["entry_sha256"],
                })
            })
            .collect(),
    )
}

fn project_frontend_bundles(registry: &Value) -> Value {
    Value::Array(
        array(&registry["frontend_bundles"])
            .iter()
            .map(|bundle| {
                json!({
                    "bundle_id": bundle["bundle_id"],
                    "bundle_sha256": bundle["bundle_sha256"],
                    "main_binary_sha256": bundle["main"]["binary_sha256"],
                })
            })
            .collect(),
    )
}

fn project_toolchain_bundles(registry: &Value) -> Value {
    Value::Array(
        array(&registry["toolchain_bundles"])
            .iter()
            .map(|bundle| {
                json!({
                    "bundle_id": bundle["bundle_id"],
                    "distribution_sha256": bundle["distribution_sha256"],
                    "execution_host_profile_id": bundle["execution_host_profile_id"],
                })
            })
            .collect(),
    )
}

fn project_tuples(registry: &Value) -> Value {
    Value::Array(
        array(&registry["tuples"])
            .iter()
            .map(|tuple| {
                let context = &tuple["semantic_context"];
                json!({
                    "source_language": context["source_language"],
                    "semantic_profile": context["semantic_profile"],
                    "semantic_parameters": context["semantic_parameters"],
                    "profile_entry_sha256": context["profile_entry_sha256"],
                    "frontend_bundle_id": tuple["frontend_bundle_id"],
                    "toolchain_bundle_id": tuple["toolchain_bundle_id"],
                    "limit_profile_id": tuple["limit_profile_id"],
                })
            })
            .collect(),
    )
}

fn project_candidate_projections(successor: &Value) -> Value {
    Value::Array(
        array(&successor["candidate_projections"])
            .iter()
            .map(|projection| {
                json!({
                    "language": projection["language"],
                    "path": projection["path"],
                    "raw_sha256": projection["sha256"],
                    "schema": projection["schema"],
                })
            })
            .collect(),
    )
}

fn assert_all_recorded_raw_hashes(value: &Value) -> usize {
    match value {
        Value::Array(values) => values.iter().map(assert_all_recorded_raw_hashes).sum(),
        Value::Object(object) => {
            let own = match (object.get("path"), object.get("raw_sha256")) {
                (Some(Value::String(path)), Some(Value::String(expected))) => {
                    let bytes = fs::read(repo_path(path))
                        .unwrap_or_else(|error| panic!("failed to read {path}: {error}"));
                    assert_eq!(hex_sha256(&bytes), *expected, "raw hash drift for {path}");
                    1
                }
                _ => 0,
            };
            own + object
                .values()
                .map(assert_all_recorded_raw_hashes)
                .sum::<usize>()
        }
        _ => 0,
    }
}

fn assert_sha256_lengths(value: &Value, pointer: &str) {
    match value {
        Value::Array(values) => {
            for (index, value) in values.iter().enumerate() {
                assert_sha256_lengths(value, &format!("{pointer}/{index}"));
            }
        }
        Value::Object(object) => {
            for (key, value) in object {
                let child = format!("{pointer}/{key}");
                if key.ends_with("sha256") {
                    assert_eq!(
                        text(value).len(),
                        64,
                        "SHA-256 must have 64 lowercase hex characters at {child}"
                    );
                    assert!(
                        text(value)
                            .bytes()
                            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte)),
                        "SHA-256 must be lowercase hexadecimal at {child}"
                    );
                }
                assert_sha256_lengths(value, &child);
            }
        }
        _ => {}
    }
}

#[derive(Debug)]
struct LedgerRow {
    status: String,
    owner: String,
    commit: String,
}

fn planned_work_items(plan: &str) -> BTreeSet<String> {
    let mut items = BTreeSet::new();
    for token in plan
        .lines()
        .filter_map(|line| line.strip_prefix("### "))
        .filter_map(|line| line.split_whitespace().next())
        .filter(|token| token.starts_with("CSHARP-03-T") && token.contains("-W"))
    {
        assert!(
            items.insert(token.to_owned()),
            "duplicate planned item {token}"
        );
    }
    items
}

fn planned_primary_owners(plan: &str) -> BTreeMap<String, String> {
    let section = plan
        .split("### 5.1 Primary test routing")
        .nth(1)
        .expect("primary routing section")
        .split("## 6.")
        .next()
        .expect("end of primary routing section");
    let mut owners = BTreeMap::new();
    for line in section.lines().filter(|line| line.starts_with("| T")) {
        let fields = markdown_fields(line);
        let owner_path = code_span(&fields[1]);
        for work_item in expand_route(&fields[0]) {
            let previous = owners.insert(work_item.clone(), format!("{owner_path}#{work_item}"));
            assert!(
                previous.is_none(),
                "duplicate planned owner for {work_item}"
            );
        }
    }
    owners
}

fn expand_route(spec: &str) -> Vec<String> {
    let spec = spec.trim_matches('`');
    let (stage, works) = spec.split_once('-').expect("Tnn-Wnn route");
    let work_numbers = if let Some((first, last)) = works.split_once('-') {
        let first = parse_work_number(first);
        let last = parse_work_number(last);
        (first..=last).collect::<Vec<_>>()
    } else {
        works.split('/').map(parse_work_number).collect::<Vec<_>>()
    };
    work_numbers
        .into_iter()
        .map(|work| format!("CSHARP-03-{stage}-W{work:02}"))
        .collect()
}

fn parse_work_number(work: &str) -> u8 {
    work.strip_prefix('W')
        .expect("W prefix")
        .parse()
        .expect("numeric W suffix")
}

fn ledger_rows(ledger: &str) -> BTreeMap<String, LedgerRow> {
    let section = ledger
        .split("<!-- work-item-ledger:start -->")
        .nth(1)
        .expect("ledger start marker")
        .split("<!-- work-item-ledger:end -->")
        .next()
        .expect("ledger end marker");
    let mut rows = BTreeMap::new();
    for line in section
        .lines()
        .filter(|line| line.starts_with("| `CSHARP-03-T"))
    {
        let fields = markdown_fields(line);
        assert_eq!(fields.len(), 4, "closed ledger row schema");
        let work_item = fields[0].trim_matches('`').to_owned();
        let row = LedgerRow {
            status: fields[1].trim_matches('`').to_owned(),
            owner: fields[2].trim_matches('`').to_owned(),
            commit: fields[3].trim_matches('`').to_owned(),
        };
        assert!(
            rows.insert(work_item.clone(), row).is_none(),
            "duplicate {work_item}"
        );
    }
    rows
}

fn markdown_fields(line: &str) -> Vec<String> {
    line.trim_matches('|')
        .split('|')
        .map(|field| field.trim().to_owned())
        .collect()
}

fn code_span(field: &str) -> &str {
    field
        .split('`')
        .nth(1)
        .expect("owner path must be the first code span")
}

fn assert_exact_keys(value: &Value, expected: &[&str]) {
    let actual = value
        .as_object()
        .expect("expected object")
        .keys()
        .map(String::as_str)
        .collect::<BTreeSet<_>>();
    let expected = expected.iter().copied().collect::<BTreeSet<_>>();
    assert_eq!(actual, expected);
}

fn array(value: &Value) -> &[Value] {
    value.as_array().expect("expected array")
}

fn text(value: &Value) -> &str {
    value.as_str().expect("expected string")
}

fn read_json(relative: &str) -> Value {
    serde_json::from_slice(
        &fs::read(repo_path(relative))
            .unwrap_or_else(|error| panic!("failed to read {relative}: {error}")),
    )
    .unwrap_or_else(|error| panic!("failed to parse {relative}: {error}"))
}

fn read_text(relative: &str) -> String {
    fs::read_to_string(repo_path(relative))
        .unwrap_or_else(|error| panic!("failed to read {relative}: {error}"))
}

fn repo_path(relative: impl AsRef<Path>) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(relative)
}

fn hex_sha256(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}
