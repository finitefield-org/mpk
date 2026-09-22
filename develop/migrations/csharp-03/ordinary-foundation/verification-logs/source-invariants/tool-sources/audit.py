from pathlib import Path
import hashlib,json,re
repo=Path('/Users/kazuyoshitoshiya/mpk');r=repo/'develop/migrations/csharp-03/ordinary-foundation';e=r/'verification-logs/source-invariants'
def read(p):return json.loads(p.read_text())
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def checked(path):
 d=read(path);assert d['status']=='passed',(str(path),d['status'])
 for run in d.get('runs',[]):
  assert run['status']=='passed';log=path.parent/run['log'];assert sha(log)==run['log_sha256'];assert run['exit_code']==0
 return d
base=checked(e/'build.json');definitions=checked(e/'definitions.json');enums=checked(e/'enums.json');preservation=checked(e/'preservation.json');quality=checked(e/'quality.json');fix=checked(e/'equation-input-fix/verification.json')
assert fix['sources_unchanged_during_build'];assert sha(Path(fix['binary']))==fix['binary_sha256'];assert all(sha(Path(x['path']))==x['sha256']for x in base['binaries'].values())
source_changes=[name for name,h in fix['source_sha256'].items()if sha(repo/name)!=h]
current_generation_build='equation-input-fix/verification.json'
current_pin_preservation=None
if source_changes:
 # A subsequent independent component adds exports while the immutable semantic
 # process continues. Reuse its observations only with exact current metadata/
 # certificate replay and an unchanged test oracle, evaluator and implementation.
 current=e.parent/'binding-defaults'
 latest=checked(current/'build.json');replay=checked(current/'preservation.json');checked(current/'quality.json')
 assert latest['sources_unchanged_during_build']
 assert all(sha(repo/name)==h for name,h in latest['source_sha256'].items())
 for name in ['crates/mpk-vc/src/csharp_practical_ordinary_source_invariants.rs','crates/mpk-vc/src/csharp_practical_ordinary_test_eval.rs','crates/mpk-vc/tests/support/csharp_practical_ordinary_source_invariant_tests.rs']:
  assert latest['source_sha256'][name]==fix['source_sha256'][name],name
 run=next(x for x in replay['runs']if x['label']=='source-invariant-preservation')
 assert 'test result: ok. 1 passed; 0 failed;'in(current/run['log']).read_text()
 assert replay['source_invariant_output_enabled'] is False
 assert sha(Path(latest['binaries']['integration']['path']))==latest['binaries']['integration']['sha256']
 current_generation_build='../binding-defaults/build.json'
 current_pin_preservation='../binding-defaults/preservation.json'

changed=[name for name,h in base['source_sha256'].items()if h!=fix['source_sha256'][name]];assert changed==['crates/mpk-vc/tests/support/csharp_practical_ordinary_source_invariant_tests.rs'],changed
first=read(e/'semantics.json');assert first['status']=='failed';assert first['runs'][2]['label']=='original-equations';assert first['runs'][2]['status']=='failed'
for run in first['runs']:
 assert sha(e/run['log'])==run['log_sha256']
for run in first['runs'][:2]:
 assert run['status']=='passed'and run['exit_code']==0;assert 'test result: ok. 1 passed; 0 failed;'in(e/run['log']).read_text()
