"""Review active inputs and every exactly retained original evidence file."""
from pathlib import Path
import hashlib
import io
import json
import sys
import tarfile
import types
import zipfile

ROOT=Path(__file__).resolve().parent
REPO=ROOT.parents[5]


def seal(path):
    digest=hashlib.sha256();size=0
    with path.open('rb') as f:
        for block in iter(lambda:f.read(1024*1024),b''):digest.update(block);size+=len(block)
    return {'path':str(path.resolve()),'bytes':size,'sha256':digest.hexdigest(),'mode':path.stat().st_mode & 0o777}


def main():
    scanner=REPO/'scripts/check-no-active-gir.sh';code=scanner.read_text().split("<<'PY'\n",1)[1].rsplit('\nPY',1)[0].rsplit('raise SystemExit(main())',1)[0]
    module=types.ModuleType('go_live_root_review');module.__file__=str(scanner);sys.modules[module.__name__]=module;sys.argv=[str(scanner),str(REPO),'strict'];exec(compile(code,str(scanner),'exec'),module.__dict__)
    matchers=module.load_matchers();manifest,_,_,_=module.validate_manifest();module.run_fixture_self_test(manifest,matchers)
    assert (len(matchers),len(module.SCOPES),len(manifest['sealed_records']),len(module.active_retained_record_exclusions(manifest)))==(127,4,39,51)
    paths={p for p in ROOT.rglob('*') if p.is_file() and p.name not in ['review.json','archived-findings.json.hex']}
    paths.update(REPO/name for name in ['go-tools/go2vir/practical_compiler_inputs_test.go','go-tools/go2vir/practical_live_root_witness_test.go','crates/mpk-cli/examples/go_practical_live_roots.rs','crates/mpk-cli/src/go_practical_source_roots.rs','crates/mpk-vc/src/go_practical_type_keys/snapshot_inputs.rs','develop/docs/09_go_practical_subset_design-todo.md','develop/docs/go-practical-implementation-traceability-ledger.md','develop/migrations/go-practical/t02-w01/README.md','develop/migrations/go-practical/t02-w01/requirements.json','develop/migrations/go-practical/t02-w01/verification.json','develop/migrations/go-practical/t02-w01/verification/source-root-matching/README.md'])
    active=[];archived=[];rows=[];archive_members=0;native_files=0;zip_members=0
    def detect(name,raw,scope):
        found=[vars(item) for item in module.detect(name,raw.decode(errors='ignore'),matchers)]
        if scope=='active':active.extend(found)
        else:archived.extend(found)
    def tar_members(payload):
        nonlocal archive_members
        with tarfile.open(fileobj=io.BytesIO(payload),mode='r:gz') as archive:
            for entry in archive:
                if entry.isfile():
                    raw=archive.extractfile(entry).read();assert len(raw)==entry.size;detect(entry.name,raw,'archived');archive_members+=1
    encoded_preflight=ROOT/'preflight/inputs.tar.gz.hex'
    for path in sorted(paths):
        assert path.is_file();raw=path.read_bytes();relative=str(path.relative_to(REPO));rows.append(seal(path));detect(relative,raw,'active')
        if path==encoded_preflight:
            data=json.loads((ROOT/'preflight-archive-storage.json').read_bytes());recovered=bytes.fromhex(raw.decode('ascii'));assert len(recovered)==data['historical_receipt']['bytes'] and hashlib.sha256(recovered).hexdigest()==data['historical_receipt']['sha256'];tar_members(recovered)
        elif path.name.endswith('.tar.gz'):tar_members(raw)
        if path.suffix=='.py':compile(raw,relative,'exec')
        if path.suffix=='.json':json.loads(raw)
    run=ROOT/'live-original';manifest=json.loads((run/'original-evidence-manifest.json').read_bytes());root=Path(manifest['root'])
    for row in manifest['files']:
        path=Path(row['path']);raw=path.read_bytes();assert len(raw)==row['bytes'] and hashlib.sha256(raw).hexdigest()==row['sha256'] and (path.stat().st_mode & 0o777)==row['mode']
        relative=str(path.relative_to(root));detect('historical-native/'+relative,raw,'archived');native_files+=1
        if raw.startswith((b'PK\x03\x04',b'PK\x05\x06')) and zipfile.is_zipfile(io.BytesIO(raw)):
            with zipfile.ZipFile(io.BytesIO(raw)) as archive:
                for member in archive.infolist():
                    content=archive.read(member);assert len(content)==member.file_size;detect('historical-native/'+relative+'!/'+member.filename,content,'archived');zip_members+=1
        if native_files%2000==0:print(json.dumps({'phase':'reviewing_original_evidence','files':native_files,'total':len(manifest['files'])}),flush=True)
    assert native_files==16880 and manifest['total_bytes']==367714473
    assert json.loads((run/'result.json').read_bytes())['status']=='passed' and json.loads((run/'independent-inspection.json').read_bytes())['status']=='passed' and json.loads((run/'restore-check.json').read_bytes())['status']=='passed'
    assert json.loads((ROOT/'preflight-go-retry/result.json').read_bytes())['status']=='passed' and json.loads((ROOT/'clippy-linux-target/result.json').read_bytes())['status']=='passed'
    assert json.loads((ROOT/'preflight/result.json').read_bytes())['status']=='failed' and json.loads((ROOT/'clippy-linux-test-cfg/result.json').read_bytes())['status']=='failed' and json.loads((ROOT/'clippy-example-test-harness/result.json').read_bytes())['status']=='failed'
    raw=(json.dumps(archived,indent=2)+'\n').encode();encoded=run/'archived-findings.json.hex';assert not encoded.exists();encoded.write_bytes(raw.hex().encode('ascii'));assert bytes.fromhex(encoded.read_text())==raw
    # The original evidence is held as encoded historical bytes; any incidental
    # archived findings are preserved in full, separately from active findings.
    findings_metadata={'count':len(archived),'raw_bytes':len(raw),'raw_sha256':hashlib.sha256(raw).hexdigest(),'encoded':seal(encoded),'exact_recovery':True}
    result={'status':'passed' if not active else 'failed','selection_reason':'Review changed Go/Rust inputs, active stored data, both retained preflight source archives and every original native evidence file and ZIP member. Record all historical findings exactly; do not change rules or exclusions and do not run the whole T gate.','inputs':rows,'active_findings':active,'archived_findings':findings_metadata,'archive_members':archive_members,'original_native_files':native_files,'original_zip_members':zip_members,'matcher_count':len(matchers),'scopes':len(module.SCOPES),'seals':len(module.active_retained_record_exclusions(manifest)),'native_Linux_release_acceptance':False,'full_T_gate':'Deferred to final T02-W04: ./scripts/check-fast.sh'}
    (ROOT/'review.json').write_text(json.dumps(result,indent=2)+'\n');print(json.dumps({'status':result['status'],'inputs':len(rows),'archive_members':archive_members,'original_native_files':native_files,'original_zip_members':zip_members,'active_findings':len(active),'archived_findings':len(archived)}),flush=True);raise SystemExit(bool(active))


if __name__=='__main__':main()
