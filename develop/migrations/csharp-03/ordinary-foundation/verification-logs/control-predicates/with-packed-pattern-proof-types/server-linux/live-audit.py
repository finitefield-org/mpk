import json,os,subprocess
from pathlib import Path
from hashlib import sha256
from datetime import datetime,timezone
repo=Path('/root/mpk-w09-packed-pattern-types-130a3ca4')
reports=Path('/root/mpk-w09-packed-pattern-types-130a3ca4-linux')
commit='130a3ca4380868953f8ec56c5aad99582715e089'
assert subprocess.check_output(['/usr/bin/git','rev-parse','HEAD'],cwd=repo,text=True).strip()==commit
assert subprocess.check_output(['/usr/bin/git','status','--porcelain'],cwd=repo,text=True).strip()==''
launch=json.loads((reports/'launch.json').read_bytes())
assert launch['commit']==commit
manifest_path=repo/'develop/migrations/csharp-03/ordinary-foundation/verification-logs/control-predicates/with-packed-pattern-proof-types/source-manifest.json'
assert sha256(manifest_path.read_bytes()).hexdigest()==launch['manifest_sha256']
manifest=json.loads(manifest_path.read_bytes())
for group in manifest:
    for name,digest in manifest[group].items():assert sha256((repo/name).read_bytes()).hexdigest()==digest,name
status=json.loads((reports/'status.json').read_bytes())
if status['status'] in ('running','preparing'): os.kill(status['supervisor_pid'],0)
processes={}
queue=[status['supervisor_pid']]
while queue and len(processes)<20:
    pid=queue.pop(0)
    if str(pid) in processes:continue
    root=Path('/proc')/str(pid)
    if not root.exists():continue
    processes[str(pid)]={'command':(root/'cmdline').read_bytes().replace(bytes([0]),b' ').decode().strip(),'state':next(line for line in (root/'status').read_text().splitlines() if line.startswith('State:'))}
    children=set()
    for task in (root/'task').iterdir():
        try:children.update(int(p) for p in (task/'children').read_text().split())
        except FileNotFoundError:pass
    queue.extend(children)
row=dict(recorded_at=datetime.now(timezone.utc).isoformat(),source_commit=commit,clean_checkout=True,source_files_verified=len(manifest['source_hashes']),fixture_files_verified=len(manifest['fixture_hashes']),manifest_sha256=launch['manifest_sha256'],status=status,processes=processes,full_t_gate='deferred to T06-W12')
print(json.dumps(row,sort_keys=True))
