use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use mpk_cert::{
    axiom_report_hash_for_report, build_axiom_report, build_export_block,
    encode::{
        AxiomReport, Certificate, CertificateHashes, Declaration, DeclarationKind, LevelNode,
        ProofNode, TermNode,
    },
    encode_certificate, export_block_hash, hash_hex,
};
use mpk_kernel::{profile_certificate_bytes, verify_certificate_bytes, KernelProfileReport};
use serde_json::Value;

const UPDATE_ENV: &str = "MPK_UPDATE_ALPHA004_PROFILE";
const ALPHA_VC_MANIFEST: &str = "fixtures/vc-alpha/manifest.json";
const PROFILE_REPORT: &str = "perf/alpha-004-fast-kernel-profile.md";

#[test]
fn alpha004_fast_kernel_profile_report_identifies_hotspots() {
    let repo_root = repo_root();
    let member_count = alpha_vc_member_count(&repo_root);
    let certificate = alpha_shaped_profile_certificate(member_count);
    let bytes = encode_certificate(&certificate);

    let verification = verify_certificate_bytes(&bytes).expect("profile workload verifies");
    assert_eq!(verification.declaration_count, member_count);

    let profile = profile_certificate_bytes(&bytes).expect("profile workload profiles");
    assert_eq!(profile.module, "Bench.Alpha004.FastKernelProfile");
    assert_eq!(profile.input_bytes, bytes.len());
    assert_eq!(profile.table_counts.declarations, member_count);
    assert_eq!(profile.table_counts.proof_nodes, member_count);
    assert_eq!(profile.table_counts.terms, 2);
    assert!(profile.timings.total_nanos > 0);
    assert!(profile.combined_cache_metrics.defeq.calls >= member_count as u64);
    assert!(profile
        .hotspots_by_elapsed()
        .iter()
        .any(|hotspot| hotspot.name == "defeq"));

    let report = render_profile_report(&profile, member_count);
    let report_path = repo_root.join(PROFILE_REPORT);
    if env::var_os(UPDATE_ENV).is_some() {
        if let Some(parent) = report_path.parent() {
            fs::create_dir_all(parent).expect("create perf report directory");
        }
        fs::write(&report_path, report).expect("write ALPHA-004 profile report");
        return;
    }

    let recorded = fs::read_to_string(&report_path).unwrap_or_else(|error| {
        panic!(
            "read ALPHA-004 profile report {}: {error}",
            report_path.display()
        )
    });
    for required in [
        "# ALPHA-004 Fast Kernel Profile",
        "## Stage timings",
        "## Cache and defeq metrics",
        "## Hotspots identified",
        "decode",
        "typecheck",
        "defeq",
        "proof-node checking",
        "fixtures/vc-alpha/manifest.json",
        &format!("Active VC member count | `{member_count}`"),
    ] {
        assert!(
            recorded.contains(required),
            "profile report must include `{required}`"
        );
    }
}

fn alpha_vc_member_count(repo_root: &Path) -> usize {
    let path = repo_root.join(ALPHA_VC_MANIFEST);
    let manifest = fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("read {}: {error}", path.display()));
    let value: Value = serde_json::from_str(&manifest)
        .unwrap_or_else(|error| panic!("decode {}: {error}", path.display()));
    let member_count = value
        .pointer("/artifacts/vc/member_count")
        .and_then(Value::as_u64)
        .expect("alpha VC manifest records member_count");
    let group_count = value
        .pointer("/artifacts/skeleton/group_count")
        .and_then(Value::as_u64)
        .expect("alpha VC manifest records group_count");
    let function_count = value
        .pointer("/source/function_count")
        .and_then(Value::as_u64)
        .expect("alpha VC manifest records function_count");
    assert_eq!(
        group_count,
        function_count * 2,
        "active skeleton contains contract and panic-free groups per function"
    );
    assert!(member_count > 0, "active VC corpus contains members");
    usize::try_from(member_count).expect("alpha VC member count fits usize")
}

fn alpha_shaped_profile_certificate(declaration_count: usize) -> Certificate {
    let mut name_table = Vec::with_capacity(declaration_count);
    let declarations = (0..declaration_count)
        .map(|index| {
            name_table.push(format!("Bench.Alpha004.Theorem{index:04}"));
            Declaration {
                name: u32::try_from(index).expect("profile declaration index fits u32"),
                kind: DeclarationKind::Theorem { ty: 1, proof: 0 },
            }
        })
        .collect::<Vec<_>>();
    let proof_node_table = (0..declaration_count)
        .map(|_| ProofNode::Exact {
            term: 0,
            expected_type: 1,
        })
        .collect::<Vec<_>>();

    finalize_certificate(Certificate {
        module: "Bench.Alpha004.FastKernelProfile".to_owned(),
        imports: Vec::new(),
        name_table,
        level_table: vec![LevelNode::Zero, LevelNode::Succ(0)],
        term_table: vec![TermNode::Sort(0), TermNode::Sort(1)],
        proof_node_table,
        declarations,
        theory_certificates: Vec::new(),
        export_block: Vec::new(),
        axiom_report: AxiomReport::default(),
        source_manifest: None,
        hashes: CertificateHashes::default(),
    })
}

