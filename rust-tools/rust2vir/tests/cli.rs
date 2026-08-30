use rust2vir_internal::cli::{parse_lower_args, RustTarget, SEMANTIC_PROFILE};
use rust2vir_internal::successor::{
    FRONTEND_ID, PROFILE_ENTRY_SHA256, PROFILE_REGISTRY_ID, PROFILE_REGISTRY_SHA256, TOOLCHAIN_ID,
};
use std::ffi::OsString;
use std::process::Command;

const SHA256: &str = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";

fn valid_arguments(source_root: &str) -> Vec<OsString> {
    [
        "lower",
        source_root,
        "--manifest-path",
        "Cargo.toml",
        "--package",
        "payment-policy",
        "--semantic-profile",
        SEMANTIC_PROFILE,
        "--target",
        "x86_64-unknown-linux-gnu",
        "--function",
        "payment_policy::rules::approved_reserve_cents",
        "--profile-registry-id",
        PROFILE_REGISTRY_ID,
        "--profile-registry-revision",
        "2",
        "--profile-registry-sha256",
        PROFILE_REGISTRY_SHA256,
        "--profile-entry-sha256",
        PROFILE_ENTRY_SHA256,
        "--frontend-bundle-id",
        FRONTEND_ID,
        "--frontend-sha256",
        SHA256,
        "--release-registry-id",
        "mpk.release.registry.v1",
        "--release-registry-sha256",
        SHA256,
        "--toolchain-bundle-id",
        TOOLCHAIN_ID,
        "--toolchain-root",
        "/mpk/toolchain",
        "--toolchain-distribution-sha256",
        SHA256,
        "--driver",
        "/mpk/frontend/rust2vir-driver",
        "--driver-sha256",
        SHA256,
        "--contract",
        "contracts/approved.json",
        "--contract",
        "contracts/rules.json",
    ]
    .into_iter()
    .map(OsString::from)
    .collect()
}

fn replace_value(arguments: &mut [OsString], option: &str, value: &str) {
    let index = arguments
        .iter()
        .position(|argument| argument == option)
        .unwrap();
    arguments[index + 1] = OsString::from(value);
}

#[test]
fn exact_lower_cli_derives_path_free_library_selection() {
    let request = parse_lower_args(valid_arguments("/mpk/source")).unwrap();
    assert_eq!(request.source_root, std::path::Path::new("/mpk/source"));
    assert_eq!(request.selection.package, "payment-policy");
    assert_eq!(request.selection.crate_name, "payment_policy");
    assert_eq!(request.selection.kind, "lib");
    assert_eq!(
        request.selection.function,
        "payment_policy::rules::approved_reserve_cents"
    );
    assert_eq!(request.target, RustTarget::X86_64UnknownLinuxGnu);
    assert_eq!(request.target.pointer_width(), 64);
    assert_eq!(
        request.contracts,
        ["contracts/approved.json", "contracts/rules.json"]
    );

    let mut i686 = valid_arguments("/another/machine/path");
    replace_value(&mut i686, "--target", "i686-unknown-linux-gnu");
    let i686 = parse_lower_args(i686).unwrap();
    assert_eq!(i686.target, RustTarget::I686UnknownLinuxGnu);
    assert_eq!(i686.target.pointer_width(), 32);
    assert_eq!(i686.selection, request.selection);
}

#[test]
fn every_selection_and_release_assertion_is_mandatory() {
    for option in [
        "--manifest-path",
        "--package",
        "--semantic-profile",
        "--target",
        "--function",
        "--profile-registry-id",
        "--profile-registry-revision",
        "--profile-registry-sha256",
        "--profile-entry-sha256",
        "--frontend-bundle-id",
        "--frontend-sha256",
        "--release-registry-id",
        "--release-registry-sha256",
        "--toolchain-bundle-id",
        "--toolchain-root",
        "--toolchain-distribution-sha256",
        "--driver",
        "--driver-sha256",
    ] {
        let mut arguments = valid_arguments("/mpk/source");
        let index = arguments
            .iter()
            .position(|argument| argument == option)
            .unwrap();
        arguments.drain(index..=index + 1);
        assert!(parse_lower_args(arguments).is_err(), "{option}");
    }

    let mut arguments = valid_arguments("/mpk/source");
    arguments.truncate(arguments.len() - 4);
    assert!(parse_lower_args(arguments).is_err());
}

#[test]
fn unknown_duplicate_and_inline_options_are_usage_errors() {
    let mut duplicate = valid_arguments("/mpk/source");
    duplicate.extend([OsString::from("--package"), OsString::from("again")]);
    assert!(parse_lower_args(duplicate).is_err());

    let mut unknown = valid_arguments("/mpk/source");
    unknown.extend([OsString::from("--jobs"), OsString::from("2")]);
    assert!(parse_lower_args(unknown).is_err());

    let mut inline = valid_arguments("/mpk/source");
    inline.push(OsString::from("--contract=contracts/extra.json"));
    assert!(parse_lower_args(inline).is_err());
}

#[test]
fn closed_profile_target_identifier_and_digest_grammars_are_enforced() {
    let mutations = [
        ("--manifest-path", "./Cargo.toml"),
        ("--package", "9vector"),
        ("--package", "vector.policy"),
        ("--semantic-profile", "mpk.rust.checked.v1"),
        ("--target", "aarch64-unknown-linux-gnu"),
        ("--function", "vector"),
        ("--function", "vector::raw#function"),
        ("--function", "vector::_"),
        ("--profile-registry-revision", "1"),
        ("--profile-entry-sha256", SHA256),
        ("--frontend-bundle-id", "Frontend.rust.v1"),
        ("--release-registry-id", "mpk..registry"),
        ("--frontend-sha256", "ABCDEF"),
        ("--toolchain-root", "relative/toolchain"),
        ("--driver", "rust2vir-driver"),
    ];
    for (option, value) in mutations {
        let mut arguments = valid_arguments("/mpk/source");
        replace_value(&mut arguments, option, value);
        assert!(parse_lower_args(arguments).is_err(), "{option}={value}");
    }
}

#[test]
fn configuration_failure_exits_two_without_json() {
    let output = Command::new(env!("CARGO_BIN_EXE_rust2vir"))
        .args(["lower", "/mpk/source", "--package", "vector"])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty());
    assert_eq!(output.stderr, b"RUST_TOOLCHAIN_ARGUMENT\n");
}

#[test]
fn help_is_not_a_lower_response() {
    let output = Command::new(env!("CARGO_BIN_EXE_rust2vir"))
        .arg("--help")
        .output()
        .unwrap();
    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    assert!(output.stdout.starts_with(b"rust2vir lower SOURCE_ROOT "));
    assert!(!output.stdout.starts_with(b"{"));
}
