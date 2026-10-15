import hashlib
import json
from pathlib import Path
b=Path(__file__).parent
h=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
results={}
for name,total in [('wrapper-program-dual-checkers',115),('control-pattern-consumer-dual-checkers',21)]:
 folder=b/name
 status=json.loads((folder/'status.json').read_text())
 reports={}
 for row in status['stages']:
  assert row['exit_code']==0 and row['verdict']=='accepted'
  p=folder/(row['stage']+'.json');q=folder/(row['case']+'.mpcert')
  assert h(p)==row['report_sha256'] and h(q)==row['input_sha256']
  assert h(folder/(row['stage']+'.stderr.txt'))==row['stderr_sha256']
  report=json.loads(p.read_text());assert report['verdict']=='accepted'
  assert report['hashes']['certificate']==hashlib.sha256(b'MPK-MODULE-CERT-0.1\0'+q.read_bytes()).hexdigest()
  if row['backend']=='rust':
   assert report['axiom_count']==0
   assert all(v==0 for v in report['axiom_report']['summary'].values())
   assert not report['axiom_report']['entries'] and not report['axiom_report']['declaration_dependencies']
  reports.setdefault(row['case'],{})[row['backend']]=report
 pairs=0
 for case,backends in reports.items():
  if set(backends)=={'rust','go'}:
   pairs+=1
   for key in ['module','declaration_count','hashes']: assert backends['rust'][key]==backends['go'][key],(case,key)
 results[name]={'reported_status':status['status'],'completed_stages_verified':len(status['stages']),
  'complete_report_pairs_independently_verified':pairs,'remaining_complete_pairs_pending':total-pairs,
  'verification':'Exact accepted report/stdout/stderr/input hashes, certificate domain hash, module/declaration/export/axiom report equality and empty Rust axiom inventory.'}
receipt={'status':'passed_completed_prefix_only_remaining_stages_pending','results':results,
 'selection_reason':'Independent read-only review of already completed actual checker reports only; no checker is rerun and pending/empty output files are not parsed or counted as passes.',
 'application_proof_ids_pending':987,'full_t01_gate':'deferred_to_final_W10','full_t06_gate':'deferred_to_W12'}
(b/'completed-checker-report-prefix-second-review.json').write_text(json.dumps(receipt,sort_keys=True,indent=2)+'\n')
print(json.dumps(receipt,indent=2))
