"""Extend the complete prior scoped review to changed and added inputs."""
from pathlib import Path
import hashlib
import io
import json
import sys
import tarfile
import types
import zipfile

ROOT=Path(__file__).resolve().parent;REPO=ROOT.parents[5];PRIOR=ROOT.parent/'analysis-pgo'


def seal(path):
    raw=path.read_bytes()
    return {'path':str(path.resolve()),'bytes':len(raw),'sha256':hashlib.sha256(raw).hexdigest(),'mode':path.stat().st_mode & 0o777}


def main():
    epoch=sys.argv[1];assert epoch.replace('-','').isalnum()
    out=ROOT/(epoch+'.json');findings_out=ROOT/(epoch+'-archived-findings.json.hex');assert not out.exists() and not findings_out.exists()
    scanner=REPO/'scripts/check-no-active-gir.sh';code=scanner.read_text().split("<<'PY'\n",1)[1].rsplit('\nPY',1)[0].rsplit('raise SystemExit(main())',1)[0]
    module=types.ModuleType('go_casefold_scoped_review');module.__file__=str(scanner);sys.modules[module.__name__]=module;sys.argv=[str(scanner),str(REPO),'strict'];exec(compile(code,str(scanner),'exec'),module.__dict__)
    matchers=module.load_matchers();manifest,_,_,_=module.validate_manifest();module.run_fixture_self_test(manifest,matchers)
    assert (len(matchers),len(module.SCOPES),len(manifest['sealed_records']),len(module.active_retained_record_exclusions(manifest)))==(127,4,39,51)
    storage=json.loads((PRIOR/'archive-storage.json').read_bytes());restored=json.loads((PRIOR/'archive-restore-check.json').read_bytes());assert storage['status']=='passed' and restored['status']=='passed' and not storage['scanner_rules_changed']
    chunks=json.loads((PRIOR/'archive-chunk-storage.json').read_bytes());chunk_restore=json.loads((PRIOR/'archive-chunk-restore-check.json').read_bytes());assert chunks['status']=='passed' and chunk_restore['status']=='passed' and not chunks['scanner_rules_changed']
    preflight=json.loads((ROOT/'archive-preflight-storage.json').read_bytes());preflight_restore=json.loads((ROOT/'archive-restore-check.json').read_bytes());assert preflight['status']=='passed' and preflight_restore['status']=='passed' and preflight_restore['restored_archives']==len(preflight['archives']) and not preflight['scanner_rules_changed']
    chunked={item['encoded_original']['path']:item for item in chunks['records']}
    def encoded_bytes(path):
        if path.exists():return path.read_bytes()
        item=chunked[str(path)];parts=[]
        for row in item['chunks']:
            part=Path(row['path']);raw=part.read_bytes()
            assert len(raw)==row['bytes'] and hashlib.sha256(raw).hexdigest()==row['sha256'] and part.stat().st_mode & 0o777==row['mode']
            parts.append(raw)
        raw=b''.join(parts);assert len(raw)==item['encoded_original']['bytes'] and hashlib.sha256(raw).hexdigest()==item['encoded_original']['sha256']
        return raw
    original_review=bytes.fromhex(Path(storage['initial_review']['encoded']['path']).read_text());assert len(original_review)==storage['initial_review']['raw_bytes'] and hashlib.sha256(original_review).hexdigest()==storage['initial_review']['raw_sha256']
    old=json.loads(original_review);assert old['status']=='failed' and len(old['active_findings'])==11 and old['archived_findings']['count']==4
    by_original={item['original']['path']:item for item in storage['archives']}
    assert {str((REPO/row['path']).resolve()) for row in old['active_findings']}==set(by_original)
    restored_paths=0;unchanged=0;changed=set()
    for row in old['inputs']:
        path=Path(row['path'])
        if str(path) in by_original:
            item=by_original[str(path)];raw=bytes.fromhex(encoded_bytes(Path(item['encoded']['path'])).decode('ascii'))
            assert len(raw)==row['bytes'] and hashlib.sha256(raw).hexdigest()==row['sha256'] and item['original']['mode']==row['mode']
            restored_paths+=1
        elif not path.exists():
            raise AssertionError(f'previously reviewed input is missing: {path}')
        elif seal(path)==row:
            unchanged+=1
        else:
            changed.add(path)
    assert restored_paths==7
    old_paths={Path(row['path']) for row in old['inputs']}
    paths={p for base in [ROOT,PRIOR] for p in base.rglob('*') if p.is_file()}
    paths.update(REPO/name for name in ['go-tools/go2vir/practical_analysis.go','go-tools/go2vir/practical_analysis_pgo_test.go','crates/mpk-vc/src/go_practical_type_keys.rs','crates/mpk-vc/src/go_practical_type_keys/analysis_vendor.rs','crates/mpk-vc/src/go_practical_type_keys/analysis_vendor/source.rs','crates/mpk-vc/tests/go_practical_analysis_pgo.rs','crates/mpk-vc/tests/go_practical_analysis_casefold.rs','crates/mpk-cli/src/go_practical_source_roots.rs','crates/mpk-cli/src/go_practical_analysis_root_binding.rs','crates/mpk-cli/tests/go_practical_analysis_root_pgo.rs','crates/mpk-cli/tests/go_practical_analysis_casefold.rs','develop/docs/go-practical-implementation-traceability-ledger.md','develop/migrations/go-practical/t02-w01/README.md','develop/migrations/go-practical/t02-w01/requirements.json','develop/migrations/go-practical/t02-w01/verification.json'])
    paths={p for p in paths if p not in old_paths or p in changed}
    active=[];archived=[];rows=[];counts={'archive_members':0,'zip_members':0,'decoded_binaries':0,'preflight_recovered_archives':0}
    def detect(name,raw,target):target.extend(vars(item) for item in module.detect(name,raw.decode(errors='ignore'),matchers))
    def zip_data(name,raw):
        if not raw.startswith((b'PK\x03\x04',b'PK\x05\x06')) or not zipfile.is_zipfile(io.BytesIO(raw)):return
        with zipfile.ZipFile(io.BytesIO(raw)) as archive:
            for member in archive.infolist():
                data=archive.read(member);assert len(data)==member.file_size;detect(name+'!/'+member.filename,data,archived);counts['zip_members']+=1
    def tar_data(name,raw):
        with tarfile.open(fileobj=io.BytesIO(raw),mode='r:*') as archive:
            for member in archive:
                if not member.isfile():continue
                data=archive.extractfile(member).read();assert len(data)==member.size;inside=name+'!/'+member.name;detect(inside,data,archived);counts['archive_members']+=1;zip_data(inside,data)
                if member.name.endswith(('.tar','.tar.gz')):tar_data(inside,data)
                elif member.name.endswith('.elf.hex'):
                    detect(inside+'!decoded-binary',bytes.fromhex(data.decode('ascii')),archived);counts['decoded_binaries']+=1
    for path in sorted(paths):
        raw=path.read_bytes();name=str(path.relative_to(REPO));rows.append(seal(path));detect(name,raw,active)
        if path.name.endswith(('.tar','.tar.gz')):tar_data(name,raw)
        elif path.name.endswith('.elf.hex'):
            detect(name+'!decoded-binary',bytes.fromhex(raw.decode('ascii')),archived);counts['decoded_binaries']+=1
        elif not path.name.endswith('.tar.gz.hex'):zip_data(name,raw)
        if path.suffix=='.py':compile(raw,name,'exec')
        if path.suffix=='.json':json.loads(raw)
    initial=preflight['initial_report'];report=bytes.fromhex(Path(initial['path']).read_text())
    assert len(report)==initial['bytes'] and hashlib.sha256(report).hexdigest()==initial['sha256']
    for item in preflight['archives']:
        parts=[]
        for row in item['chunks']:
            path=Path(row['path']);raw=path.read_bytes()
            assert len(raw)==row['bytes'] and hashlib.sha256(raw).hexdigest()==row['sha256'] and path.stat().st_mode & 0o777==row['mode'];parts.append(raw)
        raw=bytes.fromhex(b''.join(parts).decode('ascii'));original=item['original']
        assert len(raw)==original['bytes'] and hashlib.sha256(raw).hexdigest()==original['sha256']
        tar_data(str(Path(original['path']).relative_to(REPO))+'!decoded-archive',raw);counts['preflight_recovered_archives']+=1
    for directory in ['folded-final','sensitive-final','crossed-final','folded-hardlink-final','sensitive-hardlink-final','crossed-hardlink-final','hardlink-after-fix','rust-final','rust-clippy-final','cli-crossed','clippy-crossed','format-crossed']:
        assert json.loads((ROOT/directory/'result.json').read_bytes())['status']=='passed',directory
    for directory in ['native-hardlink-final','go-regression-hardlink-final','go-format-hardlink-final']:
        assert json.loads((PRIOR/directory/'result.json').read_bytes())['status']=='passed',directory
    assert json.loads((ROOT/'hardlink-before-fix/result.json').read_bytes())['status']=='failed'
    assert json.loads((ROOT/'hardlink-inspection.json').read_bytes())['status']=='passed'
    historical=json.loads(bytes.fromhex(Path(old['archived_findings']['encoded']['path']).read_text()))
    assert len(historical)==4
    complete=historical+archived
    data=(json.dumps(complete,indent=2)+'\n').encode();findings_out.write_bytes(data.hex().encode());assert bytes.fromhex(findings_out.read_text())==data
    result={'status':'passed' if not active else 'failed','selection_reason':'Verify every input of the prior complete review, restore all seven original compressed streams exactly, and scan every new or changed source/evidence file, archive member and nested ZIP using unchanged rules.','previous_review':storage['initial_review'],'previous_review_unchanged_inputs':unchanged,'previous_review_changed_inputs':len(changed),'previous_review_exactly_recovered_archives':restored_paths,'previous_archive_members':old['archive_members'],'previous_zip_members':old['zip_members'],'previous_decoded_binaries':old['decoded_binaries'],'inputs':rows,'active_findings':active,'archived_findings':{'count':len(complete),'raw_bytes':len(data),'raw_sha256':hashlib.sha256(data).hexdigest(),'encoded':seal(findings_out)},**counts,'matcher_count':len(matchers),'scopes':len(module.SCOPES),'seals':len(manifest['sealed_records']),'exclusions':len(module.active_retained_record_exclusions(manifest)),'scanner_rules_changed':False,'native_Linux_release_acceptance':False,'full_T_gate':'Deferred to final T02-W04: ./scripts/check-fast.sh'}
    out.write_text(json.dumps(result,indent=2)+'\n');print(json.dumps({'status':result['status'],'previous_unchanged':unchanged,'recovered_archives':restored_paths,'new_or_changed_files':len(rows),**counts,'active_findings':len(active),'archived_findings':len(complete)}),flush=True);raise SystemExit(bool(active))

if __name__=='__main__':main()
