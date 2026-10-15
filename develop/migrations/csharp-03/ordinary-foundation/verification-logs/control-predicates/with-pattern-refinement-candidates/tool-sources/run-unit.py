from pathlib import Path
import json,os,subprocess,time,sys
from hashlib import sha256
from datetime import datetime,timezone
repo=Path('/private/tmp/mpk-w09-packed-pattern-proofs')
out=Path('/tmp/mpk-w09-pattern-candidates')/sys.argv[1];out.mkdir(parents=True,exist_ok=True)
manifest={str(p.relative_to(repo)):sha256(p.read_bytes()).hexdigest() for p in repo.rglob('*') if p.is_file() and (p.suffix=='.rs' or p.name in ('Cargo.toml','Cargo.lock')) and 'target' not in p.parts and '.git' not in p.parts}
(out/'source-manifest.json').write_text(json.dumps(manifest,indent=2,sort_keys=True)+'\n')
command=['cargo','test','-p','mpk-vc','--lib','pattern_refinement_candidate_','--','--nocapture']
env=os.environ.copy();env['CARGO_TARGET_DIR']='/tmp/mpk-w09-pattern-candidates-target';env['MPK_W09_PATTERN_CANDIDATE_UNIT_OUTPUT']='/tmp/mpk-w09-pattern-candidates/unit-certificates'
start=time.monotonic();started=datetime.now(timezone.utc).isoformat()
with (out/'test.log').open('wb') as log:r=subprocess.run(command,cwd=repo,env=env,stdout=log,stderr=subprocess.STDOUT)
for name,digest in manifest.items():assert sha256((repo/name).read_bytes()).hexdigest()==digest,name
(out/'verification.json').write_text(json.dumps(dict(selection_reason='Validate all reconstructed path types, whole-context binding and unmodified source prefixes. Reject missing/type-swapped/source-edited candidates; linkage does not accept wrong proofs. Full T06 gate deferred to W12.',command=command,exit_code=r.returncode,started_at=started,finished_at=datetime.now(timezone.utc).isoformat(),elapsed_seconds=round(time.monotonic()-start,3),log_sha256=sha256((out/'test.log').read_bytes()).hexdigest()),indent=2)+'\n')
print((out/'test.log').read_text());raise SystemExit(r.returncode)
