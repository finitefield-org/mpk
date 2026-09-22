from pathlib import Path
import hashlib,json,re
repo=Path('/Users/kazuyoshitoshiya/mpk');r=repo/'develop/migrations/csharp-03/ordinary-foundation';base=r/'verification-logs/source-invariants';e=base/'direct-member-reads'
def read(p):return json.loads(p.read_text())
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def checked(path):
 d=read(path);assert d['status']=='passed',(path,d['status'])
 for run in d['runs']:
  assert run['status']=='passed'and run['exit_code']==0
  assert sha(path.parent/run['log'])==run['log_sha256']
 return d
receipts={mode:checked(e/(mode+'.json'))for mode in ['build','definitions','preservation','quality']}
f=e/'prefix-test-fix';fixed={mode:checked(f/(mode+'.json'))for mode in ['build','semantics','preservation']}
build=fixed['build'];previous_build=receipts['build']
assert build['sources_unchanged_during_build']and all(sha(repo/name)==h for name,h in build['source_sha256'].items())
assert previous_build['sources_unchanged_during_build']
for b in [build,previous_build]:
 for binary in b['binaries'].values():assert sha(Path(binary['path']))==binary['sha256']
test='crates/mpk-vc/tests/support/csharp_practical_ordinary_source_invariant_tests.rs'
assert {name for name,h in build['source_sha256'].items()if previous_build['source_sha256'][name]!=h}=={test}
old_test=(f/Path(test).name).read_text();new_test=(repo/test).read_text()
assert sha(f/Path(test).name)==previous_build['source_sha256'][test]
assert old_test.replace('let TermNode::Const { global, ref levels }','let TermNode::Const { ref levels, .. }').replace('        let name = &old.name_table[old.declarations[global as usize].name as usize];\n','').replace('bit(run(old, name, vec![]));','bit(core_eval::eval(old, *arg, &[]));')==new_test
failed=read(e/'semantics.json');assert failed['status']=='failed'and failed['failure']=='direct-members failed'and len(failed['runs'])==1
assert failed['runs'][0]['exit_code']==101 and failed['runs'][0]['status']=='failed'
assert sha(e/failed['runs'][0]['log'])==failed['runs'][0]['log_sha256']
assert 'explicit panic'in(e/'direct-members.log').read_text()
assert fixed['semantics']['semantic_limits']=='none'
assert receipts['preservation']['binding_default_output_enabled']is False
assert fixed['preservation']['source_invariant_output_enabled']is False
quality_context=read(f/'quality-source-context.json')
assert quality_context['source_sha256']==build['source_sha256']
assert 'quality'not in [x['mode']for x in quality_context['original_supervisor']['runs']]

old=read(e/'previous-corpus/certificates.json');new=read(r/'source-invariants/certificates.json');assert old['definitions']==new['definitions']==72;assert old['conditions']==new['conditions']==35;assert len(old['sources'])==len(new['sources'])==53
old_byid={row['id']:row for row in old['sources']}
def normalize(metadata):
 v=json.loads(json.dumps(metadata));v.pop('certificate_sha256');v.pop('static_transformers')
 for d in v['definitions']:
  for member in d['member_reads']:member['definition']='<direct member implementation>'
 return v
pins=[];changed=0;log=(e/'checkers.log').read_text()
for row in new['sources']:
 before=old_byid[row['id']];assert normalize(before['metadata'])==normalize(row['metadata']),row['id']
 path=r/'source-invariants'/f"{row['id']}.hex";raw=bytes.fromhex(path.read_text());digest=hashlib.sha256(b'MPK-MODULE-CERT-0.1\0'+raw).hexdigest();assert digest==row['metadata']['certificate_sha256']
 original=e/'previous-corpus'/path.name;oldraw=bytes.fromhex(original.read_text());assert hashlib.sha256(b'MPK-MODULE-CERT-0.1\0'+oldraw).hexdigest()==before['metadata']['certificate_sha256']
 changed+=int(raw!=oldraw)
 assert '--- PASS: TestCheckerAgreementWithRustCLISourceInvariants/'+path.name+' 'in log;assert 'Go accepted: certificate='+digest+' 'in log
 pins.append(dict(id=row['id'],certificate_sha256=digest,raw_file_sha256=sha(path),previous_certificate_sha256=before['metadata']['certificate_sha256'],bytes_changed=raw!=oldraw))
