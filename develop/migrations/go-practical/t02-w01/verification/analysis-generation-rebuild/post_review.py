"""Recheck only records updated after the complete scoped review."""
from pathlib import Path
import hashlib
import json
import sys
import types

ROOT=Path(__file__).resolve().parent;REPO=ROOT.parents[5]
scanner=REPO/'scripts/check-no-active-gir.sh';code=scanner.read_text().split("<<'PY'\n",1)[1].rsplit('\nPY',1)[0].rsplit('raise SystemExit(main())',1)[0]
module=types.ModuleType('go_generation_post_review');module.__file__=str(scanner);sys.modules[module.__name__]=module;sys.argv=[str(scanner),str(REPO),'strict'];exec(compile(code,str(scanner),'exec'),module.__dict__)
matchers=module.load_matchers();manifest,_,_,_=module.validate_manifest();module.run_fixture_self_test(manifest,matchers)
assert (len(matchers),len(module.SCOPES),len(manifest['sealed_records']),len(module.active_retained_record_exclusions(manifest)))==(127,4,39,51)
review_path=ROOT/'review.json';review=json.loads(review_path.read_bytes());assert review['status']=='passed' and not review['active_findings'] and review['archived_findings']['count']==2
changed={ROOT/'README.md',REPO/'develop/docs/go-practical-implementation-traceability-ledger.md',REPO/'develop/migrations/go-practical/t02-w01/verification.json'}
for row in review['inputs']:
 path=Path(row['path'])
 if path in changed:continue
 raw=path.read_bytes();assert (len(raw),hashlib.sha256(raw).hexdigest(),path.stat().st_mode & 0o777)==(row['bytes'],row['sha256'],row['mode']),str(path)
assert len(json.loads(bytes.fromhex((ROOT/'archived-findings.json.hex').read_text())))==2
verification=json.loads((REPO/'develop/migrations/go-practical/t02-w01/verification.json').read_bytes());checkpoint=verification['analysis_generations_same_recipe_checkpoint'];assert checkpoint['scoped_review']['sha256']==hashlib.sha256(review_path.read_bytes()).hexdigest() and checkpoint['description']['sha256']==hashlib.sha256((ROOT/'README.md').read_bytes()).hexdigest()
paths=changed|{Path(__file__).resolve(),review_path,ROOT/'archived-findings.json.hex'};findings=[];rows=[]
for path in sorted(paths):
 raw=path.read_bytes();findings.extend(vars(item) for item in module.detect(str(path.relative_to(REPO)),raw.decode(errors='ignore'),matchers));rows.append({'path':str(path),'bytes':len(raw),'sha256':hashlib.sha256(raw).hexdigest()})
 if path.suffix=='.json':json.loads(raw)
 if path.suffix=='.py':compile(raw,str(path),'exec')
result={'status':'passed' if not findings else 'failed','selection_reason':'Recheck the three records updated after the complete generation review and verify every other reviewed input remains byte-identical.','inputs':rows,'other_reviewed_inputs_unchanged':True,'findings':findings,'full_T_gate':'Deferred to final T02-W04: ./scripts/check-fast.sh'}
out=ROOT/'post-review.json';assert not out.exists();out.write_text(json.dumps(result,indent=2)+'\n');print(json.dumps({'status':result['status'],'files':len(paths),'findings':len(findings)}));raise SystemExit(bool(findings))
