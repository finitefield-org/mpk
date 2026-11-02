"""Review the additional staged-whitespace receipt after the scoped review."""
from pathlib import Path
import hashlib
import json
import sys
import types

ROOT=Path(__file__).resolve().parent;REPO=ROOT.parents[5]
scanner=REPO/'scripts/check-no-active-gir.sh';code=scanner.read_text().split("<<'PY'\n",1)[1].rsplit('\nPY',1)[0].rsplit('raise SystemExit(main())',1)[0]
module=types.ModuleType('analysis_roots_whitespace_review');module.__file__=str(scanner);sys.modules[module.__name__]=module;sys.argv=[str(scanner),str(REPO),'strict'];exec(compile(code,str(scanner),'exec'),module.__dict__)
matchers=module.load_matchers();manifest,_,_,_=module.validate_manifest();module.run_fixture_self_test(manifest,matchers)
assert (len(matchers),len(module.SCOPES),len(manifest['sealed_records']),len(module.active_retained_record_exclusions(manifest)))==(127,4,39,51)
assert json.loads((ROOT/'post-review.json').read_bytes())['status']=='passed'
paths=[Path(__file__).resolve(),ROOT/'check_staged_whitespace.py',ROOT/'staged-whitespace.json'];rows=[];findings=[]
for path in paths:
 raw=path.read_bytes();findings.extend(vars(item) for item in module.detect(str(path.relative_to(REPO)),raw.decode(errors='ignore'),matchers));rows.append({'path':str(path),'bytes':len(raw),'sha256':hashlib.sha256(raw).hexdigest()})
 if path.suffix=='.py':compile(raw,str(path),'exec')
 if path.suffix=='.json':assert json.loads(raw)['status']=='passed'
result={'status':'passed' if not findings else 'failed','selection_reason':'Review only the staged-whitespace record added after the complete scoped review; retain exact original test stdout bytes.','inputs':rows,'findings':findings,'full_T_gate':'Deferred to final T02-W04: ./scripts/check-fast.sh'}
out=ROOT/'staged-whitespace-review.json';assert not out.exists();out.write_text(json.dumps(result,indent=2)+'\n');print(json.dumps({'status':result['status'],'findings':len(findings)}));raise SystemExit(bool(findings))