fn finalize_certificate(mut certificate: Certificate) -> Certificate {
    certificate.export_block = build_export_block(&certificate).expect("export block builds");
    certificate.axiom_report = build_axiom_report(&certificate).expect("axiom report builds");
    certificate.hashes.export_hash = export_block_hash(&certificate.export_block);
    certificate.hashes.axiom_report_hash = axiom_report_hash_for_report(&certificate.axiom_report);
    certificate
}

fn render_profile_report(profile: &KernelProfileReport, member_count: usize) -> String {
    let hotspots = profile.hotspots_by_elapsed();
    let mut output = String::new();
    output.push_str("# ALPHA-004 Fast Kernel Profile\n\n");
    output.push_str("Schema: `mpk.alpha004.fast_kernel_profile.v0`\n\n");
    output.push_str("Timings are local wall-clock measurements from the Rust fast-kernel profile harness. The workload is an ALPHA-002-sized source-free certificate shape; it does not claim to prove the ALPHA VC obligations.\n\n");
    output.push_str("## Workload\n\n");
    output.push_str("| Field | Value |\n");
    output.push_str("| --- | --- |\n");
    output.push_str(&format!("| ALPHA-002 manifest | `{ALPHA_VC_MANIFEST}` |\n"));
    output.push_str(&format!("| Active VC member count | `{member_count}` |\n"));
    output.push_str(&format!("| Profile module | `{}` |\n", profile.module));
    output.push_str(&format!("| Input bytes | `{}` |\n", profile.input_bytes));
    output.push_str(&format!(
        "| Certificate hash | `{}` |\n",
        hash_hex(&profile.certificate_hash)
    ));
    output.push_str(&format!(
        "| Declarations | `{}` |\n",
        profile.table_counts.declarations
    ));
    output.push_str(&format!(
        "| Proof nodes | `{}` |\n",
        profile.table_counts.proof_nodes
    ));
    output.push_str(&format!("| Terms | `{}` |\n", profile.table_counts.terms));
    output.push_str("\n## Stage timings\n\n");
    output.push_str("| Stage | Elapsed ms | Notes |\n");
    output.push_str("| --- | ---: | --- |\n");
    output.push_str(&format!(
        "| decode | {} | canonical decode and re-encode validation |\n",
        millis(profile.timings.decode_nanos)
    ));
    output.push_str(&format!(
        "| typecheck | {} | declaration translation and core checking |\n",
        millis(profile.timings.typecheck_nanos)
    ));
    output.push_str(&format!(
        "| defeq | {} | nested cache-instrumented conversion calls |\n",
        millis(profile.combined_cache_metrics.defeq.elapsed_nanos)
    ));
    output.push_str(&format!(
        "| proof-node checking | {} | profile-gated proof-node traversal and checking |\n",
        millis(profile.timings.proof_node_check_nanos)
    ));
    output.push_str(&format!(
        "| section recompute | {} | export block, axiom report, and hash recomputation |\n",
        millis(profile.timings.section_recompute_nanos)
    ));
    output.push_str(&format!(
        "| total | {} | end-to-end profile harness time |\n",
        millis(profile.timings.total_nanos)
    ));
    output.push_str("\n## Cache and defeq metrics\n\n");
    output.push_str("| Scope | Operation | Calls | Hits | Misses | Elapsed ms |\n");
    output.push_str("| --- | --- | ---: | ---: | ---: | ---: |\n");
    push_cache_rows(
        &mut output,
        "declarations",
        &profile.declaration_cache_metrics,
    );
    push_cache_rows(&mut output, "proof nodes", &profile.proof_cache_metrics);
    push_cache_rows(&mut output, "combined", &profile.combined_cache_metrics);
    output.push_str("\n## Hotspots identified\n\n");
    for (index, hotspot) in hotspots.iter().enumerate() {
        output.push_str(&format!(
            "{}. `{}` at {} ms.\n",
            index + 1,
            hotspot.name,
            millis(hotspot.elapsed_nanos)
        ));
    }
    output.push_str("\nThe optimization follow-up should start with the largest measured stage above, then inspect nested cache key construction plus defeq hit/miss costs before changing cache layout or proof-node locality. The profile keeps the trust boundary unchanged: it measures only canonical certificate decode, core declaration checking, cached defeq, and profile-gated proof-node checking.\n");
    output
}

fn push_cache_rows(output: &mut String, scope: &str, metrics: &mpk_kernel::CheckerCacheMetrics) {
    for (operation, metric) in [
        ("infer", &metrics.infer),
        ("whnf", &metrics.whnf),
        ("defeq", &metrics.defeq),
        ("check", &metrics.check),
    ] {
        output.push_str(&format!(
            "| {scope} | {operation} | {} | {} | {} | {} |\n",
            metric.calls,
            metric.hits,
            metric.misses,
            millis(metric.elapsed_nanos)
        ));
    }
}

fn millis(nanos: u128) -> String {
    let whole = nanos / 1_000_000;
    let fractional = (nanos % 1_000_000) / 1_000;
    format!("{whole}.{fractional:03}")
}

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .components()
        .collect()
}
