import hashlib,json,os,pathlib,subprocess,time
root=pathlib.Path('/private/tmp/mpk-w09-pattern-original-probe')
repo=pathlib.Path('/private/tmp/mpk-w09-context-application-integration')
out=root/'generation-5';out.mkdir()
base=repo/'develop/migrations/csharp-03/ordinary-foundation/control-predicates/with-packed-pattern-proof-types'
inputs=[root/n for n in ['builder.rs','normalizer.rs','main.rs','probe','run.py']]+[base/n for n in ['is_binding.json','is_binding.hex']]+[repo/'crates/mpk-vc/src/csharp_practical_ordinary_ownership_proofs.rs']
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
pins={str(p):sha(p) for p in inputs};(out/'inputs.json').write_text(json.dumps(pins,indent=2)+'\n')
state=dict(status='running',supervisor_pid=os.getpid(),diagnostic_only=True,application_scope_pending=True,selection_reason='Inspect the first unresolved original is_binding path while retaining its complete environment, every original scope projection and exact original named theorem type; prior probe returned only Linkage.')
def save():(out/'status.json').write_text(json.dumps(state,indent=2)+'\n')
start=time.monotonic()
with (out/'stdout.log.txt').open('wb') as o,(out/'stderr.log.txt').open('wb') as e:
 p=subprocess.Popen([str(root/'probe'),str(repo),str(out)],stdout=o,stderr=e);state['process_pid']=p.pid;save();code=p.wait()
assert all(sha(pathlib.Path(p))==h for p,h in pins.items())
state.update(status='diagnostic_completed' if code==0 else 'diagnostic_failed',exit_code=code,elapsed_seconds=time.monotonic()-start)
if code==0:state['generation']=json.loads((out/'generation.json').read_text())
save();print(json.dumps(state))
