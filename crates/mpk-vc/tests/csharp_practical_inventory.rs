use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

const BASELINE_PATH: &str = "develop/migrations/csharp-03/baseline.json";
const INVENTORY_PATH: &str = "develop/migrations/csharp-03/artifact-consumer-inventory.json";
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
            "CSHARP-03-T01-W01" | "CSHARP-03-T01-W02" | "CSHARP-03-T01-W03"
            | "CSHARP-03-T01-W04" | "CSHARP-03-T01-W05" | "CSHARP-03-T01-W06"
            | "CSHARP-03-T01-W07" | "CSHARP-03-T01-W08" | "CSHARP-03-T01-W09" => "Complete",
            "CSHARP-03-T01-W10" => "Ready",
            _ => "Blocked",
        };
        assert_eq!(row.status, expected_status, "status drift for {work_item}");
        let expected_commit = match work_item.as_str() {
            "CSHARP-03-T01-W01" => "17275ffcba4f37d93a74fd188d9860b0a7d5f10d",
            "CSHARP-03-T01-W02" => "f84a5c6ff5122a3a5e64d9305fe999ed1f501f85",
            "CSHARP-03-T01-W03" => "4ad2cd480792d8e7cac71eb798e6b55b66bd97fb",
            "CSHARP-03-T01-W04" => "b6680168c2666be503741575c009f0a26dd0da22",
            "CSHARP-03-T01-W05" => "13415911853c0368c103bd9d5feeb8374596d724",
            "CSHARP-03-T01-W06" => "22673dbc96d8ba4f0d9a4cb97c3f2490c00d1804",
            "CSHARP-03-T01-W07" => "b0ff7daec663b95b1f88ecc1d98f0b7c1f6fdf00",
            "CSHARP-03-T01-W08" => "4ffd8b3a9918b6cae9e4d4704e4bc6b09a12cd5c",
            "CSHARP-03-T01-W09" => "SELF",
            _ => "—",
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
        "## 4. CSHARP-03-T01-W02 completion record",
        "develop/migrations/csharp-03/artifact-consumer-inventory.json",
        "Repository search fixtures: `136`",
        "inventory records 67 explicit",
        "bind 4,922 family-to-path consumer hits",
        "## 5. CSHARP-03-T01-W03 completion record",
        "develop/migrations/csharp-03/build-inputs/build-inputs.json",
        "develop/migrations/csharp-03/build-inputs/candidate-inventory.json",
        "Candidate deterministic USTAR SHA-256",
        "Private mutation checks: `12`",
        "## 6. CSHARP-03-T01-W04 completion record",
        "develop/migrations/csharp-03/probes/roslyn-data-construction.json",
        "Fourteen isolated compilation units record 181 distinct target shapes",
        "Every admitted target has a separately named upgrade mutation",
        "## 7. CSHARP-03-T01-W05 completion record",
        "develop/migrations/csharp-03/probes/roslyn-control-exception-pattern.json",
        "Eighteen isolated compilation units record 103 distinct source shapes",
        "All 40 decision graphs and 25 exception regions have distinct upgrade mutations",
        "## 8. CSHARP-03-T01-W06 completion record",
        "develop/migrations/csharp-03/probes/roslyn-dependency-generic-suspension.json",
        "Sixteen isolated compilation units record 144 distinct source shapes",
        "All 144 source shapes have distinct upgrade mutations",
        "value-type `T?` exception is immediately specialized",
        "Iterator and async observations are rejection-only",
        "## 9. CSHARP-03-T01-W07 completion record",
        "develop/migrations/csharp-03/probes/runtime-primitive-string-numeric-codec.json",
        "3,468 distinct runtime vectors",
        "154 exact operations and 26 evidence families",
        "83 culture-varying BCL differential vectors",
        "Profile-side codec results come only from the probe's closed ASCII grammars",
        "## 10. CSHARP-03-T01-W08 completion record",
        "develop/specs/CSHARP_PRACTICAL_FOUNDATION_V1.md",
        "2,051 executable specification vectors",
        "1,629 independent runtime vectors",
        "## 11. CSHARP-03-T01-W09 feasibility finding and resolution record",
        "CSHARP-03-T01-W09-F01",
        "Feasibility-amendment review findings: `0`",
        "## 12. CSHARP-03-T01-W09 completion record",
        "develop/migrations/csharp-03/freeze/profile-freeze.json",
        "700 sorted rows",
        "12 cases x 2 checkers x 2 runs = 48",
        "Final review findings: `0`",
    ] {
        assert!(ledger.contains(required), "ledger is missing {required}");
    }
}

