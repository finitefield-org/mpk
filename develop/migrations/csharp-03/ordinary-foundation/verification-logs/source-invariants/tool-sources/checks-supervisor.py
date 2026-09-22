from pathlib import Path
import json,os,subprocess,time
repo=Path('/Users/kazuyoshitoshiya/mpk');e=repo/'develop/migrations/csharp-03/ordinary-foundation/verification-logs/source-invariants';out=e/'checks-supervisor.json';d=dict(status='waiting_for_build',supervisor_pid=os.getpid(),jobs=[])
def save():out.write_text(json.dumps(d,indent=2)+'\n')
save()
while True:
 try:b=json.loads((e/'build.json').read_text())
 except json.JSONDecodeError:time.sleep(2);continue
 if b['status']=='failed':d.update(status='failed',failure='build failed');save();raise SystemExit(1)
 if b['status']=='passed':break
 time.sleep(5)
children=[]
for mode in ['definitions','semantics','preservation','enums','quality']:
 f=(e/(mode+'-supervisor.log')).open('wb');p=subprocess.Popen(['python3','/tmp/mpk-w09-source-invariants.py',mode],cwd=repo,stdin=subprocess.DEVNULL,stdout=f,stderr=subprocess.STDOUT,start_new_session=True);children.append(p);d['jobs'].append(dict(mode=mode,pid=p.pid,status='running'));save()
d['status']='running';save()
for p,row in zip(children,d['jobs']):row.update(status='passed'if p.wait()==0 else'failed');save()
d['status']='passed'if all(x['status']=='passed'for x in d['jobs'])else'failed';save()
