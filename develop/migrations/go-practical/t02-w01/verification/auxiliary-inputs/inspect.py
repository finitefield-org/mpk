"""Check exact auxiliary identities, original package coverage and proof retention."""
from pathlib import Path
import hashlib,json,re
ROOT=Path(__file__).resolve().parent
FIELDS=['c','cxx','objective_c','headers','fortran','assembly','swig','swig_cxx','system_objects']
def seal(path):
    raw=path.read_bytes();return {'path':str(path),'bytes':len(raw),'sha256':hashlib.sha256(raw).hexdigest(),'mode':path.stat().st_mode&0o777}
def read(path):return json.loads(path.read_bytes())
def compare(native,independent):
    assert not native['diagnostic']
    for field in FIELDS:assert (native[field] or [])==independent['selected'][field]
    assert (native['ignored_auxiliary'] or [])==independent['ignored_auxiliary']
def main():
    output=ROOT/'inspection.json';assert not output.exists();inputs=[]
    native_path=ROOT/'native/native-originals/cases/auxiliary-corpus.json';native=read(native_path)['cases'];inputs.append(seal(native_path))
    independent=read(ROOT/'rust-original/observations/auxiliary/native-case-comparison.json')['cases'];assert len(native)==len(independent)==178
    extensions=set();all_original_bytes=0
    for case,result in zip(native,independent):
        assert case['name']==result['case'];compare(case['expected'],result)
        for row in case['snapshot_sources']:
            raw=bytes.fromhex(row['raw_hex']);obj=row['object'];assert len(raw)==obj['bytes'] and hashlib.sha256(raw).hexdigest()==obj['raw_sha256'];all_original_bytes+=len(raw)
            if obj['kind']=='auxiliary' and '.' in obj['path']:extensions.add(obj['path'].rsplit('.',1)[-1])
    expected={'c','cc','cpp','cxx','m','h','hh','hpp','hxx','f','F','for','f90','s','S','sx','swig','swigcxx','syso'};assert expected<=extensions
    native_path=ROOT/'native/native-originals/originals/original-auxiliary-files.json';reference=read(native_path)['units'];inputs.append(seal(native_path))
    checked=read(ROOT/'rust-original/observations/auxiliary/complete-original-universe-comparison.json')['units'];assert len(reference)==len(checked)==3
    units=[];totals={field:0 for field in FIELDS};ignored=0
    for unit,actual in zip(reference,checked):
        assert unit['unit']==actual['unit'];packet_path=ROOT.parent/'snapshot-inputs'/unit['packet'];packet=read(packet_path);inputs.append(seal(packet_path))
        expected_dirs={(r['object']['capability'],r['object']['path'].rsplit('/',1)[0] if '/' in r['object']['path'] else '') for r in packet['snapshot_sources'] if r['object']['kind']=='go_file'}
        assert {(r['capability'],r['directory']) for r in unit['packages']}==expected_dirs=={(r['capability'],r['directory']) for r in actual['packages']}
        for original,result in zip(unit['packages'],actual['packages']):
            assert (original['capability'],original['directory'])==(result['capability'],result['directory']);compare(original,result)
            for field in FIELDS:totals[field]+=len(result['selected'][field])
            ignored+=len(result['ignored_auxiliary'])
        size=0;kinds={}
        for row in packet['snapshot_sources']:
            path=packet_path.parent/'original-source-objects'/row['raw_file'];raw=path.read_bytes();obj=row['object']
            assert len(raw)==obj['bytes'] and hashlib.sha256(raw).hexdigest()==obj['raw_sha256'];size+=len(raw);kinds[obj['kind']]=kinds.get(obj['kind'],0)+1
        units.append({'unit':unit['unit'],'complete_original_objects':len(packet['snapshot_sources']),'complete_original_bytes':size,'all_original_kinds':kinds,'package_directories':len(expected_dirs)})
    assert sum(u['package_directories'] for u in units)==2003
    controls=read(ROOT/'rust-original/observations/auxiliary/closed-original-buffer-controls.json');assert controls['status']=='passed' and len(controls['controls'])==2
    assert controls['all_original_names_preserved'] and not controls['fallback_filesystem_or_source_adapter']
    proofs=read(ROOT/'rust-original/observations/sources/original-proof-replay.json')['complete_original_call_proofs'];assert len(proofs)==32
    for row in proofs:
        actual=seal(Path(row['original_certificate']));assert actual['bytes']==row['bytes'] and actual['sha256']==row['raw_sha256']
    retained=read(ROOT/'rust-original/observations/snapshots/all-original-capture-units.json')['units'];assert len(retained)==2 and all(r['original_roots']==43 and r['all_source_components_and_bytes_equal'] for r in retained)
    stdout=(ROOT/'rust-original/stdout.log').read_text();counts=[int(n) for n in re.findall(r'test result: ok\. (\d+) passed; 0 failed; 0 ignored;',stdout)];assert counts==[3,2,7,2,4]
    events=[json.loads(line) for line in (ROOT/'native/stdout.log').read_text().splitlines()];roots=[r['Test'] for r in events if r.get('Action')=='pass' and r.get('Test')];assert len(roots)==2
    result={'status':'passed','driver':seal(Path(__file__).resolve()),'inputs':inputs,'native_cases':178,'recognized_extensions':sorted(expected),'auxiliary_kinds':FIELDS,'all_original_case_bytes':all_original_bytes,'original_units':units,'original_package_directories':2003,'all_original_selected_auxiliary_counts':totals,'all_original_ignored_auxiliary_count':ignored,'closed_original_buffer_controls':controls,'rust_roots':sum(counts),'native_roots':roots,'original_proof_byte_streams':32,'original_error_universes':sorted({r['error_cases'] for r in proofs}),'source_or_context_proof':False,'remaining':'Auxiliary input selection is not additional program-language support. Cgo directives/embed selection, canonical module/compiler/export/context/root authority, full foundation/source/program proofs and native Linux gates remain due.','full_T_gate':'Deferred to final T02-W04: ./scripts/check-fast.sh'}
    output.write_text(json.dumps(result,indent=2)+'\n');print(json.dumps({'status':'passed','cases':178,'packages':2003,'rust_roots':sum(counts),'selected':totals,'ignored':ignored}))
if __name__=='__main__':main()
