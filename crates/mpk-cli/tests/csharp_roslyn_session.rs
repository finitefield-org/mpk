use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

const REFERENCE_HASH: &str = "30623f64b7d85564260e62464e652bfaa89eb56e0e55193989bfb99538ba6cad";

fn repository_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn load(relative: &str) -> Value {
    let bytes = fs::read(repository_root().join(relative)).expect("read JSON");
    serde_json::from_slice(&bytes).expect("parse JSON")
}

#[test]
fn frozen_compiler_session_and_m33_are_owned_exactly() {
    let profile = load("develop/specs/vectors/csharp-profile-v0.json");
    let session = &profile["compiler_session"];
    assert_eq!(
        session["source_text"],
        json!({
            "decode": "strict UTF-8 without BOM from exact captured bytes",
            "encoding": "new UTF8Encoding(false,true)",
            "checksum_algorithm": "Sha256",
            "source_text_overload": "SourceText.From(string,Encoding,SourceHashAlgorithm)",
            "parse_text_overload": "CSharpSyntaxTree.ParseText(SourceText,CSharpParseOptions,string,CancellationToken)"
        })
    );
    assert_eq!(
        session["parse_options"],
        json!({
            "language_version_enum": "CSharp14",
            "language_version_text": "14.0",
            "source_kind": "Regular",
            "documentation_mode": "None",
            "preprocessor_symbols": [],
            "features": []
        })
    );
    assert_eq!(
        session["syntax_tree_order"],
        "selection.value.sources stored order"
    );
    assert_eq!(
        session["cfg_creation"],
        json!({
            "operation_root": "IMethodBodyOperation from exact MethodDeclarationSyntax",
            "overload": "ControlFlowGraph.Create(IMethodBodyOperation,CancellationToken)",
            "cancellation_token": "None"
        })
    );
    assert_eq!(
        session["semantic_api_options"],
        json!({
            "cancellation_token": "None",
            "ignore_accessibility": false,
            "speculative_models": false
        })
    );

    let options = &session["compilation_options"];
    for (field, expected) in [
        ("output_kind", json!("DynamicallyLinkedLibrary")),
        ("platform", json!("X64")),
        ("optimization_level", json!("Release")),
        ("check_overflow", json!(false)),
        ("nullable_context_options", json!("Disable")),
        ("allow_unsafe", json!(false)),
        ("deterministic", json!(true)),
        ("concurrent_build", json!(false)),
        ("metadata_import_options", json!("Public")),
        ("general_diagnostic_option", json!("Error")),
        ("warning_level", json!(4)),
        ("report_suppressed_diagnostics", json!(false)),
        ("references_supersede_lower_versions", json!(false)),
        ("script_class_name", json!("Script")),
        ("public_sign", json!(false)),
        ("assembly_identity_comparer", json!("Default")),
    ] {
        assert_eq!(options[field], expected, "compiler option {field}");
    }
    assert_eq!(options["specific_diagnostic_options"], json!({}));
    assert_eq!(options["global_usings"], json!([]));
    assert_eq!(options["crypto_public_key"], json!([]));
    for field in [
        "module_name",
        "main_type_name",
        "crypto_key_container",
        "crypto_key_file",
        "delay_sign",
    ] {
        assert!(options[field].is_null(), "compiler option {field}");
    }
    assert_eq!(
        options["resolvers"],
        json!({
            "source_reference_resolver": null,
            "metadata_reference_resolver": null,
            "xml_reference_resolver": null,
            "strong_name_provider": null,
            "syntax_tree_options_provider": null
        })
    );
    assert_eq!(
        options["metadata_reference_properties"],
        json!({
            "kind": "Assembly",
            "aliases": [],
            "embed_interop_types": false,
            "documentation_provider": null
        })
    );

    let expected_apis = [
        "SourceText.From",
        "CSharpSyntaxTree.ParseText",
        "SyntaxTree.GetDiagnostics",
        "CSharpCompilation.Create",
        "MetadataReference.CreateFromFile",
        "Compilation.GetDiagnostics",
        "Compilation.GetSemanticModel",
        "SemanticModel.GetDeclaredSymbol",
        "SemanticModel.GetSymbolInfo",
        "SemanticModel.GetTypeInfo",
        "SemanticModel.ClassifyConversion",
        "SemanticModel.GetOperation",
        "ControlFlowGraph.Create",
    ];
    assert_eq!(
        session["public_api_families"]
            .as_array()
            .unwrap()
            .iter()
            .map(Value::as_str)
            .collect::<Option<Vec<_>>>()
            .unwrap(),
        expected_apis
    );

    let m33 = profile["semantic_rows"]
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["row"] == "M33")
        .expect("M33 row");
    assert_eq!(m33["disposition"], "accept_under_profile_restrictions");
    assert_eq!(m33["basis"], "P01");
}

