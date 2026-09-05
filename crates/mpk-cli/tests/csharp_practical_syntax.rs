//! CSHARP-03-T03-W02: private concise-syntax and name-resolution normalization.
//!
//! The executable harness is compiled only from its exact manifest and the
//! pinned Roslyn/.NET closure. The implementation stays outside every active
//! project, bundle, registry, CLI, and application-source dependency.

use serde_json::Value;
use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

const WORK_ITEM: &str = "CSHARP-03-T03-W02";
const OWNER: &str = "crates/mpk-cli/tests/csharp_practical_syntax.rs#CSHARP-03-T03-W02";
const MANIFEST_PATH: &str = "develop/migrations/csharp-03/syntax/syntax-inputs.json";
const PACKAGE_PATH: &str = "develop/specs/vectors/csharp-practical-profile-v1.json";
const TOOLCHAIN_HASH: &str = "d4af1170b2813a5581bb0f60b65fd4e7509576093045557b88689bf7e0876b4f";

fn repository_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn read(relative: &str) -> Vec<u8> {
    fs::read(repository_root().join(relative))
        .unwrap_or_else(|error| panic!("read {relative}: {error}"))
}

fn text(relative: &str) -> String {
    String::from_utf8(read(relative)).unwrap_or_else(|error| panic!("UTF-8 {relative}: {error}"))
}

fn load(relative: &str) -> Value {
    serde_json::from_slice(&read(relative))
        .unwrap_or_else(|error| panic!("decode {relative}: {error}"))
}

fn sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

#[test]
fn csharp_03_t03_w02_inputs_are_exact_and_the_route_is_private() {
    let manifest = load(MANIFEST_PATH);
    let mut canonical = serde_json::to_vec(&manifest).expect("canonical manifest");
    canonical.push(b'\n');
    assert_eq!(read(MANIFEST_PATH), canonical, "manifest is canonical JSON");
    assert_eq!(
        manifest["schema"],
        "mpk.csharp_practical.t03_w02.syntax_inputs.v1"
    );
    assert_eq!(manifest["work_item"], WORK_ITEM);
    let files = manifest["files"].as_array().expect("manifest files");
    let expected = [
        "crates/mpk-cli/tests/csharp_practical_syntax_harness.cs",
        "csharp-tools/csharp2vir/PracticalCapture.cs",
        "csharp-tools/csharp2vir/PracticalSyntaxNormalization.cs",
    ];
    assert_eq!(files.len(), expected.len());
    for (record, expected_path) in files.iter().zip(expected) {
        assert_eq!(record["path"], expected_path);
        let bytes = read(expected_path);
        assert_eq!(record["size_bytes"].as_u64(), Some(bytes.len() as u64));
        assert_eq!(record["sha256"], sha256(&bytes));
    }

    let project = text("csharp-tools/csharp2vir/csharp2vir.csproj");
    let program = text("csharp-tools/csharp2vir/Program.cs");
    let build_inputs = text("develop/migrations/csharp-03/build-inputs/build-inputs.json");
    let active_bundle = text("release/bundles/bundle-registry.json");
    let active_semantic = text("release/bundles/semantic-profile-registry.json");
    assert!(project.contains("<EnableDefaultCompileItems>false</EnableDefaultCompileItems>"));
    for private_name in [
        "PracticalCapture.cs",
        "PracticalSyntaxNormalization.cs",
        "CSharpPracticalSyntaxNormalizer",
    ] {
        assert!(!project.contains(private_name));
        assert!(!program.contains(private_name));
    }
    assert!(!build_inputs.contains("PracticalSyntaxNormalization.cs"));
    assert!(!active_bundle.contains("mpk.csharp_practical"));
    assert!(!active_semantic.contains("mpk.csharp.practical.v1"));

    let wrapper = text("scripts/build-csharp-practical-frontend.sh");
    let script = text("scripts/csharp_practical_build_inputs.py");
    assert!(wrapper.contains("--test-syntax"));
    assert!(script.contains("def validate_syntax_inputs_value"));
    assert!(script.contains("def test_syntax()"));
    assert!(script.contains("/main:Mpk.CSharp2Vir.PracticalSyntaxHarness"));
    assert!(script.contains("CSHARP_PRACTICAL_SYNTAX_TEST_BUILD"));
    assert!(script.contains("CSHARP_PRACTICAL_SYNTAX_TEST_FAILURE"));
}

