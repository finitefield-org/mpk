"""Pin exact original Cargo output warnings; check every other staged file."""
from pathlib import Path
import datetime
import hashlib
import json
import subprocess

OUT=Path(__file__).resolve().parent
root=Path.cwd()
command=["/usr/bin/git","diff","--cached","--check"]
result=subprocess.run(command,capture_output=True,check=False)
assert not result.stderr
original={"command":command,"exit_code":result.returncode,"stdout":result.stdout.decode(),"stderr":result.stderr.decode()}
rows=[]
for line in result.stdout.decode().splitlines():
    name,position,warning=line.rsplit(":",2)
    assert warning==" new blank line at EOF." and position.isdigit()
    path=root/name;assert path.is_relative_to(OUT/"epochs") and path.name=="stdout.log"
    execution=json.loads((path.parent/"result.json").read_bytes())
    assert execution["command"][0]=="cargo" and execution["status"]in {"passed","failed"}
    raw=path.read_bytes();assert raw.endswith(b"\n\n")
    rows.append({"path":name,"sha256":hashlib.sha256(raw).hexdigest(),"bytes":len(raw),"warning":line})
assert result.returncode==(2 if rows else 0)
assert len({row["path"]for row in rows})==len(rows)
paths=sorted(row["path"]for row in rows)
command=["/usr/bin/git","diff","--cached","--check","--",".",*(":(exclude)"+name for name in paths)]
result=subprocess.run(command,capture_output=True,check=False)
assert result.returncode==0 and not result.stdout and not result.stderr
record={"status":"passed_with_original_output_retained","observed_at_utc":datetime.datetime.now(datetime.timezone.utc).isoformat(),"actual_checks":[original,{"command":command,"exit_code":result.returncode,"stdout":result.stdout.decode(),"stderr":result.stderr.decode()}],"exact_raw_output_warnings":rows,"all_other_staged_files_pass_whitespace_check":True,"original_Cargo_output_bytes_modified":False,"obsolete_IR_matcher_scope_seal_or_exclusion_changes":False}
(OUT/"git-whitespace-review.json").write_text(json.dumps(record,indent=2)+"\n")
print(json.dumps({"status":record["status"],"exact_original_Cargo_output_warnings":len(rows),"all_other_staged_files_pass_whitespace_check":True}))
