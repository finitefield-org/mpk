from pathlib import Path
import hashlib,importlib.util,json,shutil
b=Path(__file__).parent;r=Path('/private/tmp/mpk-w09-refreeze-consumer-continuation');h=lambda p:hashlib.sha256(p.read_bytes()).hexdigest();read=lambda p:json.loads(p.read_bytes());canon=lambda x:json.dumps(x,sort_keys=True,separators=(',',':'),ensure_ascii=True).encode()
plan=read(b/'final-registration-order-proof-evidence-refresh-plan.json');assert plan['status']=='prepared_final_evidence_refresh_not_executed'
assert read(b/'pattern-consumer-promotion.json')['status'].startswith('passed_promoted_54_')
assert read(b/'wrapper-consumer-promotion.json')['status'].startswith('passed_promoted_161_')
assert read(b/'ordinary-regeneration-4b/status.json')['status']=='passed_selected_source_bound_ordinary_owners'
assert len(read(b/'ordinary-regeneration-4b/status.json')['stages'])==10
linux=read(b/'registration-order-fix/linux-final-audit.json');assert linux['status']=='passed_exact_public_fix_source_linux_selected_tests_lint_format_and_68_dual_checker_stages'
assert linux['source_commit']=='fd99c03c8e3535bcc31f0b9d1441d4b74de80ebf' and linux['checker_stages']==68
linux_repo=r/'develop/migrations/csharp-03/probes/boolean-proof-elimination-refreeze/registration-order-linux-checkpoint/linux-final-audit.json';assert h(linux_repo)==h(b/'registration-order-fix/linux-final-audit.json')
local_repo=r/'develop/migrations/csharp-03/probes/boolean-proof-elimination-refreeze/registration-order-checkpoint/receipt.json';local=read(local_repo);assert local['checker_stages']==68 and local['rust_tests']==56
proof_path=r/'develop/migrations/csharp-03/probes/boolean-proof-elimination.json';assert h(proof_path)==plan['before_proof_evidence_raw_sha256'];old=read(proof_path)
new=dict(old);new.pop('content_sha256');good={'right-identity','constructor-false','constructor-true','open-motive','conjunction-left','conjunction-right','legacy-prior-constructor-value-levels','legacy-prior-theorem-proof-levels','unused-constructor-levels','prior-other-family-levels'}
rows=[]
for p in sorted((r/'fixtures/core-bool-cases').glob('*.hex')):
 data=bytes.fromhex(p.read_text());row={'id':p.stem,'path':str(p.relative_to(r)),'raw_sha256':h(p),'certificate_sha256':hashlib.sha256(data).hexdigest(),'expected':'accepted' if p.stem in good else 'core_rejected'};rows.append(row)
