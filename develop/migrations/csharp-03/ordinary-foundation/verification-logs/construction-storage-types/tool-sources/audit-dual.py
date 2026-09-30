"""Independently audit terminal dual reports and preserve exact raw receipts."""
from collections import Counter
from datetime import datetime, timezone
from hashlib import sha256
import json
from pathlib import Path
import shutil

repo=Path('/private/tmp/mpk-w09-packed-pattern-proofs')
base=repo/'develop/migrations/csharp-03/ordinary-foundation/verification-logs/construction-storage-types'
source=Path('/tmp/mpk-w09-construction-storage-types/attempt-2/certificates')
root=Path('/tmp/mpk-w09-construction-storage-types/checks')
prior=base.parent/'concrete-type-proofs/checks'
v=json.loads((root/'verification.json').read_bytes())
assert (root/'verification.json').read_bytes()==(root/'status.json').read_bytes()
assert v['status']=='passed' and not v['launches']
assert (len(v['stages']),len(v['new_stages']),len(v['retained_stages']))==(184,26,158)
assert v['previous_receipt_sha256']==sha256((prior/'verification.json').read_bytes()).hexdigest()
old={(r['backend'],r['case'],r['kind']):r for r in json.loads((prior/'verification.json').read_bytes())['stages']}
binaries={'go':Path('/tmp/mpk-w09-default-closed-proofs/go-checker'),'rust':Path('/Users/kazuyoshitoshiya/mpk/target/release/mpk')}
for backend,binary in binaries.items():
 assert sha256(binary.read_bytes()).hexdigest()==v['binaries'][backend]
manifest=json.loads((base/'source-manifest.json').read_bytes())
for group in ('source_hashes','fixture_hashes'):
 for name,digest in manifest[group].items():
  assert sha256((repo/name).read_bytes()).hexdigest()==digest,name
expected={}
for path in sorted(source.glob('*.hex')):
 name=path.stem
 data=bytes.fromhex(path.read_text())
 if name.endswith('-wrong'):
  expected[(name.removesuffix('-wrong'),'wrong')]=data
 else:
  expected[(name,'positive')]=data
  expected[(name,'hash')]=data[:-1]+bytes([data[-1]^1])
assert len(expected)==92
rows={(r['backend'],r['case'],r['kind']):r for r in v['stages']}
assert len(rows)==184 and set(rows)=={(b,c,k) for b in binaries for c,k in expected}
assert Counter(r['kind'] for r in v['stages'])=={'positive':90,'hash':90,'wrong':4}
assert [r for r in v['stages'] if 'retained_from' in r]==v['retained_stages']
assert [r for r in v['stages'] if 'retained_from' not in r]==v['new_stages']
for (case,kind),data in expected.items():
 stem=case+'-'+kind
 assert (root/(stem+'.mpcert')).read_bytes()==data
 certificate_hash=sha256(b'MPK-MODULE-CERT-0.1\0'+data).hexdigest()
 positives={}
 for backend in binaries:
  row=rows[(backend,case,kind)]
  report_bytes=(root/(stem+'-'+backend+'.json')).read_bytes()
  stderr=(root/(stem+'-'+backend+'.stderr')).read_bytes()
  assert row['binary_sha256']==v['binaries'][backend]
  assert row['input_file_sha256']==sha256(data).hexdigest()
  assert row['certificate_sha256']==certificate_hash
  assert row['report_sha256']==sha256(report_bytes).hexdigest()
  assert row['stderr_sha256']==sha256(stderr).hexdigest()
  assert row['exit_code']==(0 if kind=='positive' else 1)
  if 'retained_from' in row:
   assert row['retained_input_and_binary_exact'] is True
   assert {k:value for k,value in row.items() if k not in ('retained_from','retained_input_and_binary_exact')}==old[(backend,case,kind)]
   assert (prior/(stem+'.mpcert')).read_bytes()==data
   assert (prior/(stem+'-'+backend+'.json')).read_bytes()==report_bytes
   assert (prior/(stem+'-'+backend+'.stderr.txt')).read_bytes()==stderr
  report=json.loads(report_bytes)
  assert report['verdict']==('accepted' if kind=='positive' else 'rejected')
  if kind=='positive':
   if backend=='go':
    core=report['report']
    hashes={key:bytes(core[field]).hex() for key,field in [('export','ExportHash'),('axiom_report','AxiomReportHash'),('certificate','CertificateHash')]}
    counts=(core['Module'],core['DeclarationCount'],core['AxiomCount'])
    summary=core['AxiomReport']['Summary']
   else:
    hashes=report['hashes']
    counts=(report['module'],report['declaration_count'],report['axiom_count'])
    summary=report['axiom_report']['summary']
   assert counts[2]==0 and all(value==0 for value in summary.values())
   assert hashes['certificate']==certificate_hash
   positives[backend]=(hashes,counts)
  else:
   assert (report['certificate'] if backend=='go' else report['hashes']['certificate'])==certificate_hash
   expected_error={'go':{'hash':'hash_mismatch','wrong':'core_check'},'rust':{'hash':'KERNEL_HASH_MISMATCH','wrong':'KERNEL_CORE_CHECK'}}
   assert report['error_kind' if backend=='go' else 'error_code']==expected_error[backend][kind]
 if kind=='positive':
  assert positives['go']==positives['rust'],stem
