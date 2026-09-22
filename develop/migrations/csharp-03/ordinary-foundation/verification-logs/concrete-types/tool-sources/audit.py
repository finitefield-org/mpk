from pathlib import Path
import hashlib,json,re
repo=Path('/Users/kazuyoshitoshiya/mpk');r=repo/'develop/migrations/csharp-03/ordinary-foundation';base=r/'verification-logs/concrete-types';e=base/'review-internal-states'
def read(p):return json.loads(p.read_text())
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def checked(path):
 d=read(path);assert d['status']=='passed',(path,d['status'])
 for run in d['runs']:
  assert run['status']=='passed'and run['exit_code']==0
  assert sha(path.parent/run['log'])==run['log_sha256']
 return d
receipts={mode:checked(e/(mode+'.json'))for mode in ['build','definitions','semantics','preservation','quality']}
b=receipts['build'];assert b['sources_unchanged_during_build']
assert all(sha(repo/p)==h for p,h in b['source_sha256'].items())
for binary in b['binaries'].values():assert sha(Path(binary['path']))==binary['sha256']
old=read(base/'build.json');assert old['status']=='failed'and old['sources_unchanged_during_build']is False
assert old['runs'][0]['exit_code']==0 and sha(base/old['runs'][0]['log'])==old['runs'][0]['log_sha256']
changed={p for p,h in b['source_sha256'].items()if old['source_sha256'][p]!=h}
assert changed=={'crates/mpk-vc/tests/support/csharp_practical_ordinary_concrete_type_tests.rs'}
assert sha(e/'csharp_practical_ordinary_concrete_type_tests.rs')==old['source_sha256'][next(iter(changed))]
assert receipts['semantics']['semantic_limits']=='none'
assert receipts['preservation']['source_invariant_output_enabled']is False
manifest=read(r/'concrete-types/certificates.json');assert len(manifest['sources'])==45
log=(e/'checkers.log').read_text();pins=[];definitions=pending=proofs=0
for row in manifest['sources']:
 p=r/'concrete-types'/f"{row['id']}.hex";raw=bytes.fromhex(p.read_text());digest=hashlib.sha256(b'MPK-MODULE-CERT-0.1\0'+raw).hexdigest();m=row['metadata']
 assert digest==m['certificate_sha256'];assert '--- PASS: TestCheckerAgreementWithRustCLIConcreteTypes/'+p.name+' 'in log
 assert 'Go accepted: certificate='+digest+' 'in log
 assert len(m['definitions'])==len(m['conditions'])
 by_owner={x['sequent']['owner_id']:x['sequent']for x in m['conditions']}
 for d in m['definitions']:
  instance=d['instance'];i=instance['instance_id'];assert i==instance['type_definition']['id']==d['carrier']['type_id']
  seq=by_owner[i];assert seq['kind']=='concrete_type_equivalence'and not seq['assumptions']and len(seq['goals'])==1
  assert d['symbol']in json.dumps(seq['goals']);assert d['definition']!=d['public_domain']
 for d in m['pending_type_instances']:
  ident='binding.concrete_type_equivalence.'+d['instance_id'];assert ident in m['pending_condition_ids']and ident in m['pending_proof_ids']
 definitions+=len(m['conditions']);pending+=len(m['pending_type_instances']);proofs+=len(m['pending_proof_ids'])
 pins.append(dict(id=row['id'],certificate_sha256=digest,raw_file_sha256=sha(p),bytes=len(raw)))
