"""Inspect complete original identities, oracle coverage and proof byte retention."""
from pathlib import Path
import hashlib,json,re

REPO=Path.cwd();ROOT=Path(__file__).resolve().parent
SDK=Path('/private/tmp/mpk-go125-20261025-download/go')
SNAPSHOT=ROOT.parent/'snapshot-inputs'
def seal(path):
    raw=path.read_bytes();return {'path':str(path),'bytes':len(raw),'sha256':hashlib.sha256(raw).hexdigest(),'mode':path.stat().st_mode&0o777}
def load(path):return json.loads(path.read_bytes())
def main():
    output=ROOT/'inspection.json';assert not output.exists()
    corpus_path=ROOT/'native/native-originals/probe/build-corpus.json'
    native_path=ROOT/'native/native-originals/originals/original-build-candidates.json'
    cases=load(corpus_path)['cases'];native=load(native_path)['units']
    results=load(ROOT/'rust/observations/build/native-case-comparison.json')['cases']
    assert len(cases)==len(results)==182
    assert len({r['name'] for r in cases})==182
    for source,actual in zip(cases,results):
        raw=bytes.fromhex(source['raw_hex']);obj=source['object']
        assert len(raw)==obj['bytes'] and hashlib.sha256(raw).hexdigest()==obj['raw_sha256']
        assert actual['name']==source['name'] and actual['source_object_id']==obj['id']
        if source['diagnostic']:assert 'rejected' in actual
        else:assert actual['matched']==source['matched'] and actual['binary_only']==source['binary_only']
    syslist=SDK/'src/internal/syslist/syslist.go'
    text=syslist.read_text()
    def names(key):
        block=text.split('var '+key+' = map[string]bool{',1)[1].split('\n}',1)[0]
        return set(re.findall(r'"([a-z0-9]+)":\s+true',block))
    systems=names('KnownOS');architectures=names('KnownArch')
    assert systems=={r['name'].removeprefix('filename-os-') for r in cases if r['name'].startswith('filename-os-')}
    assert architectures=={r['name'].removeprefix('filename-arch-') for r in cases if r['name'].startswith('filename-arch-')}
    compared=load(ROOT/'rust/observations/build/original-candidate-comparison.json')['units']
    assert len(native)==len(compared)==3
    units=[];all_inputs=[corpus_path,native_path,syslist]
    for observation,actual in zip(native,compared):
        assert observation['unit']==actual['unit']
        packet_path=SNAPSHOT/observation['packet'];packet=load(packet_path);all_inputs.append(packet_path)
        sources=packet['snapshot_sources'];expected={r['object']['id']:r for r in sources if r['object']['kind'] in ['go_file','excluded_source']}
        assert len(sources)==observation['complete_original_objects']
        assert {r['object']['id'] for r in observation['candidates']}==set(expected)
        assert {r['source_object_id'] for r in actual['all_original_go_candidates']}==set(expected)
        by_id={r['source_object_id']:r for r in actual['all_original_go_candidates']}
        for row in observation['candidates']:
            obj=row['object'];assert obj==expected[obj['id']]['object'] and not row['diagnostic']
            assert by_id[obj['id']]['matched']==row['matched'] and by_id[obj['id']]['binary_only']==row['binary_only']
        total=0;kinds={}
        for row in sources:
            obj=row['object'];path=packet_path.parent/'original-source-objects'/row['raw_file']
            raw=path.read_bytes();assert len(raw)==obj['bytes'] and hashlib.sha256(raw).hexdigest()==obj['raw_sha256']
            kinds[obj['kind']]=kinds.get(obj['kind'],0)+1;total+=len(raw)
        units.append({'unit':observation['unit'],'complete_original_objects':len(sources),'complete_original_bytes':total,'all_original_kinds':kinds,'all_original_go_candidates':len(expected)})
    # Preserve the exact primary Go source witnesses and check that their bytes
    # are the original captured SDK bytes used by these complete universes.
    owned=load(SNAPSHOT/'native-final/native-originals/owned/capture-inputs.json')
    witnesses=[]
    for name in ['src/go/build/build.go','src/go/build/read.go','src/go/build/constraint/expr.go','src/go/build/constraint/vers.go','src/internal/syslist/syslist.go','src/internal/buildcfg/exp.go']:
        path=SDK/name;raw=path.read_bytes()
        rows=[r for r in owned['snapshot_sources'] if r['object']['path']==name]
        assert len(rows)==1 and hashlib.sha256(raw).hexdigest()==rows[0]['object']['raw_sha256']
        witnesses.append({'source':seal(path),'captured_original':rows[0]['object'],'raw_hex':raw.hex()})
    proofs=load(ROOT/'rust/observations/sources/original-proof-replay.json')['complete_original_call_proofs']
    assert len(proofs)==32 and {r['error_cases'] for r in proofs}=={0,70}
    for row in proofs:
        saved=seal(Path(row['original_certificate']))
        assert saved['sha256']==row['raw_sha256'] and saved['bytes']==row['bytes']
    controls=load(ROOT/'rust/observations/snapshots/coherent-target-exclusion-controls.json')['controls']
    assert len(controls)==4 and all(r['original_objects']==5490 and r['complete_rebound_contract_functions']==2 for r in controls)
    original_units=load(ROOT/'rust/observations/snapshots/all-original-capture-units.json')['units']
    assert len(original_units)==2 and all(r['original_roots']==43 and r['all_source_components_and_bytes_equal'] for r in original_units)
    language=load(ROOT/'rust/observations/languages/original-language-comparison.json')['rows'];assert len(language)==150
    stdout=(ROOT/'rust/stdout.log').read_text()
    counts=[int(n) for n in re.findall(r'test result: ok\. (\d+) passed; 0 failed; 0 ignored;',stdout)]
    assert counts==[6,2,4,3]
    events=[json.loads(line) for line in (ROOT/'native/stdout.log').read_text().splitlines()]
    roots=[e['Test'] for e in events if e.get('Action')=='pass' and e.get('Test')]
    assert len(roots)==2
    result={'status':'passed','driver':seal(Path(__file__).resolve()),'inputs':[seal(p) for p in all_inputs],
        'actual_known_os_covered':sorted(systems),'actual_known_arch_covered':sorted(architectures),
        'complete_source_witnesses':witnesses,'original_boundary_cases':182,'native_constraint_rejections':sum(bool(r['diagnostic']) for r in cases),
        'complete_original_units':units,'complete_original_go_candidates':sum(r['all_original_go_candidates'] for r in units),
        'rust_roots':sum(counts),'native_roots':roots,'coherent_source_controls':controls,'previous_language_observations':150,
        'complete_original_proof_byte_streams':32,'original_error_cases':[0,70],'source_or_context_proof':False,
        'remaining':'Necessary fixed-target source eligibility only. Import/cgo classification, complete selected package membership, canonical capture/context/descriptor/registry, launcher/compiler/export authority, full foundation/program proofs and native Linux release gates remain due.',
        'full_T_gate':'Deferred to final T02-W04: ./scripts/check-fast.sh'}
    output.write_text(json.dumps(result,indent=2)+'\n');print(json.dumps({'status':'passed','native_cases':182,'original_go_candidates':result['complete_original_go_candidates'],'rust_roots':sum(counts),'source_witnesses':len(witnesses)}))

if __name__=='__main__':main()
