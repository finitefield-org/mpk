from pathlib import Path
import datetime
import hashlib
import json
import subprocess
import sys
import time
name=sys.argv[1]
root=Path('/private/tmp')/('mpk-go-array-equality-review-'+name)
root.mkdir(exist_ok=False)
source=Path('develop/migrations/go-practical/t02-w01/verification/array-equality/review-inputs.py')
(root/'review-inputs.py').write_bytes(source.read_bytes())
(root/'driver.py').write_bytes(Path(__file__).read_bytes())
command=['python3',str(source)]
record={'command':command,'started_at_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'actual_driver_sha256':hashlib.sha256((root/'driver.py').read_bytes()).hexdigest(),'actual_review_source_sha256':hashlib.sha256((root/'review-inputs.py').read_bytes()).hexdigest(),'status':'running'}
p=root/'result.json';p.write_text(json.dumps(record,indent=2)+'\n')
start=time.monotonic()
with(root/'stdout.log').open('wb')as stdout,(root/'stderr.log').open('wb')as stderr:
    code=subprocess.run(command,stdout=stdout,stderr=stderr,check=False).returncode
record.update(status='passed'if code==0 else'failed',exit_code=code,seconds=time.monotonic()-start,finished_at_utc=datetime.datetime.now(datetime.timezone.utc).isoformat())
p.write_text(json.dumps(record,indent=2)+'\n');print(json.dumps(record));raise SystemExit(code)
