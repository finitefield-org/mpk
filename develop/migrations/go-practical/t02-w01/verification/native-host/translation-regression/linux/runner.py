"""Local emulated Linux compile/tests and actual non-native host rejection.

This is not native practical-Go release acceptance. No privileged container,
source execution, host cgroup mutation or CSharp tests are requested.
"""
from pathlib import Path
import datetime, hashlib, json, subprocess, sys, time, uuid
root=Path(__file__).resolve().parent.relative_to(Path.cwd().resolve());attempt,*scopes=sys.argv[1:]
scope=scopes[0] if scopes else 'full';assert scope in {'full','policy-only','pidfd-regression','translation-regression'}
assert attempt.replace('-','').isalnum()
out=root/attempt/'linux';out.mkdir(parents=True,exist_ok=False)
(out/'runner.py').write_bytes(Path(__file__).read_bytes())
input_path=root/'source-epochs'/attempt/'inputs.json'
if not input_path.exists():input_path=root/'inputs.json'
inputs=json.loads(input_path.read_text())
def seal(p):
 raw=Path(p).read_bytes();return {'path':str(p),'sha256':hashlib.sha256(raw).hexdigest(),'bytes':len(raw)}
name='mpk-go-native-host-'+uuid.uuid4().hex[:12]
image='sha256:0f73233bdaf6abe170bc4fdaffc4bbd27cfd1fd6332c09bd666bd0df6d18da8b'
record={'status':'running','name':name,'image':image,'scope':scope,'complete_input_receipt':seal(input_path),'sources':inputs['sources'],'checks':[],
 'started_at_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(), 'CSharp_tests_executed':False,
 'x86_64_emulated_on_local_Linux_aarch64':True,'native_Linux_release_acceptance_claimed':False,
 'R06_status':'pending','T02_W01_complete':False,'full_T02_gate':'deferred to T02-W04: ./scripts/check-fast.sh --skip-csharp',
 'selection_reason':'Compile both changed production Linux paths and run only the new Go/unchanged shared installer BPF controls, affected sandbox/accounting/cleanup/watchdog/namespace helpers and actual non-native CPU rejection before cgroup state changes. Pinned original source/manifest bytes are held fixed. No CSharp, agents, Actions, Go frontend rebuild or intermediate whole gate.'}
result=out/'result.json';result.write_text(json.dumps(record,indent=2)+'\n')
prefix=['/usr/local/bin/docker','run','--rm','--name',name,'--platform','linux/amd64','--pull=never',
 '--network=none','--read-only','--user','501:501','--cap-drop=ALL','--security-opt=no-new-privileges',
 '--memory=6g','--cpus=2','--pids-limit=256','--tmpfs','/tmp:rw,exec,nosuid,nodev,size=2g,mode=1777',
 '--mount','type=bind,source='+str(Path.cwd())+',target=/mpk,readonly',
 '--mount','type=bind,source=/private/tmp/mpk-go-practical-native-host-cargo,target=/cargo',
 '--mount','type=bind,source=/private/tmp/mpk-go-practical-policy-linux-target,target=/target',
 '-e','CARGO_HOME=/cargo','-e','CARGO_TARGET_DIR=/target','-e','CARGO_NET_OFFLINE=true',
 '-e','CARGO_INCREMENTAL=0','-e','RUST_TEST_THREADS=1','-e','HOME=/tmp','--workdir','/mpk',image]
commands=[
 ('host','/bin/sh',['-c','uname -a; cat /proc/cpuinfo; rustc --version; cargo --version']),
 ('policy','cargo',['test','--offline','--locked','-p','mpk-linux-sandbox','--lib','policy::tests::','--','--nocapture','--test-threads=1']),
 ('selected','cargo',['test','--offline','--locked','-p','mpk-cli','--lib','frontend_sandbox::','--','--nocapture','--test-threads=1']),
 ('lint','cargo',['clippy','--offline','--locked','-p','mpk-cli','-p','mpk-linux-sandbox','--lib','--','-D','warnings']),
]
if scope=='policy-only':commands=commands[:2]
if scope=='pidfd-regression':
 commands=commands[:1]+[('pidfd-regression','cargo',['test','--offline','--locked','-p','mpk-linux-sandbox','--lib','go_policy::tests::go_policy_preserves_original_initial_pidfd_probe','--','--exact','--nocapture','--test-threads=1'])]
if scope=='translation-regression':
 commands=commands[:1]+[('translation-regression','cargo',['test','--offline','--locked','-p','mpk-cli','--lib','frontend_sandbox::go_practical::tests::go_practical_observed_translation_is_not_native_cpu_evidence','--','--exact','--nocapture','--test-threads=1'])]
for label,program,args in commands:
 command=prefix.copy();command[command.index('--name')+1]=name+'-'+label;command += [program,*args]
 print(json.dumps({'phase':label,'container':name+'-'+label}),flush=True);start=time.monotonic()
 with (out/(label+'.stdout.log')).open('wb') as stdout,(out/(label+'.stderr.log')).open('wb') as stderr:
  code=subprocess.run(command,stdout=stdout,stderr=stderr,check=False).returncode
 record['checks'].append({'label':label,'command':command,'exit_code':code,'seconds':time.monotonic()-start,
  'stdout':seal(out/(label+'.stdout.log')),'stderr':seal(out/(label+'.stderr.log'))})
 result.write_text(json.dumps(record,indent=2)+'\n')
 if code:break
record['status']='passed' if all(r['exit_code']==0 for r in record['checks']) and len(record['checks'])==len(commands) else 'failed'
record['whole_inputs_unchanged_during_execution']=all(seal(r['path'])['sha256']==r['sha256'] and seal(r['path'])['bytes']==r['bytes'] for r in inputs['sources'])
if not record['whole_inputs_unchanged_during_execution']:record['status']='failed'
record['finished_at_utc']=datetime.datetime.now(datetime.timezone.utc).isoformat();result.write_text(json.dumps(record,indent=2)+'\n')
print(json.dumps({'status':record['status'],'checks':[(r['label'],r['exit_code'],r['seconds']) for r in record['checks']]}))
raise SystemExit(0 if record['status']=='passed' else 1)
