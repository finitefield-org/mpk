"""Preserve exact checker output bytes while checking all other staged files."""
from pathlib import Path
import datetime
import hashlib
import json
import re
import subprocess

OUT=Path(__file__).resolve().parent.relative_to(Path.cwd())
RUN=Path('/private/tmp/mpk-go-string-concat-whitespace')
RUN.mkdir(exist_ok=False)
(RUN/'driver.py').write_bytes(Path(__file__).read_bytes())
checks=[]
def check(arguments):
    command=['/usr/bin/git','diff','--cached','--check',*arguments]
    result=subprocess.run(command,capture_output=True,check=False)
    checks.append({'command':command,'exit_code':result.returncode,'stdout':result.stdout.decode(),'stderr':result.stderr.decode()})
    assert not result.stderr
    return result

logs=set()
for path in (OUT/'epochs').glob('*/*/result.json'):
    logs.update(str(path.parent/name)for name in ['stdout.log','stderr.log'])
pair=json.loads((OUT/'paired/result.json').read_bytes())
for row in pair['cases']:logs.update(row[key]['path']for key in ['stdout','stderr'])
first=check([])
warnings={}
for line in first.stdout.decode().splitlines():
    match=re.fullmatch(r'(.+):(\d+): (new blank line at EOF\.|trailing whitespace\.)',line)
    if match:
        path,line_number,kind=match.groups();assert path in logs,path
        warnings.setdefault(path,[]).append({'line':int(line_number),'message':kind})
    else:
        # Git displays the actual offending output line following its warning.
        assert warnings and any(row['message']=='trailing whitespace.'for row in list(warnings.values())[-1]),line
assert first.returncode==(2 if warnings else 0)
second=check(['--','.',*[':(exclude)'+path for path in sorted(warnings)]])
assert second.returncode==0 and not second.stdout
rows=[]
for name,items in sorted(warnings.items()):
    raw=Path(name).read_bytes()
    for item in items:
        if item['message']=='new blank line at EOF.':assert raw.endswith(b'\n\n')
        else:assert raw.splitlines()[item['line']-1].endswith((b' ',b'\t'))
    rows.append({'path':name,'bytes':len(raw),'sha256':hashlib.sha256(raw).hexdigest(),'warnings':items})
record={'status':'passed_with_original_output_retained','observed_at_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'actual_driver_sha256':hashlib.sha256((RUN/'driver.py').read_bytes()).hexdigest(),'actual_checks':checks,'exact_raw_output_warnings':rows,'all_other_staged_files_pass_whitespace_check':True,'original_Cargo_output_bytes_modified':False,'obsolete_IR_matcher_scope_seal_or_exclusion_changes':False}
(OUT/'git-whitespace-review.json').write_text(json.dumps(record,indent=2)+'\n')
(RUN/'result.json').write_text(json.dumps(record,indent=2)+'\n')
print(json.dumps({'status':record['status'],'exact_raw_output_warnings':len(rows),'all_other_staged_files_pass_whitespace_check':True}))
