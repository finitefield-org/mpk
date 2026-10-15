from pathlib import Path
import hashlib,json,os,re,subprocess,time
b=Path(__file__).parent;r=Path('/private/tmp/mpk-w09-refreeze-consumer-continuation')
h=lambda p:hashlib.sha256(p.read_bytes()).hexdigest();read=lambda p:json.loads(p.read_bytes())
refresh=read(b/'final-registration-order-proof-evidence-refresh.json')
assert refresh['status']=='passed_final_registration_order_evidence_refresh_exact_709_vectors_and_non_evidence_freeze_behavior'
assert refresh['all_709_complete_original_vector_rows_unchanged'] and refresh['checker_stages']==68
assert read(b/'pattern-consumer-promotion.json')['fresh_checker_stages_passed']==42
assert read(b/'wrapper-consumer-promotion.json')['fresh_matching_checker_acceptance_stages']==230
assert read(b/'ordinary-regeneration-4b/status.json')['status']=='passed_selected_source_bound_ordinary_owners'
assert (r/'develop/specs/CSHARP_PRACTICAL_SHARED_ARTIFACTS_V1.md').read_bytes()==(b/'final-shared-artifacts-specification-candidate.md').read_bytes()
out=b/'final-w09-selected-validation';out.mkdir(exist_ok=False)
selection='The final proof evidence changes Boolean cases/source pins and regenerates private freeze evidence while preserving all709 complete rows. Run six W09 primary freeze/spec tests plus the two original recursor/capacity normal-pin reproducibility owners. Unchanged W08 foundation tests and still-blocked W10 publication owners are excluded. Prior selected core/checker/source-owner tests retain exact execution provenance. Whole gate is deferred to final T01-W10.'
paths=set()
for root in ['crates','develop/probes/csharp-03']:
 for p in (r/root).rglob('*'):
  if p.is_file() and p.suffix in ['.rs','.toml','.py'] and '__pycache__' not in p.parts:paths.add(p.relative_to(r).as_posix())
for p in ['Cargo.toml','Cargo.lock','develop/migrations/csharp-03/freeze/profile-freeze.json','develop/migrations/csharp-03/freeze/profile-freeze-vectors.json','develop/migrations/csharp-03/probes/boolean-proof-elimination.json','develop/migrations/csharp-03/probes/checker-capacity.json','develop/migrations/csharp-03/probes/recursor-feasibility.json','develop/specs/CSHARP_PRACTICAL_SHARED_ARTIFACTS_V1.md']:paths.add(p)
manifest={p:h(r/p) for p in sorted(paths)}
(out/'source-manifest.json').write_text(json.dumps({'hashes':manifest,'selection_reason':selection},sort_keys=True,indent=2)+'\n')
state={'status':'running','stages':[],'selection_reason':selection,'original_application_proof_ids_pending':987,'full_t01_gate':'deferred_to_final_W10','full_t06_gate':'deferred_to_W12','supervisor_pid':os.getpid()}
def save():(out/'status.json').write_text(json.dumps(state,sort_keys=True,indent=2)+'\n')
def verify():
 for p,sha in manifest.items():assert h(r/p)==sha,p
env=os.environ.copy()
for key in list(env):
 if key.startswith(('MPK_W09_','MPK_T06_')) or key=='MPK_W14_REQUEST_OUTPUT':env.pop(key)
env.update(CARGO_TARGET_DIR='/private/tmp/mpk-w09-bool-cases-registration-order-target',CARGO_INCREMENTAL='0',PYTHONDONTWRITEBYTECODE='1')
commands=[('private-freeze-check',['python3','develop/probes/csharp-03/profile_freeze.py','--check'],None),('eight-original-w09-owners',['/Users/kazuyoshitoshiya/.cargo/bin/cargo','test','-p','mpk-vc','--test','csharp_practical_spec','csharp_03_t01_w09_','--','--test-threads=1'],'8'),('selected-spec-clippy',['/Users/kazuyoshitoshiya/.cargo/bin/cargo','clippy','-p','mpk-vc','--test','csharp_practical_spec','--','-D','warnings'],None)]
try:
 for stage,command,count in commands:
  verify();state['stage']=stage;save();start=time.monotonic();log=out/(stage+'.log.txt')
  with log.open('wb') as stream:
   process=subprocess.Popen(command,cwd=r,env=env,stdout=stream,stderr=subprocess.STDOUT);state['process_pid']=process.pid;save();code=process.wait()
  counts=re.findall(r'test result: ok\. (\d+) passed; 0 failed;',log.read_text())
  row={'stage':stage,'command':command,'exit_code':code,'elapsed_seconds':time.monotonic()-start,'log_sha256':h(log),'test_counts':counts};state['stages'].append(row);save();assert code==0,row
  if count is not None:assert counts==[count],row
 verify();state.update(status='passed_final_w09_eight_original_normal_pin_owners_freeze_check_and_selected_clippy',rust_owner_tests=8,fixture_or_source_changes=0)
except BaseException as error:state.update(status='failed',error=repr(error));raise
finally:save()
