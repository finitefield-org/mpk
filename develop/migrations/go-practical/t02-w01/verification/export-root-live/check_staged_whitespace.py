"""Check the staged change without altering the original test output."""
from pathlib import Path
import hashlib
import json
import subprocess

ROOT = Path(__file__).resolve().parent
REPO = ROOT.parents[5]
path = 'develop/migrations/go-practical/t02-w01/verification/export-root-live/projection-original/test.stdout'
full_command = ['/usr/bin/git', 'diff', '--cached', '--check']
full = subprocess.run(full_command, cwd=REPO, capture_output=True, text=True, check=False)
expected = path + ':11: new blank line at EOF.\n'
assert full.returncode == 2 and full.stdout == expected and not full.stderr
original = (REPO / path).read_bytes()
receipt = json.loads((ROOT / 'projection-original/result.json').read_bytes())['stdout']
assert len(original) == receipt['bytes'] and hashlib.sha256(original).hexdigest() == receipt['sha256']
staged_command = ['/usr/bin/git', 'show', ':' + path]
staged = subprocess.run(staged_command, cwd=REPO, capture_output=True, check=False)
assert staged.returncode == 0 and staged.stdout == original and not staged.stderr
selected_command = ['/usr/bin/git', 'diff', '--cached', '--check', '--', '.', ':(exclude)' + path]
selected = subprocess.run(selected_command, cwd=REPO, capture_output=True, text=True, check=False)
result = {'status': 'passed' if selected.returncode == 0 else 'failed', 'selection_reason': 'Check every staged change while preserving the exact original test output, whose final blank line is part of the recorded bytes.', 'full_command': full_command, 'full_exit_code': full.returncode, 'full_stdout': full.stdout, 'preserved_original_evidence_path': path, 'original_stdout_bytes': len(original), 'original_stdout_sha256': hashlib.sha256(original).hexdigest(), 'staged_original_bytes_match': True, 'selected_command': selected_command, 'selected_exit_code': selected.returncode, 'selected_stdout': selected.stdout, 'selected_stderr': selected.stderr, 'original_evidence_bytes_modified': False}
out = ROOT / 'staged-whitespace.json'
assert not out.exists()
out.write_text(json.dumps(result, indent=2) + '\n')
print(json.dumps({'status': result['status'], 'preserved_original_evidence_paths': 1}), flush=True)
raise SystemExit(result['status'] != 'passed')
