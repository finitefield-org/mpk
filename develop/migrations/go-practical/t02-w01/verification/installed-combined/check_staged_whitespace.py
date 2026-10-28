"""Check staged changes while preserving the exact original linker diagnostic."""
from pathlib import Path
import hashlib
import json
import re
import subprocess

ROOT = Path(__file__).resolve().parent
REPO = ROOT.parents[5]
relative = 'develop/migrations/go-practical/t02-w01/verification/installed-combined/scope-original/cargo-rustc-clippy-wrapper.stderr'
full_command = ['/usr/bin/git', 'diff', '--cached', '--check']
full = subprocess.run(full_command, cwd=REPO, capture_output=True, text=True, check=False)
assert full.returncode == 2 and not full.stderr
lines = full.stdout.splitlines()
assert len(lines) == 4
for index in (0, 2):
    assert re.fullmatch(re.escape(relative) + r':\d+: trailing whitespace\.', lines[index])
    assert lines[index + 1].startswith('+')
receipt = json.loads((ROOT / 'scope-original/result.json').read_bytes())['checks']['cargo-rustc-clippy-wrapper']['stderr']
path = REPO / relative
raw = path.read_bytes()
assert len(raw) == receipt['bytes'] and hashlib.sha256(raw).hexdigest() == receipt['sha256'] and path.stat().st_mode & 0o777 == receipt['mode']
staged = subprocess.run(['/usr/bin/git', 'show', ':' + relative], cwd=REPO, capture_output=True, check=False)
assert staged.returncode == 0 and staged.stdout == raw and not staged.stderr
selected_command = ['/usr/bin/git', 'diff', '--cached', '--check', '--', '.', ':(exclude)' + relative]
selected = subprocess.run(selected_command, cwd=REPO, capture_output=True, text=True, check=False)
result = {'status': 'passed' if selected.returncode == 0 else 'failed', 'selection_reason': 'Check all staged changes while preserving every original linker diagnostic byte, including its trailing spaces.', 'full_command': full_command, 'full_exit_code': full.returncode, 'full_stdout': full.stdout, 'preserved_original_evidence': {'path': relative, 'bytes': len(raw), 'sha256': receipt['sha256'], 'mode': receipt['mode']}, 'staged_original_bytes_match': True, 'selected_command': selected_command, 'selected_exit_code': selected.returncode, 'selected_stdout': selected.stdout, 'selected_stderr': selected.stderr, 'original_evidence_bytes_modified': False}
out = ROOT / 'staged-whitespace.json'
assert not out.exists()
out.write_text(json.dumps(result, indent=2) + '\n')
print(json.dumps({'status': result['status'], 'preserved_original_evidence_paths': 1}), flush=True)
raise SystemExit(result['status'] != 'passed')
