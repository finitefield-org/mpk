"""Only affected value/statement/selection consumers; each child saves its inputs."""
from pathlib import Path
import json,subprocess
OUT=Path(__file__).resolve().parent;RUN=OUT/'affected-driver';RUN.mkdir(exist_ok=False);(RUN/'driver.py').write_bytes(Path(__file__).read_bytes())
rows=[]
for mode in ['rust-scalar','rust-aggregates','rust-methods','rust-assignments','rust-returns','rust-builtins','rust-bodies','rust-selection','clippy','rust-format','go-format','vet']:
 result=subprocess.run(['python3',str(OUT/'run-checks.py'),'controls',mode],check=False)
 rows.append({'mode':mode,'exit_code':result.returncode});(RUN/'result.json').write_text(json.dumps({'status':'running','children':rows},indent=2)+'\n')
record={'status':'passed'if all(row['exit_code']==0 for row in rows)else'failed','children':rows,'selection_reason':'Shared value engine context records and body range handling affect these consumers. Parser/scopes/constants/source-type algorithms, core/ordinary and unrelated languages are unchanged. Full T02 gate remains W04.'};(RUN/'result.json').write_text(json.dumps(record,indent=2)+'\n');print(json.dumps(record));raise SystemExit(record['status']!='passed')
