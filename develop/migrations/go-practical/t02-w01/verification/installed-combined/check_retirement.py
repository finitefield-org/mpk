"""Check the actual original live root paths after the completed run."""
from pathlib import Path
import json
import os
import subprocess
import sys

ROOT = Path(__file__).resolve().parent
REPO = ROOT.parents[5]
IMAGE = 'sha256:7cf1e580ef5539f03b58560753e8ab84c8c360960d99dff714004aa98f203977'
epoch = sys.argv[1]
assert epoch.replace('-', '').isalnum()
run = ROOT / epoch
result = json.loads((run / 'result.json').read_bytes())
inputs = json.loads((run / 'inputs.json').read_bytes())
assert result['status'] == 'passed'
source = json.loads((run / 'source-coord/roots-ready.json').read_bytes())
export = json.loads((run / 'export-coord/roots-ready.json').read_bytes())
rows = source['roots'] + export['roots']
assert len(rows) == 4 and all(Path(row['Directory']).is_absolute() and Path(row['Directory']).parts[:2] == ('/', 'tmp') for row in rows)
paths = [row['Directory'] for row in rows]
script = 'import json,os,sys\nresults=[]\nfor p in sys.argv[1:]:\n try: os.lstat(p)\n except FileNotFoundError: results.append({"path":p,"absent":True})\n else: results.append({"path":p,"absent":False})\nprint(json.dumps(results))\n'
command = ['/usr/local/bin/docker', 'run', '--pull=never', '--rm', '--platform', 'linux/arm64', '--network', 'none', '--read-only', '--cap-drop', 'ALL', '--security-opt', 'no-new-privileges', '--user', f'{os.getuid()}:{os.getgid()}', '--mount', f'type=volume,src={inputs["volume"]},dst=/tmp,readonly,volume-nocopy', IMAGE, 'python3', '-c', script, *paths]
checked = subprocess.run(command, cwd=REPO, capture_output=True, text=True, check=False)
assert checked.returncode == 0 and not checked.stderr
observed = json.loads(checked.stdout)
assert observed == [{'path': path, 'absent': True} for path in paths]
receipt = {'status': 'passed', 'selection_reason': 'Check every actual source and generated export root on its original shared volume after the live run, without recreating or changing any entries.', 'command': command, 'original_root_paths': rows, 'post_run_observations': observed, 'all_paths_absent_after_run': True, 'original_volume_modified': False, 'native_Linux_release_acceptance': False}
out = run / 'retirement-check.json'
assert not out.exists()
out.write_text(json.dumps(receipt, indent=2) + '\n')
print(json.dumps({'status': receipt['status'], 'original_roots_absent': len(observed)}), flush=True)
