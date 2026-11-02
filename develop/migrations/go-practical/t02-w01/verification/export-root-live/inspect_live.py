"""Independently rehash and connect the actual live export and complete application evidence."""
from pathlib import Path
import hashlib
import importlib.util
import json
import sys

ROOT=Path(__file__).resolve().parent
REPO=ROOT.parents[5]
sys.dont_write_bytecode=True
spec=importlib.util.spec_from_file_location('go_snapshot_prior',ROOT.parent/'analysis-namespace/inspect_snapshot.py');prior=importlib.util.module_from_spec(spec);spec.loader.exec_module(prior)


def seal(path):
    raw=path.read_bytes();return {'path':str(path.resolve()),'bytes':len(raw),'sha256':hashlib.sha256(raw).hexdigest()}


def wire(packet):
    raw=bytes.fromhex(packet['snapshot_hex']);data=json.loads(raw)
    assert prior.canonical(data)==raw and prior.native_digest(data)==data['snapshot_sha256']
    body={k:v for k,v in data.items() if k!='content_sha256'}
    assert hashlib.sha256(b'MPK-PRIVATE-GO-SOURCE-SNAPSHOT-INPUTS-1.0\0'+prior.canonical(body)).hexdigest()==data['content_sha256']
    return data


def main():
    run=ROOT/'live-original';report=json.loads((run/'result.json').read_bytes());assert report['status']=='passed' and report['original_inputs_unchanged'] and report['registered_sdk_unchanged']
    evidence=Path(json.loads((run/'original-evidence-manifest.json').read_bytes())['root'])/'versioned-owned'
    current=json.loads((run/'coord/snapshot-ready.json').read_bytes());full=json.loads((evidence/'complete-original-module-graph-inputs.json').read_bytes());a=wire(current);b=wire(full)
    caps=set(a['capabilities']);assert caps=={'original-go-package-exports'} and set(b['capabilities'])==caps|{'workspace/main','workspace/dep','original-go-sdk'}
    assert a['directories']==[d for d in b['directories'] if d['capability'] in caps]
    assert a['objects']==[o for o in b['objects'] if o['capability'] in caps]
    assert len(a['objects'])==181 and len(b['objects'])==5697
    assert all(o['kind']=='export_data' for o in b['objects'] if o['capability']=='original-go-package-exports')
    rows={r['object']['id']:r for r in full['snapshot_sources']};assert len(rows)==len(b['objects']) and len(current['snapshot_sources'])==len(a['objects'])
    total=0
    for obj in b['objects']:
        row=rows[obj['id']];assert row['object']==obj and row['raw_file']==obj['id']+'.raw'
        raw=(evidence/'original-source-objects'/row['raw_file']).read_bytes();assert len(raw)==obj['bytes'] and hashlib.sha256(raw).hexdigest()==obj['raw_sha256']
        identity={k:v for k,v in obj.items() if k!='id'};assert hashlib.sha256(b'MPK-GO-SOURCE-OBJECT-1.0\0'+prior.canonical(identity)).hexdigest()==obj['id'];total+=len(raw)
    assert all(rows[row['object']['id']]==row for row in current['snapshot_sources'])
    compiler=json.loads(bytes.fromhex(full['compiler_inputs_hex']));release=json.loads((REPO/'release/bundles/candidates/go.json').read_bytes())['toolchain_bundles'][0]
    assert compiler['inventory']==release['inventory'] and compiler['host']=='linux_amd64' and len(full['compiler_files'])==5577
    compiler_bytes=0
    for row,expected in zip(full['compiler_files'],release['inventory']['files']):
        assert row['entry']==expected and row['raw_file']==expected['sha256']+'.raw'
        raw=(evidence/'original-compiler-files'/row['raw_file']).read_bytes();assert len(raw)==expected['size_bytes'] and hashlib.sha256(raw).hexdigest()==expected['sha256'];compiler_bytes+=len(raw)
    assert compiler_bytes==166354960
    selection=json.loads(bytes.fromhex(full['selection_hex']));exports=json.loads(bytes.fromhex(full['export_inputs_hex']))
    assert len(selection['roots'])==1 and len(full['overlays'])==0 and len(exports['packages'])==182
    result=report['matched'];assert result['status']=='passed' and result['opened_before_capture'] and result['original_source_objects']==181 and result['native_snapshot_sha256']==a['snapshot_sha256'] and not result['authorized_launcher_claimed'] and not result['application_proof_claimed']
    start=json.loads((run/'coord/roots-ready.json').read_bytes());opened=json.loads((run/'coord/opened.json').read_bytes());assert start['capture_started'] is False and len(start['roots'])==1 and start['compiler_output_prepared'] is True and opened['status']=='passed' and opened['capabilities']==1 and opened['opened_before_capture'] is True
    output=(run/'go.stdout').read_text();assert output.index('independent observer mutated only copied export inputs')<output.index('--- PASS: TestPracticalInstalledCompilerInputsKeepWholeRegisteredSDKAndPackages') and current['copied_input_mutation_probe'] is True
    storage=json.loads((run/'evidence-storage.json').read_bytes());restore=json.loads((run/'restore-check.json').read_bytes());assert storage['status']=='passed' and restore['status']=='passed' and storage['original_files']==restore['restored_files']
    audit={'status':'passed','selection_reason':'Recompute every native snapshot framing, source object identity and complete SDK byte; compare the pre-capture-held live export snapshot with the corresponding capability in the actual complete application/export owner.','live_result':seal(run/'result.json'),'snapshot_packet':seal(run/'coord/snapshot-ready.json'),'complete_application_packet':seal(evidence/'complete-original-module-graph-inputs.json'),'evidence_storage':seal(run/'evidence-storage.json'),'restoration':seal(run/'restore-check.json'),'root_matched_export_objects':len(a['objects']),'root_matched_directories':len(a['directories']),'root_matched_entries':sum(len(d['entries']) for d in a['directories']),'complete_application_source_objects':5697,'other_original_source_objects':len(b['objects'])-len(a['objects']),'all_full_source_object_bytes_and_ids_recomputed':True,'total_source_bytes':total,'registered_sdk_files':5577,'registered_sdk_bytes':compiler_bytes,'all_registered_sdk_files_rehashed':True,'original_packages':182,'original_root_selections':1,'original_contract_overlays':0,'launcher_authority_proved':False,'live_export_root_matched_at_capture':True,'production_export_root_authorized':False,'application_proof_claimed':False,'native_Linux_release_acceptance':False,'full_T_gate':'Deferred to final T02-W04: ./scripts/check-fast.sh'}
    out=run/'independent-inspection.json';assert not out.exists();out.write_text(json.dumps(audit,indent=2)+'\n');print(json.dumps({k:audit[k] for k in ['status','root_matched_export_objects','complete_application_source_objects','registered_sdk_files','original_packages']}),flush=True)


if __name__=='__main__':main()
