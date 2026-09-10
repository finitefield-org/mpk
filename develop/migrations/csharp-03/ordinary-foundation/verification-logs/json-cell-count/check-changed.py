from pathlib import Path
import subprocess,os,json,hashlib,sys,time
repo=Path('/Users/kazuyoshitoshiya/mpk')
base=repo/'develop/migrations/csharp-03/ordinary-foundation'
plan=json.loads((base/'unit-4-json-cell-count-replacements.json').read_text())
result_path=base/'verification-logs/json-cell-count/changed-checker-results.json'
results=json.loads(result_path.read_text()) if result_path.exists() else []
def certificate_hash(path):
 return hashlib.sha256(b'MPK-MODULE-CERT-0.1\0'+bytes.fromhex(path.read_text())).hexdigest()
for group in plan['checker_groups']:
 family=group['family']
 expected={r['file']:r['new_sha256'] for r in plan['rows'] if r['family']==family and r['file'] in group['files']}
 actual={name:certificate_hash(base/family/name) for name in group['files']}
 assert actual==expected,('candidate changed',family)
 if any(r['family']==family and r['exit_code']==0 and r['candidates']==actual for r in results):continue
 log=Path('/tmp/mpk-w09-json-cell-count-checkers-'+family+'-v1.log')
 args=['go','test','-timeout','0','-tags','checkeragreement','-run',group['run_filter'],'-count=1','-v']
 print(f"Starting {family}: {len(actual)} changed candidates",flush=True)
 start=time.time()
 with log.open('wb') as out:
  result=subprocess.run(args,cwd=repo/'go-tools/mpk-checker-ref',env={**os.environ,'GOCACHE':'/tmp/mpk-w09-go-cache'},stdout=out,stderr=subprocess.STDOUT)
 data=log.read_bytes();retained=base/'verification-logs/json-cell-count'/log.name;retained.write_bytes(data)
 results.append(dict(family=family,candidates=actual,command=args,exit_code=result.returncode,seconds=round(time.time()-start,2),log=str(log),retained_log=str(retained.relative_to(repo)),log_sha256=hashlib.sha256(data).hexdigest()))
 temp=result_path.with_suffix('.tmp');temp.write_text(json.dumps(results,indent=2)+'\n');temp.replace(result_path)
 print(f"Finished {family}: exit{result.returncode}",flush=True)
 if result.returncode:sys.exit(result.returncode)
