from pathlib import Path
import concurrent.futures,hashlib,json,os,subprocess,threading,time
repo=Path('/Users/kazuyoshitoshiya/mpk');root=repo/'develop/migrations/csharp-03/ordinary-foundation';e=root/'verification-logs/unit-4-count-pin-refresh';p=e/'transient-rust-build-retries.json';lock=threading.Lock()
def sha(p):return hashlib.sha256(Path(p).read_bytes()).hexdigest()
r=dict(status='running',supervisor_pid=os.getpid(),cause='Three pre-existing corpus checks invoked cargo run while a new source-invariant field was temporarily named value_id instead of TypedValueRef.id. Library cargo check passed after the correction. Original failed logs are retained. Rerun full original Go/Rust acceptance, identical report comparison and corruption rejection for each failed byte set.',full_gate='deferred_to_T06_W12',runs=[])
for batch in ['consumers-batch-1','consumers-batch-2']:
 d=json.loads((e/(batch+'-checkers.json')).read_text())
 for previous in d['runs']:
  if previous['status']!='failed':continue
  log=e/previous['log'];assert sha(log)==previous['log_sha256'];assert 'has no field named `value_id`' in log.read_text()
  name=previous['corpus']+'/'+previous['selected_files'][0];row=next(x for x in d['byte_sets']if x['representative']==name);r['runs'].append(dict(previous_batch=batch+'-checkers.json',previous_log=previous['log'],previous_log_sha256=previous['log_sha256'],command=previous['command'],test=previous['test'],representative=name,occurrences=row['occurrences'],raw_bytes_sha256=row['raw_bytes_sha256'],certificate_sha256=row['certificate_sha256'],status='pending'))
assert len(r['runs'])==3
r['fixed_source_sha256']=sha(repo/'crates/mpk-vc/src/csharp_practical_ordinary_source_invariants.rs')
def save():
 q=p.with_suffix('.pending');q.write_text(json.dumps(r,indent=2)+'\n');os.replace(q,p)
def check(row):
 for file in row['occurrences']:
  raw=bytes.fromhex((root/file).read_text());assert hashlib.sha256(raw).hexdigest()==row['raw_bytes_sha256'];assert hashlib.sha256(b'MPK-MODULE-CERT-0.1\0'+raw).hexdigest()==row['certificate_sha256']
def one(row):
 check(row);log=e/('build-retry-'+row['representative'].replace('/','-')+'.log');env=os.environ.copy();env['GOCACHE']='/tmp/mpk-w09-unit4-go-cache';start=time.monotonic()
 with log.open('wb') as f:
  child=subprocess.Popen(row['command'],cwd=repo/'go-tools/mpk-checker-ref',env=env,stdin=subprocess.DEVNULL,stdout=f,stderr=subprocess.STDOUT)
  with lock:row.update(status='running',pid=child.pid,log=log.name);save()
  code=child.wait()
 passed=code==0 and '--- PASS: '+row['test']+'/'+row['representative'].split('/')[1]+' ' in log.read_text();check(row)
 with lock:row.update(status='passed'if passed else 'failed',exit_code=code,elapsed_seconds=round(time.monotonic()-start,3),log_sha256=sha(log));save()
 return passed
save()
with concurrent.futures.ThreadPoolExecutor(max_workers=2)as pool:results=list(pool.map(one,r['runs']))
r['status']='passed'if all(results)else'failed';save()
