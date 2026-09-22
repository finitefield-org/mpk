from pathlib import Path
import hashlib,json
repo=Path('/Users/kazuyoshitoshiya/mpk');r=repo/'develop/migrations/csharp-03/ordinary-foundation';e=r/'verification-logs/unit-4-count-pin-refresh'
def read(p):return json.loads(p.read_text())
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def terminal(run,path,files):
 assert run['status']=='passed'and run['exit_code']==0
 log=path.parent/run['log'];assert sha(log)==run['log_sha256'];text=log.read_text()
 for file in files:assert '--- PASS: '+run['test']+'/'+file+' 'in text
 for marker in ['Go accepted: certificate=','Rust accepted identical bytes and report:','Go rejected changed hash:','Rust rejected changed hash:',' axioms=0 ']:assert text.count(marker)==len(files),(log,marker)
 return text
batches={name:read(e/name)for name in ['consumers-batch-1-checkers.json','consumers-batch-2-checkers.json']}
assert all(d['status']in ['passed','failed']for d in batches.values()),'Both live batch supervisors must finish first.'
retry_path=e/'transient-rust-build-retries.json';retries=read(retry_path);assert retries['status']=='passed'
assert sha(r/'verification-logs/source-invariants/direct-member-reads/csharp_practical_ordinary_source_invariants.rs')==retries['fixed_source_sha256']
byte_sets={};coverage={};failed={}
for name,d in batches.items():
 assert len(d['byte_sets'])==d['distinct_byte_sequences']
 for b in d['byte_sets']:
  assert b['representative'] not in byte_sets;byte_sets[b['representative']]=b
 for run in d['runs']:
  if run['status']=='passed':
   log=terminal(run,e/name,run['selected_files'])
   for file in run['selected_files']:
    key=run['corpus']+'/'+file;b=byte_sets[key];assert 'Go accepted: certificate='+b['certificate_sha256']+' 'in log
    assert key not in coverage;coverage[key]=dict(receipt=name,log=run['log'],log_sha256=run['log_sha256'])
  else:
   assert run['status']=='failed';assert len(run['selected_files'])==1
   path=e/run['log'];assert sha(path)==run['log_sha256'];assert 'has no field named `value_id`'in path.read_text()
   key=run['corpus']+'/'+run['selected_files'][0];assert key not in failed;failed[key]=(name,run)
assert len(failed)==3 and len(retries['runs'])==3
for run in retries['runs']:
 key=run['representative'];name,original=failed.pop(key);b=byte_sets[key]
 assert run['previous_batch']==name and run['previous_log']==original['log']and run['previous_log_sha256']==original['log_sha256']
 assert run['command']==original['command']and run['certificate_sha256']==b['certificate_sha256']and run['raw_bytes_sha256']==b['raw_bytes_sha256']and run['occurrences']==b['occurrences']
 log=terminal(run,retry_path,[key.split('/')[1]]);assert 'Go accepted: certificate='+b['certificate_sha256']+' 'in log
 assert key not in coverage;coverage[key]=dict(receipt=retry_path.name,log=run['log'],log_sha256=run['log_sha256'],previous_failed_receipt=name,previous_failed_log=original['log'])
assert not failed and set(coverage)==set(byte_sets)
occurrences=set();hashes=set();rows=[]
for key,b in sorted(byte_sets.items()):
 assert b['certificate_sha256']not in hashes;hashes.add(b['certificate_sha256'])
 for file in b['occurrences']:
  assert file not in occurrences;occurrences.add(file);raw=bytes.fromhex((r/file).read_text())
  assert hashlib.sha256(raw).hexdigest()==b['raw_bytes_sha256'];assert hashlib.sha256(b'MPK-MODULE-CERT-0.1\0'+raw).hexdigest()==b['certificate_sha256']
 rows.append(dict(**b,verification=coverage[key]))
assert len(rows)==224 and len(occurrences)==274
result=dict(status='passed',scope='Terminal identical-byte Go/Rust acceptance, identical reports, zero axioms and full hash-corruption rejection for all additional count consumers. Earlier interruptions and three transient Rust compilation failures remain retained.',distinct_byte_sequences=224,changed_occurrences=274,full_retries=3,original_batch_status={k:v['status']for k,v in batches.items()},evidence={name:sha(e/name)for name in [*batches,retry_path.name]},byte_sets=rows,full_gate='deferred_to_T06_W12')
p=e/'additional-consumers/checker-verification.json';p.write_text(json.dumps(result,indent=2)+'\n')
p=e/'additional-consumers/verification.json';d=read(p);d['status']='regeneration_audit_promotion_and_checkers_passed';d['terminal_checker_verification']='checker-verification.json';d['checker_note']='All 224 current distinct byte sets (274 occurrences) passed unchanged Go/Rust acceptance and hash-corruption rejection. Three original Cargo failures are retained and covered by complete terminal reruns; original batch statuses are not rewritten.';p.write_text(json.dumps(d,indent=2)+'\n')
print(json.dumps({k:v for k,v in result.items()if k!='byte_sets'}))
