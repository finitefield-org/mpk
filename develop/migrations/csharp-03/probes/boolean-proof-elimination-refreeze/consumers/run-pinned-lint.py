import hashlib,json,os,pathlib,subprocess,time
base=pathlib.Path(__file__).parent;repo=pathlib.Path('/private/tmp/mpk-w09-context-application-integration');out=base/'pinned-lint';out.mkdir(exist_ok=False)
state={'status':'running','selection_reason':'The changed cfg(test) snapshot fixture path and affected VC/CLI test code require one focused library regression plus scoped lint/format; renewed full T01 gate is deferred to final W10.','stages':[],'full_t01_gate':'deferred to renewed T01-W10','full_t06_gate':'deferred to T06-W12'}
manifest=json.loads((base/'pinned-consumer-source-manifest.json').read_bytes());h=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
def verify():
 for p,sha in manifest['hashes'].items():assert h(repo/p)==sha,p
def save():(out/'status.json').write_text(json.dumps(state,sort_keys=True,indent=2)+'\n')
try:
 verify();env=os.environ.copy();env['CARGO_TARGET_DIR']='/private/tmp/mpk-w09-bool-cases-target'
 for k in list(env):
  if k.startswith(('MPK_T06_','MPK_W09_')):env.pop(k)
 cargo='/Users/kazuyoshitoshiya/.cargo/bin/cargo'
 jobs=[('shared-json-library-fixture',[cargo,'test','-p','mpk-vc','--lib','csharp_03_t06_w09_json_and_collection_folds_share_budget','--','--nocapture']),('vc-clippy',[cargo,'clippy','-p','mpk-vc','--test','csharp_practical_vc','--','-D','warnings']),('cli-clippy',[cargo,'clippy','-p','mpk-cli','--test','csharp_practical_boundary','--test','csharp_practical_boundary_transition','--test','csharp_practical_domain','--','-D','warnings']),('format',[cargo,'fmt','-p','mpk-vc','-p','mpk-cli','--','--check'])]
 for label,cmd in jobs:
  state['stage']=label;t=time.monotonic()
  with (out/(label+'.log.txt')).open('wb') as log:
   p=subprocess.Popen(cmd,cwd=repo,env=env,stdout=log,stderr=subprocess.STDOUT);state['process_pid']=p.pid;save();code=p.wait()
  state['stages'].append({'stage':label,'exit_code':code,'log_sha256':h(out/(label+'.log.txt')),'elapsed_seconds':time.monotonic()-t});save();print(label+': '+str(code),flush=True);assert code==0,label;verify()
 state['status']='passed_selected_library_fixture_and_affected_lint_format'
except BaseException as e:state['status']='failed';state['error']=repr(e);raise
finally:save()
