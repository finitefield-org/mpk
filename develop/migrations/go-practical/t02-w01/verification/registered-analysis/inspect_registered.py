"""Independently reconcile old source archive, capture and registered builds."""
from pathlib import Path
import hashlib
import json
import tarfile

ROOT=Path(__file__).resolve().parent;REPO=ROOT.parents[5];INSTALLED=ROOT.parent/'installed-compiler'
OLD=Path('/private/tmp/mpk-installed-compiler-205-go-source')
REFERENCE=Path('/private/tmp/mpk-installed-compiler-205-original-go-output/frontend/bin/go2vir')


def main():
    receipt=json.loads((INSTALLED/'original-go-source.json').read_bytes());archive=INSTALLED/'original-go-source.tar';native=json.loads((ROOT/'native-fixed/native.json').read_bytes());built=json.loads((ROOT/'rebuild-original/result.json').read_bytes());matched=json.loads((ROOT/'roots-original/result.json').read_bytes())
    assert hashlib.sha256(archive.read_bytes()).hexdigest()==receipt['archive_sha256']
    expected={str(Path(row['path']).relative_to(OLD)):row for row in receipt['source_files']};assert len(expected)==269
    source={};directory_count=0
    with tarfile.open(archive) as tar:
        for member in tar:
            if member.isdir():directory_count+=1;continue
            assert member.isfile() and member.name in expected and member.name not in source
            raw=tar.extractfile(member).read();row=expected[member.name]
            assert len(raw)==member.size==row['bytes'] and hashlib.sha256(raw).hexdigest()==row['sha256']
            source[member.name]=raw
    assert set(source)==set(expected) and directory_count==61
    assert sum(p.startswith('go-tools/go2vir/') for p in source)==242
    original={row['object']['id']:row for row in native['original_sources']};assert len(original)==196
    for row in original.values():
        raw=(ROOT/'native-fixed/original-bytes'/row['raw_file']).read_bytes();object=row['object']
        assert len(raw)==object['bytes'] and hashlib.sha256(raw).hexdigest()==object['raw_sha256']
    build=native['build_objects'];assert len(build)==184 and len({row['path'] for row in build})==184
    for row in build:
        assert row['capability']=='analysis-frontend' and original[row['id']]['object']==row
        exact=source['go-tools/go2vir/'+row['path']]
        assert len(exact)==row['bytes'] and hashlib.sha256(exact).hexdigest()==row['raw_sha256']
        assert (ROOT/'native-fixed/original-bytes'/original[row['id']]['raw_file']).read_bytes()==exact
    wire=json.loads(bytes.fromhex(native['snapshot_hex']));dirs=len(wire['directories']);entries=sum(len(d['entries']) for d in wire['directories'])
    assert len(wire['objects'])==196 and built['status']=='passed' and matched['status']=='passed'
    reference=REFERENCE.read_bytes();candidate=json.loads((REPO/'release/bundles/candidates/go.json').read_bytes())['frontend_bundles'][0]
    registered=next(row for row in candidate['inventory']['files'] if row['path']=='bin/go2vir')
    assert len(reference)==registered['size_bytes']==9004378 and hashlib.sha256(reference).hexdigest()==registered['sha256']=='ccc4a984d41b5da7b3a6838cbfaeee3eb23f675a2e4d7fb5d4f9fcd7e597d538'
    for i in range(2):
        raw=bytes.fromhex((ROOT/f'rebuild-original/frontend-{i}.elf.hex').read_text());assert raw==reference
    result={'status':'passed','selection_reason':'Independently rehash all original archive files, compare every Go-derived build object to its complete historical source bytes, reconcile the full held-root test and both complete reproduced binaries with the registered inventory.','registered_epoch':receipt['basis_commit'],'original_full_source_files':269,'original_frontend_files':242,'archive_directories':directory_count,'captured_objects':len(original),'captured_directories':dirs,'captured_entries':entries,'build_objects':len(build),'frontend_go_files':native['original_frontend_go_files'],'vendor_files':native['original_vendor_files'],'registered_frontend_bytes':len(reference),'registered_frontend_sha256':registered['sha256'],'two_complete_binaries_equal_registered':True,'earlier_test_root_comparison_passed':True,'current_source_tree_attributed_to_registered_binary':False,'production_launcher_authority_proved':False,'application_proof_proved':False,'native_Linux_release_acceptance':False,'full_T_gate':'Deferred to final T02-W04: ./scripts/check-fast.sh'}
    out=ROOT/'inspection.json';assert not out.exists();out.write_text(json.dumps(result,indent=2)+'\n');print(json.dumps(result),flush=True)


if __name__=='__main__':main()