#[test]
fn csharp_03_t03_w02_normalizer_owns_the_complete_closed_handoff() {
    let source = text("csharp-tools/csharp2vir/PracticalSyntaxNormalization.cs");
    for required in [
        "CSharpPracticalCapture.Validate",
        "PracticalNormalizedSyntax",
        "PracticalNormalizedCallable",
        "PracticalExactTypeBinding",
        "PracticalExactTypeNormalizer",
        "topLevelNullability",
        "SymbolEqualityComparer.IncludeNullability",
        "normalizedAnnotatedTypes",
        "normalizedNotAnnotatedTypes",
        "canonicalKey ??= BuildCanonicalKey(this)",
        "var wrapperIds = new List<string>()",
        "var pending = new Stack<PracticalNormalizedType>()",
        "var pending = new Stack<IOperation>()",
        "child_count",
        "CopyCanonicalBytes",
        "CopyBodyBytes",
        "mpk.csharp_practical.normalized_syntax.v1",
        "ValidateExpressionBodies",
        "MethodDeclarationSyntax",
        "PropertyDeclarationSyntax",
        "GetAccessorDeclaration",
        "expression_body_kind",
        "ValidateVarContexts",
        "VariableDeclarationSyntax",
        "LocalDeclarationStatementSyntax",
        "ForStatementSyntax",
        "AnonymousObjectCreationExpressionSyntax",
        "ImplicitArrayCreationExpressionSyntax",
        "ImplicitObjectCreationExpressionSyntax",
        "ImplicitStackAllocArrayCreationExpressionSyntax",
        "CollectionExpressionSyntax",
        "DefaultLiteralExpression",
        "target_typed_inference",
        "var_inference_type",
        "nullable_inference",
        "ValidateImportsAndDirectives",
        "GlobalKeyword",
        "StaticKeyword",
        "ExternAliasDirectiveSyntax",
        "UsingStatementSyntax",
        "UsingKeyword",
        "NullableDirectiveTriviaSyntax",
        "EnableKeyword",
        "source_directive",
        "IsMpkNamespace",
        "CanonicalOperationType",
        "intrinsic_argument:System.StringComparison",
        "intrinsic_argument:System.MidpointRounding",
        "ReferenceInventorySha256",
        "MetadataReference.CreateFromImage",
    ] {
        assert!(source.contains(required), "missing W02 rule {required}");
    }

    let capture = source.find("CSharpPracticalCapture.Validate").unwrap();
    let imports = source.find("ValidateImportsAndDirectives(state)").unwrap();
    let arrows = source.find("ValidateExpressionBodies(state)").unwrap();
    let inference = source.find("ValidateVarContexts(state)").unwrap();
    let artifact = source
        .find("new PracticalSyntaxModel(state, closure).Build()")
        .unwrap();
    assert!(capture < imports);
    assert!(imports < arrows);
    assert!(arrows < inference);
    assert!(inference < artifact);
    assert!(!source.contains("Microsoft.CodeAnalysis.CSharp.Scripting"));
    assert!(!source.contains("CSharpScript"));
}

#[test]
fn csharp_03_t03_w02_executable_vectors_cover_equivalence_and_rejection() {
    let harness = text("crates/mpk-cli/tests/csharp_practical_syntax_harness.cs");
    for owner in [
        "ExpressionBodiesNormalizeExactly",
        "VarLocalsExposeExactTypes",
        "NamespaceImportsAndNullableNormalizeExactly",
        "InferenceFailuresAreClosed",
        "ImportFormsAreClosed",
        "DirectiveFormsAreClosed",
        "ForeachRemainsDeferred",
        "ArtifactsAreDeterministicImmutableAndSanitized",
        "AssertEquivalent",
    ] {
        assert!(harness.contains(owner), "missing executable owner {owner}");
    }
    for mutation in [
        "public int Value => value",
        "get => value",
        "init => this.value = value",
        "var number = input + 1",
        "string? copy = input",
        "using System;",
        "using Alias = Business.Helpers",
        "using static Business.Helpers",
        "global using System",
        "using global::System",
        "extern alias External",
        "using Missing.Namespace",
        "using var stream",
        "PracticalCapturedInputKind.GeneratedSource",
        "#nullable enable",
        "// retained application comment",
        "#nullable disable",
        "#nullable restore",
        "#nullable enable annotations",
        "#nullable enable warnings",
        "#if true",
        "#define FEATURE",
        "#undef FEATURE",
        "#pragma warning disable",
        "#line 1",
        "#region practical",
        "#error STOP",
        "#warning STOP",
        "new[] { input, 1L }",
        "ref var alias = ref value",
        "Identity(default)",
        "Identity([input])",
        "Identity(new(input))",
        "new { Value = input }",
        "dynamic value = input",
        "foreach (",
        "new[] { \"int value\", \"var value\" }",
    ] {
        assert!(harness.contains(mutation), "missing mutation {mutation}");
    }
}

