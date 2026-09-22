from pathlib import Path
import subprocess,sys,json,os
repo=Path('/Users/kazuyoshitoshiya/mpk');e=repo/'develop/migrations/csharp-03/ordinary-foundation/verification-logs/concrete-operations';path=e/'supervisor.json';assert not path.exists();d=dict(status='running',pid=os.getpid(),runs=[],full_gate='deferred_to_T06_W12')
def save():path.write_text(json.dumps(d,indent=2)+'\n')
def start(mode):
 f=(e/(mode+'-supervisor.log')).open('wb');p=subprocess.Popen([sys.executable,'/tmp/mpk-w09-concrete-operations.py',mode],cwd=repo,stdin=subprocess.DEVNULL,stdout=f,stderr=subprocess.STDOUT);row=dict(mode=mode,pid=p.pid,status='running');d['runs'].append(row);save();return p,f,row
def finish(job):
 p,f,row=job;code=p.wait();f.close();row.update(status='passed'if code==0 else'failed',exit_code=code);save();return code
save()
if finish(start('build'))==0:
 jobs=[start(mode)for mode in ['definitions','routing','real','preservation']]
 for j in jobs:finish(j)
 finish(start('quality'))
d['status']='passed'if len(d['runs'])==6 and all(r['status']=='passed'for r in d['runs'])else'failed';save()