#[test]
fn csharp_03_t01_w02_inventory_closes_every_artifact_and_consumer_edge() {
    let inventory = read_json(INVENTORY_PATH);
    assert_exact_keys(
        &inventory,
        &[
            "schema",
            "work_item",
            "observed_source",
            "route_classes",
            "identity_families",
            "explicit_edges",
            "search_policy",
            "bundle_members",
            "cli_routes",
            "api_routes",
            "atomic_migration_set",
            "whole_image_rollback_set",
            "closure",
        ],
    );
    assert_eq!(
        text(&inventory["schema"]),
        "mpk.csharp_practical.artifact_consumer_inventory.v1"
    );
    assert_eq!(text(&inventory["work_item"]), "CSHARP-03-T01-W02");
    assert_eq!(
        inventory["observed_source"]["commit"],
        "17275ffcba4f37d93a74fd188d9860b0a7d5f10d"
    );
    assert_eq!(
        inventory["observed_source"]["tree"],
        "957b38264b0e149fa6050b0c5d692ee4b1761001"
    );
    assert_eq!(inventory["observed_source"]["baseline"], BASELINE_PATH);

    let route_classes = object(&inventory["route_classes"]);
    assert_eq!(
        route_classes
            .keys()
            .map(String::as_str)
            .collect::<BTreeSet<_>>(),
        BTreeSet::from(["active", "private"])
    );

    let required_families = string_set(&inventory["closure"]["required_families"]);
    assert_eq!(
        required_families,
        BTreeSet::from([
            "semantic_registry".to_owned(),
            "semantic_context".to_owned(),
            "semantic_parameters".to_owned(),
            "selection".to_owned(),
            "profile_contract".to_owned(),
            "source_artifact".to_owned(),
            "foundation".to_owned(),
            "vir".to_owned(),
            "frontend_protocol".to_owned(),
            "source_map".to_owned(),
            "source_manifest".to_owned(),
            "vc_skeleton".to_owned(),
            "release".to_owned(),
            "policy_evidence".to_owned(),
            "program_assembly".to_owned(),
            "ai".to_owned(),
            "api".to_owned(),
        ])
    );

    let planned_items = planned_work_items(&read_text(PLAN_PATH));
    let search_fixtures = array(&inventory["search_policy"]["fixtures"]);
    assert_eq!(
        search_fixtures.len(),
        136,
        "repository search inventory drift"
    );
    assert_eq!(
        search_fixtures
            .iter()
            .map(|fixture| {
                fixture["expected_count"]
                    .as_u64()
                    .expect("search count must be an unsigned integer")
            })
            .sum::<u64>(),
        4_922,
        "family-to-path inventory total drift"
    );
    let fixture_by_id = search_fixtures
        .iter()
        .map(|fixture| (text(&fixture["id"]).to_owned(), fixture))
        .collect::<BTreeMap<_, _>>();
    let fixture_ids = fixture_by_id.keys().cloned().collect::<BTreeSet<_>>();
    assert_eq!(
        fixture_ids.len(),
        search_fixtures.len(),
        "duplicate search fixture ID"
    );

    let families = array(&inventory["identity_families"]);
    assert_eq!(families.len(), 17, "identity-family inventory drift");
    let mut actual_families = BTreeSet::new();
    let mut referenced_fixtures = BTreeSet::new();
    for family in families {
        assert_exact_keys(
            family,
            &[
                "id",
                "current_identities",
                "current_hash_domains",
                "successor_requirement",
                "identity_freeze_owner",
                "implementation_owners",
                "search_fixture_ids",
            ],
        );
        let family_id = text(&family["id"]);
        assert!(
            actual_families.insert(family_id.to_owned()),
            "duplicate identity family {family_id}"
        );
        assert!(
            !array(&family["current_identities"]).is_empty(),
            "{family_id} lacks a current identity"
        );
        assert_eq!(
            array(&family["current_identities"]).len(),
            string_set(&family["current_identities"]).len(),
            "{family_id} repeats a current identity"
        );
        assert_eq!(
            array(&family["current_hash_domains"]).len(),
            string_set(&family["current_hash_domains"]).len(),
            "{family_id} repeats a current hash domain"
        );
        assert!(
            !text(&family["successor_requirement"]).is_empty(),
            "{family_id} lacks a successor requirement"
        );
        assert_eq!(text(&family["identity_freeze_owner"]), "CSHARP-03-T01-W09");
        assert!(
            !array(&family["implementation_owners"]).is_empty(),
            "{family_id} lacks an implementation owner"
        );
        for owner in array(&family["implementation_owners"]) {
            let owner = text(owner);
            assert!(planned_items.contains(owner), "unknown owner {owner}");
        }
        assert!(
            !array(&family["search_fixture_ids"]).is_empty(),
            "{family_id} lacks a repository search fixture"
        );
        for fixture_id in array(&family["search_fixture_ids"]) {
            let fixture_id = text(fixture_id);
            assert!(
                fixture_ids.contains(fixture_id),
                "{family_id} references missing search fixture {fixture_id}"
            );
            assert!(
                referenced_fixtures.insert(fixture_id.to_owned()),
                "search fixture {fixture_id} is assigned to multiple families"
            );
        }
        for token in array(&family["current_identities"])
            .iter()
            .chain(array(&family["current_hash_domains"]).iter())
            .map(text)
            .filter(|token| {
                token.starts_with("mpk.") || token.starts_with("MPK-") || token.starts_with("Std.")
            })
        {
            assert!(
                array(&family["search_fixture_ids"])
                    .iter()
                    .any(|fixture_id| {
                        let fixture = fixture_by_id
                            .get(text(fixture_id))
                            .expect("family search fixture must exist");
                        token.starts_with(text(&fixture["needle"]))
                    }),
                "{family_id} identity/domain token has no search coverage: {token}"
            );
        }
    }
    assert_eq!(actual_families, required_families);
    assert_eq!(referenced_fixtures, fixture_ids);
    assert_installed_identity_inventory(families);
    assert_current_hash_domain_inventory(families);

    let required_roles = string_set(&inventory["closure"]["required_roles"]);
    let allowed_operations = string_set(&inventory["closure"]["edge_operations"]);
    assert_eq!(
        allowed_operations,
        BTreeSet::from(["hash".to_owned(), "read".to_owned(), "write".to_owned()])
    );
    let migration_set = text(&inventory["atomic_migration_set"]["id"]);
    let mut edge_ids = BTreeSet::new();
    let mut observed_edge_families = BTreeSet::new();
    let mut observed_roles = BTreeSet::new();
    let mut observed_operations = BTreeSet::new();
    let mut observed_routes = BTreeSet::new();
    let explicit_edges = array(&inventory["explicit_edges"]);
    assert_eq!(explicit_edges.len(), 67, "explicit edge inventory drift");
    for edge in explicit_edges {
        assert_exact_keys(
            edge,
            &[
                "id",
                "family",
                "path",
                "anchors",
                "operations",
                "roles",
                "route",
                "migration_owner",
                "migration_set",
            ],
        );
        let edge_id = text(&edge["id"]);
        assert!(
            edge_ids.insert(edge_id.to_owned()),
            "duplicate edge {edge_id}"
        );
        assert!(
            required_families.contains(text(&edge["family"])),
            "edge {edge_id} has unknown family"
        );
        observed_edge_families.insert(text(&edge["family"]).to_owned());
        let path = text(&edge["path"]);
        let contents = read_text(path);
        assert!(
            !array(&edge["anchors"]).is_empty(),
            "edge {edge_id} lacks anchors"
        );
        for anchor in array(&edge["anchors"]) {
            let anchor = text(anchor);
            assert!(
                contents.contains(anchor),
                "edge {edge_id} lost anchor {anchor} in {path}"
            );
        }
        assert!(
            !array(&edge["operations"]).is_empty(),
            "edge {edge_id} lacks operations"
        );
        for operation in array(&edge["operations"]) {
            let operation = text(operation);
            assert!(
                allowed_operations.contains(operation),
                "edge {edge_id} has unknown operation {operation}"
            );
            observed_operations.insert(operation.to_owned());
        }
        assert!(
            !array(&edge["roles"]).is_empty(),
            "edge {edge_id} lacks roles"
        );
        for role in array(&edge["roles"]) {
            let role = text(role);
            assert!(
                required_roles.contains(role),
                "edge {edge_id} has unknown role {role}"
            );
            observed_roles.insert(role.to_owned());
        }
        let route = text(&edge["route"]);
        assert!(
            route_classes.contains_key(route),
            "unknown edge route {route}"
        );
        let (classified_route, _) = classify_search_path(&inventory["search_policy"], path)
            .unwrap_or_else(|| panic!("explicit edge {edge_id} has no path route"));
        assert_eq!(
            route, classified_route,
            "explicit edge {edge_id} route disagrees with path classification"
        );
        observed_routes.insert(route.to_owned());
        assert!(
            planned_items.contains(text(&edge["migration_owner"])),
            "edge {edge_id} has an unknown migration owner"
        );
        assert_eq!(text(&edge["migration_set"]), migration_set);
    }
    assert_eq!(observed_edge_families, required_families);
    assert_eq!(observed_roles, required_roles);
    assert_eq!(observed_operations, allowed_operations);
    assert_eq!(
        observed_routes,
        BTreeSet::from(["active".to_owned(), "private".to_owned()])
    );

    let search_routes = assert_repository_search_fixtures(&inventory, &required_families);
    assert_eq!(
        search_routes,
        BTreeSet::from(["active".to_owned(), "private".to_owned()])
    );
    assert_bundle_members_and_routes(&inventory);
    assert_atomic_migration_and_rollback(&inventory, &required_families, &planned_items);

    let closure = &inventory["closure"];
    assert_exact_keys(
        closure,
        &[
            "required_families",
            "required_roles",
            "edge_operations",
            "unowned_edges",
            "unclassified_routes",
            "unassigned_atomic_members",
            "unassigned_rollback_members",
        ],
    );
    for field in [
        "unowned_edges",
        "unclassified_routes",
        "unassigned_atomic_members",
        "unassigned_rollback_members",
    ] {
        assert!(
            array(&closure[field]).is_empty(),
            "closure field {field} is not empty"
        );
    }
}

