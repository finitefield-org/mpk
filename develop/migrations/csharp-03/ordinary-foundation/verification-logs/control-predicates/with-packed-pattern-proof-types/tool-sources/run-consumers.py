import os, json, subprocess, sys, time
from pathlib import Path
from hashlib import sha256
from datetime import datetime, timezone
root=Path('/private/tmp/mpk-w09-packed-pattern-proofs')
out=Path('/tmp/mpk-w09-packed-pattern/local-consumers')
out.mkdir(parents=True,exist_ok=True)
env=os.environ.copy()
env['CARGO_TARGET_DIR']='/tmp/mpk-w09-packed-pattern-target'
rows=[]
reason='The changed private generation selector is shared by the existing pattern APIs. Run their scope/projection units and ownership foundation unit, compile all test targets with Clippy, and check formatting. The source integration already runs separately. T06-wide gate is deferred to W12.'
for name, command in [('scope-units',['cargo','test','-p','mpk-vc','--lib','pattern_','--','--nocapture']),('ownership-unit',['cargo','test','-p','mpk-vc','--lib','ownership_normalization_preserves_free_selector_under_binder','--','--nocapture']),('clippy',['cargo','clippy','-p','mpk-vc','--lib','--tests','--','-D','warnings']),('fmt',['cargo','fmt','--all','--','--check'])]:
    log=out/(name+'.log')
    start=time.monotonic()
    started=datetime.now(timezone.utc).isoformat()
    with log.open('wb') as f:
        result=subprocess.run(command,cwd=root,env=env,stdout=f,stderr=subprocess.STDOUT)
    rows.append(dict(stage=name,command=command,started_at=started,finished_at=datetime.now(timezone.utc).isoformat(),exit_code=result.returncode,elapsed_seconds=round(time.monotonic()-start,3),log_sha256=sha256(log.read_bytes()).hexdigest()))
    (out/'verification.json').write_text(json.dumps(dict(selection_reason=reason,status='running' if result.returncode==0 else 'failed',stages=rows),indent=2)+'\n')
    print(name,result.returncode,flush=True)
    if result.returncode: sys.exit(result.returncode)
(out/'verification.json').write_text(json.dumps(dict(selection_reason=reason,status='passed',stages=rows),indent=2)+'\n')
