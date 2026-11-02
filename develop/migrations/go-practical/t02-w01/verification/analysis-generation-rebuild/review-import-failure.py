"""Review the same-recipe comparison and both complete original source epochs."""
from pathlib import Path
import hashlib
import io
import json
import sys
import tarfile
import types
import zipfile

ROOT=Path(__file__).resolve().parent;REPO=ROOT.parents[5];VERIFY=ROOT.parent


def seal(path):
    h=hashlib.sha256();size=0
    with path.open('rb') as f:
        for block in iter(lambda:f.read(1024*1024),b''):h.update(block);size+=len(block)
    return {'path':str(path.resolve()),'bytes':size,'sha256':h.hexdigest(),'mode':path.stat().st_mode & 0o777}


def main():
    scanner=REPO/'scripts/check-no-active-gir.sh';code=scanner.read_text().split("<<'PY'\n",1)[1].rsplit('\nPY',1)[0].rsplit('raise SystemExit(main())',1)[0]
    module=types.ModuleType('go_same_recipe_review');module.__file__=str(scanner);sys.modules[module.__name__]=module;sys.argv=[str(scanner),str(REPO),'strict'];exec(compile(code,str(scanner),'exec'),module.__dict__)
    matchers=module.load_matchers();manifest,_,_,_=module.validate_manifest();module.run_fixture_self_test(manifest,matchers)
    assert (len(matchers),len(module.SCOPES),len(manifest['sealed_records']),len(module.active_retained_record_exclusions(manifest)))==(127,4,39,51)
    paths={p for p in ROOT.rglob('*') if p.is_file() and p.name not in ('review.json','archived-findings.json.hex')}
    paths.update(REPO/name for name in ['develop/docs/go-practical-implementation-traceability-ledger.md','develop/migrations/go-practical/t02-w01/README.md','develop/migrations/go-practical/t02-w01/requirements.json','develop/migrations/go-practical/t02-w01/verification.json'])
    active=[];archived=[];rows=[];archive_members=0;zip_members=0;source_objects=0;decoded_binaries=0
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
    for p in sorted(paths):
        raw=p.read_bytes();name=str(p.relative_to(REPO));rows.append(seal(p));detect(name,raw,active)
        if p.name.endswith('.elf.hex'):
            data=bytes.fromhex(raw.decode('ascii'));assert len(data)==9004642;detect(name+'!decoded-elf',data,archived);decoded_binaries+=1
        if p.suffix=='.py':compile(raw,name,'exec')
        if p.suffix=='.json':json.loads(raw)
    for label,directory in [('registered',VERIFY/'registered-analysis/native-fixed'),('later',VERIFY/'analysis-namespace/native')]:
        packet=json.loads((directory/'native.json').read_bytes())
        for row in packet['original_sources']:
            path=directory/'original-bytes'/row['raw_file'];raw=path.read_bytes();obj=row['object'];assert len(raw)==obj['bytes'] and hashlib.sha256(raw).hexdigest()==obj['raw_sha256'];detect(label+'/'+row['raw_file'],raw,archived);zip_data(label+'/'+row['raw_file'],raw);source_objects+=1
    for label,path in [('registered',VERIFY/'installed-compiler/original-go-source.tar'),('later',VERIFY/'analysis-build-list/native/inputs.tar.gz')]:tar_data(label, path.read_bytes())
    assert source_objects==409 and decoded_binaries==2
    for name in ['same-flags-original/result.json','inspection.json','inspection-presence.json']:assert json.loads((ROOT/name).read_bytes())['status']=='passed'
    assert json.loads((ROOT/'inspect-output-exists.json').read_bytes())['status']=='failed'
    findings=(json.dumps(archived,indent=2)+'\n').encode();encoded=ROOT/'archived-findings.json.hex';assert not encoded.exists();encoded.write_bytes(findings.hex().encode());assert bytes.fromhex(encoded.read_text())==findings
    result={'status':'passed' if not active else 'failed','selection_reason':'Review changed comparison scripts/records, both exact original source captures and their complete source archives, nested ZIP members, and both decoded binary streams. Historical findings are retained exactly; scanner rules and exclusions do not change.','inputs':rows,'active_findings':active,'archived_findings':{'count':len(archived),'raw_bytes':len(findings),'raw_sha256':hashlib.sha256(findings).hexdigest(),'encoded':seal(encoded)},'original_source_objects':source_objects,'archive_members':archive_members,'zip_members':zip_members,'decoded_complete_binaries':decoded_binaries,'matcher_count':len(matchers),'scopes':len(module.SCOPES),'seals':len(manifest['sealed_records']),'exclusions':len(module.active_retained_record_exclusions(manifest)),'native_Linux_release_acceptance':False,'full_T_gate':'Deferred to final T02-W04: ./scripts/check-fast.sh'}
    (ROOT/'review.json').write_text(json.dumps(result,indent=2)+'\n');print(json.dumps({'status':result['status'],'files':len(rows),'original_source_objects':source_objects,'archive_members':archive_members,'zip_members':zip_members,'decoded_complete_binaries':decoded_binaries,'active_findings':len(active),'archived_findings':len(archived)}),flush=True);raise SystemExit(bool(active))


if __name__=='__main__':main()