#[test]
fn csharp_03_t01_w02_search_fixtures_reject_added_or_deleted_consumers() {
    let inventory = read_json(INVENTORY_PATH);
    let policy = &inventory["search_policy"];
    let search_index = repository_search_index(policy);
    for fixture in array(&policy["fixtures"]) {
        let id = text(&fixture["id"]);
        let observed = repository_search_paths(&search_index, text(&fixture["needle"]));
        assert_search_fingerprint(fixture, &observed).expect("checked-in consumer set");

        let mut deleted = observed.clone();
        deleted
            .pop()
            .unwrap_or_else(|| panic!("search fixture {id} must own a known consumer"));
        assert!(
            assert_search_fingerprint(fixture, &deleted).is_err(),
            "deleting a known consumer must fail search fixture {id}"
        );

        let mut added = observed;
        added.push(format!("fixtures/csharp/__synthetic_{id}_consumer__.json"));
        added.sort();
        assert!(
            assert_search_fingerprint(fixture, &added).is_err(),
            "adding a known consumer must fail search fixture {id}"
        );
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

fn assert_repository_search_fixtures(
    inventory: &Value,
    required_families: &BTreeSet<String>,
) -> BTreeSet<String> {
    let policy = &inventory["search_policy"];
    assert_exact_keys(
        policy,
        &[
            "roots",
            "ignored_directory_names",
            "path_route_rules",
            "fixtures",
        ],
    );
    let search_roots = string_set(&policy["roots"]);
    assert_eq!(
        search_roots,
        BTreeSet::from([
            "alpha-release-report".to_owned(),
            "crates".to_owned(),
            "csharp-tools".to_owned(),
            "develop/specs".to_owned(),
            "examples".to_owned(),
            "fixtures".to_owned(),
            "fuzz".to_owned(),
            "go-tools".to_owned(),
            "java-tools".to_owned(),
            "proofs".to_owned(),
            "release-report.json".to_owned(),
            "release/build-inputs".to_owned(),
            "release/bundles".to_owned(),
            "rust-tools".to_owned(),
            "scripts".to_owned(),
        ]),
        "repository search-root inventory drift"
    );
    assert_eq!(
        array(&policy["roots"]).len(),
        search_roots.len(),
        "duplicate repository search root"
    );
    for root in array(&policy["roots"]) {
        let root = text(root);
        assert!(
            repo_path(root).exists(),
            "search root does not exist: {root}"
        );
    }

    let mut rule_ids = BTreeSet::new();
    for rule in array(&policy["path_route_rules"]) {
        assert_exact_keys(rule, &["id", "match", "pattern", "route", "role"]);
        let id = text(&rule["id"]);
        assert!(rule_ids.insert(id.to_owned()), "duplicate route rule {id}");
        assert!(
            matches!(text(&rule["match"]), "prefix" | "suffix" | "contains"),
            "unknown route-rule matcher for {id}"
        );
        assert!(
            matches!(text(&rule["route"]), "active" | "private"),
            "unknown route class for {id}"
        );
    }

    let mut routes = BTreeSet::new();
    let mut errors = Vec::new();
    let search_index = repository_search_index(policy);
    for fixture in array(&policy["fixtures"]) {
        assert_exact_keys(
            fixture,
            &[
                "id",
                "family",
                "needle",
                "expected_count",
                "expected_paths_sha256",
            ],
        );
        let id = text(&fixture["id"]);
        let family = text(&fixture["family"]);
        assert!(
            required_families.contains(family),
            "{id} has unknown family"
        );
        let paths = repository_search_paths(&search_index, text(&fixture["needle"]));
        if let Err(error) = assert_search_fingerprint(fixture, &paths) {
            errors.push(error);
        }
        for path in &paths {
            let (route, role) = classify_search_path(policy, path)
                .unwrap_or_else(|| panic!("unowned search edge {id}:{path}"));
            assert!(
                matches!(
                    role,
                    "producer" | "validator" | "bundle_member" | "fixture" | "test" | "api_route"
                ),
                "search edge {id}:{path} has invalid role {role}"
            );
            routes.insert(route.to_owned());
        }
    }
    assert!(
        errors.is_empty(),
        "repository consumer search drift:\n{}",
        errors.join("\n")
    );
    routes
}

fn assert_installed_identity_inventory(families: &[Value]) {
    let registry = read_json("release/bundles/semantic-profile-registry.json");
    let profiles = array(&registry["profiles"]);

    let mut registry_identities = BTreeSet::from([
        "mpk.semantic_profile.entry.v1".to_owned(),
        "mpk.semantic_profile.registry.limits.v1".to_owned(),
        "mpk.semantic_profile.registry.v1".to_owned(),
    ]);
    let mut parameter_identities = BTreeSet::new();
    let mut selection_identities = BTreeSet::new();
    let mut contract_identities = BTreeSet::from([
        "mpk.csharp.contract.v0".to_owned(),
        "mpk.go.contract.v0".to_owned(),
        "mpk.java.contract.v0".to_owned(),
        "mpk.rust.contract.v0".to_owned(),
    ]);
    for profile in profiles {
        registry_identities.insert(text(&profile["semantic_profile"]).to_owned());
        parameter_identities.insert(text(&profile["semantic_parameters_schema"]).to_owned());
        selection_identities.insert(text(&profile["selection_schema"]).to_owned());
        contract_identities.extend(
            object(&profile["contracts"])
                .values()
                .map(text)
                .map(str::to_owned),
        );
    }

    assert_eq!(
        string_set(&identity_family(families, "semantic_registry")["current_identities"]),
        registry_identities
    );
    assert_eq!(
        string_set(&identity_family(families, "semantic_parameters")["current_identities"]),
        parameter_identities
    );
    assert_eq!(
        string_set(&identity_family(families, "selection")["current_identities"]),
        selection_identities
    );
    assert_eq!(
        string_set(&identity_family(families, "profile_contract")["current_identities"]),
        contract_identities
    );
}

fn assert_current_hash_domain_inventory(families: &[Value]) {
    let expected = BTreeMap::from([
        (
            "semantic_registry",
            vec![
                "MPK-SEMANTIC-PROFILE-ENTRY-1.0",
                "MPK-SEMANTIC-PROFILE-REGISTRY-1.0",
            ],
        ),
        (
            "selection",
            vec!["MPK-CSHARP-SELECTION-0.1", "MPK-JAVA-SELECTION-0.1"],
        ),
        (
            "profile_contract",
            vec![
                "MPK-CONTRACT-0.1",
                "MPK-CONTRACT-1.0",
                "MPK-CSHARP-CONTRACT-SIDECAR-0.1",
                "MPK-JAVA-CONTRACT-SIDECAR-0.1",
            ],
        ),
        (
            "vir",
            vec![
                "MPK-CONTRACT-0.1",
                "MPK-CONTRACT-1.0",
                "MPK-VIR-0.1",
                "MPK-VIR-1.0",
            ],
        ),
        (
            "frontend_protocol",
            vec![
                "MPK-RUST-DRIVER-PAYLOAD-0.1",
                "MPK-RUST-DRIVER-PAYLOAD-1.0",
                "MPK-RUST-DRIVER-REQUEST-0.1",
                "MPK-RUST-DRIVER-REQUEST-1.0",
            ],
        ),
        (
            "source_map",
            vec!["MPK-SOURCE-MAP-0.1", "MPK-SOURCE-MAP-1.0"],
        ),
        (
            "source_manifest",
            vec![
                "MPK-INPUT-SET-0.1",
                "MPK-RUST-SOURCE-INVENTORY-0.1",
                "MPK-SOURCE-MANIFEST-0.1",
                "MPK-SOURCE-MANIFEST-1.0",
            ],
        ),
        ("vc_skeleton", vec!["MPK-VC-1.0", "MPK-VC-2.0"]),
        (
            "release",
            vec![
                "MPK-BUNDLE-CONTENT-0.1",
                "MPK-BUNDLE-REGISTRY-0.1",
                "MPK-BUNDLE-REGISTRY-1.0",
                "MPK-CSHARP-REFERENCE-INVENTORY-0.1",
                "MPK-CSHARP-TOOLCHAIN-INPUTS-0.1",
                "MPK-JAVA-TOOLCHAIN-INPUTS-0.1",
                "MPK-RUST-BUILD-INPUTS-0.1",
            ],
        ),
        (
            "program_assembly",
            vec![
                "MPK-AXIOM-REPORT-0.1",
                "MPK-DECL-0.1",
                "MPK-LEVEL-0.1",
                "MPK-MODULE-CERT-0.1",
                "MPK-MODULE-EXPORT-0.1",
                "MPK-PROOF-NODE-0.1",
                "MPK-SOURCE-MANIFEST-0.1",
                "MPK-TERM-0.1",
                "MPK-THEORY-CERT-0.1",
            ],
        ),
    ]);
    for family in families {
        let id = text(&family["id"]);
        let actual = string_set(&family["current_hash_domains"]);
        let expected = expected
            .get(id)
            .map(|domains| domains.iter().map(|domain| (*domain).to_owned()).collect())
            .unwrap_or_default();
        assert_eq!(actual, expected, "current hash-domain drift for {id}");
    }
}

fn identity_family<'a>(families: &'a [Value], id: &str) -> &'a Value {
    families
        .iter()
        .find(|family| text(&family["id"]) == id)
        .unwrap_or_else(|| panic!("missing identity family {id}"))
}

