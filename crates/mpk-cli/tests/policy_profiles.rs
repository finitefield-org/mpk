use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use mpk_api::{PolicyAxiomProfile, PolicyStrategyProfile, ProofProfile};
use mpk_cert::encode::AxiomCategory;
use mpk_cli::policy_profile::{
    approved_axioms, strategy_registry, summary_only_axiom_report_is_permitted,
    validate_explainer_profile_selection, validate_package_axiom_profiles,
    validate_package_release_profiles, validate_policy_profile_selection, PolicyProfileErrorKind,
    PolicyProfileField, PolicyProfileRecognition, PolicyProfileSelection,
};
use mpk_vc::{SemanticProfile, SourceLanguage};
use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
struct PackageManifestFixture {
    policy: PackagePolicyFixture,
}

#[derive(Clone, Debug, Deserialize)]
struct PackagePolicyFixture {
    checker_profile: String,
    allowed_axiom_profiles: Vec<String>,
}

#[derive(Clone, Debug, Deserialize)]
struct ReleasePolicyFixture {
    source_language: String,
    semantic_profile: String,
    strategy_profile: String,
    checker_profile: String,
    axiom_profile: String,
}

impl ReleasePolicyFixture {
    fn selection(&self) -> PolicyProfileSelection<'_> {
        PolicyProfileSelection {
            strategy_profile: &self.strategy_profile,
            checker_profile: &self.checker_profile,
            source_language: &self.source_language,
            semantic_profile: &self.semantic_profile,
            axiom_profile: &self.axiom_profile,
        }
    }
}

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn read_fixture<T: for<'de> Deserialize<'de>>(relative: &str) -> T {
    let bytes = fs::read(repo_root().join(relative)).expect("fixture is readable");
    serde_json::from_slice(&bytes).expect("fixture has the expected shape")
}

