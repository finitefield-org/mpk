import hashlib,json,os,pathlib,re,subprocess,time
b=pathlib.Path(__file__).parent;r=pathlib.Path('/private/tmp/mpk-w09-refreeze-consumer-continuation');out=b/'standalone-promoted-pin-checks';out.mkdir(exist_ok=False);h=lambda p:hashlib.sha256(p.read_bytes()).hexdigest();promotion=json.loads((b/'standalone-program-promotion.json').read_bytes());jobs=[('calendar-data','csharp_03_t06_w09_calendar_data_candidates'),('integer-data','csharp_03_t06_w09_integer_data_candidates'),('decimal-data','csharp_03_t06_w09_decimal_data_candidates'),('floating-data','csharp_03_t06_w09_floating_data_candidates'),('lifted-data','csharp_03_t06_w09_lifted_data_candidates'),('source-value-data','csharp_03_t06_w09_source_value_data_candidates'),('string-data','csharp_03_t06_w09_string_data_candidates'),('structural-data','csharp_03_t06_w09_structural_data_candidates'),('contract-scopes','csharp_03_t06_w09_contract_expression_method_and_loop_scopes'),('contract-old-result','csharp_03_t06_w09_contract_expression_old_and_result_values'),('multiowner-public','csharp_03_t06_w09_source_clause_multiowner_carriers'),('source-clauses','csharp_03_t06_w09_source_clauses_original_construction_invariants'),('json-products','csharp_03_t06_w09_json_products_sources_and_dependency_preservation'),('json-calendar','csharp_03_t06_w09_json_calendar_source_captures')];paths=list(r.glob('crates/mpk-vc/src/**/*.rs'))+list(r.glob('crates/mpk-vc/tests/**/*.rs'))+[r/x['path'] for x in promotion['changes']];manifest={str(p.relative_to(r)):h(p) for p in sorted(set(paths))};(out/'manifest.json').write_text(json.dumps(manifest,sort_keys=True,indent=2)+'\n');env=os.environ.copy()
for k in list(env):
 if k.startswith(('MPK_W09_','MPK_T06_')) or k=='MPK_W14_REQUEST_OUTPUT':env.pop(k)
env['CARGO_TARGET_DIR']='/private/tmp/mpk-w09-refreeze-consumer-target';state={'status':'running','selection_reason':'Check only the fourteen selected affected owner branches after actual standalone metadata/certificate promotion. Use existing candidate branches for eight scalar/data owners to preserve full source/import/mutation and exact pin checks without repeating their unchanged native-value matrices. Check contract scope/old-result, public multiowner, source-clause, JSON product and calendar consumers whose names or aggregate metadata change. Every changed certificate already passes its complete graph and both checkers; whole gates remain deferred to final T01-W10/T06-W12.','stages':[],'whole_gate':'deferred_to_final_T01_W10_and_T06_W12'}
def save():(out/'status.json').write_text(json.dumps(state,sort_keys=True,indent=2)+'\n')
def verify():
 for p,sha in manifest.items():assert h(r/p)==sha,p
save()
try:
 verify()
 for label,selector in jobs:
  state['stage']=label;start=time.monotonic();cmd=['/Users/kazuyoshitoshiya/.cargo/bin/cargo','test','-p','mpk-vc','--test','csharp_practical_vc',selector,'--','--nocapture','--test-threads=1']
  with (out/(label+'.log.txt')).open('wb') as log:
   p=subprocess.Popen(cmd,cwd=r,env=env,stdout=log,stderr=subprocess.STDOUT);state['process_pid']=p.pid;save();code=p.wait()
  counts=re.findall(r'test result: ok\. (\d+) passed; 0 failed;', (out/(label+'.log.txt')).read_text());row={'stage':label,'selector':selector,'exit_code':code,'test_counts':counts,'elapsed_seconds':time.monotonic()-start,'log_sha256':h(out/(label+'.log.txt'))};state['stages'].append(row);save();assert code==0 and counts==['1'],row;verify();print(label+': passed',flush=True)
 state['status']='passed_all_fourteen_selected_promoted_owner_pin_and_dependency_tests'
except BaseException as e:state['status']='failed';state['error']=repr(e);raise
finally:save()