fn assert_search_fingerprint(fixture: &Value, paths: &[String]) -> Result<(), String> {
    let id = text(&fixture["id"]);
    let expected_count = fixture["expected_count"]
        .as_u64()
        .expect("search count must be an unsigned integer") as usize;
    let expected_hash = text(&fixture["expected_paths_sha256"]);
    let actual_hash = search_path_set_hash(paths);
    if paths.len() == expected_count && actual_hash == expected_hash {
        Ok(())
    } else {
        Err(format!(
            "{id}: expected count={expected_count} sha256={expected_hash}; actual count={} sha256={actual_hash}",
            paths.len()
        ))
    }
}

fn repository_search_index(policy: &Value) -> Vec<(String, String)> {
    let root = repo_path("");
    let ignored = string_set(&policy["ignored_directory_names"]);
    let mut files = Vec::new();
    for search_root in array(&policy["roots"]) {
        collect_search_files(&root.join(text(search_root)), &ignored, &mut files);
    }
    files.sort();
    files.dedup();
    files
        .into_iter()
        .filter_map(|path| {
            fs::read(&path).ok().map(|bytes| {
                let relative = path
                    .strip_prefix(&root)
                    .expect("search path must remain under repository root")
                    .to_string_lossy()
                    .replace('\\', "/");
                (relative, String::from_utf8_lossy(&bytes).into_owned())
            })
        })
        .collect()
}

