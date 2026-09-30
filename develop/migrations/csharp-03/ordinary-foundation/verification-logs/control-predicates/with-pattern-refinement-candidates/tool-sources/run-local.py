import os,json,subprocess,time,sys
from pathlib import Path
from datetime import datetime,timezone
from hashlib import sha256
repo=Path('/private/tmp/mpk-w09-packed-pattern-proofs')
out=Path('/tmp/mpk-w09-pattern-candidates/local-final');out.mkdir(parents=True,exist_ok=True)
manifest={str(p.relative_to(repo)):sha256(p.read_bytes()).hexdigest() for p in repo.rglob('*') if p.is_file() and (p.suffix=='.rs' or p.name in ('Cargo.toml','Cargo.lock')) and 'target' not in p.parts and '.git' not in p.parts}
(out/'source-manifest.json').write_text(json.dumps(manifest,indent=2,sort_keys=True)+'\n')
env=os.environ.copy();env['CARGO_TARGET_DIR']='/tmp/mpk-w09-pattern-candidates-target';env['MPK_W09_PACKED_PATTERN_OUTPUT']='/tmp/mpk-w09-pattern-candidates/pins'
rows=[]
selection='Test the new candidate envelope and all affected scope/projection consumers, then bind all 113 reconstructed original paths and reject all helper-only original certificates across 18 contexts. Preserve all old declaration bodies and source goals. Clippy includes library and integration targets. Full T06 gate deferred to W12.'
for label,command,count in [('scope-units',['cargo','test','-p','mpk-vc','--lib','pattern_','--','--nocapture'],7),('original-contexts',['cargo','test','-p','mpk-vc','--test','csharp_practical_vc','csharp_03_t06_w09_packed_pattern_proof_types_cover_complete_original_environments','--','--nocapture'],1),('clippy',['cargo','clippy','-p','mpk-vc','--lib','--tests','--','-D','warnings'],0),('format',['cargo','fmt','--all','--','--check'],0)]:
    for name,digest in manifest.items():assert sha256((repo/name).read_bytes()).hexdigest()==digest,name
    started=datetime.now(timezone.utc).isoformat();start=time.monotonic();log=out/(label+'.log')
    (out/'status.json').write_text(json.dumps(dict(status='running',stage=label,supervisor_pid=os.getpid(),selection_reason=selection,stages=rows),indent=2)+'\n')
    with log.open('wb') as f:r=subprocess.run(command,cwd=repo,env=env,stdout=f,stderr=subprocess.STDOUT)
    data=log.read_bytes()
    rows.append(dict(stage=label,command=command,exit_code=r.returncode,started_at=started,finished_at=datetime.now(timezone.utc).isoformat(),elapsed_seconds=round(time.monotonic()-start,3),log_sha256=sha256(data).hexdigest()))
    for name,digest in manifest.items():assert sha256((repo/name).read_bytes()).hexdigest()==digest,name
    status='running' if r.returncode==0 else 'failed'
    (out/'status.json').write_text(json.dumps(dict(status=status,stage=label,supervisor_pid=os.getpid(),selection_reason=selection,stages=rows),indent=2)+'\n')
    print(label,r.returncode,flush=True)
    if r.returncode:raise SystemExit(r.returncode)
    if count:assert f'test result: ok. {count} passed; 0 failed;' in data.decode(),label
(out/'status.json').write_text(json.dumps(dict(status='passed',selection_reason=selection,unique_tests_passed=8,stages=rows),indent=2)+'\n')
