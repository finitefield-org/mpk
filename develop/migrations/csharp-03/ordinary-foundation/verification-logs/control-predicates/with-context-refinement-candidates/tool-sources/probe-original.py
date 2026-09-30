from pathlib import Path
from hashlib import sha256
from datetime import datetime,timezone
import json,os,subprocess,time
repo=Path('/private/tmp/mpk-w09-packed-pattern-proofs');out=Path('/tmp/mpk-w09-context-refinement/original-probe');out.mkdir(exist_ok=True)
p=repo/'crates/mpk-vc/tests/support/csharp_practical_ordinary_control_pattern_packed_proof_tests.rs';prior=p.read_bytes()
probe=r'''
#[test]
fn csharp_03_t06_w09_refinement_generation_original_probe() {
    std::thread::Builder::new().stack_size(64 * 1024 * 1024).spawn(|| {
        let mut visited = 0;
        each_context(|id, vir| {
            if id != "is_binding" { return; }
            visited += 1;
            let p = generate_csharp_practical_ordinary_control_predicates_with_packed_pattern_proof_types(vir).unwrap();
            let required = csharp_practical_ordinary_pattern_refinement_candidate_theorems(&p).unwrap();
            let outcome = generate_csharp_practical_ordinary_pattern_refinement_candidate(&p);
            let report = match outcome {
                Ok(candidate) => {
                    let checked = mpk_kernel::verify_certificate_bytes(candidate.certificate_bytes()).unwrap();
                    serde_json::json!({"status":"generated_and_rust_checked", "context":id, "required_theorems":required.len(), "axiom_count":checked.axiom_count, "proof_check_pending":candidate.proof_check_pending(), "application_scope_pending":true})
                },
                Err(error) => serde_json::json!({"status":"original_proofs_pending", "context":id, "required_theorems":required.len(), "error":format!("{error:?}"), "application_scope_pending":true}),
            };
            std::fs::write("/tmp/mpk-w09-context-refinement/original-probe/outcome.json", serde_json::to_vec_pretty(&report).unwrap()).unwrap();
            println!("Original source capability: {report}");
        });
        assert_eq!(visited,1);
    }).unwrap().join().unwrap();
}
'''
(out/'probe.rs').write_text(probe)
p.write_bytes(prior+probe.encode())
manifest={str(x.relative_to(repo)):sha256(x.read_bytes()).hexdigest() for x in repo.rglob('*') if x.is_file() and (x.suffix=='.rs' or x.name in ('Cargo.toml','Cargo.lock')) and 'target' not in x.parts and '.git' not in x.parts}
(out/'source-manifest.json').write_text(json.dumps(manifest,indent=2,sort_keys=True)+'\n')
command=['cargo','test','-p','mpk-vc','--test','csharp_practical_vc','csharp_03_t06_w09_refinement_generation_original_probe','--','--nocapture'];env=os.environ.copy();env['CARGO_TARGET_DIR']='/tmp/mpk-w09-pattern-candidates-target'
start=time.monotonic();started=datetime.now(timezone.utc).isoformat()
try:
 with (out/'test.log').open('wb') as log:r=subprocess.run(command,cwd=repo,env=env,stdout=log,stderr=subprocess.STDOUT)
 for name,digest in manifest.items():assert sha256((repo/name).read_bytes()).hexdigest()==digest,name
finally:
 assert p.read_bytes()==prior+probe.encode()
 p.write_bytes(prior)
(out/'execution.json').write_text(json.dumps(dict(command=command,exit_code=r.returncode,started_at=started,finished_at=datetime.now(timezone.utc).isoformat(),elapsed_seconds=round(time.monotonic()-start,3),log_sha256=sha256((out/'test.log').read_bytes()).hexdigest(),temporary_probe_source_sha256=sha256(probe.encode()).hexdigest(),test_file_restored_sha256=sha256(p.read_bytes()).hexdigest(),selection_reason='Measure actual complete candidate generation for the original reconstructed is_binding source; a successful diagnostic process is not a discharged source proof. Other contexts and all application proofs remain pending.'),indent=2)+'\n')
print((out/'test.log').read_text());raise SystemExit(r.returncode)
