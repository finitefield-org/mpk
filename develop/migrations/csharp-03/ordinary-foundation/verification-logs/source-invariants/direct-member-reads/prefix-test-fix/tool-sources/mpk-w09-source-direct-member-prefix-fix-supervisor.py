from pathlib import Path
import subprocess,sys,json,os,time
repo=Path('/Users/kazuyoshitoshiya/mpk');e=repo/'develop/migrations/csharp-03/ordinary-foundation/verification-logs/source-invariants/direct-member-reads/prefix-test-fix';p=e/'supervisor.json';assert not p.exists()
d=dict(status='running',pid=os.getpid(),runs=[],full_gate='deferred_to_T06_W12')
def save():p.write_text(json.dumps(d,indent=2)+'\n')
def start(mode):
 f=(e/(mode+'-supervisor.log')).open('wb');c=subprocess.Popen([sys.executable,'/tmp/mpk-w09-source-direct-member-prefix-fix.py',mode],cwd=repo,stdin=subprocess.DEVNULL,stdout=f,stderr=subprocess.STDOUT);d['runs'].append(dict(mode=mode,pid=c.pid,status='running'));save();return c,f,d['runs'][-1]
def finish(job):
 c,f,row=job;code=c.wait();f.close();row.update(status='passed'if code==0 else'failed',exit_code=code);save();return code
save()
if finish(start('build'))==0:
 jobs=[start(mode)for mode in ['semantics','preservation']]
 for j in jobs:finish(j)
 # Initial supervisor runs targeted quality on the current unchanged sources.
d['status']='passed'if all(r['status']=='passed'for r in d['runs'])else'failed';save()
