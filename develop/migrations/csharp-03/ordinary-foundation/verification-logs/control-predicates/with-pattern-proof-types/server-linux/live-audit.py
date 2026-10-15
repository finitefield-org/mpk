import json,os
from datetime import datetime,timezone
from pathlib import Path
r=Path('/root/mpk-w09-pattern-proof-types-ef4f0123-linux')
s=json.loads((r/'status.json').read_text())
seen=set();processes=[]
def visit(pid):
 if pid in seen:return
 seen.add(pid);os.kill(pid,0)
 p=Path('/proc')/str(pid)
 state=next(line for line in (p/'status').read_text().splitlines() if line.startswith('State:'))
 assert 'Z (zombie)' not in state
 processes.append(dict(pid=pid,comm=(p/'comm').read_text().strip(),state=state))
 for child in (p/'task'/str(pid)/'children').read_text().split():visit(int(child))
visit(s['supervisor_pid'])
print(json.dumps(dict(checked_at=datetime.now(timezone.utc).isoformat(),processes=processes,status=s['status'],stage=s.get('stage'),terminal_stages=[dict(stage=x['stage'],exit_code=x['exit_code'],log_sha256=x['log_sha256']) for x in s['stages']]),sort_keys=True))
