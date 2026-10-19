"""Independent current Go verification of the same actual full-width word bytes."""
from pathlib import Path
import datetime,hashlib,json,re,subprocess,time

repo=Path.cwd();root=Path(__file__).resolve().parent.relative_to(repo.resolve());out=root/'paired-final';out.mkdir(exist_ok=False)
def seal(p):
 p=Path(p);b=p.read_bytes();return {'path':str(p),'sha256':hashlib.sha256(b).hexdigest(),'bytes':len(b)}
original_path=root/'reviewed/rust-tests/result.json';original=json.loads(original_path.read_text())
assert original['status']=='failed' and original['exit_code']==101
assert '4 passed; 1 failed' in (root/'reviewed/rust-tests/stdout.log').read_text()
for source in original['sources']:
 saved=seal(root/'reviewed/rust-tests/inputs'/source['path']);assert saved['sha256']==source['sha256'] and saved['bytes']==source['bytes']
 if source['path']!='crates/mpk-vc/tests/go_practical_word_foundation.rs':assert seal(source['path'])==source
focused_path=root/'final/rust-claims/result.json';focused=json.loads(focused_path.read_text());assert focused['status']=='passed' and focused['tests_passed']==1
for source in focused['sources']:assert seal(source['path'])==source
old_bad=root/'reviewed/rust-tests/certificates/divisor-guard-8.rust.json';bad=json.loads(old_bad.read_text());assert bad['error_code']=='KERNEL_CANONICAL_CERTIFICATE'
previous_path=root.parent/'core-interfaces/paired/result.json';previous=json.loads(previous_path.read_text());assert previous['status']=='passed'
binary=previous['go_binary'];assert seal(binary['path'])==binary
for source in previous['go_production_sources']:assert seal(source['path'])==source
positive={'word-foundation','retained-range-basis','honest-different-word-component'}
retained=positive|{'Boolean-count-'+str(w) for w in [8,16,32,64]}|{'wrong-adder-definition'}
old_dir=root/'reviewed/rust-tests/certificates';cases=[old_dir/(name+'.mpcert') for name in sorted(retained)];assert len(cases)==8
new_cases=sorted((root/'final/rust-claims/certificates').glob('*.mpcert'));assert len(new_cases)==12 and [seal(p) for p in new_cases]==focused['certificates']
cases=sorted(cases+new_cases,key=lambda p:p.stem);assert len(cases)==20 and len({p.stem for p in cases})==20
assert (old_dir/'word-foundation.mpcert').read_bytes()==(root/'initial/rust-tests/certificates/word-foundation.mpcert').read_bytes()
rows=[]
for p in cases:
 name=p.stem;accepted=name in positive;rust_path=p.with_suffix('.rust.json');r=json.loads(rust_path.read_text());assert r['verdict']==('accepted' if accepted else 'rejected')
 command=[binary['path'],'verify',str(p)];start=time.monotonic();run=subprocess.run(command,stdout=subprocess.PIPE,stderr=subprocess.PIPE,timeout=60,check=False)
 stdout=out/(name+'.stdout.log');stderr=out/(name+'.stderr.log');stdout.write_bytes(run.stdout);stderr.write_bytes(run.stderr);g=json.loads(run.stdout)
 assert not run.stderr and run.returncode==(0 if accepted else 1) and g['verdict']==r['verdict'],(name,r,g)
 if accepted:
  assert r['axiom_count']==0 and g.get('axiom_count',0)==0 and r['hashes']==g['hashes'] and r['declaration_count']==g['declaration_count'],(name,r,g)
 else:
  detail=json.loads(r['error_detail']);assert r['error_code']=='KERNEL_CORE_CHECK' and detail['code']=='CORE_TYPE_MISMATCH' and detail['details']['kind']=='check_type_mismatch',(name,r,g)
  inferred=detail['details']['inferred_kind'];expected=detail['details']['expected_kind'];kinds={'sort','var','const','app','lam','pi','let'}
  assert inferred in kinds and expected in kinds and g['error_kind']=='core_check' and re.fullmatch(r'term [0-9]+ inferred '+re.escape(inferred)+' but expected '+re.escape(expected),g['error_detail']),(name,r,g)
 rows.append({'name':name,'certificate':seal(p),'rust_report':seal(rust_path),'command':command,'exit_code':run.returncode,'seconds':time.monotonic()-start,'stdout':seal(stdout),'stderr':seal(stderr),'rust_observation':r,'go_observation':g,'same_original_bytes':True})
record={'schema':'mpk.go_practical.paired_word_foundation.v1','status':'passed','observed_at_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'original_execution':seal(original_path),'focused_corrected_claim_execution':seal(focused_path),'original_canonical_negative_not_counted':seal(old_bad),'selection_reason':'All complete word circuits/basic laws, exact import/retained Range, actual1640-input model/oracle and typed count/adder roots passed. Only false-claim root initially stopped on an invalid helper declaration name. Retain original inputs/diagnostic, replace those helper names and rerun that affected12-candidate root. Current unchanged Go checker independently checks all20 final actual byte streams; three acceptances have exact three hashes and zero axioms, all17 negatives must be core type mismatches, never format/resource results.','unchanged_go_build_evidence':seal(previous_path),'go_binary':binary,'unchanged_go_production_sources':previous['go_production_sources'],'cases':rows,'accepted_zero_axiom':3,'rejected_core_type_mismatch':17,'CSharp_tests_executed':False,'T02_W01_complete':False,'public_profile_activated':False,'full_numeric_or_source_program_proof_claimed':False,'full_T02_gate':'deferred to W04: check-fast.sh --skip-csharp'}
(out/'result.json').write_text(json.dumps(record,indent=2)+'\n');print(json.dumps({'status':'passed','same_byte_cases':20,'accepted_zero_axiom':3,'rejected_core_type_mismatch':17}))