#[test]
fn csharp_03_t03_w02_published_owner_and_foreach_routing_are_exact() {
    let package = load(PACKAGE_PATH);
    let owners = package["downstream_work_item_owners"]
        .as_array()
        .expect("downstream owners");
    let owner = owners
        .iter()
        .find(|entry| entry["work_item"] == WORK_ITEM)
        .expect("W02 downstream owner");
    assert_eq!(owner["primary_test_owner"], OWNER);
    assert_eq!(
        owner["requirement_anchor"],
        "develop/docs/08_csharp_practical_subset_design-todo.md#CSHARP-03-T03-W02"
    );
    assert_eq!(owner["entry_state_at_publication"], "serially_blocked");
    let owns = owner["owns"].as_str().expect("owns");
    let exit_gate = owner["exit_gate"].as_str().expect("exit gate");
    let verification = owner["verification"].as_str().expect("verification");
    assert!(owns.contains("reusable exact-type normalization handoff"));
    assert!(owns.contains("positive `foreach` source form"));
    assert!(owns.contains("remains T04-W02-owned"));
    assert!(exit_gate.contains("byte-identical to explicit equivalents"));
    assert!(exit_gate.contains("emit no partial artifacts"));
    assert!(verification.contains("conditional-compilation"));
    assert!(verification.contains("positive `foreach`-variable cases are not claimed"));

    let vectors = package["vectors"].as_array().expect("vectors");
    assert_eq!(
        vectors
            .iter()
            .filter(|vector| vector["implementation_owner"] == WORK_ITEM)
            .count(),
        0,
        "W02 owns no frozen package vector rows"
    );
    let plan = text("develop/docs/08_csharp_practical_subset_design-todo.md");
    assert!(plan.contains("positive `foreach` source form, including `var` in that position"));
    assert!(plan.contains("T04-W02-owned."));
}

#[test]
fn csharp_03_t03_w02_pinned_roslyn_harness_passes_when_cache_is_present() {
    let package = load("develop/migrations/csharp-03/build-inputs/build-inputs.json");
    let archives = package["toolchain_inputs"]["archives"]
        .as_array()
        .expect("archives");
    // Validate the manifest shape on every host before the Linux-only runner.
    if !cfg!(target_os = "linux") {
        return;
    }
    let cache = repository_root()
        .join("release/build-input-cache/csharp")
        .join(TOOLCHAIN_HASH)
        .join("archives");
    let present = archives
        .iter()
        .filter(|archive| {
            let suffix = match archive["kind"].as_str().expect("archive kind") {
                "tar.gz" => ".tar.gz",
                "nupkg" => ".nupkg",
                kind => panic!("unexpected archive kind {kind}"),
            };
            cache
                .join(format!(
                    "{}{}",
                    archive["id"].as_str().expect("archive id"),
                    suffix
                ))
                .is_file()
        })
        .count();
    assert!(
        present == 0 || present == archives.len(),
        "partial pinned C# archive cache"
    );
    if present == 0 {
        return;
    }

    let output = Command::new(repository_root().join("scripts/build-csharp-practical-frontend.sh"))
        .arg("--test-syntax")
        .env_clear()
        .env("PATH", "/usr/bin:/bin")
        .output()
        .expect("run practical syntax harness");
    assert!(
        output.status.success(),
        "stdout={} stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}