#[test]
fn candidate_uses_only_the_frozen_public_roslyn_boundary() {
    let root = repository_root();
    let session = fs::read_to_string(root.join("csharp-tools/csharp2vir/RoslynSession.cs"))
        .expect("read Roslyn session implementation");
    let adapters = fs::read_to_string(root.join("csharp-tools/csharp2vir/RoslynAdapters.cs"))
        .expect("read Roslyn adapter implementation");
    let program = fs::read_to_string(root.join("csharp-tools/csharp2vir/Program.cs"))
        .expect("read inactive candidate program");
    let combined = format!("{session}\n{adapters}");

    for required in [
        "SourceText.From(",
        "CSharpSyntaxTree.ParseText(",
        "syntaxTree.GetDiagnostics(CancellationToken.None)",
        "CSharpCompilation.Create(",
        "MetadataReference.CreateFromFile(",
        "compilation.GetDiagnostics(CancellationToken.None)",
        "session.Compilation.GetSemanticModel(",
        "ignoreAccessibility: false",
        "semanticModel.GetDeclaredSymbol(declaration, CancellationToken.None)",
        "semanticModel.GetSymbolInfo(expression, CancellationToken.None)",
        "semanticModel.GetTypeInfo(expression, CancellationToken.None)",
        "destination,\n                isExplicitInSource)",
        "semanticModel.GetOperation(syntax, CancellationToken.None)",
        "operation is not IMethodBodyOperation methodBody",
        "ControlFlowGraph.Create(methodBody, CancellationToken.None)",
    ] {
        assert!(
            combined.contains(required),
            "missing exact API call {required}"
        );
    }

    for forbidden in [
        "BindingFlags.NonPublic",
        "MSBuildWorkspace",
        "CSharpCommandLineParser",
        "MetadataReference.CreateFromImage",
        "MetadataReference.CreateFromStream",
        "CSharpScript",
        ".Emit(",
        "Process.Start",
        "ToFullString",
        "SyntaxKind.",
        "DescendantNodes",
    ] {
        assert!(
            !combined.contains(forbidden),
            "forbidden Roslyn boundary {forbidden}"
        );
    }

    assert!(session.contains("ExpectedCount = 167"));
    assert!(session.contains("ExpectedTotalBytes = 6_046_008"));
    assert!(session.contains(REFERENCE_HASH));
    assert!(session.contains("ExpectedCanonicalBytes = 24_670"));
    assert!(session.contains("ReferencesSupersedeLowerVersions is an internal getter"));
    assert!(program.contains("RoslynSessionFactory.Parse(selection, sources)"));
    assert!(program.contains("RoslynSessionFactory.Compile("));
    assert!(program.contains("CSharpSubset.Validate(selection, compilationSession)"));
    assert!(!program.contains("RoslynPublicApi"));
}

#[test]
fn executable_harness_and_build_gate_own_session_drift() {
    let root = repository_root();
    let harness =
        fs::read_to_string(root.join("crates/mpk-cli/tests/csharp_roslyn_session_harness.cs"))
            .expect("read executable Roslyn harness");
    for owner in [
        "OnlyFrozenRoslynAssembliesLoad",
        "ExactSourceAndTreeSession",
        "ExactCompilationAndReferences",
        "DiagnosticPhaseOrdering",
        "OptionAndReferenceDriftFailClosed",
        "PublicSemanticApisAreExact",
    ] {
        assert!(harness.contains(owner), "missing harness owner {owner}");
    }

    let script = fs::read_to_string(root.join("scripts/csharp_build_inputs.py"))
        .expect("read C# build gate");
    assert!(script.contains("validate_roslyn_session_implementation"));
    assert!(script.contains("run_roslyn_tests=True"));
    assert!(script.contains("csharp2vir-roslyn-tests.dll"));
    assert!(script.contains("str(reference_root)"));

    let manifest = load("develop/specs/vectors/manifest.json");
    let record = manifest["vectors"]
        .as_array()
        .unwrap()
        .iter()
        .find(|record| record["path"] == "develop/specs/vectors/csharp-profile-v0.json")
        .expect("C# vector manifest record");
    assert!(record["implementation_test_owners"]
        .as_array()
        .unwrap()
        .iter()
        .any(|owner| owner == "crates/mpk-cli/tests/csharp_roslyn_session.rs"));
}

#[test]
fn provisioned_offline_closure_executes_the_roslyn_harness() {
    if !cfg!(target_os = "linux") {
        return;
    }

    let root = repository_root();
    let profile = load("develop/specs/vectors/csharp-profile-v0.json");
    let hash = profile["toolchain_inputs"]["toolchain_inputs_sha256"]
        .as_str()
        .unwrap();
    let cache = root
        .join("release/build-input-cache/csharp")
        .join(hash)
        .join("archives");
    let archives = profile["toolchain_inputs"]["archives"].as_array().unwrap();
    let present = archives
        .iter()
        .filter(|record| {
            let suffix = match record["kind"].as_str().unwrap() {
                "tar.gz" => ".tar.gz",
                "nupkg" => ".nupkg",
                kind => panic!("unexpected archive kind {kind}"),
            };
            cache
                .join(format!("{}{}", record["id"].as_str().unwrap(), suffix))
                .is_file()
        })
        .count();
    assert!(
        present == 0 || present == archives.len(),
        "partial C# archive cache"
    );
    if present == 0 {
        return;
    }

    let output = Command::new(root.join("scripts/build-csharp-frontend.sh"))
        .arg("--test-roslyn")
        .env_clear()
        .env("PATH", "/usr/bin:/bin")
        .output()
        .expect("execute pinned Roslyn session harness");
    assert!(
        output.status.success(),
        "stdout={} stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output.stdout.is_empty());
    assert!(output.stderr.is_empty());
}
