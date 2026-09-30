"""Launch the requested Linux replay from an exact published source commit."""
from datetime import datetime, timezone
from hashlib import sha256
import json
from pathlib import Path
import subprocess
import sys
import time


commit = sys.argv[1]
assert len(commit) == 40 and all(c in '0123456789abcdef' for c in commit)
script = Path(__file__).with_name('launch-linux.py').read_bytes()
out = Path('/tmp/mpk-w09-concrete-operation-proofs/server-launch-' + commit[:8])
assert not out.exists()
out.mkdir(parents=True)
command = ['ssh', '-i', '/Users/kazuyoshitoshiya/ffvps.pem', '-o', 'BatchMode=yes',
           '-o', 'ConnectTimeout=20', 'root@162.43.92.154', 'python3', '-', commit]
started = datetime.now(timezone.utc).isoformat()
start = time.monotonic()
result = subprocess.run(command, input=script, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
(out / 'stdout.log').write_bytes(result.stdout)
(out / 'stderr.log').write_bytes(result.stderr)
receipt = dict(command=command, exit_code=result.returncode, started_at=started,
               finished_at=datetime.now(timezone.utc).isoformat(),
               elapsed_seconds=round(time.monotonic() - start, 3),
               script_sha256=sha256(script).hexdigest(),
               stdout_sha256=sha256(result.stdout).hexdigest(),
               stderr_sha256=sha256(result.stderr).hexdigest())
(out / 'execution.json').write_text(json.dumps(receipt, indent=2, sort_keys=True) + '\n')
if result.returncode:
    print(result.stderr.decode())
    raise SystemExit(result.returncode)
launch = json.loads(next(line for line in reversed(result.stdout.decode().splitlines()) if line.startswith('{')))
assert launch['commit'] == commit and launch['status'] == 'launched'
(out / 'launch.json').write_text(json.dumps(launch, indent=2, sort_keys=True) + '\n')
print(json.dumps(launch, sort_keys=True))
