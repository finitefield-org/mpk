from pathlib import Path
from hashlib import sha256
from datetime import datetime,timezone
import json,os,subprocess,time,sys
repo=Path('/private/tmp/mpk-w09-packed-pattern-proofs')
out=Path('/tmp/mpk-w09-context-refinement')/sys.argv[1];out.mkdir(parents=True,exist_ok=True)
manifest={str(p.relative_to(repo)):sha256(p.read_bytes()).hexdigest() for p in repo.rglob('*') if p.is_file() and (p.suffix=='.rs' or p.name in ('Cargo.toml','Cargo.lock')) and 'target' not in p.parts and '.git' not in p.parts}
(out/'source-manifest.json').write_text(json.dumps(manifest,indent=2,sort_keys=True)+'\n')
env=os.environ.copy();env['CARGO_TARGET_DIR']='/tmp/mpk-w09-pattern-candidates-target';env['MPK_W09_CONTEXT_BOOLEAN_UNIT_OUTPUT']='/tmp/mpk-w09-context-refinement/context-certificates';env['MPK_W09_PATTERN_CANDIDATE_UNIT_OUTPUT']='/tmp/mpk-w09-context-refinement/candidate-certificates'
selection='Verify proof-producing contextual Boolean reduction with actual equality proof terms, free branch values and let substitution; missing/conflicting facts reject and wrong proof terms fail core checking. Test full candidate generation plus all candidate linkage regressions and the shared substitution consumer. Also replay the exact ownership-proof and closed-default consumers against their unchanged original pins because they use the same normalizer. Clippy includes tests. Original source/application proofs and the full T06-W12 gate remain pending.'
rows=[]
for label,command,count in [('context-unit',['cargo','test','-p','mpk-vc','--lib','context_boolean_normalization_','--','--nocapture'],1),('candidate-units',['cargo','test','-p','mpk-vc','--lib','pattern_refinement_candidate_','--','--nocapture'],3),('ownership-unit',['cargo','test','-p','mpk-vc','--lib','ownership_normalization_preserves_free_selector_under_binder','--','--nocapture'],1),('ownership-consumer',['cargo','test','-p','mpk-vc','--test','csharp_practical_vc','csharp_03_t06_w09_ownership_proof_candidates','--','--nocapture'],1),('closed-default-consumer',['cargo','test','-p','mpk-vc','--test','csharp_practical_vc','csharp_03_t06_w09_binding_defaults_closed_boolean_proof_candidates','--','--nocapture'],1),('clippy',['cargo','clippy','-p','mpk-vc','--lib','--tests','--','-D','warnings'],0),('format',['cargo','fmt','--all','--','--check'],0)]:
 for name,digest in manifest.items():assert sha256((repo/name).read_bytes()).hexdigest()==digest,name
 log=out/(label+'.log');started=datetime.now(timezone.utc).isoformat();start=time.monotonic()
 (out/'status.json').write_text(json.dumps(dict(status='running',stage=label,supervisor_pid=os.getpid(),selection_reason=selection,stages=rows),indent=2)+'\n')
 with log.open('wb') as f:r=subprocess.run(command,cwd=repo,env=env,stdout=f,stderr=subprocess.STDOUT)
 rows.append(dict(stage=label,command=command,exit_code=r.returncode,started_at=started,finished_at=datetime.now(timezone.utc).isoformat(),elapsed_seconds=round(time.monotonic()-start,3),log_sha256=sha256(log.read_bytes()).hexdigest()))
 for name,digest in manifest.items():assert sha256((repo/name).read_bytes()).hexdigest()==digest,name
 (out/'status.json').write_text(json.dumps(dict(status='running' if r.returncode==0 else 'failed',stage=label,selection_reason=selection,stages=rows),indent=2)+'\n')
 print(label,r.returncode,flush=True)
 if r.returncode:print(log.read_text());raise SystemExit(r.returncode)
 if count:assert f'test result: ok. {count} passed; 0 failed;' in log.read_text(),label
(out/'status.json').write_text(json.dumps(dict(status='passed',unique_tests_passed=7,selection_reason=selection,stages=rows),indent=2)+'\n')
