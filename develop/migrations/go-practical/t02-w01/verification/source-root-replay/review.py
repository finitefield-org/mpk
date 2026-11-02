"""Review the historical fixture correction and exactly recoverable archives."""
from pathlib import Path
import hashlib
import io
import json
import sys
import tarfile
import types
import zipfile

ROOT = Path(__file__).resolve().parent
REPO = ROOT.parents[5]
OLD = ROOT.parent/'analysis-build-list'


def seal(path):
    raw = path.read_bytes()
    return {'path':str(path.resolve()), 'bytes':len(raw), 'sha256':hashlib.sha256(raw).hexdigest(), 'mode':path.stat().st_mode & 0o777}


def main():
    scanner = REPO/'scripts/check-no-active-gir.sh'
    code = scanner.read_text().split("<<'PY'\n",1)[1].rsplit('\nPY',1)[0].rsplit('raise SystemExit(main())',1)[0]
    module = types.ModuleType('source_root_replay_review'); module.__file__ = str(scanner); sys.modules[module.__name__] = module; sys.argv = [str(scanner), str(REPO), 'strict']
    exec(compile(code,str(scanner),'exec'),module.__dict__)
    matchers = module.load_matchers(); manifest,_,_,_ = module.validate_manifest(); module.run_fixture_self_test(manifest,matchers)
    assert (len(matchers), len(module.SCOPES), len(manifest['sealed_records']), len(module.active_retained_record_exclusions(manifest))) == (127,4,39,51)
    paths = {p for p in ROOT.rglob('*') if p.is_file() and p.name != 'review.json'}
    paths.update(REPO/name for name in ['crates/mpk-cli/tests/go_practical_source_roots.rs', 'develop/docs/09_go_practical_subset_design-todo.md', 'develop/docs/go-practical-implementation-traceability-ledger.md', 'develop/migrations/go-practical/t02-w01/README.md', 'develop/migrations/go-practical/t02-w01/requirements.json', 'develop/migrations/go-practical/t02-w01/verification.json', 'develop/migrations/go-practical/t02-w01/verification/source-root-matching/README.md', 'develop/migrations/go-practical/t02-w01/verification/analysis-build-list/README.md'])
    stored = json.loads((ROOT/'archive-storage.json').read_bytes()); assert stored['status']=='passed' and stored['findings']['count']==3
    encoded_paths = {OLD/name/'inputs.tar.gz.hex' for name in ['go-checks','prelaunch']}; paths.update(encoded_paths)
    native = ROOT.parent/'analysis-namespace/native'; packet=json.loads((native/'native.json').read_bytes()); originals={r['object']['id']:r for r in packet['original_sources']}
    zip_paths={native/'original-bytes'/originals[i]['raw_file'] for i in packet['archive_source_ids']};paths.update(zip_paths)
    rows=[];findings=[];members=0;zip_members=0;restored_archives=0

    def scan(name,raw):
        nonlocal members
        findings.extend(vars(item) for item in module.detect(name,raw.decode(errors='ignore'),matchers));members+=1

    def scan_tar(payload, compressed):
        with tarfile.open(fileobj=io.BytesIO(payload),mode='r:gz' if compressed else 'r:') as archive:
            for entry in archive:
                if not entry.isfile():continue
                data=archive.extractfile(entry).read();assert len(data)==entry.size
                if entry.name.endswith('/original-go-source.tar'):
                    scan_tar(data,False)
                elif entry.name.endswith('analysis-build-list/native/inputs.tar.gz'):
                    scan(entry.name,data);scan_tar(data,True)
                else:scan(entry.name,data)

    for path in sorted(paths):
        assert path.is_file();raw=path.read_bytes();relative=str(path.relative_to(REPO));rows.append(seal(path))
        findings.extend(vars(item) for item in module.detect(relative,raw.decode(errors='ignore'),matchers))
        if path in encoded_paths:
            recovered=bytes.fromhex(raw.decode('ascii')); row=next(r for r in stored['files'] if r['encoded']['path']==str(path.resolve()))
            assert len(recovered)==row['historical_receipt']['bytes'] and hashlib.sha256(recovered).hexdigest()==row['historical_receipt']['sha256'] and row['exact_recovery']
            scan_tar(recovered,True);restored_archives+=1
        elif path.name.endswith('.tar.gz'):
            scan_tar(raw,True)
        elif path in zip_paths:
            with zipfile.ZipFile(io.BytesIO(raw)) as archive:
                for entry in archive.infolist():
                    data=archive.read(entry);assert len(data)==entry.file_size;scan(entry.filename,data);zip_members+=1
        if path.suffix=='.py':compile(raw,relative,'exec')
        if path.suffix=='.json':json.loads(raw)
    assert zip_members==1751 and restored_archives==2
    for name in ['original','detached','clippy-linux-target']:
        result=json.loads((ROOT/name/'result.json').read_bytes());assert result['status']=='passed' and result['original_inputs_unchanged']
    assert 'test result: ok. 2 passed;' in (ROOT/'detached/stdout.log').read_text()
    assert json.loads((ROOT/'format.json').read_bytes())['status']=='passed'
    restored=json.loads((ROOT/'restore-check.json').read_bytes());assert restored['status']=='passed' and restored['exact_recovery'] and len(restored['outputs'])==3
    result={'status':'passed' if not findings else 'failed','selection_reason':'Review the detached full historical fixture, exact byte recovery, changed test, all archived input members and scope records. The three known matches in the original compressed byte streams are retained in encoded form; scanner rules and exclusions are unchanged.','inputs':rows,'archive_members':members,'original_zip_members':zip_members,'exactly_recovered_archives':restored_archives,'findings':findings,'matcher_count':len(matchers),'scopes':len(module.SCOPES),'seals':len(manifest['sealed_records']),'exclusions':len(module.active_retained_record_exclusions(manifest)),'native_Linux_release_acceptance':False,'full_T_gate':'Deferred to final T02-W04: ./scripts/check-fast.sh'}
    (ROOT/'review.json').write_text(json.dumps(result,indent=2)+'\n');print(json.dumps({'status':result['status'],'inputs':len(rows),'archive_members':members,'original_zip_members':zip_members,'exactly_recovered_archives':restored_archives,'findings':len(findings)}));raise SystemExit(bool(findings))


if __name__=='__main__':main()