fn repository_search_paths(search_index: &[(String, String)], needle: &str) -> Vec<String> {
    assert!(!needle.is_empty(), "search needle must not be empty");
    let mut matches = search_index
        .iter()
        .filter(|(_, contents)| contents.contains(needle))
        .map(|(path, _)| path.clone())
        .collect::<Vec<_>>();
    matches.sort();
    matches
}

fn collect_search_files(path: &Path, ignored: &BTreeSet<String>, files: &mut Vec<PathBuf>) {
    let metadata = fs::symlink_metadata(path)
        .unwrap_or_else(|error| panic!("failed to inspect {}: {error}", path.display()));
    if metadata.file_type().is_symlink() {
        return;
    }
    if metadata.is_file() {
        files.push(path.to_path_buf());
        return;
    }
    if path
        .file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| ignored.contains(name))
    {
        return;
    }
    let entries = fs::read_dir(path)
        .unwrap_or_else(|error| panic!("failed to list {}: {error}", path.display()));
    for entry in entries {
        let entry = entry.unwrap_or_else(|error| {
            panic!("failed to read an entry under {}: {error}", path.display())
        });
        collect_search_files(&entry.path(), ignored, files);
    }
}

fn search_path_set_hash(paths: &[String]) -> String {
    let mut hasher = Sha256::new();
    for path in paths {
        hasher.update(path.as_bytes());
        hasher.update(b"\n");
    }
    hasher
        .finalize()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn classify_search_path<'a>(policy: &'a Value, path: &str) -> Option<(&'a str, &'a str)> {
    for rule in array(&policy["path_route_rules"]) {
        let pattern = text(&rule["pattern"]);
        let matched = match text(&rule["match"]) {
            "prefix" => path.starts_with(pattern),
            "suffix" => path.ends_with(pattern),
            "contains" => path.contains(pattern),
            _ => false,
        };
        if matched {
            return Some((text(&rule["route"]), text(&rule["role"])));
        }
    }
    None
}

