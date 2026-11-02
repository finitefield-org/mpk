"""Inspect complete original compiler metadata, buffers and resealed controls."""
from pathlib import Path
import hashlib,json,re
ROOT=Path(__file__).resolve().parent
INPUTS={}
def seal(p):
    raw=p.read_bytes();return {'path':str(p),'bytes':len(raw),'sha256':hashlib.sha256(raw).hexdigest(),'mode':p.stat().st_mode&0o777}
def raw(p):INPUTS[str(p)]=seal(p);return p.read_bytes()
def read(p):return json.loads(raw(p))
def events(name):return [json.loads(l) for l in raw(ROOT/name/'stdout.log').decode().splitlines()]
def main():
    output=ROOT/'inspection.json';assert not output.exists()
    epochs={p.parent.name:read(p) for p in ROOT.glob('*/result.json')};assert len(epochs)==10
    assert epochs['native']['status']=='failed' and epochs['native']['consumed_inputs_unchanged']
    assert all(v['status']=='passed' and (v['consumed_inputs_unchanged'] or v['expected_formatter_mutation']) for k,v in epochs.items() if k!='native')
    first=events('native');assert any(r.get('Test')=='TestPracticalExportInputsKeepWholeOriginalCompilerUniverse' and r['Action']=='pass' for r in first)
    failures=[r for r in first if r['Action']=='fail' and r.get('Test')];assert [r['Test'] for r in failures]==['TestPracticalCaptureInputsKeepOriginalOwnedUniverse']
    assert any('file exists' in r.get('Output','') and '/owned' in r.get('Output','') for r in first)
    final=events('native-capture');assert any(r.get('Test')=='TestPracticalCaptureInputsKeepOriginalOwnedUniverse' and r['Action']=='pass' for r in final) and not any(r['Action']=='fail' for r in final)
    assert (ROOT/'go-format-check/stdout.log').stat().st_size==0
    directory=ROOT/'native/native-originals/owned';packet=read(directory/'complete-original-export-inputs.json');wire=json.loads(bytes.fromhex(packet['export_inputs_hex']));ownership=read(directory/'original-export-ownership.json')
    original={};total=0;kinds={}
    for row in packet['snapshot_sources']:
        obj=row['object'];data=raw(directory/'original-source-objects'/row['raw_file']);assert len(data)==obj['bytes'] and hashlib.sha256(data).hexdigest()==obj['raw_sha256'];original[obj['id']]=(obj,data);total+=len(data);kinds[obj['kind']]=kinds.get(obj['kind'],0)+1
    assert len(original)==ownership['complete_original_objects']==5663 and len(wire['packages'])==ownership['packages']==180
    assert ownership['returned_buffers_independent'] and ownership['owner_retirement_keeps_handoff'] and ownership['closed_and_nil_owners_rejected'] and not ownership['original_initialization_or_unused_body_executed']
    linked=read(ROOT/'rust-original/observations/exports/whole-original-export-linkage.json');assert linked['original_objects']==5663 and linked['original_selected_contract_functions']==1 and linked['metadata_hex']==wire['metadata_hex'] and linked['diagnostics_hex']==wire['diagnostics_hex'] and not linked['source_or_context_program_proof']
    decoder=json.JSONDecoder();text=bytes.fromhex(wire['metadata_hex']).decode();rows=[]
    while text.strip():row,end=decoder.raw_decode(text.lstrip());rows.append(row);text=text.lstrip()[end:]
    assert len(rows)==180 and len({r['ImportPath'] for r in rows})==180
    by_name={r['ImportPath']:r for r in rows};exports={p['package']:p for p in linked['exports']};assert len(exports)==179 and 'unsafe' not in exports and 'net/http' in exports and any(p.startswith('vendor/') for p in exports)
    archive_bytes=0
    for package in wire['packages']:
        row=by_name[package['import_path']];id=package['export_source_object_id']
        if id is None:assert package['import_path']=='unsafe' and not row.get('Export') and not row.get('BuildID');continue
        obj,data=original[id];assert exports[package['import_path']]['object']==obj and exports[package['import_path']]['bytes']==len(data)
        assert data.startswith(b'!<arch>\n') and data[8:24].strip()==b'__.PKGDEF';size=int(data[56:66].strip());body=data[68:68+size];header,build,*_=body.split(b'\n',2)
        assert header.startswith(b'go object linux amd64 go1.25.0 GOAMD64=v1 X:') and json.loads(build.removeprefix(b'build id '))==row['BuildID'];archive_bytes+=len(data)
    controls=read(ROOT/'rust-original/observations/exports/coherent-native-export-controls.json');assert len(controls['cases'])==21 and controls['original_input_unchanged']
    for row in controls['cases']:changed=json.loads(bytes.fromhex(row['complete_changed_export_inputs_hex']));assert changed!=wire and row['error']
    archives=read(ROOT/'rust-original/observations/exports/coherent-original-archive-controls.json');assert len(archives['cases'])==4 and archives['original_objects']==5663
    original_capture=json.loads(bytes.fromhex(packet['capture_hex']))
    for row in archives['cases']:
        data=bytes.fromhex(row['complete_changed_archive_hex']);obj=row['changed_object'];assert len(data)==obj['bytes'] and hashlib.sha256(data).hexdigest()==obj['raw_sha256'] and row['same_original_source_selection_and_root_contract']
        changed_capture=json.loads(bytes.fromhex(row['changed_capture_hex']));changed_exports=json.loads(bytes.fromhex(row['changed_export_hex']));assert changed_capture['selection_inputs_sha256']==original_capture['selection_inputs_sha256'] and changed_exports['source_capture_inputs_sha256']==changed_capture['content_sha256']
    proofs=read(ROOT/'rust-original/observations/sources/original-proof-replay.json')['complete_original_call_proofs'];assert len(proofs)==32
    for row in proofs:data=raw(Path(row['original_certificate']));assert len(data)==row['bytes'] and hashlib.sha256(data).hexdigest()==row['raw_sha256']
    units=read(ROOT/'rust-original/observations/snapshots/all-original-capture-units.json')['units'];assert len(units)==2 and all(r['original_roots']==43 and r['all_source_components_and_bytes_equal'] for r in units)
    counts=[int(n) for n in re.findall(r'test result: ok\. (\d+) passed; 0 failed; 0 ignored;',raw(ROOT/'rust-original/stdout.log').decode())];assert counts==[6,3,9,4]
    witnesses=[];sdk=Path('/private/tmp/mpk-go125-20261025-download/go')
    for name in ['src/cmd/go/internal/load/pkg.go','src/cmd/internal/archive/archive.go','src/cmd/internal/buildid/buildid.go']:
        obj,data=next((o,b) for o,b in original.values() if o['path']==name);assert data==(sdk/name).read_bytes();witnesses.append({'path':name,'object':obj,'raw_hex':data.hex()})
    result={'status':'passed','driver':seal(Path(__file__).resolve()),'inputs':list(INPUTS.values()),'source_epochs':10,'original_failed_harness_epoch_retained':True,'failure':'The second original capture test attempted the same exclusive owned output directory after the new export test passed; no product assertion or source/compiler failure. Its unchanged test then passed under a separate evidence directory.','actual_native_passed_roots':['TestPracticalExportInputsKeepWholeOriginalCompilerUniverse','TestPracticalCaptureInputsKeepOriginalOwnedUniverse'],'affected_rust_roots':sum(counts),'original_packages':180,'original_export_archives':179,'original_source_objects':5663,'original_source_bytes':total,'original_export_bytes':archive_bytes,'original_source_kinds':kinds,'same_original_root_contracts':1,'metadata_control_cases':21,'coherent_archive_control_cases':4,'original_proof_byte_streams':32,'original_roots_per_unit':43,'original_error_case_universes':sorted({r['error_cases'] for r in proofs}),'complete_original_source_witnesses':witnesses,'remaining':'Independent compiler-image/stdlib/analysis inventory and root/module/context/descriptor/registry authority, complete source/export semantics and all source/IR/state/storage/frame/callee/program proofs, all56 foundation families/all169 demands, native Linux gates and remaining T02-T11 remain due.','full_T_gate':'Deferred to final T02-W04: ./scripts/check-fast.sh'}
    output.write_text(json.dumps(result,indent=2)+'\n');print(json.dumps({k:result[k] for k in ['status','source_epochs','original_packages','original_export_archives','original_source_objects','original_source_bytes','affected_rust_roots']}))
if __name__=='__main__':main()
