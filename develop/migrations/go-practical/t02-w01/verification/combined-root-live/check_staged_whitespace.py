"""Check staged changes while preserving exact original test output bytes."""
from pathlib import Path
import hashlib
import json
import re
import subprocess

ROOT = Path(__file__).resolve().parent
REPO = ROOT.parents[5]
allowed = {
    'downstream-original/rust-original-consumer.stdout': ('downstream-original/result.json', 'rust-original-consumer'),
    'preflight-control/rust-complete-owner.stdout': ('preflight-control/result.json', 'rust-complete-owner'),
    'preflight-final/rust-native-names.stdout': ('preflight-final/result.json', 'rust-native-names'),
    'preflight-original/rust-original-projection.stdout': ('preflight-original/result.json', 'rust-original-projection'),
}
prefix = str(ROOT.relative_to(REPO)) + '/'
full_command = ['/usr/bin/git', 'diff', '--cached', '--check']
full = subprocess.run(full_command, cwd=REPO, capture_output=True, text=True, check=False)
assert full.returncode == 2 and not full.stderr
paths = set()
for line in full.stdout.splitlines():
    match = re.fullmatch(r'(.+):(\d+): new blank line at EOF\.', line)
    assert match and match.group(1).startswith(prefix), line
    relative = match.group(1)[len(prefix):]
    assert relative in allowed and relative not in paths
    paths.add(relative)
assert paths == set(allowed)
saved = []
for relative in sorted(paths):
    result_name, check = allowed[relative]
    receipt = json.loads((ROOT / result_name).read_bytes())['checks'][check]['stdout']
    path = ROOT / relative
    raw = path.read_bytes()
    assert len(raw) == receipt['bytes'] and hashlib.sha256(raw).hexdigest() == receipt['sha256'] and path.stat().st_mode & 0o777 == receipt['mode']
    staged = subprocess.run(['/usr/bin/git', 'show', ':' + prefix + relative], cwd=REPO, capture_output=True, check=False)
    assert staged.returncode == 0 and staged.stdout == raw and not staged.stderr
    saved.append({'path': prefix + relative, 'bytes': len(raw), 'sha256': receipt['sha256'], 'mode': receipt['mode']})
selected_command = ['/usr/bin/git', 'diff', '--cached', '--check', '--', '.'] + [':(exclude)' + prefix + path for path in sorted(paths)]
selected = subprocess.run(selected_command, cwd=REPO, capture_output=True, text=True, check=False)
result = {'status': 'passed' if selected.returncode == 0 else 'failed', 'selection_reason': 'Check all staged changes while preserving the exact original Go/Rust test logs, including their final blank lines.', 'full_command': full_command, 'full_exit_code': full.returncode, 'full_stdout': full.stdout, 'preserved_original_evidence': saved, 'staged_original_bytes_match': True, 'selected_command': selected_command, 'selected_exit_code': selected.returncode, 'selected_stdout': selected.stdout, 'selected_stderr': selected.stderr, 'original_evidence_bytes_modified': False}
out = ROOT / 'staged-whitespace.json'
assert not out.exists()
out.write_text(json.dumps(result, indent=2) + '\n')
print(json.dumps({'status': result['status'], 'preserved_original_evidence_paths': len(saved)}), flush=True)
raise SystemExit(result['status'] != 'passed')