fn go(checker: &'static str) -> PolicyProfileSelection<'static> {
    PolicyProfileSelection {
        strategy_profile: "payment-policy-alpha",
        checker_profile: checker,
        source_language: "go",
        semantic_profile: "mpk.go.fixed.v0",
        axiom_profile: "zero-axiom",
    }
}

fn rust(checker: &'static str) -> PolicyProfileSelection<'static> {
    PolicyProfileSelection {
        strategy_profile: "payment-policy-rust-alpha",
        checker_profile: checker,
        source_language: "rust",
        semantic_profile: "mpk.rust.checked.v0",
        axiom_profile: "mvp-theory",
    }
}

#[test]
fn policy_profile_registry_has_exact_go_then_rust_rows() {
    let rows = strategy_registry();
    assert_eq!(rows.len(), 2);
    assert_eq!(
        (
            rows[0].strategy_profile,
            rows[0].source_language,
            rows[0].semantic_profile,
            rows[0].axiom_profile,
        ),
        (
            PolicyStrategyProfile::PaymentPolicyAlpha,
            SourceLanguage::Go,
            SemanticProfile::GoFixedV0,
            PolicyAxiomProfile::ZeroAxiom,
        )
    );
    assert_eq!(
        (
            rows[1].strategy_profile,
            rows[1].source_language,
            rows[1].semantic_profile,
            rows[1].axiom_profile,
        ),
        (
            PolicyStrategyProfile::PaymentPolicyRustAlpha,
            SourceLanguage::Rust,
            SemanticProfile::RustCheckedV0,
            PolicyAxiomProfile::MvpTheory,
        )
    );

    for checker in ["core-bootstrap", "mvp-structural", "mvp-strict"] {
        assert_eq!(
            validate_policy_profile_selection(go(checker))
                .unwrap()
                .checker_profile
                .canonical_name(),
            checker
        );
        assert_eq!(
            validate_policy_profile_selection(rust(checker))
                .unwrap()
                .checker_profile
                .canonical_name(),
            checker
        );
    }
}

#[test]
fn every_crossed_known_strategy_tuple_rejects_as_crossed() {
    let crossed = [
        PolicyProfileSelection {
            source_language: "rust",
            ..go("mvp-strict")
        },
        PolicyProfileSelection {
            semantic_profile: "mpk.rust.checked.v0",
            ..go("mvp-strict")
        },
        PolicyProfileSelection {
            axiom_profile: "mvp-theory",
            ..go("mvp-strict")
        },
        PolicyProfileSelection {
            source_language: "go",
            ..rust("mvp-strict")
        },
        PolicyProfileSelection {
            semantic_profile: "mpk.go.fixed.v0",
            ..rust("mvp-strict")
        },
        PolicyProfileSelection {
            axiom_profile: "zero-axiom",
            ..rust("mvp-strict")
        },
    ];
    for selection in crossed {
        let error = validate_policy_profile_selection(selection).unwrap_err();
        assert_eq!(error.kind(), PolicyProfileErrorKind::CrossedTuple);
    }
}

#[test]
fn unknown_profile_values_are_not_normalized() {
    let cases = [
        (
            PolicyProfileSelection {
                strategy_profile: "payment-policy-future-alpha",
                ..rust("mvp-strict")
            },
            PolicyProfileField::StrategyProfile,
        ),
        (
            PolicyProfileSelection {
                checker_profile: "future-checker",
                ..rust("mvp-strict")
            },
            PolicyProfileField::CheckerProfile,
        ),
        (
            PolicyProfileSelection {
                source_language: "future-language",
                ..rust("mvp-strict")
            },
            PolicyProfileField::SourceLanguage,
        ),
        (
            PolicyProfileSelection {
                semantic_profile: "mpk.rust.future.v0",
                ..rust("mvp-strict")
            },
            PolicyProfileField::SemanticProfile,
        ),
        (
            PolicyProfileSelection {
                axiom_profile: "future-axiom",
                ..rust("mvp-strict")
            },
            PolicyProfileField::AxiomProfile,
        ),
    ];
    for (selection, field) in cases {
        let error = validate_policy_profile_selection(selection).unwrap_err();
        assert_eq!(error.kind(), PolicyProfileErrorKind::Unknown);
        assert_eq!(error.field(), field);
    }
}

#[test]
fn explainer_only_sanitizes_a_genuinely_unknown_authorized_strategy() {
    let future = PolicyProfileSelection {
        strategy_profile: "payment-policy-future-alpha",
        ..rust("mvp-strict")
    };
    assert_eq!(
        validate_explainer_profile_selection(future, true).unwrap(),
        PolicyProfileRecognition::UnrecognizedStrategy
    );
    assert_eq!(
        validate_explainer_profile_selection(future, false)
            .unwrap_err()
            .kind(),
        PolicyProfileErrorKind::Unknown
    );

    let crossed_known = PolicyProfileSelection {
        strategy_profile: "payment-policy-alpha",
        ..rust("mvp-strict")
    };
    assert_eq!(
        validate_explainer_profile_selection(crossed_known, true)
            .unwrap_err()
            .kind(),
        PolicyProfileErrorKind::CrossedTuple
    );
}

#[test]
fn rust_package_and_release_fixtures_admit_exact_evidence_selection() {
    let package: PackageManifestFixture =
        read_fixture("fixtures/package-manifest/valid/rust-policy-package.json");
    let release: ReleasePolicyFixture =
        read_fixture("fixtures/policy-profiles/rust-release-policy.json");

    assert_eq!(package.policy.checker_profile, "mvp-strict");
    assert_eq!(package.policy.allowed_axiom_profiles, ["mvp-theory"]);
    let validated = validate_package_release_profiles(
        release.selection(),
        &package.policy.checker_profile,
        &package.policy.allowed_axiom_profiles,
        release.selection(),
    )
    .unwrap();
    assert_eq!(validated.checker_profile, ProofProfile::MvpStrict);
    assert_eq!(validated.axiom_profile, PolicyAxiomProfile::MvpTheory);
}

#[test]
fn package_check_accepts_rust_policy_manifest_fixture() {
    let output = Command::new(env!("CARGO_BIN_EXE_mpk"))
        .current_dir(repo_root())
        .args([
            "package",
            "check",
            "fixtures/package-manifest/valid/rust-policy-package.json",
        ])
        .output()
        .expect("mpk package check runs");
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        "ok package=Example.Rust.PolicyPackage imports=1 certificates=1\n"
    );
}

#[test]
fn package_template_selects_only_the_rust_axiom_profile() {
    let template =
        fs::read_to_string(repo_root().join("develop/templates/module_manifest.yaml")).unwrap();
    assert!(template.contains("  checker_profile: mvp-strict\n"));
    assert!(template.contains("  allowed_axiom_profiles:\n    - mvp-theory\n"));
    assert!(!template.contains("    - zero-axiom\n"));
}

#[test]
fn rust_package_gate_rejects_active_evidence_and_manifest_mismatches() {
    let package: PackageManifestFixture =
        read_fixture("fixtures/package-manifest/valid/rust-policy-package.json");
    let release: ReleasePolicyFixture =
        read_fixture("fixtures/policy-profiles/rust-release-policy.json");

    let active_checker_mismatch = PolicyProfileSelection {
        checker_profile: "mvp-structural",
        ..release.selection()
    };
    assert!(validate_package_release_profiles(
        release.selection(),
        &package.policy.checker_profile,
        &package.policy.allowed_axiom_profiles,
        active_checker_mismatch,
    )
    .is_err());

    let active_axiom_mismatch = PolicyProfileSelection {
        axiom_profile: "zero-axiom",
        ..release.selection()
    };
    assert!(validate_package_release_profiles(
        release.selection(),
        &package.policy.checker_profile,
        &package.policy.allowed_axiom_profiles,
        active_axiom_mismatch,
    )
    .is_err());

    let crossed_evidence = PolicyProfileSelection {
        strategy_profile: "payment-policy-alpha",
        ..release.selection()
    };
    assert_eq!(
        validate_package_release_profiles(
            crossed_evidence,
            &package.policy.checker_profile,
            &package.policy.allowed_axiom_profiles,
            release.selection(),
        )
        .unwrap_err()
        .kind(),
        PolicyProfileErrorKind::CrossedTuple
    );

    assert!(validate_package_release_profiles(
        release.selection(),
        "mvp-structural",
        &package.policy.allowed_axiom_profiles,
        release.selection(),
    )
    .is_err());
    assert!(validate_package_release_profiles(
        release.selection(),
        &package.policy.checker_profile,
        &["zero-axiom".to_owned()],
        release.selection(),
    )
    .is_err());
}

#[test]
fn package_axiom_allowlists_are_closed_and_evidence_cannot_broaden_them() {
    assert!(validate_package_axiom_profiles(&["mvp-theory".to_owned()]).is_ok());
    for invalid in [
        vec!["future-axiom".to_owned()],
        vec!["mvp-theory".to_owned(), "mvp-theory".to_owned()],
    ] {
        assert!(validate_package_axiom_profiles(&invalid).is_err());
    }

    assert!(validate_package_release_profiles(
        rust("mvp-strict"),
        "mvp-strict",
        &["zero-axiom".to_owned()],
        rust("mvp-strict"),
    )
    .is_err());
}

#[test]
fn mvp_theory_adds_no_category_and_approves_no_unreviewed_identity() {
    assert_eq!(
        [
            AxiomCategory::CoreAxiom,
            AxiomCategory::BuiltinTheoryAxiom,
            AxiomCategory::GoSemanticsAxiom,
            AxiomCategory::ExternalAxiom,
        ]
        .map(AxiomCategory::canonical_name),
        [
            "CoreAxiom",
            "BuiltinTheoryAxiom",
            "GoSemanticsAxiom",
            "ExternalAxiom",
        ]
    );
    assert!(approved_axioms(PolicyAxiomProfile::MvpTheory).is_empty());
    assert!(summary_only_axiom_report_is_permitted(
        PolicyAxiomProfile::MvpTheory,
        0
    ));
    assert!(!summary_only_axiom_report_is_permitted(
        PolicyAxiomProfile::MvpTheory,
        1
    ));
}
