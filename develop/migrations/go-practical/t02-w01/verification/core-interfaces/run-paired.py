"""Verify the same fresh ordinary core bytes with the current independent Go checker."""
from pathlib import Path
import datetime,hashlib,json,subprocess,time
root=Path(__file__).resolve().parent.relative_to(Path.cwd().resolve());out=root/'paired';out.mkdir(exist_ok=False)
def seal(p):
 p=Path(p);raw=p.read_bytes();return {'path':str(p),'sha256':hashlib.sha256(raw).hexdigest(),'bytes':len(raw)}
rust_path=root/'final/rust-tests/result.json';rust=json.loads(rust_path.read_text());assert rust['status']=='passed' and rust['tests_passed']==4
for item in rust['sources']:assert seal(item['path'])==item
build_path=root/'initial/go-build/result.json';build=json.loads(build_path.read_text());assert build['status']=='passed';binary=build['checker_binary'];assert seal(binary['path'])==binary
production_sources=[seal(p) for p in sorted(Path('go-tools/mpk-checker-ref').rglob('*.go')) if not p.name.endswith('_test.go')]+[seal('go-tools/mpk-checker-ref/go.mod')]
rows=[];positive={'core-foundation','retained-boolean-basis','unused-interface-nodes'}
certificates=sorted((root/'final/rust-tests/certificates').glob('*.mpcert'));assert len(certificates)==18
for certificate in certificates:
 name=certificate.stem;rust_report=certificate.with_suffix('.rust.json');r=json.loads(rust_report.read_text());accepted=name in positive;assert r['verdict']==('accepted' if accepted else 'rejected')
 command=[binary['path'],'verify',str(certificate)];started=time.monotonic();run=subprocess.run(command,stdout=subprocess.PIPE,stderr=subprocess.PIPE,timeout=60)
 stdout=out/(name+'.stdout.log');stderr=out/(name+'.stderr.log');stdout.write_bytes(run.stdout);stderr.write_bytes(run.stderr)
 go=json.loads(run.stdout);assert not run.stderr and run.returncode==(0 if accepted else 1) and go['verdict']==r['verdict'],(name,go,r)
 if accepted:
  assert r['axiom_count']==0 and go.get('axiom_count',0)==0 and go['hashes']==r['hashes'] and go['declaration_count']==r['declaration_count']
 else:
  assert go['error_kind']=='core_check' and r['error_code']=='KERNEL_CORE_CHECK' and json.loads(r['error_detail'])['code'].startswith('CORE_'),(name,r,go)
 rows.append({'name':name,'certificate':seal(certificate),'rust_report':seal(rust_report),'command':command,'exit_code':run.returncode,'seconds':time.monotonic()-started,'stdout':seal(stdout),'stderr':seal(stderr),'rust_observation':r,'go_observation':go,'same_original_bytes':True})
record={'schema':'mpk.go_practical.paired_core_foundation.v1','status':'passed','observed_at_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'rust_execution':seal(rust_path),'current_go_build':seal(build_path),'go_binary':binary,'go_production_sources':production_sources,'cases':rows,'accepted_zero_axiom':len(positive),'rejected_in_core':len(rows)-len(positive),'CSharp_tests_executed':False,'T02_W01_complete':False,'public_profile_activated':False,'source_program_proof_claimed':False,'native_Linux_acceptance_claimed':False,'full_T02_gate':'deferred to W04: check-fast.sh --skip-csharp','scope':'Actual ordinary core basis and both exact frozen additive interfaces. Same full bytes/hashes, arbitrary-Nat legacy Nat.rec identity proved using Nat.induct, conditional Bool FalseElim and unchanged Boolean basis. Complete Go foundation/descriptor/context and source/application correspondence remain due.'}
(out/'result.json').write_text(json.dumps(record,indent=2)+'\n');print(json.dumps({'status':'passed','same_byte_cases':len(rows),'accepted_zero_axiom':len(positive),'rejected_in_core':len(rows)-len(positive)}))
