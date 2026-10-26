"""Inspect all original embed cases, preserved resources and unaffected proof streams."""
from pathlib import Path
import base64,hashlib,json,re
ROOT=Path(__file__).resolve().parent
def seal(p):
    raw=p.read_bytes();return {'path':str(p),'bytes':len(raw),'sha256':hashlib.sha256(raw).hexdigest(),'mode':p.stat().st_mode&0o777}
def read(p):return json.loads(p.read_bytes())
def main():
    output=ROOT/'inspection.json';assert not output.exists();inputs=[]
    path=ROOT/'native/native-originals/matrix/embedded-source-corpus.json';corpus=read(path);cases=corpus['cases'];inputs.append(seal(path))
    assert len(cases)==54 and sum(r['accepted'] for r in cases)==27
    all_case_bytes=0;resource_identities=0
    for case in cases:
        raw_files={k:base64.b64decode(v,validate=True) for k,v in case['whole_original_files'].items()};all_case_bytes+=sum(map(len,raw_files.values()))
        native_stdout=base64.b64decode(case['native_stdout'],validate=True);base64.b64decode(case['native_stderr'],validate=True)
        if not case['accepted']:assert 'error' in case;continue
        native=json.loads(native_stdout);assert case['files']==native['EmbedFiles']
        packet=case['snapshot'];objects={s['object']['id']:s for s in packet['snapshot_sources']}
        for row in packet['snapshot_sources']:
            obj=row['object'];raw=bytes.fromhex(row['raw_hex']);assert raw==raw_files[obj['path']]
            assert len(raw)==obj['bytes'] and hashlib.sha256(raw).hexdigest()==obj['raw_sha256']
        assert len(case['files'])==len(case['objects'])
        for file,obj in zip(case['files'],case['objects']):assert obj['path']==file and objects[obj['id']]['object']==obj;resource_identities+=1
        assert sorted({file for files in case['pattern_files'].values() for file in files})==case['files']
    assert corpus['original_roots_removed_before_successful_replay'] and not corpus['caller_source_adapter_or_fallback']
    directory=ROOT/'native/native-originals/owned';packet_path=directory/'complete-embedded-capture.json';packet=read(packet_path);inputs.append(seal(packet_path))
    native=read(directory/'original-embedded-resources.json');assert len(native['selected'])==4
    assert native['returned_inventory_independent'] and len(native['controls'])==3
    all_bytes=0;kinds={}
    for row in packet['snapshot_sources']:
        obj=row['object'];raw=(directory/'original-source-objects'/row['raw_file']).read_bytes()
        assert len(raw)==obj['bytes'] and hashlib.sha256(raw).hexdigest()==obj['raw_sha256'];all_bytes+=len(raw);kinds[obj['kind']]=kinds.get(obj['kind'],0)+1
    assert len(packet['snapshot_sources'])==native['all_original_objects']==5535
    independent=read(ROOT/'rust/observations/snapshots/whole-native-embedded-resources.json');assert independent['status']=='passed' and independent['complete_original_objects']==5535
    assert independent['original_resources']==native['selected'] and independent['original_selected_contract_functions']==1
    assert not independent['independent_resource_pattern_selection_claimed'] and not independent['source_or_context_proof']
    assert next(r for r in independent['original_resources'] if r['object']['path']=='assets/binary.dat')['raw_hex']=='000100ff80'
    witnesses=[];sdk=Path('/private/tmp/mpk-go125-20261025-download/go')
    for name in ['src/cmd/go/internal/load/pkg.go','src/path/match.go','src/cmd/vendor/golang.org/x/mod/module/module.go']:
        raw=(sdk/name).read_bytes();rows=[s for s in packet['snapshot_sources'] if s['object']['path']==name];assert len(rows)==1 and hashlib.sha256(raw).hexdigest()==rows[0]['object']['raw_sha256']
        witnesses.append({'path':name,'object':rows[0]['object'],'raw_hex':raw.hex()})
    proofs=read(ROOT/'rust/observations/sources/original-proof-replay.json')['complete_original_call_proofs'];assert len(proofs)==32
    for row in proofs:
        saved=seal(Path(row['original_certificate']));assert saved['sha256']==row['raw_sha256'] and saved['bytes']==row['bytes']
    retained=read(ROOT/'rust/observations/snapshots/all-original-capture-units.json')['units'];assert len(retained)==2 and all(r['original_roots']==43 and r['all_source_components_and_bytes_equal'] for r in retained)
    stdout=(ROOT/'rust/stdout.log').read_text();counts=[int(n) for n in re.findall(r'test result: ok\. (\d+) passed; 0 failed; 0 ignored;',stdout)];assert counts==[9,4]
    events=[json.loads(line) for line in (ROOT/'native/stdout.log').read_text().splitlines()];passed=[r['Test'] for r in events if r.get('Action')=='pass' and r.get('Test')];roots=[n for n in passed if '/' not in n];assert len(roots)==7 and not [r for r in events if r.get('Action')=='fail']
    result={'status':'passed','driver':seal(Path(__file__).resolve()),'inputs':inputs,'original_embed_cases':54,'accepted_original_cases':27,'rejected_original_cases':27,'complete_case_source_bytes':all_case_bytes,'original_selected_resource_identities':resource_identities,'actual_original_owner_objects':5535,'actual_original_owner_bytes':all_bytes,'all_original_owner_source_kinds':kinds,'original_embedded_resources':native,'independent_whole_byte_handoff':independent,'complete_pinned_primary_source_witnesses':witnesses,'affected_rust_roots':sum(counts),'native_roots':roots,'native_subtests':len(passed)-len(roots),'original_proof_byte_streams':32,'original_error_case_universes':sorted({r['error_cases'] for r in proofs}),'remaining':'Native original resource selection/capture/replay and independent full-byte handoff only. Independent Rust pattern/resource selection, canonical capture/context/compiler/export/descriptor/registry and root authority, full foundation/source/program proofs and native Linux gates remain due.','full_T_gate':'Deferred to final T02-W04: ./scripts/check-fast.sh'}
    output.write_text(json.dumps(result,indent=2)+'\n');print(json.dumps({'status':'passed','embed_cases':54,'owner_objects':5535,'native_roots':len(roots),'native_subtests':len(passed)-len(roots),'rust_roots':sum(counts),'resource_identities':resource_identities}))
if __name__=='__main__':main()