metadata=[json.loads(p.read_bytes()) for p in source.glob('*.json')]
assert len(metadata)==45
assert sum(len(m['proofs']) for m in metadata)==87
assert sum(len(m['pending_type_instances']) for m in metadata)==0
assert sum(len(m['pending_proof_ids']) for m in metadata)==987
assert all(m['proof_check_pending'] and m['application_scope_pending'] for m in metadata)
domains=[d for m in metadata for d in m.get('construction_storage_domains',[])]
assert len(domains)==6 and all(d['private_storage_only'] and d['ownership_pending'] for d in domains)
expected_catalog=json.loads((base/'expected-certificates.json').read_bytes())
assert len(expected_catalog)==92
assert {p.name:sha256(p.read_bytes()).hexdigest() for p in source.iterdir()}==expected_catalog
first=source.parent.parent/'attempt-1/certificates'
assert {p.name:sha256(p.read_bytes()).hexdigest() for p in first.iterdir()}==expected_catalog
dest=base/'checks'
assert not dest.exists()
dest.mkdir()
for p in sorted(root.iterdir()):
 assert p.is_file()
 name=p.name+'.txt' if p.suffix=='.stderr' else p.name
 shutil.copyfile(p,dest/name)
 assert (dest/name).read_bytes()==p.read_bytes()
audit=dict(status='passed',recorded_at=datetime.now(timezone.utc).isoformat(),stage_count=184,
           new_stages=26,retained_stages=158,positive_stages=90,hash_rejections=90,core_rejections=4,
           source_contexts=45,supplied_concrete_type_proofs=87,concrete_type_instances_pending=0,
           private_storage_domains=6,source_ownership_pending=True,application_proofs_pending=987,
           all_positive_axiom_counts_zero=True,all_positive_hashes_counts_match=True,
           current_inputs_and_binaries_verified=True,retained_raw_reports_and_stderr_exact=True,
           final_inputs_byte_identical_to_first_passing_proof_export=True,
           production_and_proof_generator_unchanged_after_first_export=True,
           test_only_repair='crates/mpk-vc/tests/support/csharp_practical_ordinary_construction_type_tests.rs',
           source_files_verified=len(manifest['source_hashes']),fixture_files_verified=len(manifest['fixture_hashes']),
           verification_sha256=sha256((root/'verification.json').read_bytes()).hexdigest(),
           full_t_gate='deferred to T06-W12')
(base/'dual-audit.json').write_text(json.dumps(audit,indent=2,sort_keys=True)+chr(10))
shutil.copyfile(Path(__file__),base/'tool-sources/audit-dual.py')
verification=json.loads((base/'verification.json').read_bytes())
verification.update(status='passed_local_targeted_and_dual_verification',
                    dual_checker_verification='passed: 184 audited stages (26 fresh, 158 exact retained)',
                    dual_checker_receipt='checks/verification.json',dual_checker_audit='dual-audit.json')
(base/'verification.json').write_text(json.dumps(verification,indent=2,sort_keys=True)+chr(10))
review=base.parent.parent/'unit-7-construction-storage-types-review.md'
s=review.read_text()
before='Dual-checker verification awaits independent final-input audit. Requested Linux\nverification remains pending an exact published-source terminal receipt.'
after='Both unchanged checkers pass the 26 affected stages. The independent terminal\naudit verifies all 184 stages, including 158 retained only after exact current\ninput/binary/raw-report/stderr comparison. All positive hashes and module/\ndeclaration/axiom counts agree with zero axioms; both checkers reject all hash\nmutations and both typed wrong proofs at core checking. Their actual inputs\nexactly match the final proof exports after the semantic test fixture repair.\nRequested Linux verification remains pending an exact published-source terminal receipt.'
assert s.count(before)==1
review.write_text(s.replace(before,after))
todo=repo/'develop/docs/08_csharp_practical_subset_design-todo.md'
s=todo.read_text()
before='173 fixture hashes. Dual-checker final-input audit and exact published-source\nLinux replay remain pending terminal receipts.'
after='173 fixture hashes. The independently audited dual-checker receipt passes all\n184 stages (26 fresh and 158 retained only after exact input/binary/raw-report/\nstderr comparison), with matching positive hashes/counts, zero axioms and both\ntyped core rejections. Exact published-source Linux replay remains pending.'
assert s.count(before)==1
todo.write_text(s.replace(before,after))
print(json.dumps(audit,sort_keys=True))
