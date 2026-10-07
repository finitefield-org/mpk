import hashlib,json,os,pathlib,subprocess,time
b=pathlib.Path(__file__).parent;r=pathlib.Path('/private/tmp/mpk-w09-refreeze-consumer-continuation');out=b/'pattern-consumer-selected-quality';out.mkdir(exist_ok=False);h=lambda p:hashlib.sha256(p.read_bytes()).hexdigest();paths=list(r.glob('crates/mpk-vc/src/**/*.rs'))+list(r.glob('crates/mpk-vc/tests/**/*.rs'));manifest={str(p.relative_to(r)):h(p) for p in paths};(out/'source-manifest.json').write_text(json.dumps(manifest,sort_keys=True,indent=2)+'\n');env=os.environ.copy()
for k in list(env):
 if k.startswith(('MPK_W09_','MPK_T06_')) or k=='MPK_W14_REQUEST_OUTPUT':env.pop(k)
env['CARGO_TARGET_DIR']='/private/tmp/mpk-w09-refreeze-consumer-target';state={'status':'running','stages':[],'selection_reason':'Run Clippy only for the affected C# practical VC integration test target after adding the original-condition historical identity comparison and two optional exact predecessor input directories. Run format and check immutable source hashes; no whole gate or unrelated test targets are selected.','whole_gate':'deferred_to_final_T01_W10_and_T06_W12'}
def save():(out/'status.json').write_text(json.dumps(state,sort_keys=True,indent=2)+'\n')
save()
try:
 for label,cmd in [('clippy',['/Users/kazuyoshitoshiya/.cargo/bin/cargo','clippy','-p','mpk-vc','--test','csharp_practical_vc','--','-D','warnings']),('format',['/Users/kazuyoshitoshiya/.cargo/bin/cargo','fmt','--all','--','--check'])]:
  state['stage']=label;save();start=time.monotonic()
  with (out/(label+'.log.txt')).open('wb') as log:code=subprocess.run(cmd,cwd=r,env=env,stdout=log,stderr=subprocess.STDOUT).returncode
  state['stages'].append({'stage':label,'exit_code':code,'elapsed_seconds':time.monotonic()-start,'log_sha256':h(out/(label+'.log.txt'))});save();assert code==0,label
 assert all(h(r/p)==sha for p,sha in manifest.items());state['status']='passed_affected_vc_test_clippy_and_format_unchanged_sources'
except BaseException as e:state['status']='failed';state['error']=repr(e);raise
finally:save()
print(json.dumps(state))
