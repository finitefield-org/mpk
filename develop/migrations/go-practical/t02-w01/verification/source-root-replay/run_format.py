"""Check the changed original-archive replay test's formatting."""
from pathlib import Path
import hashlib
import json
import subprocess

ROOT = Path(__file__).resolve().parent
REPO = ROOT.parents[5]
file = REPO / 'crates/mpk-cli/tests/go_practical_source_roots.rs'
raw = file.read_bytes()
command = ['rustfmt', '--edition=2021', '--config', 'skip_children=true', '--check', str(file.relative_to(REPO))]
run = subprocess.run(command, cwd=REPO, capture_output=True, check=False)
assert file.read_bytes() == raw
result = {'status':'passed' if run.returncode == 0 else 'failed', 'selection_reason':'Format-check the changed root replay test only.', 'command':command, 'input_sha256':hashlib.sha256(raw).hexdigest(), 'input_bytes':len(raw), 'exit_code':run.returncode, 'stdout':run.stdout.decode(), 'stderr':run.stderr.decode(), 'full_T_gate':'Deferred to final T02-W04: ./scripts/check-fast.sh'}
out = ROOT/'format.json'; assert not out.exists(); out.write_text(json.dumps(result,indent=2)+'\n'); print(json.dumps({'status':result['status']})); raise SystemExit(run.returncode)
