from pathlib import Path
import hashlib,json,os,subprocess,time
repo=Path('/Users/kazuyoshitoshiya/mpk');e=repo/'develop/migrations/csharp-03/ordinary-foundation/verification-logs/binding-defaults';p=e/'quality-inventory-fix.json';assert not p.exists();build=json.loads((e/'build.json').read_text());env=os.environ.copy();env.update(build['profile'])
def sha(p):return hashlib.sha256(Path(p).read_bytes()).hexdigest()
changes={name:sha(repo/name)for name,h in build['source_sha256'].items()if sha(repo/name)!=h};assert list(changes)==['crates/mpk-vc/tests/csharp_practical_inventory.rs'],changes
d=dict(status='running',supervisor_pid=os.getpid(),selection_reason='Initial Clippy passed. Inventory found the new default test as one additional Std namespace consumer; preserve the exact previous 143-path fingerprint, add only that test, update 144 paths and total 4967, rerun all five inventory tests and pending format. No other Rust source, certificate or semantic oracle changed.',source_changes_since_build=changes,metadata_hashes={name:sha(repo/name)for name in ['develop/migrations/csharp-03/artifact-consumer-inventory.json','develop/migrations/csharp-03/data-phase/historical-inventory-cache-correction.json']},runs=[],full_gate='deferred_to_T06_W12')
def save():p.write_text(json.dumps(d,indent=2)+'\n')
def run(label,cmd,expected=()):
 log=e/(label+'.log');row=dict(label=label,command=cmd,status='running',log=log.name);d['runs'].append(row);save();start=time.monotonic()
 with log.open('wb')as f:
  child=subprocess.Popen(cmd,cwd=repo,env=env,stdin=subprocess.DEVNULL,stdout=f,stderr=subprocess.STDOUT);row['pid']=child.pid;save();code=child.wait()
 text=log.read_text();passed=code==0 and all(s in text for s in expected);row.update(status='passed'if passed else'failed',exit_code=code,elapsed_seconds=time.monotonic()-start,log_sha256=sha(log));save()
 if not passed:raise RuntimeError(label+' failed')
save()
try:
 run('inventory-fixed',['cargo','test','-p','mpk-vc','--test','csharp_practical_inventory'],['test result: ok. 5 passed; 0 failed;'])
 run('format-fixed',['cargo','fmt','--all','--','--check'])
 d['status']='passed'
except Exception as ex:d.update(status='failed',failure=str(ex))
save()