assert log.count('Rust accepted identical bytes and report:')==53 and log.count('Rust rejected changed hash:')==53 and log.count('Go rejected changed hash:')==53 and log.count(' axioms=0 ')==53
members=re.search(r'source invariant direct members: (\d+) exact beta/eta comparisons and constant-subcube checks, (\d+) deep members',(f/'direct-members.log').read_text());assert members and int(members[2])>0
sem=re.search(r'source invariant original equations: (\d+) observations, (\d+) enum rejection cases',(f/'original-equations.log').read_text());assert sem
assert 'source invariant nested public clauses: 39 observations'in(f/'nested-clauses.log').read_text()
# The enum compiler and helper certificate are unaffected by member getters.
current=(repo/'crates/mpk-vc/src/csharp_practical_ordinary_source_invariants.rs').read_text();previous=(e/'csharp_practical_ordinary_source_invariants.rs').read_text()
start='fn emit_enum_case(';end='pub fn generate_csharp_practical_ordinary_source_invariants('
assert current[current.index(start):current.index(end)]==previous[previous.index(start):previous.index(end)]
enums=checked(base/'enums.json');enum=bytes.fromhex((r/'source-invariant-enums/enum-cases.hex').read_text());enum_sha=hashlib.sha256(b'MPK-MODULE-CERT-0.1\0'+enum).hexdigest();assert 'Go accepted: certificate='+enum_sha+' 'in(base/'enum-checkers.log').read_text()
record=dict(status='passed_scoped_component',work_item='CSHARP-03-T06-W09',internal_unit=4,source_contexts=53,type_equations=72,binding_conditions=35,changed_source_certificates=changed,checker_certificates=53,zero_axioms=True,proofs_discharged=0,direct_member_comparisons=int(members[1]),deep_members=int(members[2]),original_equation_observations=int(sem[1]),enum_rejection_cases=int(sem[2]),nested_clause_observations=39,preserved_enum_helper_observations=868,binding_default_pins_preserved=45,manifest_sha256=sha(r/'source-invariants/certificates.json'),current_build='direct-member-reads/prefix-test-fix/build.json',current_source_hashes_verified=True,pins=pins,evidence={**{mode+'.json':sha(e/(mode+'.json'))for mode in receipts},**{'prefix-test-fix/'+mode+'.json':sha(f/(mode+'.json'))for mode in fixed}},retained_failed_test='semantics.json',diagnostic='../deep-equation-diagnostic/finding.json',previous_corpus='previous-corpus',scope='Original source public bodies and W06 source-invariant conditions with exact direct member projections. All source/member/type identities, public clauses, domain definitions and pending proofs are unchanged. No unit or W09 completion is implied.',full_gate='deferred_to_T06_W12')
(e/'verification.json').write_text(json.dumps(record,indent=2)+'\n');(base/'verification.json').write_text(json.dumps(dict(record,current_verification='direct-member-reads/verification.json'),indent=2)+'\n')
p=r/'unit-4-source-invariant-progress.json';d=read(p);d.update(status='passed_scoped_component',current_verification='verification-logs/source-invariants/direct-member-reads/verification.json');d['direct_member_read_fix'].update(status='passed_scoped_component',remaining='Full W03 partial-clause integration and every application proof remain pending.');d['semantic_verification']={k:record[k]for k in ['direct_member_comparisons','deep_members','original_equation_observations','enum_rejection_cases','nested_clause_observations']};d['semantic_verification']['status']='passed';d['generation']=dict(status='passed',source_contexts=53,type_equations=72,binding_conditions=35,manifest_sha256=record['manifest_sha256'],evidence='verification-logs/source-invariants/direct-member-reads/definitions.json');d['semantic_input_correction']['status']='superseded_by_full_direct_member_equation_pass';d['initial_quality']['current_test_input_fix']='Current targeted Clippy and format passed after the test-only prefix correction; see direct-member-reads/quality.json.';d['checker_verification']['evidence']=['verification-logs/source-invariants/direct-member-reads/definitions.json','verification-logs/source-invariants/enums.json'];d['remaining']=[x for x in d['remaining']if not x.startswith('Finish current')];p.write_text(json.dumps(d,indent=2)+'\n')
# The union is unchanged: only member implementation names/certificate bytes vary.
p=base.parent/'binding-defaults/remaining-w06-conditions.json';d=read(p);d['inputs']['source-invariants']['raw_sha256']=record['manifest_sha256'];d['source_invariant_verification']='passed_scoped_component; source-invariants/direct-member-reads/verification.json';d['new_component_verification']='Default and source-invariant scoped components passed. No application proof is discharged.';p.write_text(json.dumps(d,indent=2)+'\n')
print(json.dumps({k:v for k,v in record.items()if k not in ['pins','evidence']}))
