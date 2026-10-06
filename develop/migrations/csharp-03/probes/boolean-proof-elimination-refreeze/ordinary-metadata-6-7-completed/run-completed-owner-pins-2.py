import hashlib,json,os,pathlib,re,subprocess,time
b=pathlib.Path(__file__).parent;r=pathlib.Path('/private/tmp/mpk-w09-refreeze-consumer-continuation');out=b/'completed-owner-pin-checks-2';out.mkdir(exist_ok=False);promotion=json.loads((b/'completed-definition-owner-promotion.json').read_bytes());h=lambda p:hashlib.sha256(p.read_bytes()).hexdigest();jobs=[]
for owner in promotion['owners']:
 plan=json.loads((b/f"ordinary-regeneration-plan-{owner['run']}.json").read_bytes());row=next(x for x in plan if x['destination']==owner['family']);jobs.append((owner['family'],row['selector']))
paths=list(r.glob('crates/mpk-vc/src/**/*.rs'))+list(r.glob('crates/mpk-vc/tests/**/*.rs'))+[r/x['path'] for x in promotion['metadata_files_promoted']];paths.extend(p for p in (r/'develop/migrations/csharp-03/ordinary-foundation').rglob('*') if p.is_file() and p.suffix=='.hex' and 'verification-logs' not in p.parts);manifest={str(p.relative_to(r)):h(p) for p in sorted(set(paths))};(out/'source-manifest.json').write_text(json.dumps(manifest,sort_keys=True,indent=2)+'\n');env=os.environ.copy()
for k in list(env):
 if k.startswith(('MPK_W09_','MPK_T06_')) or k=='MPK_W14_REQUEST_OUTPUT':env.pop(k)
env['CARGO_TARGET_DIR']='/private/tmp/mpk-w09-refreeze-consumer-target';state={'status':'running','selection_reason':'Check only the thirteen actually promoted metadata corpora through their normal owner pin branches. Their complete generation/import/mutation stages already pass on the unchanged source; no other scalar/native value matrix or whole gate is repeated. Formatting is checked once; Rust source is unchanged and the affected VC-test Clippy already passes at the preceding checkpoint.','stages':[],'full_t01_gate':'deferred_to_final_W10','full_t06_gate':'deferred_to_W12'}
def save():(out/'status.json').write_text(json.dumps(state,sort_keys=True,indent=2)+'\n')
def verify():
 for p,sha in manifest.items():assert h(r/p)==sha,p
save()
try:
 verify()
 for label,selector in jobs+ [('format',None)]:
  label=label.replace('/','--');cmd=['/Users/kazuyoshitoshiya/.cargo/bin/cargo','test','-p','mpk-vc','--test','csharp_practical_vc',selector,'--','--nocapture','--test-threads=1'] if selector else ['/Users/kazuyoshitoshiya/.cargo/bin/cargo','fmt','--all','--check'];state['stage']=label;start=time.monotonic()
  with (out/(label+'.log.txt')).open('wb') as log:
   p=subprocess.Popen(cmd,cwd=r,env=env,stdout=log,stderr=subprocess.STDOUT);state['process_pid']=p.pid;save();code=p.wait()
  counts=re.findall(r'test result: ok\. (\d+) passed; 0 failed;', (out/(label+'.log.txt')).read_text());row={'stage':label,'selector':selector,'exit_code':code,'elapsed_seconds':time.monotonic()-start,'test_counts':counts,'log_sha256':h(out/(label+'.log.txt'))};state['stages'].append(row);save();assert code==0 and (selector is None or counts==['1']),label;verify();print(label+': '+str(code),flush=True)
 state['status']='passed_13_promoted_owner_pin_tests_and_format'
except BaseException as e:state['status']='failed';state['error']=repr(e);raise
finally:save()
