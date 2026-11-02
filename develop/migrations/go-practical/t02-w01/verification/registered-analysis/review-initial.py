"""Review new evidence and every retained archive without changing scan rules."""
from pathlib import Path
import hashlib
import io
import json
import sys
import tarfile
import types
import zipfile

ROOT=Path(__file__).resolve().parent;REPO=ROOT.parents[5]


def seal(path):
    h=hashlib.sha256();size=0
    with path.open('rb') as f:
        for block in iter(lambda:f.read(1024*1024),b''):h.update(block);size+=len(block)
    return {'path':str(path.resolve()),'bytes':size,'sha256':h.hexdigest(),'mode':path.stat().st_mode & 0o777}


def main():
    scanner=REPO/'scripts/check-no-active-gir.sh';code=scanner.read_text().split("<<'PY'\n",1)[1].rsplit('\nPY',1)[0].rsplit('raise SystemExit(main())',1)[0]
    module=types.ModuleType('go_registered_analysis_review');module.__file__=str(scanner);sys.modules[module.__name__]=module;sys.argv=[str(scanner),str(REPO),'strict'];exec(compile(code,str(scanner),'exec'),module.__dict__)
    matchers=module.load_matchers();manifest,_,_,_=module.validate_manifest();module.run_fixture_self_test(manifest,matchers)
    assert (len(matchers),len(module.SCOPES),len(manifest['sealed_records']),len(module.active_retained_record_exclusions(manifest)))==(127,4,39,51)
    paths={p for p in ROOT.rglob('*') if p.is_file() and p.name not in ('review.json','archived-findings.json.hex')}
    paths.update(REPO/name for name in ['go-tools/go2vir/practical_registered_analysis_test.go','crates/mpk-vc/tests/go_practical_registered_analysis.rs','crates/mpk-cli/tests/go_practical_registered_roots.rs','develop/docs/go-practical-implementation-traceability-ledger.md','develop/migrations/go-practical/t02-w01/README.md','develop/migrations/go-practical/t02-w01/requirements.json','develop/migrations/go-practical/t02-w01/verification.json'])
    active=[];archived=[];rows=[];archive_members=0;zip_members=0;elf_streams=0
    def detect(name,raw,target):target.extend(vars(item) for item in module.detect(name,raw.decode(errors='ignore'),matchers))
    def zip_data(name,raw):
        nonlocal zip_members
        if not raw.startswith((b'PK\x03\x04',b'PK\x05\x06')) or not zipfile.is_zipfile(io.BytesIO(raw)):return
        with zipfile.ZipFile(io.BytesIO(raw)) as z:
            for member in z.infolist():
                data=z.read(member);assert len(data)==member.file_size;detect(name+'!/'+member.filename,data,archived);zip_members+=1
    def tar_data(name,raw):
        nonlocal archive_members
        with tarfile.open(fileobj=io.BytesIO(raw),mode='r:*') as tar:
            for member in tar:
                if not member.isfile():continue
                data=tar.extractfile(member).read();assert len(data)==member.size;inside=name+'!/'+member.name;detect(inside,data,archived);archive_members+=1;zip_data(inside,data)
                if member.name.endswith('.tar') or member.name.endswith('.tar.gz'):tar_data(inside,data)
    for p in sorted(paths):
        raw=p.read_bytes();name=str(p.relative_to(REPO));rows.append(seal(p));detect(name,raw,active)
        if p.name.endswith('.tar.gz') or p.name.endswith('.tar'):tar_data(name,raw)
        elif p.name.endswith('.elf.hex'):
            data=bytes.fromhex(raw.decode('ascii'));assert len(data)==9004378;detect(name+'!decoded-elf',data,archived);elf_streams+=1
        else:zip_data(name,raw)
        if p.suffix=='.py':compile(raw,name,'exec')
        if p.suffix=='.json':json.loads(raw)
    for name in ['native-fixed','native-after-path','rebuild-original','roots-original','rust-original','rust-clippy-original','cli-clippy-original','go-vet-original','go-vet-after-path']:
        assert json.loads((ROOT/name/'result.json').read_bytes())['status']=='passed',name
    assert json.loads((ROOT/'native-original/result.json').read_bytes())['status']=='failed'
    for name in ['formats.json','formats-after-path.json','inspection.json','native-epochs-equal.json']:
        assert json.loads((ROOT/name).read_bytes())['status']=='passed',name
    assert elf_streams==2
    findings=(json.dumps(archived,indent=2)+'\n').encode();encoded=ROOT/'archived-findings.json.hex';assert not encoded.exists();encoded.write_bytes(findings.hex().encode());assert bytes.fromhex(encoded.read_text())==findings
    result={'status':'passed' if not active else 'failed','selection_reason':'Review every changed source, complete retained original inputs, nested source archives, ZIP members and exact decoded historical binaries. Keep any binary/archive findings separately with exact recovery; change no scanner rules or exclusions.','inputs':rows,'active_findings':active,'archived_findings':{'count':len(archived),'raw_bytes':len(findings),'raw_sha256':hashlib.sha256(findings).hexdigest(),'encoded':seal(encoded)},'archive_members':archive_members,'zip_members':zip_members,'decoded_complete_binaries':elf_streams,'matcher_count':len(matchers),'scopes':len(module.SCOPES),'seals':len(manifest['sealed_records']),'exclusions':len(module.active_retained_record_exclusions(manifest)),'native_Linux_release_acceptance':False,'full_T_gate':'Deferred to final T02-W04: ./scripts/check-fast.sh'}
    (ROOT/'review.json').write_text(json.dumps(result,indent=2)+'\n');print(json.dumps({'status':result['status'],'files':len(rows),'archive_members':archive_members,'zip_members':zip_members,'decoded_complete_binaries':elf_streams,'active_findings':len(active),'archived_findings':len(archived)}),flush=True);raise SystemExit(bool(active))


if __name__=='__main__':main()
