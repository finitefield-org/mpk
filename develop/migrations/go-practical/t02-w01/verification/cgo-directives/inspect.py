"""Inspect original cgo-disabled documentation cases and complete source retention."""
from pathlib import Path
import hashlib,json,re
ROOT=Path(__file__).resolve().parent
def seal(path):
    raw=path.read_bytes();return {'path':str(path),'bytes':len(raw),'sha256':hashlib.sha256(raw).hexdigest(),'mode':path.stat().st_mode&0o777}
def read(path):return json.loads(path.read_bytes())
def compare(native,actual):
    if native['diagnostic']:assert 'error' in actual
    else:
        assert native['name']==actual['name']
        assert (native['go_files'] or [])==actual['active'] and (native['ignored_go_files'] or [])==actual['excluded']
def main():
    output=ROOT/'inspection.json';assert not output.exists();inputs=[]
    path=ROOT/'native/native-originals/headers/cgo-directive-corpus.json';native=read(path)['cases'];inputs.append(seal(path))
    actual=read(ROOT/'rust-original/observations/cgo/native-cgo-comparison.json')['cases'];assert len(native)==len(actual)==88
    case_bytes=0
    for reference,checked in zip(native,actual):
        assert reference['name']==checked['case'];compare(reference['expected'],checked)
        for row in reference['snapshot_sources']:
            raw=bytes.fromhex(row['raw_hex']);obj=row['object'];assert len(raw)==obj['bytes'] and hashlib.sha256(raw).hexdigest()==obj['raw_sha256'];case_bytes+=len(raw)
    path=ROOT/'native/native-originals/originals/original-cgo-package-files.json';native=read(path)['units'];inputs.append(seal(path))
    actual=read(ROOT/'rust-original/observations/cgo/original-cgo-universes.json')['units'];assert len(native)==len(actual)==3
    units=[];cgo_files=0
    for reference,checked in zip(native,actual):
        assert reference['unit']==checked['unit'];packet_path=ROOT.parent/'snapshot-inputs'/reference['packet'];packet=read(packet_path);inputs.append(seal(packet_path))
        expected={(r['object']['capability'],r['object']['path'].rsplit('/',1)[0] if '/' in r['object']['path'] else '') for r in packet['snapshot_sources'] if r['object']['kind']=='go_file'}
        assert {(r['capability'],r['directory']) for r in reference['packages']}==expected=={(r['capability'],r['directory']) for r in checked['packages']}
        for a,b in zip(reference['packages'],checked['packages']):
            assert (a['capability'],a['directory'])==(b['capability'],b['directory']);compare(a,b);cgo_files+=len(b.get('cgo_ignored',[]))
        kinds={};total=0
        for row in packet['snapshot_sources']:
            raw=(packet_path.parent/'original-source-objects'/row['raw_file']).read_bytes();obj=row['object']
            assert len(raw)==obj['bytes'] and hashlib.sha256(raw).hexdigest()==obj['raw_sha256'];total+=len(raw);kinds[obj['kind']]=kinds.get(obj['kind'],0)+1
        units.append({'unit':reference['unit'],'complete_original_objects':len(packet['snapshot_sources']),'complete_original_bytes':total,'original_source_kinds':kinds,'package_directories':len(expected)})
    assert sum(r['package_directories'] for r in units)==2003
    owned_path=ROOT/'native/native-originals/owned/complete-cgo-capture.json';owned=read(owned_path);inputs.append(seal(owned_path))
    owned_bytes=0
    for row in owned['snapshot_sources']:
        raw=(owned_path.parent/'original-source-objects'/row['raw_file']).read_bytes();obj=row['object']
        assert len(raw)==obj['bytes'] and hashlib.sha256(raw).hexdigest()==obj['raw_sha256'];owned_bytes+=len(raw)
    control=read(ROOT/'rust-original/observations/snapshots/coherent-excluded-cgo-directive.json')
    assert control['status']=='passed' and control['same_original_source_selection_and_two_contract_candidates'] and not control['cgo_enabled']
    assert control['complete_original_objects']==len(owned['snapshot_sources']) and control['original_excluded_source']['kind']==control['changed_excluded_source']['kind']=='excluded_source'
    assert control['original_excluded_source']['path']==control['changed_excluded_source']['path']=='cgo.go'
    witnesses=[];sdk=Path('/private/tmp/mpk-go125-20261025-download/go')
    for name in ['src/go/build/build.go','src/go/build/read.go','src/go/parser/parser.go','src/go/ast/ast.go','src/go/scanner/scanner.go']:
        raw=(sdk/name).read_bytes();matches=[r for r in owned['snapshot_sources'] if r['object']['path']==name]
        assert len(matches)==1 and hashlib.sha256(raw).hexdigest()==matches[0]['object']['raw_sha256']
        witnesses.append({'path':name,'original_source_object':matches[0]['object'],'raw_hex':raw.hex()})
    proof_path=ROOT/'rust-original/observations/sources/original-proof-replay.json';proofs=read(proof_path)['complete_original_call_proofs'];inputs.append(seal(proof_path));assert len(proofs)==32
    for row in proofs:
        actual=seal(Path(row['original_certificate']));assert actual['sha256']==row['raw_sha256'] and actual['bytes']==row['bytes']
    retained=read(ROOT/'rust-original/observations/snapshots/all-original-capture-units.json')['units'];assert len(retained)==2 and all(r['original_roots']==43 and r['all_source_components_and_bytes_equal'] for r in retained)
    stdout=(ROOT/'rust-original/stdout.log').read_text();counts=[int(n) for n in re.findall(r'test result: ok\. (\d+) passed; 0 failed; 0 ignored;',stdout)];assert counts==[3,2,2,8,2,4]
    events=[json.loads(line) for line in (ROOT/'native/stdout.log').read_text().splitlines()];go_roots=[r['Test'] for r in events if r.get('Action')=='pass' and r.get('Test')];assert len(go_roots)==3
    result={'status':'passed','driver':seal(Path(__file__).resolve()),'inputs':inputs,'native_cases':88,'native_directive_rejections':31,'complete_case_source_bytes':case_bytes,'complete_original_units':units,'all_original_package_directories':2003,'all_original_cgo_disabled_files':cgo_files,'new_complete_owned_snapshot_objects':len(owned['snapshot_sources']),'new_complete_owned_snapshot_bytes':owned_bytes,'complete_original_source_witnesses':witnesses,'coherent_excluded_directive_control':control,'affected_rust_roots':sum(counts),'native_roots':go_roots,'original_proof_byte_streams':32,'original_error_universes':sorted({r['error_cases'] for r in proofs}),'cgo_enabled':False,'remaining':'Original import-C documentation validation only. Embed selection and canonical module/compiler/export/context/launcher authority, full foundation/source/program proofs and native Linux gates remain due.','full_T_gate':'Deferred to final T02-W04: ./scripts/check-fast.sh'}
    output.write_text(json.dumps(result,indent=2)+'\n');print(json.dumps({'status':'passed','cases':88,'original_packages':2003,'cgo_disabled_files':cgo_files,'rust_roots':sum(counts),'owned_objects':len(owned['snapshot_sources'])}))
if __name__=='__main__':main()
