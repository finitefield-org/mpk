from pathlib import Path
import hashlib,json,re,collections,sys
repo=Path('/Users/kazuyoshitoshiya/mpk');r=repo/'develop/migrations/csharp-03/ordinary-foundation';base=r/'verification-logs/concrete-operations';e=base;review=base/'review-isolated-mutations'
def read(p):return json.loads(p.read_text())
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def checked(path):
 d=read(path);assert d['status']=='passed',(path,d['status'])
 for run in d['runs']:
  assert run['status']=='passed'and run['exit_code']==0
  assert sha(path.parent/run['log'])==run['log_sha256']
 return d
receipt_paths={mode:e/(mode+'.json')for mode in ['definitions','routing','real','preservation']};receipt_paths.update({mode:review/(mode+'.json')for mode in ['build','isolated','quality']});receipt_paths['definition_preservation']=review/'definitions.json';receipt_paths['initial_build']=e/'build.json'
receipts={mode:checked(path)for mode,path in receipt_paths.items()};b=receipts['build'];assert b['sources_unchanged_during_build'];assert all(sha(repo/p)==h for p,h in b['source_sha256'].items())
initial=receipts['initial_build'];assert initial['sources_unchanged_during_build'];changed={p for p,h in initial['source_sha256'].items()if b['source_sha256'][p]!=h};test_path='crates/mpk-vc/tests/support/csharp_practical_ordinary_concrete_operation_tests.rs';assert changed=={test_path}
old_test=review/Path(test_path).name;assert sha(old_test)==initial['source_sha256'][test_path]
expected=old_test.read_text().replace('for d in rows.iter().filter(|d| d["carrier"]["type_id"] == id) {','if let Some(d) = rows.iter().find(|d| d["carrier"]["type_id"] == id) {')
assert (repo/test_path).read_text().split('// A test-only closed IEEE word.')[0].rstrip()==expected.rstrip()
for binary in initial['binaries'].values():assert sha(Path(binary['path']))==binary['sha256']
failed_quality=read(e/'quality.json');assert failed_quality['status']=='failed'and failed_quality['failure']=='clippy failed';assert len(failed_quality['runs'])==1;clippy=failed_quality['runs'][0];assert clippy['exit_code']!=0 and sha(e/clippy['log'])==clippy['log_sha256'];assert 'clippy::never_loop'in(e/clippy['log']).read_text()
assert receipts['definition_preservation']['corpus_output_enabled']is False and receipts['isolated']['semantic_limits']=='none'
assert 'isolated concrete operations: 1 NaN payload changes, 1 malformed source inputs'in(review/'isolated.log').read_text()
for binary in b['binaries'].values():assert sha(Path(binary['path']))==binary['sha256']
assert receipts['routing']['semantic_limits']==receipts['real']['semantic_limits']=='none'
manifest=read(r/'concrete-operations/certificates.json');assert len(manifest['sources'])==45
assert receipts['definition_preservation']['unchanged_certificate_corpus']=={x['id']:x['metadata']['certificate_sha256']for x in manifest['sources']}
log=(e/'checkers.log').read_text();pins=[];definitions=pending=proofs=0;reasons=collections.Counter();arities=collections.Counter()
for row in manifest['sources']:
 p=r/'concrete-operations'/f"{row['id']}.hex";raw=bytes.fromhex(p.read_text());digest=hashlib.sha256(b'MPK-MODULE-CERT-0.1\0'+raw).hexdigest();m=row['metadata']
 assert digest==m['certificate_sha256'];assert '--- PASS: TestCheckerAgreementWithRustCLIConcreteOperations/'+p.name+' 'in log;assert 'Go accepted: certificate='+digest+' 'in log
 assert len(m['definitions'])==len(m['conditions']);by_owner={x['sequent']['owner_id']:x['sequent']for x in m['conditions']}
 originals={o['id']:(i['instance_id'],o)for i in m['instances']for o in i['operation_definitions']};assert len(originals)==len(m['definitions'])+len(m['pending_operations'])
 for d in m['definitions']+m['pending_operations']:
  ident=d['component']['operation_id'];owner,recipe=originals.pop(ident);assert owner==d['instance_id']and recipe==d['recipe'];assert d['component']['argument_type_ids']==recipe['argument_type_ids']and d['component']['result_type_id']==recipe['normal_result_type_id'];assert [f['label']for f in d['component']['failures']]==recipe['error_precedence']
  if 'reasons'in d:
   assert d['reasons'];reasons.update(d['reasons']);condition='binding.concrete_definition_equivalence.'+ident;assert condition in m['pending_condition_ids']and condition in m['pending_proof_ids']
  else:
   assert ident in by_owner;seq=by_owner[ident];assert seq['kind']=='concrete_definition_equivalence';assert [s['type_id']for s in seq['subjects']]==recipe['argument_type_ids'];arities[len(seq['subjects'])]+=1
   assert len(seq['goals'])==2 and len(seq['assumptions'])==len(seq['subjects']);assert d['concrete_symbol']in json.dumps(seq['goals'])and d['all_outcomes_symbol']in json.dumps(seq['goals']);assert d['normal_definition']!=d['concrete_definition']
 assert not originals
 definitions+=len(m['conditions']);pending+=len(m['pending_operations']);proofs+=len(m['pending_proof_ids']);pins.append(dict(id=row['id'],certificate_sha256=digest,raw_file_sha256=sha(p),bytes=len(raw)))
