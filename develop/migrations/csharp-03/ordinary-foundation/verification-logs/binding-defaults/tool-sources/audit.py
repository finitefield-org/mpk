from pathlib import Path
import hashlib,json,re
repo=Path('/Users/kazuyoshitoshiya/mpk');r=repo/'develop/migrations/csharp-03/ordinary-foundation';e=r/'verification-logs/binding-defaults'
def read(p):return json.loads(p.read_text())
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def checked(path):
 d=read(path);assert d['status']=='passed',(path,d['status'])
 for run in d['runs']:
  assert run['status']=='passed'and run['exit_code']==0
  assert sha(path.parent/run['log'])==run['log_sha256']
 return d
receipts={mode:checked(e/(mode+'.json'))for mode in ['build','definitions','semantics','preservation']}
build=receipts['build'];assert build['sources_unchanged_during_build']
initial_quality=read(e/'quality.json');assert initial_quality['status']=='failed'
assert [(x['label'],x['status'])for x in initial_quality['runs']]==[('clippy','passed'),('inventory','failed')]
for run in initial_quality['runs']:assert sha(e/run['log'])==run['log_sha256']
assert initial_quality['runs'][0]['exit_code']==0
repair=checked(e/'quality-inventory-fix.json')
assert list(repair['source_changes_since_build'])==['crates/mpk-vc/tests/csharp_practical_inventory.rs']
assert all(sha(repo/p)==repair['source_changes_since_build'].get(p,h) for p,h in build['source_sha256'].items())
assert all(sha(repo/p)==h for p,h in repair['metadata_hashes'].items())

for binary in build['binaries'].values():assert sha(Path(binary['path']))==binary['sha256']
assert receipts['preservation']['source_invariant_output_enabled'] is False
manifest=read(r/'binding-defaults/certificates.json');assert len(manifest['sources'])==45
log=(e/'checkers.log').read_text();pins=[];defaults=conditions=pending=proofs=0
for row in manifest['sources']:
 path=r/'binding-defaults'/f"{row['id']}.hex";raw=bytes.fromhex(path.read_text());digest=hashlib.sha256(b'MPK-MODULE-CERT-0.1\0'+raw).hexdigest();m=row['metadata']
 assert digest==m['certificate_sha256'];assert '--- PASS: TestCheckerAgreementWithRustCLIBindingDefaults/'+path.name+' 'in log
 assert 'Go accepted: certificate='+digest+' 'in log
 defaults+=sum(x['definition']is not None for x in m['actual_defaults']);conditions+=len(m['conditions']);pending+=len(m['pending_defaults']);proofs+=len(m['pending_proof_ids'])
 pins.append(dict(id=row['id'],certificate_sha256=digest,raw_file_sha256=sha(path),bytes=len(raw)))
assert log.count('Rust accepted identical bytes and report:')==45
assert log.count('Rust rejected changed hash:')==45 and log.count('Go rejected changed hash:')==45
assert log.count(' axioms=0 ')==45
assert conditions==manifest['conditions']and pending==sum(manifest['pending_defaults'].values())
assert conditions+pending==35 and proofs==987
semantic=(e/'closed-conditions.log').read_text();match=re.search(r'binding default observations: (\d+) actual defaults, (\d+) bits, (\d+) closed conditions, (\d+) unproven declaration flags, (\d+) invalid source with matching arm, (\d+) valid source with wrong arm',semantic);assert match
metrics=dict(zip(['actual_defaults','bits','closed_conditions','unproven_declaration_flags','invalid_source_matching_arm','valid_source_wrong_arm'],map(int,match.groups())))
assert metrics['actual_defaults']==defaults and metrics['closed_conditions']==conditions
assert metrics['invalid_source_matching_arm']>0 and metrics['valid_source_wrong_arm']>0
record=dict(status='passed_scoped_component',work_item='CSHARP-03-T06-W09',internal_unit=4,scope='Original W06 closed actual-default definitions. No proof is discharged; source-use obligations, all remaining definitions and complete unit/W09 closure remain open.',source_contexts=45,conditions=conditions,pending_defaults=pending,pending_default_reasons=manifest['pending_defaults'],proofs_pending=proofs,proofs_discharged=0,semantics=metrics,zero_axioms=True,checker_certificates=45,source_invariant_pins_preserved=53,manifest_sha256=sha(r/'binding-defaults/certificates.json'),pins=pins,current_build='build.json',current_source_hashes_verified=True,inventory_registration_repair='quality-inventory-fix.json',evidence={name:sha(e/name)for name in [*[mode+'.json'for mode in receipts],'quality.json','quality-inventory-fix.json']},maximum_terms=max(x['terms']for x in manifest['sources']),maximum_declarations=max(x['declarations']for x in manifest['sources']),full_gate='deferred_to_T06_W12')
(e/'verification.json').write_text(json.dumps(record,indent=2)+'\n')
p=r/'unit-4-binding-default-progress.json';d=read(p);d.update(status='passed_scoped_component',current_verification='verification-logs/binding-defaults/verification.json',source_contexts=45,conditions=conditions,pending_defaults=pending,semantics=metrics,checker_certificates=45,source_invariant_pins_preserved=53);d['remaining']=[x for x in d['remaining']if not x.startswith('Finish targeted')];p.write_text(json.dumps(d,indent=2)+'\n')
# Keep previous three-component inventory historical; this is a new four-component
# definition union. Identical proof IDs and source/VC hashes are mandatory.
prior=read(e.parent/'source-invariants/remaining-w06-conditions.json');d=prior.copy();d['scope']='Only the original 45 retained binding contexts. Definitions are available across four separate components, not one application certificate. All 987 proofs remain pending.';d['inputs']=dict(prior['inputs']);d['inputs']['binding-defaults']=dict(path='binding-defaults/certificates.json',raw_sha256=sha(r/'binding-defaults/certificates.json'));byid={x['id']:x['metadata']for x in manifest['sources']};available={};remaining={}
for row in d['sources']:
 m=byid[row['id']];assert m['source_ir_sha256']==row['source_ir_sha256']and m['binding_vc_sha256']==row['binding_vc_sha256'];assert sorted(m['pending_proof_ids'])==sorted(row['pending_proof_ids'])
 current=set(row['separate_component_conditions']);current.update(x['sequent']['id']for x in m['conditions']);row['separate_component_conditions']=sorted(current);row['pending_condition_ids']=sorted(set(row['pending_proof_ids'])-current)
 for target,ids in [(available,current),(remaining,row['pending_condition_ids'])]:
  for ident in ids:
   kind=ident.split('.')[1];target[kind]=target.get(kind,0)+1
for kind,value in prior['available_condition_kinds'].items():assert available[kind]==value
assert available['actual_default']==conditions
d.update(available_condition_kinds=dict(sorted(available.items())),remaining_condition_kinds=dict(sorted(remaining.items())),available_conditions=sum(available.values()),remaining_conditions=sum(remaining.values()),new_component_verification='passed_scoped_component; binding-defaults/verification.json. Source-invariant full-equation semantic verification remains separately tracked and is not upgraded here.')
assert d['available_conditions']+d['remaining_conditions']==987
(e/'remaining-w06-conditions.json').write_text(json.dumps(d,indent=2)+'\n')
print(json.dumps({k:v for k,v in record.items()if k not in ['pins','evidence']}))
