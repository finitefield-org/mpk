import hashlib,json,os,pathlib,re,subprocess,time
b=pathlib.Path(__file__).parent;r=pathlib.Path('/private/tmp/mpk-w09-refreeze-consumer-continuation');out=b/'ordinary-regeneration-13';out.mkdir(exist_ok=False);goldens=out/'goldens';goldens.mkdir();h=lambda p:hashlib.sha256(p.read_bytes()).hexdigest();previous=json.loads((b/'ordinary-source-manifest-12.json').read_bytes());paths={r/p for p in previous['hashes']};paths|=set(r.glob('crates/mpk-vc/src/**/*.rs'))|set(r.glob('crates/mpk-vc/tests/**/*.rs'));paths.add(r/'develop/migrations/csharp-03/probes/boolean-proof-elimination-refreeze/pattern-current-identity-names.json');paths|=set((r/'develop/migrations/csharp-03/ordinary-foundation/control-predicates/with-pattern-captures').glob('*'));paths|=set((r/'develop/migrations/csharp-03/ordinary-foundation/control-predicates/with-pattern-observations').glob('*'));paths={p for p in paths if p.is_file()};manifest={'hashes':{str(p.relative_to(r)):h(p) for p in sorted(paths)},'selection_reason':'Select only the four original eighteen-source pattern consumers affected by approved-freeze identity names: primitive conditions, routes, unpacked proof types and packed proof types. Preserve all source/value/import/mutation/complete-environment checks. The historical 61-condition comparison rebinds only the independently derived, SHA-pinned names; all eighteen complete typed predecessor declaration closures pass separately. Optional inputs retain exact canonical parent equality and unchanged default paths. Do not rerun unaffected scalar/native matrices. Whole gates remain final T01-W10/T06-W12.'};(b/'ordinary-source-manifest-13.json').write_text(json.dumps(manifest,sort_keys=True,indent=2)+'\n');state={'status':'running','stages':[],'selection_reason':manifest['selection_reason'],'application_proof_ids_pending':987,'full_t01_gate':'deferred_to_final_W10','full_t06_gate':'deferred_to_W12'}
def save():(out/'status.json').write_text(json.dumps(state,sort_keys=True,indent=2)+'\n')
def verify():
 for p,sha in manifest['hashes'].items():assert h(r/p)==sha,p
jobs=[('pattern-primitives','csharp_03_t06_w09_pattern_sources_original_conditions','MPK_W09_CONTROL_PATTERN_SOURCE_OUTPUT','with-pattern-primitives'),('pattern-routes','csharp_03_t06_w09_pattern_routes_original_conditions','MPK_W09_CONTROL_PATTERN_ROUTE_OUTPUT','with-pattern-routes'),('unpacked-pattern-proof-types','csharp_03_t06_w09_pattern_proof_types_preserve_original_goals_and_premises','MPK_W09_PATTERN_PROOF_TYPES_OUTPUT','unpacked-proof-types'),('packed-pattern-proof-types','csharp_03_t06_w09_packed_pattern_proof_types_cover_complete_original_environments','MPK_W09_PACKED_PATTERN_OUTPUT','with-packed-pattern-proof-types')]
save()
try:
 verify()
 for label,selector,variable,family in jobs:
  env=os.environ.copy()
  for k in list(env):
   if k.startswith(('MPK_W09_','MPK_T06_')) or k=='MPK_W14_REQUEST_OUTPUT':env.pop(k)
  env['CARGO_TARGET_DIR']='/private/tmp/mpk-w09-refreeze-consumer-target';env[variable]=str(goldens/family)
  if label=='pattern-routes':env['MPK_W09_CONTROL_PATTERN_PRIMITIVE_INPUT']=str(goldens/'with-pattern-primitives')
  if label=='unpacked-pattern-proof-types':env['MPK_W09_CONTROL_PATTERN_ROUTE_INPUT']=str(goldens/'with-pattern-routes')
  state['stage']=label;start=time.monotonic();cmd=['/Users/kazuyoshitoshiya/.cargo/bin/cargo','test','-p','mpk-vc','--test','csharp_practical_vc',selector,'--','--nocapture','--test-threads=1']
  with (out/(label+'.log.txt')).open('wb') as log:
   p=subprocess.Popen(cmd,cwd=r,env=env,stdout=log,stderr=subprocess.STDOUT);state['process_pid']=p.pid;save();code=p.wait()
  counts=re.findall(r'test result: ok\. (\d+) passed; 0 failed;',(out/(label+'.log.txt')).read_text());row={'stage':label,'selector':selector,'destination':family,'exit_code':code,'test_counts':counts,'elapsed_seconds':time.monotonic()-start,'log_sha256':h(out/(label+'.log.txt'))};state['stages'].append(row);save();assert code==0 and counts==['1'],row
  row['generated_files']={str(p.relative_to(goldens/family)):h(p) for p in sorted((goldens/family).rglob('*')) if p.is_file()};verify();save();print(label+': passed',flush=True)
 state['status']='passed_all_four_affected_original_pattern_source_and_proof_type_owners'
except BaseException as e:state['status']='failed';state['error']=repr(e);raise
finally:save()