fn assert_bundle_members_and_routes(inventory: &Value) {
    let members = &inventory["bundle_members"];
    assert_exact_keys(
        members,
        &[
            "descriptor_paths",
            "binary_members",
            "frontend_bundle_ids",
            "toolchain_bundle_ids",
            "inventory_sets",
            "tuple_keys",
        ],
    );
    for path in array(&members["descriptor_paths"]) {
        let path = text(path);
        assert!(
            repo_path(path).is_file(),
            "missing bundle descriptor {path}"
        );
    }
    assert_eq!(
        string_set(&members["descriptor_paths"]).len(),
        array(&members["descriptor_paths"]).len(),
        "duplicate bundle descriptor path"
    );
    let binary_members = array(&members["binary_members"]);
    assert_eq!(
        binary_members.len(),
        1,
        "checked binary member inventory drift"
    );
    let binary_member = &binary_members[0];
    assert_exact_keys(
        binary_member,
        &["family", "path", "raw_sha256", "roles", "route"],
    );
    assert_eq!(text(&binary_member["family"]), "program_assembly");
    assert_eq!(text(&binary_member["route"]), "active");
    assert_eq!(
        string_set(&binary_member["roles"]),
        BTreeSet::from([
            "bundle_member".to_owned(),
            "parser".to_owned(),
            "validator".to_owned(),
        ])
    );
    let binary_path = text(&binary_member["path"]);
    assert_eq!(binary_path, "release/checkers/mpk-checker-ref-linux-amd64");
    assert_eq!(
        text(&binary_member["raw_sha256"]),
        hex_sha256(&fs::read(repo_path(binary_path)).expect("checked binary member"))
    );

    let registry = read_json("release/bundles/bundle-registry.json");
    let frontend_ids = array(&registry["frontend_bundles"])
        .iter()
        .map(|bundle| text(&bundle["bundle_id"]).to_owned())
        .collect::<BTreeSet<_>>();
    let toolchain_ids = array(&registry["toolchain_bundles"])
        .iter()
        .map(|bundle| text(&bundle["bundle_id"]).to_owned())
        .collect::<BTreeSet<_>>();
    assert_eq!(frontend_ids, string_set(&members["frontend_bundle_ids"]));
    assert_eq!(toolchain_ids, string_set(&members["toolchain_bundle_ids"]));
    let mut expected_inventory_sets = BTreeMap::new();
    for inventory_set in array(&members["inventory_sets"]) {
        assert_exact_keys(inventory_set, &["bundle_id", "file_count", "paths_sha256"]);
        let bundle_id = text(&inventory_set["bundle_id"]);
        let file_count = inventory_set["file_count"]
            .as_u64()
            .expect("bundle member count must be u64");
        let paths_sha256 = text(&inventory_set["paths_sha256"]);
        assert!(
            paths_sha256.len() == 64
                && paths_sha256
                    .bytes()
                    .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte)),
            "bundle path-set SHA-256 is malformed for {bundle_id}"
        );
        assert!(
            expected_inventory_sets
                .insert(bundle_id.to_owned(), (file_count, paths_sha256.to_owned()),)
                .is_none(),
            "duplicate bundle inventory set for {bundle_id}"
        );
    }
    let mut observed_inventory_sets = BTreeMap::new();
    for bundle in array(&registry["frontend_bundles"])
        .iter()
        .chain(array(&registry["toolchain_bundles"]).iter())
    {
        let bundle_id = text(&bundle["bundle_id"]);
        let files = array(&bundle["inventory"]["files"]);
        assert!(
            !files.is_empty(),
            "bundle inventory is empty for {bundle_id}"
        );
        let mut paths = BTreeSet::new();
        for file in files {
            let path = text(&file["path"]);
            assert!(
                paths.insert(path),
                "duplicate bundle-inventory member {bundle_id}:{path}"
            );
        }
        let sorted_paths = paths
            .into_iter()
            .map(str::to_owned)
            .collect::<Vec<String>>();
        observed_inventory_sets.insert(
            bundle_id.to_owned(),
            (files.len() as u64, search_path_set_hash(&sorted_paths)),
        );
    }
    assert_eq!(observed_inventory_sets, expected_inventory_sets);

    let tuple_keys = array(&registry["tuples"])
        .iter()
        .map(|tuple| {
            let context = &tuple["semantic_context"];
            format!(
                "{}|{}|{}",
                text(&context["source_language"]),
                text(&context["semantic_profile"]),
                text(&context["semantic_parameters"]["value"]["target_id"])
            )
        })
        .collect::<Vec<_>>();
    assert_eq!(
        tuple_keys,
        array(&members["tuple_keys"])
            .iter()
            .map(|value| text(value).to_owned())
            .collect::<Vec<_>>()
    );

    let main = read_text("crates/mpk-cli/src/main.rs");
    let mut cli_routes = BTreeSet::new();
    for route in array(&inventory["cli_routes"]) {
        assert_exact_keys(
            route,
            &["route", "path", "dispatch_anchor", "handler", "route_class"],
        );
        assert_eq!(text(&route["path"]), "crates/mpk-cli/src/main.rs");
        assert_eq!(text(&route["route_class"]), "active");
        assert!(
            cli_routes.insert(text(&route["route"]).to_owned()),
            "duplicate CLI route"
        );
        let dispatch_anchor = text(&route["dispatch_anchor"]);
        assert!(
            main.contains(dispatch_anchor),
            "CLI route lost dispatch anchor {dispatch_anchor}"
        );
        let handler = text(&route["handler"]);
        assert!(main.contains(handler), "CLI route lost handler {handler}");
    }
    assert_eq!(
        cli_routes,
        BTreeSet::from([
            "mpk __mpk_frontend_probe_v0".to_owned(),
            "mpk __mpk_frontend_sandbox_v0".to_owned(),
            "mpk axiom-report".to_owned(),
            "mpk check".to_owned(),
            "mpk explain".to_owned(),
            "mpk package check".to_owned(),
            "mpk package verify-certs".to_owned(),
            "mpk policy scan".to_owned(),
            "mpk policy verify".to_owned(),
            "mpk verify".to_owned(),
        ])
    );
    let run_dispatch = main
        .split_once("fn run(args: Vec<String>)")
        .and_then(|(_, suffix)| suffix.split_once("fn explain_route").map(|(body, _)| body))
        .expect("top-level CLI dispatcher");
    assert_eq!(
        run_dispatch.matches("=>").count(),
        11,
        "top-level CLI dispatcher gained or lost an action/help route"
    );
    let main_dispatch = main
        .split_once("fn main()")
        .and_then(|(_, suffix)| suffix.split_once("fn run(args").map(|(body, _)| body))
        .expect("internal frontend CLI dispatcher");
    assert_eq!(
        main_dispatch
            .matches("return ExitCode::from(mpk_cli::run_frontend_")
            .count(),
        2,
        "internal frontend CLI dispatcher gained or lost a route"
    );

    let api_source = read_text("crates/mpk-api/src/successor_api.rs");
    let api_route_table = api_source
        .split_once("pub const SUCCESSOR_ROUTES")
        .and_then(|(_, suffix)| suffix.split_once("];").map(|(table, _)| table))
        .expect("successor API route table");
    let compact_api = api_route_table.split_whitespace().collect::<String>();
    let mut api_routes = BTreeSet::new();
    for route in array(&inventory["api_routes"]) {
        assert_exact_keys(route, &["method", "path"]);
        let method = text(&route["method"]);
        let path = text(&route["path"]);
        assert!(matches!(method, "GET" | "POST"));
        assert!(
            api_routes.insert(format!("{method} {path}")),
            "duplicate API route {method} {path}"
        );
        let source_route = format!("route(\"{method}\",\"{path}\"");
        assert!(
            compact_api.contains(&source_route),
            "API route drift for {method} {path}"
        );
    }
    assert_eq!(api_routes.len(), 33, "successor API route inventory drift");
    assert_eq!(
        compact_api.matches("route(\"").count(),
        api_routes.len(),
        "successor API table has an unowned or missing route"
    );
}

