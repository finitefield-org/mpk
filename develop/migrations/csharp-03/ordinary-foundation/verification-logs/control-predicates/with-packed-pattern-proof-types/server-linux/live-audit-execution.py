from pathlib import Path
import subprocess,json,time
from datetime import datetime,timezone
from hashlib import sha256
base=Path('/private/tmp/mpk-w09-packed-pattern-proofs/develop/migrations/csharp-03/ordinary-foundation/verification-logs/control-predicates/with-packed-pattern-proof-types/server-linux')
script=base/'live-audit.py'
command=['ssh','-i','/Users/kazuyoshitoshiya/ffvps.pem','-o','BatchMode=yes','-o','ConnectTimeout=20','root@162.43.92.154','python3','-']
start=time.monotonic();started=datetime.now(timezone.utc).isoformat()
with (base/'live-audit.log.txt').open('wb') as log:result=subprocess.run(command,input=script.read_bytes(),stdout=log,stderr=subprocess.STDOUT)
receipt=dict(command=command,exit_code=result.returncode,started_at=started,finished_at=datetime.now(timezone.utc).isoformat(),elapsed_seconds=round(time.monotonic()-start,3),source_script_sha256=sha256(script.read_bytes()).hexdigest(),log_sha256=sha256((base/'live-audit.log.txt').read_bytes()).hexdigest())
(base/'live-audit-execution.json').write_text(json.dumps(receipt,indent=2)+'\n')
print((base/'live-audit.log.txt').read_text(),flush=True)
raise SystemExit(result.returncode)
