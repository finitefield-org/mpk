"""Check staged code whitespace while preserving exact test stdout bytes."""
from pathlib import Path
import json
import subprocess

ROOT = Path(__file__).resolve().parent
REPO = ROOT.parents[5]
full_command = ['/usr/bin/git', 'diff', '--cached', '--check']
full = subprocess.run(full_command, cwd=REPO, capture_output=True, text=True, check=False)
expected = {'develop/migrations/go-practical/t02-w01/verification/capture-binding/' + path for path in ['binding-after-attribute/stdout.log:8: new blank line at EOF.', 'binding-original/stdout.log:8: new blank line at EOF.', 'roots-original/stdout.log:7: new blank line at EOF.']}
assert full.returncode == 2 and set(full.stdout.splitlines()) == expected and not full.stderr
selected_command = ['/usr/bin/git', 'diff', '--cached', '--check', '--', '.', ':(exclude)develop/migrations/go-practical/t02-w01/verification/capture-binding/**/stdout.log']
selected = subprocess.run(selected_command, cwd=REPO, capture_output=True, text=True, check=False)
result = {'status':'passed' if selected.returncode == 0 else 'failed','selection_reason':'Check all staged code and records while preserving the exact original test stdout streams; separately record the full check and its three final blank-line observations.','full_command':full_command,'full_exit_code':full.returncode,'full_stdout':full.stdout,'selected_command':selected_command,'selected_exit_code':selected.returncode,'selected_stdout':selected.stdout,'selected_stderr':selected.stderr,'original_stdout_bytes_modified':False}
out=ROOT/'staged-whitespace.json'; assert not out.exists(); out.write_text(json.dumps(result,indent=2)+'\n'); print(json.dumps({'status':result['status'],'preserved_original_stdout_observations':len(expected)})); raise SystemExit(result['status'] != 'passed')