assert len(rows)==31 and sum(x['expected']=='accepted' for x in rows)==10
by_id={x['id']:x for x in rows}
for row in old['cases']:assert row==by_id[row['id']],row['id']
source_paths=set(old['core_source_hashes']);source_paths.add('go-tools/mpk-checker-ref/core_check.go');assert len(source_paths)==36
new.update(source_commit=linux['source_commit'],accepted_cases=10,rejected_cases=21,cases=rows,core_source_hashes={p:h(r/p) for p in sorted(source_paths)})
new['proof_interface']=dict(old['proof_interface']);new['proof_interface']['universe_arguments']='empty at inference, reduction and cases registration of all preceding reachable checked types/definition values/theorem proofs'
new['local']={'receipt_path':str(local_repo.relative_to(r)),'receipt_raw_sha256':h(local_repo),'rust_tests':56,'go_top_level_tests':16,'checker_stages':68}
new['linux']={'audit_path':str(linux_repo.relative_to(r)),'audit_raw_sha256':h(linux_repo),'rust_tests':linux['rust_selected_tests'],'go_top_level_tests':linux['go_top_level_tests'],'checker_stages':68,'git_blob_files_verified':linux['git_blob_files_verified']}
new['previous_implementation_evidence']={'path':'develop/migrations/csharp-03/probes/boolean-proof-elimination-refreeze/registration-order-proof-evidence-predecessor/boolean-proof-elimination.json','raw_sha256':h(proof_path),'source_commit':old['source_commit'],'accepted_cases':old['accepted_cases'],'rejected_cases':old['rejected_cases']}
new['content_sha256']=hashlib.sha256(b'MPK-CSHARP-BOOL-PROOF-ELIMINATION-1.0\0'+canon(new)).hexdigest()
candidate=plan['freeze_generator_candidate'];generator=r/candidate['production_path'];assert h(generator)==candidate['before_raw_sha256'];assert h(b/candidate['path'])==candidate['raw_sha256']
old_vectors=read(r/'develop/migrations/csharp-03/freeze/profile-freeze-vectors.json');old_freeze=read(r/'develop/migrations/csharp-03/freeze/profile-freeze.json');assert len(old_vectors['vectors'])==709
# Compute the exact complete result before changing any repository input.
prospective_proof=b/'final-registration-order-proof-evidence-candidate.json';prospective_proof.write_bytes(canon(new)+b'\n')
spec=importlib.util.spec_from_file_location('original_profile_freeze_paths',generator);original_module=importlib.util.module_from_spec(spec);spec.loader.exec_module(original_module)
spec=importlib.util.spec_from_file_location('prospective_profile_freeze',b/candidate['path']);module=importlib.util.module_from_spec(spec);spec.loader.exec_module(module)
for name in ['ROOT','FREEZE','VECTORS','INVENTORY','FOUNDATION','RECURSOR','CAPACITY','LIMIT_SOURCE']:setattr(module,name,getattr(original_module,name))
module.BOOLEAN_PROOFS=prospective_proof
fresh=module.make_freeze();vectors=module.make_vectors(fresh)
a={k:v for k,v in old_vectors.items() if k!='freeze_content_sha256'};z={k:v for k,v in vectors.items() if k!='freeze_content_sha256'};assert a==z,'complete original709vector rows changed'
a={k:v for k,v in old_freeze.items() if k not in ['evidence','content_sha256']};z={k:v for k,v in fresh.items() if k not in ['evidence','content_sha256']};assert a==z,'non-evidence freeze behavior changed'
changed=[k for k in old_freeze['evidence'] if old_freeze['evidence'][k]!=fresh['evidence'][k]];assert set(changed)=={'freeze_generator_raw_sha256','boolean_proof_evidence_raw_sha256','boolean_proof_evidence_content_sha256'},changed
archive=r/'develop/migrations/csharp-03/probes/boolean-proof-elimination-refreeze/registration-order-proof-evidence-predecessor';archive.mkdir(exist_ok=False);old_files={}
for name in ['develop/migrations/csharp-03/probes/boolean-proof-elimination.json','develop/probes/csharp-03/profile_freeze.py','develop/migrations/csharp-03/freeze/profile-freeze.json','develop/migrations/csharp-03/freeze/profile-freeze-vectors.json']:
 p=r/name;dst=archive/p.name;shutil.copy2(p,dst);assert h(p)==h(dst);old_files[p.name]=h(dst)
(archive/'file-manifest.json').write_text(json.dumps({'schema':'mpk.evidence_file_manifest.v1','files':old_files},indent=2,sort_keys=True)+'\n')
proof_path.write_bytes(canon(new)+b'\n');shutil.copy2(b/candidate['path'],generator)
spec=importlib.util.spec_from_file_location('final_profile_freeze',generator);actual_module=importlib.util.module_from_spec(spec);spec.loader.exec_module(actual_module)
assert actual_module.make_freeze()==fresh
assert actual_module.make_vectors(fresh)==vectors
module.FREEZE.write_bytes(module.bytes_of(fresh));module.VECTORS.write_bytes(module.bytes_of(vectors))
receipt={'status':'passed_final_registration_order_evidence_refresh_exact_709_vectors_and_non_evidence_freeze_behavior','before_proof_evidence_raw_sha256':plan['before_proof_evidence_raw_sha256'],'proof_evidence_raw_sha256':h(proof_path),'proof_evidence_content_sha256':new['content_sha256'],'core_source_pin_count':36,'shared_positive_cases':10,'shared_negative_cases':21,'predecessor_cases':3,'checker_stages':68,'original_19_case_records_retained_exactly':True,'all_709_complete_original_vector_rows_unchanged':True,'non_evidence_freeze_behavior_unchanged':True,'changed_freeze_evidence_fields':changed,'fixture_or_application_source_changes':0,'application_proof_ids_pending':987,'full_t01_gate':'deferred_to_final_W10','full_t06_gate':'deferred_to_W12'}
(b/'final-registration-order-proof-evidence-refresh.json').write_text(json.dumps(receipt,indent=2,sort_keys=True)+'\n');print(receipt['status'])