fn assert_atomic_migration_and_rollback(
    inventory: &Value,
    required_families: &BTreeSet<String>,
    planned_items: &BTreeSet<String>,
) {
    let migration = &inventory["atomic_migration_set"];
    assert_exact_keys(
        migration,
        &[
            "id",
            "activation_owner",
            "producer_migration_owner",
            "consumer_migration_owner",
            "member_families",
            "activation_units",
            "forbidden_partial_states",
        ],
    );
    assert_eq!(
        text(&migration["id"]),
        "csharp-practical-successor-whole-release"
    );
    for owner in [
        "activation_owner",
        "producer_migration_owner",
        "consumer_migration_owner",
    ] {
        assert!(
            planned_items.contains(text(&migration[owner])),
            "unknown atomic migration owner"
        );
    }
    assert_eq!(
        string_set(&migration["member_families"]),
        *required_families
    );
    assert_eq!(
        array(&migration["member_families"]).len(),
        required_families.len(),
        "duplicate atomic migration family"
    );
    assert!(!array(&migration["activation_units"]).is_empty());
    assert!(!array(&migration["forbidden_partial_states"]).is_empty());

    let rollback = &inventory["whole_image_rollback_set"];
    assert_exact_keys(
        rollback,
        &[
            "id",
            "source_baseline",
            "source_commit",
            "source_tree",
            "member_families",
            "restore_units",
            "rollback_rule",
            "partial_rollback_forbidden",
        ],
    );
    assert_eq!(text(&rollback["source_baseline"]), BASELINE_PATH);
    assert_eq!(
        rollback["source_commit"],
        inventory["observed_source"]["commit"]
    );
    assert_eq!(
        rollback["source_tree"],
        inventory["observed_source"]["tree"]
    );
    assert_eq!(string_set(&rollback["member_families"]), *required_families);
    assert_eq!(
        array(&rollback["member_families"]).len(),
        required_families.len(),
        "duplicate rollback family"
    );
    assert_eq!(rollback["partial_rollback_forbidden"], true);
    assert!(!array(&rollback["restore_units"]).is_empty());
    assert!(
        text(&rollback["rollback_rule"]).contains("entire installed image"),
        "rollback must be whole-image only"
    );
}

fn object(value: &Value) -> &serde_json::Map<String, Value> {
    value.as_object().expect("expected object")
}

fn string_set(value: &Value) -> BTreeSet<String> {
    array(value)
        .iter()
        .map(|value| text(value).to_owned())
        .collect()
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
