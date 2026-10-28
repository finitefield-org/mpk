"""Format-check the changed production and integration Rust files only."""
from pathlib import Path
import hashlib
import json
import subprocess

ROOT = Path(__file__).resolve().parent
REPO = ROOT.parents[5]
paths = [REPO / name for name in ['crates/mpk-cli/src/lib.rs', 'crates/mpk-cli/src/go_practical_capture_binding.rs', 'crates/mpk-cli/tests/go_practical_capture_binding.rs', 'crates/mpk-vc/src/go_practical_type_keys/snapshot_inputs.rs']]
def seal(path):
    raw = path.read_bytes()
    return {'path': str(path), 'bytes': len(raw), 'sha256': hashlib.sha256(raw).hexdigest()}
before = [seal(p) for p in paths]
command = ['rustfmt', '--edition', '2021', '--check', '--config', 'skip_children=true', *map(str, paths)]
run = subprocess.run(command, cwd=REPO, capture_output=True, check=False)
after = [seal(p) for p in paths]
result = {'status': 'passed' if run.returncode == 0 and before == after else 'failed', 'selection_reason': 'Format-check only the changed Rust modules, accessor and binding integration test; unrelated modules are not recursively reformatted.', 'command': command, 'inputs': before, 'inputs_unchanged': before == after, 'exit_code': run.returncode, 'stdout': run.stdout.decode(), 'stderr': run.stderr.decode(), 'full_T_gate': 'Deferred to final T02-W04: ./scripts/check-fast.sh'}
out = ROOT / 'format.json'; assert not out.exists(); out.write_text(json.dumps(result, indent=2) + '\n')
print(json.dumps({'status': result['status'], 'files': len(paths)})); raise SystemExit(result['status'] != 'passed')
