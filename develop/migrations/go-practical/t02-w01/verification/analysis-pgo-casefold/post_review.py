"""Recheck records updated after the complete incremental PGO review."""
from pathlib import Path
import hashlib
import json
import sys
import types

ROOT=Path(__file__).resolve().parent;REPO=ROOT.parents[5];PRIOR=ROOT.parent/'analysis-pgo'
scanner=REPO/'scripts/check-no-active-gir.sh';code=scanner.read_text().split("<<'PY'\n",1)[1].rsplit('\nPY',1)[0].rsplit('raise SystemExit(main())',1)[0]
module=types.ModuleType('go_casefold_post_review');module.__file__=str(scanner);sys.modules[module.__name__]=module;sys.argv=[str(scanner),str(REPO),'strict'];exec(compile(code,str(scanner),'exec'),module.__dict__)
matchers=module.load_matchers();manifest,_,_,_=module.validate_manifest();module.run_fixture_self_test(manifest,matchers)
assert (len(matchers),len(module.SCOPES),len(manifest['sealed_records']),len(module.active_retained_record_exclusions(manifest)))==(127,4,39,51)
review_path=ROOT/'review-final.json';review=json.loads(review_path.read_bytes());assert review['status']=='passed' and not review['active_findings']
review_stdout=json.loads((ROOT/'review-final.stdout').read_bytes());assert review_stdout['status']=='passed' and review_stdout['active_findings']==0 and (ROOT/'review-final.stderr').read_bytes()==b''
changed={ROOT/'README.md',PRIOR/'README.md',ROOT/'check_staged_whitespace.py',ROOT/'staged-whitespace.json',ROOT/'review-final.stdout',ROOT/'review-final.stderr',Path(__file__).resolve(),REPO/'develop/docs/go-practical-implementation-traceability-ledger.md',REPO/'develop/migrations/go-practical/t02-w01/README.md',REPO/'develop/migrations/go-practical/t02-w01/requirements.json',REPO/'develop/migrations/go-practical/t02-w01/verification.json'}
for row in review['inputs']:
    path=Path(row['path'])
    if path in changed:continue
    raw=path.read_bytes();assert (len(raw),hashlib.sha256(raw).hexdigest(),path.stat().st_mode & 0o777)==(row['bytes'],row['sha256'],row['mode']),str(path)
storage=json.loads((PRIOR/'archive-storage.json').read_bytes());chunks=json.loads((PRIOR/'archive-chunk-storage.json').read_bytes())
encoded_chunks={item['encoded_original']['path']:item for item in chunks['records']}
archives={item['original']['path']:item for item in storage['archives']}
initial=json.loads(bytes.fromhex(Path(storage['initial_review']['encoded']['path']).read_text()))
incremental_paths={Path(item['path']) for item in review['inputs']}
old_unchanged=0
for row in initial['inputs']:
    path=Path(row['path'])
    if path in changed or path in incremental_paths:
        continue
    if str(path) in archives:
        item=archives[str(path)];encoded=Path(item['encoded']['path'])
        if encoded.exists():data=encoded.read_bytes()
        else:
            chunked=encoded_chunks[str(encoded)];data=b''.join(Path(piece['path']).read_bytes() for piece in chunked['chunks'])
            assert len(data)==chunked['encoded_original']['bytes'] and hashlib.sha256(data).hexdigest()==chunked['encoded_original']['sha256']
        raw=bytes.fromhex(data.decode('ascii'));assert item['original']['mode']==row['mode']
    else:
        raw=path.read_bytes();assert path.stat().st_mode & 0o777==row['mode']
    assert len(raw)==row['bytes'] and hashlib.sha256(raw).hexdigest()==row['sha256'],str(path)
    old_unchanged+=1
assert old_unchanged==review['previous_review_unchanged_inputs']+review['previous_review_exactly_recovered_archives']
verification=json.loads((REPO/'develop/migrations/go-practical/t02-w01/verification.json').read_bytes());checkpoint=verification['analysis_pgo_filesystem_lookup_checkpoint']
assert checkpoint['scoped_review']['sha256']==hashlib.sha256(review_path.read_bytes()).hexdigest() and checkpoint['description']['sha256']==hashlib.sha256((ROOT/'README.md').read_bytes()).hexdigest()
assert verification['analysis_pgo_mode_checkpoint']['description']['sha256']==hashlib.sha256((PRIOR/'README.md').read_bytes()).hexdigest()
paths=changed|{Path(__file__).resolve(),review_path,Path(review['archived_findings']['encoded']['path'])};findings=[];rows=[]
for path in sorted(paths):
    raw=path.read_bytes();findings.extend(vars(item) for item in module.detect(str(path.relative_to(REPO)),raw.decode(errors='ignore'),matchers));rows.append({'path':str(path),'bytes':len(raw),'sha256':hashlib.sha256(raw).hexdigest()})
    if path.suffix=='.json':json.loads(raw)
    if path.suffix=='.py':compile(raw,str(path),'exec')
result={'status':'passed' if not findings else 'failed','selection_reason':'Recheck records updated after the complete incremental review and verify all other original and incremental review inputs remain byte-identical.','inputs':rows,'other_reviewed_inputs_unchanged':True,'prior_review_inputs_rechecked':old_unchanged,'findings':findings,'full_T_gate':'Deferred to final T02-W04: ./scripts/check-fast.sh'}
out=ROOT/'post-review.json';assert not out.exists();out.write_text(json.dumps(result,indent=2)+'\n');print(json.dumps({'status':result['status'],'files':len(paths),'findings':len(findings)}),flush=True);raise SystemExit(bool(findings))