assert 'non-Bool major'in(e/'original-equations.log').read_text()
original=(e/'equation-input-fix/original-equations.log').read_text();match=re.search(r'source invariant original equations: (\d+) observations, (\d+) enum rejection cases',original);assert match and'test result: ok. 1 passed; 0 failed;'in original
assert'source invariant nested public clauses: 39 observations'in(e/'nested-clauses.log').read_text();assert'source invariant enum underlying bits: 868 observations'in(e/'enum-bits.log').read_text()
manifest=read(r/'source-invariants/certificates.json');assert len(manifest['sources'])==53;assert manifest['conditions']==35 and manifest['definitions']==72
log=(e/'checkers.log').read_text();pins={}
for row in manifest['sources']:
 path=r/'source-invariants'/f"{row['id']}.hex";raw=bytes.fromhex(path.read_text());digest=hashlib.sha256(b'MPK-MODULE-CERT-0.1\0'+raw).hexdigest();assert digest==row['metadata']['certificate_sha256'];assert'--- PASS: TestCheckerAgreementWithRustCLISourceInvariants/'+path.name+' 'in log;assert'Go accepted: certificate='+digest+' 'in log;pins[str(path.relative_to(r))]={'raw_file_sha256':sha(path),'certificate_sha256':digest,'bytes':len(raw)}
assert log.count('Rust accepted identical bytes and report:')==53 and log.count('Rust rejected changed hash:')==53
path=r/'source-invariant-enums/enum-cases.hex';raw=bytes.fromhex(path.read_text());digest=hashlib.sha256(b'MPK-MODULE-CERT-0.1\0'+raw).hexdigest();elog=(e/'enum-checkers.log').read_text();assert'Go accepted: certificate='+digest+' 'in elog;assert'--- PASS: TestCheckerAgreementWithRustCLISourceInvariantEnums/enum-cases.hex 'in elog;assert'Rust rejected changed hash:'in elog;pins[str(path.relative_to(r))]={'raw_file_sha256':sha(path),'certificate_sha256':digest,'bytes':len(raw)}
record=dict(status='passed_scoped_component',scope='Original W02 source invariant body definitions and original W06 source_invariant conditions with all proof obligations pending. This does not close unit 4, W09 or partial-clause definedness integration.',source_contexts=53,type_equations=72,binding_conditions=35,original_equation_observations=int(match[1]),enum_rejection_cases=int(match[2]),nested_clause_observations=39,enum_bit_observations=868,checker_source_certificates=53,checker_helper_certificates=1,proofs_discharged=0,maximum_terms=max(x['terms']for x in manifest['sources']),maximum_declarations=max(x['declarations']for x in manifest['sources']),manifest_sha256=sha(r/'source-invariants/certificates.json'),pins=pins,semantic_build='equation-input-fix/verification.json',current_build=current_generation_build,current_pin_preservation=current_pin_preservation,source_changes_since_semantic_build=source_changes,initial_build='build.json',current_source_hashes_verified=True,evidence={k:sha(e/k) for k in ['build.json','definitions.json','enums.json','preservation.json','quality.json','semantics.json','equation-input-fix/verification.json']},review='unit-4-source-invariant-review.md',full_gate='deferred_to_T06_W12',initial_semantic_failure='Retained: malformed zero-depth test value. Corrected typed input; complete original-equation rerun passed. Production and generated certificates unchanged.')
(e/'verification.json').write_text(json.dumps(record,indent=2)+'\n')
p=r/'unit-4-source-invariant-progress.json';d=read(p);d['status']='passed_scoped_component';d['current_verification']='verification-logs/source-invariants/verification.json';d['semantic_input_correction']['status']='passed_full_rerun';d['semantic_verification']={k:record[k]for k in ['original_equation_observations','enum_rejection_cases','nested_clause_observations','enum_bit_observations']};d['semantic_verification']['status']='passed';d['initial_quality']['current_test_input_fix']='Targeted Clippy and format passed. Subsequent component export changes are covered by current source hashes, unchanged semantic oracle/evaluator/implementation hashes and exact 53-context metadata/certificate replay; see current_pin_preservation in the verification record.';d['remaining']=[x for x in d['remaining']if not x.startswith('Finish current')];p.write_text(json.dumps(d,indent=2)+'\n')
p=e/'remaining-w06-conditions.json';d=read(p);d['new_component_verification']='passed_scoped_component; verification-logs/source-invariants/verification.json';p.write_text(json.dumps(d,indent=2)+'\n')
print(json.dumps({k:v for k,v in record.items()if k not in ['pins','evidence']}))
