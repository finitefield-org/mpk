"""Independent byte/member/coverage inspection of complete actual observations."""
from pathlib import Path
import hashlib,json,re
ROOT=Path(__file__).resolve().parent
def seal(path):
    raw=path.read_bytes();return {'path':str(path),'bytes':len(raw),'sha256':hashlib.sha256(raw).hexdigest(),'mode':path.stat().st_mode&0o777}
def read(path):return json.loads(path.read_bytes())
def main():
    output=ROOT/'inspection.json';assert not output.exists();inputs=[]
    header_path=ROOT/'native-final/native-originals/headers/package-header-corpus.json'
    header=read(header_path)['cases'];actual=read(ROOT/'rust-original/observations/packages/header-comparison.json')['cases']
    assert len(header)==len(actual)==39;inputs.append(seal(header_path))
    names=[('go_files','active'),('ignored_go_files','excluded'),('test_go_files','tests'),('external_test_go_files','external_tests')]
    def compare(expected,observed):
        if expected['diagnostic']:assert 'rejected' in observed
        else:
            assert expected['name']==observed['name'] and expected['binary_only']==observed['binary_only']
            for native,key in names:assert (expected[native] or [])==observed[key]
            for key in ['imports','test_imports']:assert (expected[key] or [])==observed[key]
            assert (expected['external_test_imports'] or [])==observed['external_test_imports']
    source_bytes=0
    for original,observed in zip(header,actual):
        assert original['name']==observed['case'];compare(original['expected'],observed)
        for row in original['snapshot_sources']:
            raw=bytes.fromhex(row['raw_hex']);obj=row['object']
            assert len(raw)==obj['bytes'] and hashlib.sha256(raw).hexdigest()==obj['raw_sha256'];source_bytes+=len(raw)
    native_path=ROOT/'native/native-originals/originals/original-package-files.json';native=read(native_path)['units'];inputs.append(seal(native_path))
    independent=read(ROOT/'rust-original/observations/packages/original-universe-comparison.json')['units'];assert len(native)==len(independent)==3
    units=[]
    for original,observed in zip(native,independent):
        assert original['unit']==observed['unit'];packet_path=ROOT.parent/'snapshot-inputs'/original['packet'];packet=read(packet_path);inputs.append(seal(packet_path))
        expected={(r['object']['capability'],r['object']['path'].rsplit('/',1)[0] if '/' in r['object']['path'] else '') for r in packet['snapshot_sources'] if r['object']['kind']=='go_file'}
        assert {(r['capability'],r['directory']) for r in original['packages']}==expected
        assert {(r['capability'],r['directory']) for r in observed['packages']}==expected
        assert len(original['packages'])==len(observed['packages'])==len(expected)
        for reference,result in zip(original['packages'],observed['packages']):
            assert (reference['capability'],reference['directory'])==(result['capability'],result['directory']);compare(reference,result)
        total=0;kinds={}
        for row in packet['snapshot_sources']:
            path=packet_path.parent/'original-source-objects'/row['raw_file'];raw=path.read_bytes();obj=row['object']
            assert len(raw)==obj['bytes'] and hashlib.sha256(raw).hexdigest()==obj['raw_sha256'];total+=len(raw);kinds[obj['kind']]=kinds.get(obj['kind'],0)+1
        units.append({'unit':original['unit'],'complete_original_objects':len(packet['snapshot_sources']),'complete_original_bytes':total,'all_original_kinds':kinds,'package_directories':len(expected),'native_errors':sum(bool(r['diagnostic']) for r in original['packages'])})
    assert sum(u['package_directories'] for u in units)==2003
    # Inspect exact range-data equality independently of Rust's binary search.
    native_ranges=re.findall(r'\(0x([0-9a-f]+),\s*0x([0-9a-f]+)\)',(ROOT/'graphic-final.stdout').read_text())
    implementation=Path.cwd()/'crates/mpk-vc/src/go_practical_type_keys/source_package/graphic.rs'
    rust_ranges=re.findall(r'\(0x([0-9a-f]+),\s*0x([0-9a-f]+)\)',implementation.read_text())
    assert native_ranges==rust_ranges
    previous=-1
    for lo,hi in native_ranges:
        lo=int(lo,16);hi=int(hi,16);assert previous<lo<=hi<=0x10ffff;previous=hi
    omitted_path=ROOT/'rust-original/observations/snapshots/coherent-unselected-file-omission.json';omitted=read(omitted_path);inputs.append(seal(omitted_path))
    assert omitted['status']=='passed' and omitted['original_snapshot_unchanged'] and omitted['original_and_rebound_selected_contract_functions']==2
    assert omitted['omitted_original_object']['path']=='unselected.go'
    proofs=read(ROOT/'rust-original/observations/sources/original-proof-replay.json')['complete_original_call_proofs'];assert len(proofs)==32
    for proof in proofs:
        actual=seal(Path(proof['original_certificate']));assert actual['sha256']==proof['raw_sha256'] and actual['bytes']==proof['bytes']
    preserved=read(ROOT/'rust-original/observations/snapshots/all-original-capture-units.json')['units'];assert len(preserved)==2 and all(r['original_roots']==43 and r['all_source_components_and_bytes_equal'] for r in preserved)
    text=(ROOT/'rust-original/stdout.log').read_text();counts=[int(n) for n in re.findall(r'test result: ok\. (\d+) passed; 0 failed; 0 ignored;',text)];assert counts==[3,2,3,7,2,4,3]
    assert 'accepted=5714, rejected=139, exact positive tokens=7702601' in text
    assert 'accepted=2969, rejected=27, exact syntax nodes=5731905' in text
    corpus=read(ROOT/'restored-corpora.json');assert [r['case_directories'] for r in corpus['corpora']]==[5853,2996]
    result={'status':'passed','driver':seal(Path(__file__).resolve()),'inputs':inputs,'original_header_cases':39,'header_native_rejections':sum(bool(r['expected']['diagnostic']) for r in header),'all_original_header_source_bytes':source_bytes,'original_units':units,'all_original_package_directories':2003,'exact_graphic_range_count':len(native_ranges),'rust_roots':sum(counts),'complete_original_lexical_cases':5853,'complete_original_syntax_cases':2996,'exact_positive_tokens':7702601,'exact_positive_syntax_nodes':5731905,'omitted_original_source_control':omitted,'original_proof_streams':32,'original_error_universes':sorted({r['error_cases'] for r in proofs}),'remaining':'Go header/file-set membership only. Auxiliary/directive/embed selection, canonical module/compiler/export/context/root authority, full foundation/source/program proofs and native Linux release gates remain due.','full_T_gate':'Deferred to final T02-W04: ./scripts/check-fast.sh'}
    output.write_text(json.dumps(result,indent=2)+'\n');print(json.dumps({'status':'passed','headers':39,'original_packages':2003,'rust_roots':sum(counts),'graphic_ranges':len(native_ranges)}))
if __name__=='__main__':main()
