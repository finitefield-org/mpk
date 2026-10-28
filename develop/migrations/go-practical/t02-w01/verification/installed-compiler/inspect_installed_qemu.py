"""Independently inspect the complete installed-root observations and controls."""
from pathlib import Path
import hashlib
import json
import sys

ROOT = Path(__file__).resolve().parent
REPO = ROOT.parents[5]
sys.dont_write_bytecode = True
sys.path.insert(0, str(ROOT))
from run_scoped import seal, write
sys.path.insert(0, str(REPO/'scripts'))
import successor_release_bundles as release


def main():
    out=ROOT/'installed-qemu-inspection.json'
    assert not out.exists()
    run=ROOT/'installed-qemu-binding'
    result=json.loads((run/'result.json').read_bytes())
    assert result['status']=='passed' and result['original_inputs_unchanged'] and result['installed_registry_and_sdk_restored']
    assert not result['native_Linux_release_acceptance']
    commands=json.loads((run/'commands.json').read_bytes())
    expected=['uninstalled-executable','original-snapshot','original-binding','coherent-wrong-host','crossed-darwin-owner','changed-installed-sdk-byte','missing-rust-namespace-member','alternate-registry-validates','alternate-valid-registry-pin']
    assert [row['name'] for row in commands]==expected and result['checks']==expected
    observations={}
    for row in commands:
        assert row['exit_code']==0 and row['stdout']==seal(row['stdout']['path']) and row['stderr']==seal(row['stderr']['path'])
        assert Path(row['stderr']['path']).read_bytes()==b''
        observations[row['name']]=json.loads(Path(row['stdout']['path']).read_bytes())
    registry,candidates=release.validate_release_models()
    go=candidates['go']['toolchain_bundles'][0]
    snapshot=observations['original-snapshot']
    bound=observations['original-binding']
    assert snapshot['status']=='accepted' and snapshot['inventory']==go['inventory']
    assert bound['status']=='accepted' and bound['release_registry']['registry_sha256']==registry['registry_sha256']
    assert bound['files']==5577 and bound['bytes']==166354960 and bound['sdk_sources']==5480 and bound['sdk_source_directories']==715
    assert bound['original_objects']==5697 and bound['original_packages']==182 and bound['compiler_record']['inventory']==go['inventory']
    for name in ['uninstalled-executable','changed-installed-sdk-byte','missing-rust-namespace-member','alternate-valid-registry-pin']:
        assert observations[name]=={'status':'rejected','phase':'installed','error':'Release'}
    for name in ['coherent-wrong-host','crossed-darwin-owner']:
        assert observations[name]=={'status':'rejected','phase':'binding','error':'Inputs'}
    original_wire=bound['compiler_record']
    changed=json.loads((run/'changed-host.json').read_bytes())
    assert changed['host']=='darwin_arm64' and original_wire['host']=='linux_amd64'
    assert {k:v for k,v in changed.items() if k not in ['host','content_sha256']}=={k:v for k,v in original_wire.items() if k not in ['host','content_sha256']}
    assert hashlib.sha256(b'MPK-PRIVATE-GO-COMPILER-IMAGE-INPUTS-1.0\0'+release.canonical({k:v for k,v in changed.items() if k!='content_sha256'})).hexdigest()==changed['content_sha256']
    alternate=json.loads((run/'alternate-registry.json').read_bytes())
    assert observations['alternate-registry-validates']=={'status':'accepted','phase':'registry-validation','registry_sha256':alternate['registry_sha256']}
    assert alternate['registry_sha256'] != registry['registry_sha256']
    assert hashlib.sha256(release.REGISTRY_DOMAIN+release.canonical({k:v for k,v in alternate.items() if k!='registry_sha256'})).hexdigest()==alternate['registry_sha256']
    assert alternate['frontend_bundles']==registry['frontend_bundles'] and alternate['tuples']==registry['tuples']
    original_rust=next(x for x in registry['toolchain_bundles'] if x['bundle_id'].startswith('toolchain.rust.'))
    assert original_rust==next(x for x in alternate['toolchain_bundles'] if x['bundle_id']==original_rust['bundle_id'])
    changed_go=next(x for x in alternate['toolchain_bundles'] if x['bundle_id']==go['bundle_id'])
    changed_file=next(x for x in changed_go['inventory']['files'] if x['path']=='go/bin/go')
    actual=seal(ROOT/'alternate-registry-validation/changed-go-binary')
    assert changed_file['sha256']==actual['sha256'] and changed_file['size_bytes']==actual['bytes']
    assert next(x for x in changed_go['components'] if x['name']=='go')['binary_sha256']==actual['sha256']
    assert hashlib.sha256(release.CONTENT_DOMAIN+release.canonical(changed_go['inventory'])).hexdigest()==changed_go['distribution_sha256']
    rust=json.loads((ROOT/'rust-finalized-bundles/result.json').read_bytes())
    assert rust['status']=='passed' and rust['consumed_inputs_unchanged']
    assert rust['complete_toolchain']==candidates['rust']['toolchain_bundles'][0]['inventory']
    assert rust['complete_frontend']==candidates['rust']['frontend_bundles'][0]['inventory']
    root_check=json.loads((ROOT/'whole-snapshot-diagnostic/result.json').read_bytes())
    assert root_check['status']=='passed' and root_check['whole_originals_unchanged'] and root_check['observation']['same_selected_original_source_and_root']
    summary={'status':'passed','driver':seal(__file__),'installed_result':seal(run/'result.json'),'commands':seal(run/'commands.json'),'go_toolchain_files':5577,'go_toolchain_bytes':166354960,'rust_toolchain_files':len(rust['complete_toolchain']['files']),'rust_toolchain_bytes':sum(x['size_bytes'] for x in rust['complete_toolchain']['files']),'original_packages':182,'original_source_objects':5697,'root_selections':1,'contract_overlays':0,'controls':expected,'actual_installed_root_exercised':True,'local_linux_emulation_only':True,'emulator':'QEMU 10.0.2 user mode on Linux arm64','native_Linux_release_acceptance':False,'public_practical_profile_activated':False,'whole_repository_gate_run':False,'full_T_gate':'Deferred to final T02-W04: ./scripts/check-fast.sh'}
    write(out,summary)
    print(json.dumps({k:summary[k] for k in ['status','go_toolchain_files','rust_toolchain_files','original_packages','controls']}))


if __name__=='__main__':
    main()
