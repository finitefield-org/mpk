"""Check only the Rust files changed by source-root matching."""
from pathlib import Path
import hashlib
import json
import subprocess

ROOT = Path(__file__).resolve().parent
REPO = ROOT.parents[5]
files = ['crates/mpk-vc/src/go_practical_type_keys/snapshot_inputs.rs', 'crates/mpk-cli/src/lib.rs', 'crates/mpk-cli/src/go_practical_source_roots.rs', 'crates/mpk-cli/tests/go_practical_source_roots.rs']
command = ['rustfmt', '--edition=2021', '--config', 'skip_children=true', '--check', *files]
def seal(path):
    raw = path.read_bytes()
    return {'path': str(path.relative_to(REPO)), 'bytes': len(raw), 'sha256': hashlib.sha256(raw).hexdigest()}
before = [seal(REPO / name) for name in files]
run = subprocess.run(command, cwd=REPO, capture_output=True, check=False)
assert before == [seal(REPO / name) for name in files]
result = {'status': 'passed' if run.returncode == 0 else 'failed', 'selection_reason': 'Check formatting only for the modified snapshot view and its Linux CLI/test consumers.', 'command': command, 'inputs': before, 'exit_code': run.returncode, 'stdout': run.stdout.decode(), 'stderr': run.stderr.decode(), 'full_T_gate': 'Deferred to final T02-W04: ./scripts/check-fast.sh'}
out = ROOT / 'format.json'; assert not out.exists(); out.write_text(json.dumps(result, indent=2) + '\n'); print(json.dumps({'status': result['status'], 'files': len(files)})); raise SystemExit(run.returncode)
