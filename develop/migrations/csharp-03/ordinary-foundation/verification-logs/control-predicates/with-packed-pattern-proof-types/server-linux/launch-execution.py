import json,subprocess,time,sys
from pathlib import Path
from hashlib import sha256
from datetime import datetime,timezone
base=Path('/private/tmp/mpk-w09-packed-pattern-proofs/develop/migrations/csharp-03/ordinary-foundation/verification-logs/control-predicates/with-packed-pattern-proof-types')
script=base/'tool-sources/launch-linux.py'
out=base/'server-linux';out.mkdir(exist_ok=True)
commit='130a3ca4380868953f8ec56c5aad99582715e089'
command=['ssh','-i','/Users/kazuyoshitoshiya/ffvps.pem','-o','BatchMode=yes','-o','ConnectTimeout=20','root@162.43.92.154','python3','-',commit]
start=time.monotonic();started=datetime.now(timezone.utc).isoformat()
with (out/'launch-execution.log.txt').open('wb') as log:
    result=subprocess.run(command,input=script.read_bytes(),stdout=log,stderr=subprocess.STDOUT)
receipt=dict(command=command,started_at=started,finished_at=datetime.now(timezone.utc).isoformat(),elapsed_seconds=round(time.monotonic()-start,3),exit_code=result.returncode,source_script_sha256=sha256(script.read_bytes()).hexdigest(),log_sha256=sha256((out/'launch-execution.log.txt').read_bytes()).hexdigest(),source_commit=commit)
(out/'launch-execution.json').write_text(json.dumps(receipt,indent=2)+'\n')
print((out/'launch-execution.log.txt').read_text(),flush=True)
raise SystemExit(result.returncode)
