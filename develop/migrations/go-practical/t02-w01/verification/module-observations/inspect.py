"""Inspect complete original resolver/source/export universes and controls."""
from pathlib import Path
import hashlib,json,re
ROOT=Path(__file__).resolve().parent
INPUTS={}
def seal(p):
    data=p.read_bytes();return {'path':str(p),'bytes':len(data),'sha256':hashlib.sha256(data).hexdigest(),'mode':p.stat().st_mode&0o777}
def raw(p):INPUTS[str(p)]=seal(p);return p.read_bytes()
def read(p):return json.loads(raw(p))
def stream(data):
    decoder=json.JSONDecoder();text=data.decode();rows=[]
    while text.strip():
        text=text.lstrip();row,end=decoder.raw_decode(text);rows.append(row);text=text[end:]
    return rows

def main():
    output=ROOT/'inspection.json';assert not output.exists()
    epochs={p.parent.name:read(p) for p in ROOT.glob('*/result.json')}
    assert all(r['status']=='passed' and (r['consumed_inputs_unchanged'] or r['expected_formatter_mutation']) for r in epochs.values())
    native=[json.loads(line) for line in raw(ROOT/'native/stdout.log').splitlines()]
    passed=[r['Test'] for r in native if r['Action']=='pass' and r.get('Test')]
    assert set(passed)=={'TestPracticalExportInputsKeepWholeOriginalCompilerUniverse','TestPracticalModuleObservationsKeepOriginalReplacementGraph','TestPracticalModuleObservationsKeepOriginalVersionedReplacement'}
    assert not any(r['Action']=='fail' for r in native)
    assert not raw(ROOT/'go-format-check/stdout.log')
    owners=[];witnesses=[]
    for name in ['owned','modules-owned','versioned-owned']:
        directory=ROOT/'native/native-originals'/name
        packet=read(directory/('complete-original-export-inputs.json' if name=='owned' else 'complete-original-module-observations.json'))
        objects={};total=0
        for row in packet['snapshot_sources']:
            obj=row['object'];data=raw(directory/'original-source-objects'/row['raw_file'])
            assert len(data)==obj['bytes'] and hashlib.sha256(data).hexdigest()==obj['raw_sha256'] and obj['id'] not in objects
            objects[obj['id']]=(obj,data);total+=len(data)
        export=json.loads(bytes.fromhex(packet['export_inputs_hex']));packages=stream(bytes.fromhex(export['metadata_hex']))
        assert len(packages)==len(export['packages']) and len({r['ImportPath'] for r in packages})==len(packages)
        item={'owner':name,'objects':len(objects),'raw_bytes':total,'packages':len(packages),'exports':sum(r['export_source_object_id'] is not None for r in export['packages'])}
        if name!='owned':
            wire=json.loads(bytes.fromhex(packet['module_observations_hex']));rows=stream(bytes.fromhex(wire['metadata_hex']));origins={r['capability']:r['path'] for r in export['origins']}
            assert wire['source_export_inputs_sha256']==export['content_sha256'] and len(rows)==len(wire['modules'])
            native_by_path={r['Path']:r for r in rows};assert len(rows)==len(native_by_path)
            observed=read(ROOT/'rust-original/observations/exports'/('whole-original-module-linkage.json' if name=='modules-owned' else 'whole-original-versioned-module-linkage.json'))
            assert observed['metadata_hex']==wire['metadata_hex'] and observed['diagnostics_hex']==wire['diagnostics_hex']
            assert len(observed['manifests'])==len(rows) and not any(r.get('Error') for r in rows)
            for selected in wire['modules']:
                row=native_by_path[selected['path']];target=row.get('Replace',row);obj,data=objects[selected['manifest_source_object_id']]
                assert obj['kind']=='go_mod' and observed['manifests'][selected['path']]==obj
                assert row.get('Version','')==selected['version'] and row['GoMod']==target['GoMod']==origins[obj['capability']].rstrip('/')+'/'+obj['path']
                assert target.get('Path','')==selected['replacement_path'] if 'Replace' in row else not selected['replacement_path']
                assert target.get('Version','')==selected['replacement_version'] if 'Replace' in row else not selected['replacement_version']
            assert sum(bool(r.get('Main')) for r in rows)==1
            assert observed.get('original_selected_contract_functions',1)==1
            if name=='modules-owned':
                assert len(rows)==3 and observed['manifests']['example.com/alias']['path']=='local-a/go.mod'
                assert any(o['path']=='wildcard/go.mod' for o,_ in objects.values())
                main=next(data for obj,data in objects.values() if obj['id']==observed['manifests']['example.com/policy']['id'])
                assert b'example.com/alias => ./wildcard' in main and b'example.com/alias v1.0.0 =>' in main
                assert main.count(b'example.com/unselected => ./wildcard')==2 and b'example.com/policy => ./wildcard' in main
                assert any(r['ImportPath']=='net/http' for r in packages)
            else:
                assert len(rows)==2 and observed['manifests']['example.com/Dep']['path']=='cache/download/example.com/!dep/@v/v1.2.0.mod'
                cache=observed['whole_original_cache_sources'];assert len(cache)==4
                for obj in cache:assert objects[obj['id']][0]==obj
                assert any(o['path'].endswith('.zip') for o in cache) and any(o['kind']=='go_sum' for o,_ in objects.values())
            item['modules']=len(rows)
        if name=='modules-owned':
            for path in ['src/cmd/go/internal/modload/modfile.go','src/cmd/vendor/golang.org/x/mod/modfile/read.go','src/cmd/vendor/golang.org/x/mod/modfile/rule.go']:
                obj,data=next((o,b) for o,b in objects.values() if o['path']==path)
                assert data==(Path('/private/tmp/mpk-go125-20261025-download/go')/path).read_bytes()
                witnesses.append({'path':path,'object':obj,'raw_hex':data.hex()})
        owners.append(item)
    controls=read(ROOT/'rust-original/observations/exports/coherent-original-module-controls.json')['observations'];assert len(controls)==24
    original=read(ROOT/'native/native-originals/modules-owned/complete-original-module-observations.json');original_wire=json.loads(bytes.fromhex(original['module_observations_hex']))
    for row in controls:
        changed=json.loads(bytes.fromhex(row['complete_changed_module_observations_hex']));assert changed!=original_wire and row['original_capture_and_one_Root_contract_unchanged']
        if 'complete_export_inputs_hex' in row:
            changed_export=json.loads(bytes.fromhex(row['complete_export_inputs_hex']))
            if row['case']!='export-owner':assert changed['source_export_inputs_sha256']==changed_export['content_sha256']
            assert row['error']
    proof_rows=read(ROOT/'rust-original/observations/sources/original-proof-replay.json')['complete_original_call_proofs'];assert len(proof_rows)==32
    for row in proof_rows:
        data=raw(Path(row['original_certificate']));assert len(data)==row['bytes'] and hashlib.sha256(data).hexdigest()==row['raw_sha256']
    units=read(ROOT/'rust-original/observations/snapshots/all-original-capture-units.json')['units'];assert len(units)==2 and all(r['original_roots']==43 and r['all_source_components_and_bytes_equal'] for r in units)
    counts=[int(n) for n in re.findall(r'test result: ok\. (\d+) passed; 0 failed; 0 ignored;',raw(ROOT/'rust-original/stdout.log').decode())];assert counts==[6,6,9,4]
    result={'status':'passed','driver':seal(Path(__file__).resolve()),'inputs':list(INPUTS.values()),'source_epochs':len(epochs),'native_passed_roots':passed,'owners':owners,'affected_rust_roots':sum(counts),'coherent_module_controls':len(controls),'original_proof_byte_streams':32,'original_roots_per_unit':43,'original_error_case_universes':sorted({r['error_cases'] for r in proof_rows}),'complete_original_source_witnesses':witnesses,'remaining':'Independent MVS, complete modfile and module identity/version validation, ZIP/sum/graph/root/compiler/context authority, full source/export/program semantics, all56 foundation/all169 demands, native Linux gates and T02-T11 remain due.','full_T_gate':'Deferred to final T02-W04: ./scripts/check-fast.sh'}
    output.write_text(json.dumps(result,indent=2)+'\n');print(json.dumps({k:result[k] for k in ['status','source_epochs','owners','affected_rust_roots','coherent_module_controls']}))
if __name__=='__main__':main()
