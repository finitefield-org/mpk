"""Wait for terminal source checks before the final independent large observation."""
from pathlib import Path
import json,subprocess,time
ROOT=Path(__file__).resolve().parent
for epoch in ['final-large','producer-replay','format-name-fixed','lint-fixed']:
    while True:
        p=ROOT/epoch/'result.json'
        if p.exists():
            state=json.loads(p.read_bytes())
            if state['status']!='running':break
        time.sleep(1)
    assert state['status']=='passed',(epoch,state)
command=['python3',str(ROOT/'run-oracle.py'),'oracle-large','final-large']
assert subprocess.run(command,check=False).returncode==0
while True:
    state=json.loads((ROOT/'paired-large/result.json').read_bytes())
    if state['status']!='running':break
    time.sleep(1)
assert state['status']=='passed'
assert subprocess.run(['python3',str(ROOT/'summarize.py')],check=False).returncode==0
