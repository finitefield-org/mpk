"""Independently audit terminal dual reports and preserve exact raw receipts."""
from collections import Counter
from datetime import datetime, timezone
from hashlib import sha256
import json
from pathlib import Path
import shutil

repo=Path('/private/tmp/mpk-w09-packed-pattern-proofs')
base=repo/'develop/migrations/csharp-03/ordinary-foundation/verification-logs/concrete-allocation-proofs'
source=Path('/tmp/mpk-w09-concrete-allocation-proofs/attempt-5/certificates')
root=Path('/tmp/mpk-w09-concrete-allocation-proofs/checks')
prior=base.parent/'concrete-operation-transport/checks'
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
assert sum(len(m['proofs']) for m in metadata)==442
assert sum(len(m['pending_operations']) for m in metadata)==25
assert sum(len(m['pending_proof_ids']) for m in metadata)==987
assert all(m['proof_check_pending'] and m['application_scope_pending'] for m in metadata)
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
           source_contexts=45,supplied_concrete_operation_proofs=442,concrete_operations_pending=25,
           application_proofs_pending=987,all_positive_axiom_counts_zero=True,
           all_positive_hashes_counts_match=True,current_inputs_and_binaries_verified=True,
           retained_raw_reports_and_stderr_exact=True,source_files_verified=len(manifest['source_hashes']),
           fixture_files_verified=len(manifest['fixture_hashes']),
           verification_sha256=sha256((root/'verification.json').read_bytes()).hexdigest(),
           full_t_gate='deferred to T06-W12')
(base/'dual-audit.json').write_text(json.dumps(audit,indent=2,sort_keys=True)+'\n')
shutil.copyfile(Path(__file__),base/'tool-sources/audit-dual.py')
verification=json.loads((base/'verification.json').read_bytes())
verification.update(status='passed_local_targeted_and_dual_verification',
                    dual_checker_verification='passed: 184 audited stages (26 fresh, 158 exact retained)',
                    dual_checker_receipt='checks/verification.json',dual_checker_audit='dual-audit.json')
(base/'verification.json').write_text(json.dumps(verification,indent=2,sort_keys=True)+'\n')
review=base.parent.parent/'unit-7-concrete-allocation-proofs-review.md'
s=review.read_text()
s=s.replace('operation pins, Clippy and crate/support format also pass.','operation pins. Clippy and crate/support format also pass.')
s=s.replace('Dual checking will\nretain 158 stages only after exact input/binary/raw-report/stderr comparison and\nexecute the 26 affected positive/hash/wrong-proof stages. Dual-checker and\nrequested Linux verification remain pending terminal receipts.',
'''Both unchanged checkers pass the 26 affected positive/hash/wrong-proof stages.
The independent terminal audit verifies all 184 stages, including 158 retained
only after exact current input, binary, raw report and stderr comparison. All
45 positive certificates have matching module/declaration/axiom counts and all
three hashes, with zero axioms. Both checkers reject all hash corruptions and
the two typed wrong-normal results at core checking. Requested Linux verification
remains pending an exact published-source terminal receipt.''')
review.write_text(s)
todo=repo/'develop/docs/08_csharp_practical_subset_design-todo.md'
s=todo.read_text().replace('Dual-checker and exact published-source Linux replay remain pending terminal\nreceipts. The 25 remaining operations, six internal type instances, seven',
'''Both unchanged checkers pass the 26 affected stages; 158 unchanged stages are
retained only after exact input/binary/raw-report/stderr comparison. The terminal
184-stage audit confirms matching hashes/counts, zero axioms, all hash rejections
and both typed wrong-normal core rejections. Exact published-source Linux replay
remains pending. The 25 remaining operations, six internal type instances, seven''')
todo.write_text(s)
print(json.dumps(audit,sort_keys=True))
