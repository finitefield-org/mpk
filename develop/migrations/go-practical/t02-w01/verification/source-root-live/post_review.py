"""Review only the records written after the complete original-data review."""
from pathlib import Path
import hashlib
import json
import sys
import types

ROOT=Path(__file__).resolve().parent
REPO=ROOT.parents[5]
scanner=REPO/'scripts/check-no-active-gir.sh';code=scanner.read_text().split("<<'PY'\n",1)[1].rsplit('\nPY',1)[0].rsplit('raise SystemExit(main())',1)[0]
module=types.ModuleType('go_live_post_review');module.__file__=str(scanner);sys.modules[module.__name__]=module;sys.argv=[str(scanner),str(REPO),'strict'];exec(compile(code,str(scanner),'exec'),module.__dict__)
matchers=module.load_matchers();review=json.loads((ROOT/'review.json').read_bytes());assert review['status']=='passed' and len(review['active_findings'])==0 and review['archived_findings']['count']==0
changed={ROOT/'README.md',REPO/'develop/docs/go-practical-implementation-traceability-ledger.md',REPO/'develop/migrations/go-practical/t02-w01/verification.json'}
for row in review['inputs']:
 path=Path(row['path'])
 if path in changed:continue
 digest=hashlib.sha256();size=0
 with path.open('rb') as f:
  for block in iter(lambda:f.read(1024*1024),b''):digest.update(block);size+=len(block)
 assert (size,digest.hexdigest(),path.stat().st_mode & 0o777)==(row['bytes'],row['sha256'],row['mode']),str(path)
paths=changed|{Path(__file__).resolve(),ROOT/'review.json',ROOT/'review-final/result.json',ROOT/'review-final/stdout.log',ROOT/'review-final/stderr.log',ROOT/'live-original/archived-findings.json.hex'}
findings=[];rows=[]
for p in sorted(paths):
 raw=p.read_bytes();relative=str(p.relative_to(REPO));findings.extend(vars(item) for item in module.detect(relative,raw.decode(errors='ignore'),matchers));rows.append({'path':str(p.resolve()),'bytes':len(raw),'sha256':hashlib.sha256(raw).hexdigest()})
 if p.suffix=='.json':json.loads(raw)
 if p.suffix=='.py':compile(raw,relative,'exec')
transcript=json.loads((ROOT/'review-final/result.json').read_bytes());assert transcript['status']=='passed' and transcript['review_sha256']==hashlib.sha256((ROOT/'review.json').read_bytes()).hexdigest()
assert json.loads(bytes.fromhex((ROOT/'live-original/archived-findings.json.hex').read_text()))==[]
result={'status':'passed' if not findings else 'failed','selection_reason':'Recheck only the three scope records and review output written after the full 16880-file review; verify every other reviewed input remains byte-identical.','inputs':rows,'earlier_reviewed_inputs_unchanged':True,'findings':findings,'full_T_gate':'Deferred to final T02-W04: ./scripts/check-fast.sh'}
out=ROOT/'post-review.json';assert not out.exists();out.write_text(json.dumps(result,indent=2)+'\n');print(json.dumps({'status':result['status'],'changed_and_new_files':len(paths),'findings':len(findings)}),flush=True);raise SystemExit(bool(findings))
