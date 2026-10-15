import hashlib,json,os,pathlib,re,subprocess,time
b=pathlib.Path(__file__).parent;repo=pathlib.Path('/private/tmp/mpk-w09-refreeze-metadata-integration');out=b/'integration-checkpoint-checks';out.mkdir(exist_ok=False);env=os.environ.copy();env['CARGO_TARGET_DIR']='/private/tmp/mpk-w09-refreeze-metadata-target';h=lambda p:hashlib.sha256(p.read_bytes()).hexdigest();paths=list(repo.glob('crates/mpk-vc/src/**/*.rs'))+list(repo.glob('crates/mpk-vc/tests/**/*.rs'));paths += [p for p in (repo/'develop/migrations/csharp-03/ordinary-foundation').rglob('*') if p.is_file() and p.suffix in ['.json','.hex'] and 'verification-logs' not in p.parts];manifest={str(p.relative_to(repo)):h(p) for p in sorted(set(paths))};(out/'source-manifest.json').write_text(json.dumps(manifest,sort_keys=True,indent=2)+'\n');state={'status':'running','selection_reason':'Review the two repaired parser baselines and four separated certificate-owner entries; rerun only six affected scalar/codec pin consumers and the unchanged aggregate mutation audit, then lint the affected VC test target and check formatting. All other generation outputs have exact promotion audits. Whole gates stay deferred to final T01-W10 and T06-W12.','full_t01_gate':'deferred_to_final_W10','full_t06_gate':'deferred_to_W12','stages':[]}
def save():(out/'status.json').write_text(json.dumps(state,sort_keys=True,indent=2)+'\n')
def verify():
 for p,sha in manifest.items():assert h(repo/p)==sha,p
cargo='/Users/kazuyoshitoshiya/.cargo/bin/cargo';jobs=[]
for family in ['integer_parsers','integer_formats','hex_codecs','decimal_formats','decimal_fixed_formats']:
 selector='csharp_03_t06_w09_'+family+'_pinned_sources';jobs.append((family+'-pins',[cargo,'test','-p','mpk-vc','--test','csharp_practical_vc',selector,'--','--nocapture','--test-threads=1'],1))
jobs.append(('calendar-codec-pins',[cargo,'test','-p','mpk-vc','--test','csharp_practical_vc','csharp_03_t06_w09_calendar_codecs_pinned_bytes','--','--nocapture','--test-threads=1'],1));jobs.append(('aggregate-mutation-audit',[cargo,'test','-p','mpk-vc','--test','csharp_practical_vc','csharp_03_t06_w09_aggregate_regroup_audit_mutations','--','--nocapture','--test-threads=1'],1));jobs.append(('clippy-vc-test',[cargo,'clippy','-p','mpk-vc','--test','csharp_practical_vc','--','-D','warnings'],None));jobs.append(('format',[cargo,'fmt','--all','--','--check'],None));save()
try:
 verify()
 for label,cmd,count in jobs:
  state['stage']=label;save();start=time.monotonic()
  with (out/(label+'.log.txt')).open('wb') as log:code=subprocess.run(cmd,cwd=repo,env=env,stdout=log,stderr=subprocess.STDOUT).returncode
  observed=re.findall(r'test result: ok\. (\d+) passed; 0 failed;', (out/(label+'.log.txt')).read_text());row=dict(stage=label,command=cmd,exit_code=code,test_counts=observed,elapsed_seconds=time.monotonic()-start,log_sha256=h(out/(label+'.log.txt')));state['stages'].append(row);save();print(label+': '+str(code),flush=True);assert code==0 and (count is None or observed==[str(count)]),label;verify()
 state['status']='passed_selected_pins_mutation_audit_lint_and_format'
except BaseException as e:state['status']='failed';state['error']=repr(e);raise
finally:save()