assert definitions==manifest['conditions']==81 and pending==manifest['pending_type_conditions']==6 and proofs==987
for marker in ['Rust accepted identical bytes and report:','Rust rejected changed hash:','Go rejected changed hash:',' axioms=0 ']:assert log.count(marker)==45
match=re.search(r'concrete type predicates: (\d+) observations, (\d+) admitted, (\d+) rejected, (\d+) false mutated conditions',(e/'predicates.log').read_text());assert match
metrics=dict(zip(['observations','admitted','rejected','false_mutated_conditions'],map(int,match.groups())))
assert metrics['observations']==metrics['false_mutated_conditions']==405 and metrics['admitted']>0 and metrics['rejected']>0
assert metrics['admitted']+metrics['rejected']==405
format=read(base/'go-format.json');assert format['status']=='passed'and sha(repo/format['file'])==format['source_sha256']
record=dict(status='passed_scoped_component',work_item='CSHARP-03-T06-W09',internal_unit=4,source_contexts=45,conditions=81,pending_internal_type_conditions=6,original_type_conditions=87,proofs_pending=987,proofs_discharged=0,semantics=metrics,checker_certificates=45,zero_axioms=True,source_invariant_pins_preserved=53,manifest_sha256=sha(r/'concrete-types/certificates.json'),pins=pins,current_build='review-internal-states/build.json',current_source_hashes_verified=True,invalidated_initial_build='build.json',review_correction='review-internal-states/finding.json',evidence={mode+'.json':sha(e/(mode+'.json'))for mode in receipts},maximum_terms=max(x['terms']for x in manifest['sources']),maximum_declarations=max(x['declarations']for x in manifest['sources']),scope='Exact closed type recipe links and original W06 predicates. Internal construction states remain explicitly pending. No application proof, concrete operation/outcome equivalence, or unit/W09 completion is supplied.',full_gate='deferred_to_T06_W12')
(e/'verification.json').write_text(json.dumps(record,indent=2)+'\n');(base/'verification.json').write_text(json.dumps(dict(record,current_verification='review-internal-states/verification.json'),indent=2)+'\n')
p=r/'unit-4-concrete-type-progress.json';d=read(p);d.update(status='passed_scoped_component',current_verification='verification-logs/concrete-types/verification.json',source_contexts=45,conditions=81,pending_internal_type_conditions=6,semantics=metrics,checker_certificates=45,source_invariant_pins_preserved=53);d['remaining']=[x for x in d['remaining']if not x.startswith('Complete scoped')];d['remaining'].insert(0,'Six original internal construction-state type conditions still require their ownership model.');p.write_text(json.dumps(d,indent=2)+'\n')
# Extend the verified four-component definition inventory without changing any
# original source/VC identity or counting a pending proof as discharged.
prior=read(base.parent/'binding-defaults/remaining-w06-conditions.json');d=json.loads(json.dumps(prior));d['scope']='Only the original 45 binding contexts. Available definitions span five separate ordinary components, not a complete application certificate. All 987 proofs remain pending.'
for input in d['inputs'].values():assert sha(r/input['path'])==input['raw_sha256']
d['inputs']['concrete-types']=dict(path='concrete-types/certificates.json',raw_sha256=record['manifest_sha256']);byid={x['id']:x['metadata']for x in manifest['sources']};available={};remaining={}
for row in d['sources']:
 m=byid[row['id']];assert m['source_ir_sha256']==row['source_ir_sha256']and m['binding_vc_sha256']==row['binding_vc_sha256'];assert sorted(m['pending_proof_ids'])==sorted(row['pending_proof_ids'])
 current=set(row['separate_component_conditions']);new={x['sequent']['id']for x in m['conditions']};assert not current&new;current.update(new);row['separate_component_conditions']=sorted(current);row['pending_condition_ids']=sorted(set(row['pending_proof_ids'])-current)
 for target,ids in [(available,current),(remaining,row['pending_condition_ids'])]:
  for ident in ids:
   kind=ident.split('.')[1];target[kind]=target.get(kind,0)+1
for kind,value in prior['available_condition_kinds'].items():assert available[kind]==value
assert available['concrete_type_equivalence']==81 and remaining['concrete_type_equivalence']==6
d.update(available_condition_kinds=dict(sorted(available.items())),remaining_condition_kinds=dict(sorted(remaining.items())),available_conditions=sum(available.values()),remaining_conditions=sum(remaining.values()),new_component_verification='Concrete type scoped component passed; concrete-types/verification.json. Six construction type conditions and all application proofs remain pending.')
assert d['available_conditions']==445 and d['remaining_conditions']==542
(base/'remaining-w06-conditions.json').write_text(json.dumps(d,indent=2)+'\n')
print(json.dumps({k:v for k,v in record.items()if k not in ['pins','evidence']}))
