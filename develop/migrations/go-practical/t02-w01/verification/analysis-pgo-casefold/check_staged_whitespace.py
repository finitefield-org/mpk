"""Check staged changes while retaining exact native evidence output bytes."""
from pathlib import Path
import json
import re
import subprocess

ROOT=Path(__file__).resolve().parent;REPO=ROOT.parents[5]
full_command=['/usr/bin/git','diff','--cached','--check'];full=subprocess.run(full_command,cwd=REPO,capture_output=True,text=True,check=False)
assert full.returncode in (0,2) and not full.stderr
paths=set()
allow_context=False
for line in full.stdout.splitlines():
    found=re.fullmatch(r'(.+):(\d+): (?:new blank line at EOF\.|trailing whitespace\.|space before tab in indent\.)',line)
    if not found:
        assert allow_context and line.startswith('+'), line
        allow_context=False
        continue
    allow_context=line.endswith(('trailing whitespace.','space before tab in indent.'))
    path=found.group(1)
    assert path.startswith(('develop/migrations/go-practical/t02-w01/verification/analysis-pgo/','develop/migrations/go-practical/t02-w01/verification/analysis-pgo-casefold/')) and path.endswith(('.log','.stdout','.stderr','.raw')),line
    paths.add(path)
selected_command=['/usr/bin/git','diff','--cached','--check','--','.']+[':(exclude)'+path for path in sorted(paths)]
selected=subprocess.run(selected_command,cwd=REPO,capture_output=True,text=True,check=False)
result={'status':'passed' if selected.returncode==0 else 'failed','selection_reason':'Check all staged changes and preserve exact original native logs and raw inputs without normalizing their bytes.','full_command':full_command,'full_exit_code':full.returncode,'full_stdout':full.stdout,'preserved_original_evidence_paths':sorted(paths),'selected_command':selected_command,'selected_exit_code':selected.returncode,'selected_stdout':selected.stdout,'selected_stderr':selected.stderr,'original_evidence_bytes_modified':False}
out=ROOT/'staged-whitespace.json';assert not out.exists();out.write_text(json.dumps(result,indent=2)+'\n');print(json.dumps({'status':result['status'],'preserved_evidence_paths':len(paths)}),flush=True);raise SystemExit(result['status']!='passed')