assert definitions==manifest['conditions']==436 and pending==manifest['pending_operation_conditions']==31 and proofs==987
assert reasons==dict(application_currency_predicate=1,internal_construction_state=30,source_ownership=24);assert set(arities)=={0,1,2,3,4}
for marker in ['Rust accepted identical bytes and report:','Rust rejected changed hash:','Go rejected changed hash:',' axioms=0 ']:assert log.count(marker)==45
match=re.search(r'ordered outcomes: (\d+) operations, (\d+) assignments, (\d+) single-failure changes, (\d+) correctly masked later changes',(e/'routing.log').read_text());assert match
routing=dict(zip(['operations','assignments','single_failure_changes','masked_later_changes'],map(int,match.groups())));assert routing['masked_later_changes']>0
match=re.search(r'real operation values: (\d+) normal observations, (\d+) closed operations, (\d+) NaN observation, (\d+) rejected-input guards',(e/'real.log').read_text());assert match
real=dict(zip(['normal_observations','closed_operations','nan_observations','rejected_input_guards'],map(int,match.groups())));assert real['nan_observations']==1
preserved={name:dict(certificate_count=len(list((r/name).glob('*.hex'))),manifest_sha256=sha(r/name/'certificates.json'))for name in ['structural-foundations','structural-boundary','structural-public','concrete-types']};assert sum(x['certificate_count']for x in preserved.values())==96
record=dict(status='passed_scoped_component',work_item='CSHARP-03-T06-W09',internal_unit=4,source_contexts=45,conditions=436,pending_operation_conditions=31,original_operation_conditions=467,pending_reasons=dict(reasons),operand_arities=dict(arities),proofs_pending=987,proofs_discharged=0,routing=routing,real_values=real,isolated_mutations=dict(nan_payload_changes=1,malformed_source_inputs=1),retained_semantics='Only the definition test lookup syntax changed and one independent test was appended. Existing routing and real-value test functions/helpers and all production sources remain exact; archived original binaries and all 45 current corpus pins are verified.',initial_lint_failure='quality.json',review_correction='review-isolated-mutations/finding.json',checker_certificates=45,zero_axioms=True,preserved=preserved,manifest_sha256=sha(r/'concrete-operations/certificates.json'),pins=pins,current_build=str((review/'build.json').relative_to(base)),current_source_hashes_verified=True,evidence={str(path.relative_to(base)):sha(path)for path in receipt_paths.values()},maximum_terms=max(x['terms']for x in manifest['sources']),maximum_declarations=max(x['declarations']for x in manifest['sources']),scope='Exact original closed operation recipes, normal values, first failures/success and original W06 conditions. Construction ownership/state and application currency prerequisites remain explicitly pending. No application proof or unit/W09 completion is supplied.',full_gate='deferred_to_T06_W12')
(base/'verification.json').write_text(json.dumps(record,indent=2)+'\n')
p=r/'unit-4-concrete-operation-progress.json';d=read(p);d.update(status='passed_scoped_component',current_verification='verification-logs/concrete-operations/verification.json',source_contexts=45,conditions=436,pending_operation_conditions=31,routing=routing,real_values=real,checker_certificates=45,preserved=preserved);d['verification']='Complete targeted terminal evidence, unchanged original recipes/sequents, mutation checks, prior-byte preservation, both checkers and quality checks passed. All application proofs remain pending.';p.write_text(json.dumps(d,indent=2)+'\n')
prior=read(base.parent/'concrete-types/remaining-w06-conditions.json');d=json.loads(json.dumps(prior));d['scope']='Only the original 45 binding contexts. Available definitions span six separate ordinary components, not a complete application certificate. All 987 proofs remain pending.'
for item in d['inputs'].values():assert sha(r/item['path'])==item['raw_sha256']
d['inputs']['concrete-operations']=dict(path='concrete-operations/certificates.json',raw_sha256=record['manifest_sha256']);byid={x['id']:x['metadata']for x in manifest['sources']};available=collections.Counter();remaining=collections.Counter()
for row in d['sources']:
 m=byid[row['id']];assert m['source_ir_sha256']==row['source_ir_sha256']and m['binding_vc_sha256']==row['binding_vc_sha256'];assert sorted(m['pending_proof_ids'])==sorted(row['pending_proof_ids'])
 current=set(row['separate_component_conditions']);new={x['sequent']['id']for x in m['conditions']};assert not current&new;current.update(new);row['separate_component_conditions']=sorted(current);row['pending_condition_ids']=sorted(set(row['pending_proof_ids'])-current)
 available.update(i.split('.')[1]for i in current);remaining.update(i.split('.')[1]for i in row['pending_condition_ids'])
for kind,value in prior['available_condition_kinds'].items():assert available[kind]==value
assert available['concrete_definition_equivalence']==436 and remaining['concrete_definition_equivalence']==31
d.update(available_condition_kinds=dict(sorted(available.items())),remaining_condition_kinds=dict(sorted(remaining.items())),available_conditions=sum(available.values()),remaining_conditions=sum(remaining.values()),new_component_verification='Concrete operation scoped component passed; concrete-operations/verification.json. Construction state/ownership, application currency predicates and all application proofs remain pending.')
assert d['available_conditions']==881 and d['remaining_conditions']==106
(base/'remaining-w06-conditions.json').write_text(json.dumps(d,indent=2)+'\n')
print(json.dumps({k:v for k,v in record.items()if k not in ['pins','evidence']}))
