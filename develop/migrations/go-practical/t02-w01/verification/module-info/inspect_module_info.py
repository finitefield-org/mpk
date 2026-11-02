"""Audit the full original application owners and Go/Rust info observations."""
from pathlib import Path
import hashlib
import importlib.util
import json
import sys

ROOT=Path(__file__).resolve().parent
sys.dont_write_bytecode=True
spec=importlib.util.spec_from_file_location('prior_snapshot_inspection', ROOT.parent/'analysis-namespace/inspect_snapshot.py')
prior=importlib.util.module_from_spec(spec);spec.loader.exec_module(prior)
sys.path.insert(0,str(ROOT.parent/'installed-compiler'))
from run_scoped import seal


def main():
    packet=json.loads((ROOT/'original-inputs.json').read_bytes());assert packet['status']=='passed' and packet['file_count']==22323
    for row in packet['files']:
        assert seal(Path(row['path']))==row
    native=json.loads((ROOT/'native/native.json').read_bytes())
    assert native['original_source_objects']==5490 and native['original_root_or_vcs_authority'] is False
    names={};used=set()
    for row in native['observations']:
        rec=row['source'];obj=rec['object'];raw=(ROOT/'native/observed-bytes'/rec['raw_file']).read_bytes()
        assert obj['capability']=='workspace/dep' and obj['path']==native['original_info_path'] and obj['kind']=='metadata'
        assert len(raw)==obj['bytes'] and hashlib.sha256(raw).hexdigest()==obj['raw_sha256'] and rec['raw_file']==obj['raw_sha256']+'.raw'
        identity={k:v for k,v in obj.items() if k!='id'}
        assert hashlib.sha256(b'MPK-GO-SOURCE-OBJECT-1.0\0'+prior.canonical(identity)).hexdigest()==obj['id']
        assert row['name'] not in names;names[row['name']]=row;used.add(rec['raw_file'])
    assert len(names)==14 and set(p.name for p in (ROOT/'native/observed-bytes').glob('*.raw'))==used
    assert names['original']['source']['object']['id']==native['original_source_id']
    assert names['lowercase-version']['accepted'] and names['unicode-version']['accepted']
    assert not names['time-number']['accepted'] and not names['time-invalid']['accepted'] and not names['origin-invalid']['accepted']
    results=json.loads((ROOT/'generic-tests/observations/original-application-module-info-controls.json').read_bytes())
    assert results['status']=='passed' and results['original_application_and_sdk_sources_preserved'] and not results['root_or_vcs_authority']
    for row in results['cases']:
        original=names[row['case']];assert row['accepted']==original['accepted']
        assert row['changed_info']==original['source']['object'] and row['original_info_id']==native['original_source_id']
        assert row['original_source_objects']==5490 and row['original_root_contracts']==1
    assert len(results['cases'])==len(names)
    for name in ['prelaunch','native','go-format-write','go-checks','rust-format','generic-check','generic-clippy','consumer-tests','owner-tests','rebind-tests','generic-tests']:
        result=json.loads((ROOT/name/'result.json').read_bytes())
        assert result['status']=='passed' and result.get('consumed_inputs_unchanged',True) and result.get('complete_original_test_inputs_unchanged',True)
    output=(ROOT/'consumer-tests/stdout.log').read_text()
    assert output.count('test result: ok. 2 passed;')==3 and 'test result: ok. 1 passed;' in output
    for name in ['owner-tests','rebind-tests','generic-tests']:
        assert 'test result: ok. 1 passed;' in (ROOT/name/'stdout.log').read_text()
    result={'status':'passed','driver':seal(Path(__file__).resolve()),'native':seal(ROOT/'native/native.json'),'rust_application_controls':seal(ROOT/'generic-tests/observations/original-application-module-info-controls.json'),'original_application_and_sdk_files':len(packet['files']),'original_application_objects':5490,'original_root_contracts':1,'observations':{name:row['accepted'] for name,row in names.items()},'observations_count':len(names),'accepted':sum(row['accepted'] for row in names.values()),'rejected':sum(not row['accepted'] for row in names.values()),'all_original_application_and_sdk_files_rehashed':True,'all_original_info_and_control_identities_recomputed':True,'original_root_or_vcs_authenticated':False,'source_or_context_proof_claimed':False,'native_Linux_release_acceptance':False,'whole_repository_gate_run':False,'full_T_gate':'Deferred to final T02-W04: ./scripts/check-fast.sh'}
    out=ROOT/'inspection.json';assert not out.exists();out.write_text(json.dumps(result,indent=2)+'\n')
    print(json.dumps({k:result[k] for k in ['status','original_application_and_sdk_files','original_application_objects','observations_count','accepted','rejected']}))


if __name__=='__main__':main()
