"""Compare complete source and reproducible binary epochs independently."""
from pathlib import Path
import hashlib
import json

ROOT=Path(__file__).resolve().parent;VERIFY=ROOT.parent


def source_rows(packet,directory):
    rows={}
    for entry in packet['original_sources']:
        obj=entry['object'];raw=(directory/'original-bytes'/entry['raw_file']).read_bytes()
        assert len(raw)==obj['bytes'] and hashlib.sha256(raw).hexdigest()==obj['raw_sha256'] and obj['id'] not in rows
        rows[obj['id']]=(obj,raw)
    return rows


def normalize(command):
    result=[]
    for value in command:
        if value.startswith('type=bind,src=') and ',dst=/source,readonly' in value:result.append('type=bind,src=<restored-source>,dst=/source,readonly')
        elif value.startswith('type=bind,src=') and ',dst=/out' in value:result.append('type=bind,src=<output>,dst=/out')
        else:result.append(value)
    return result


def main():
    old_dir=VERIFY/'registered-analysis/native-fixed';new_dir=VERIFY/'analysis-namespace/native'
    old=json.loads((old_dir/'native.json').read_bytes());current=json.loads((new_dir/'native.json').read_bytes());new_list=json.loads((VERIFY/'analysis-build-list/native/native.json').read_bytes())
    old_sources=source_rows(old,old_dir);new_sources=source_rows(current,new_dir);assert len(old_sources)==196 and len(new_sources)==213
    assert set(old['source_ids'])==set(old_sources) and set(new_list['source_ids'])==set(new_sources)
    old_snapshot=json.loads(bytes.fromhex(old['snapshot_hex']));new_snapshot=json.loads(bytes.fromhex(new_list['snapshot_hex']))
    assert {o['id'] for o in old_snapshot['objects']}==set(old_sources) and {o['id'] for o in new_snapshot['objects']}==set(new_sources)
    for captured in (old_snapshot,new_snapshot):
        top=next(d for d in captured['directories'] if d['capability']=='analysis-frontend' and d['path']=='')
        assert b'default.pgo' not in [bytes.fromhex(e['name_hex']) for e in top['entries']]
    def build(packet,sources):
        rows={}
        for obj in packet['build_objects']:
            assert obj['id'] in sources and sources[obj['id']][0]==obj and obj['path'] not in rows
            rows[obj['path']]=sources[obj['id']]
        return rows
    a=build(old,old_sources);b=build(new_list,new_sources);assert len(a)==184 and len(b)==201
    common=set(a)&set(b);same=[];changed=[]
    def detail(path,item):return {'path':path,'object_id':item[0]['id'],'sha256':item[0]['raw_sha256'],'bytes':len(item[1]),'kind':item[0]['kind']}
    for path in sorted(common):
        assert a[path][0]['capability']==b[path][0]['capability'] and a[path][0]['kind']==b[path][0]['kind']
        if a[path][1]==b[path][1]:
            assert a[path][0]==b[path][0];same.append(path)
        else:changed.append({'path':path,'registered':detail(path,a[path]),'later':detail(path,b[path])})
    added=[detail(path,b[path]) for path in sorted(set(b)-set(a))];removed=[detail(path,a[path]) for path in sorted(set(a)-set(b))]
    assert (len(same),len(changed),len(added),len(removed))==(178,6,17,0)
    previous=VERIFY/'registered-analysis/rebuild-original';latest=ROOT/'same-flags-original'
    previous_result=json.loads((previous/'result.json').read_bytes());latest_result=json.loads((latest/'result.json').read_bytes());assert previous_result['status']==latest_result['status']=='passed'
    previous_commands=json.loads((previous/'commands.json').read_bytes());latest_commands=json.loads((latest/'commands.json').read_bytes())
    for index in (1,3):assert normalize(previous_commands[index]['command'])==normalize(latest_commands[index]['command'])
    binary_groups=[]
    for folder in (previous,latest):
        originals=[bytes.fromhex((folder/f'frontend-{i}.elf.hex').read_text()) for i in range(2)];assert originals[0]==originals[1]
        binary_groups.append(originals[0])
    cross_equal=binary_groups[0]==binary_groups[1]
    hashes=[hashlib.sha256(value).hexdigest() for value in binary_groups]
    assert hashes[0]=='ccc4a984d41b5da7b3a6838cbfaeee3eb23f675a2e4d7fb5d4f9fcd7e597d538'
    result={'status':'passed','selection_reason':'Independently compare every original captured source buffer and build object across both complete epochs, then check both independent full binary streams under exactly the same normalized build command.','registered_objects':len(old_sources),'later_objects':len(new_sources),'registered_build_objects':len(a),'later_build_objects':len(b),'common_byte_equal':len(same),'common_changed':changed,'added':added,'removed':removed,'all_original_buffers_rehashed':True,'normalized_build_commands_exactly_equal':True,'default_pgo_absent_in_both_observed_frontend_roots':True,'each_epoch_has_two_identical_complete_binaries':True,'cross_epoch_binaries_equal':cross_equal,'registered_binary':{'bytes':len(binary_groups[0]),'sha256':hashes[0]},'later_binary':{'bytes':len(binary_groups[1]),'sha256':hashes[1]},'specific_causal_source_change_proved':False,'current_live_checkout_identified':False,'trusted_launcher_authority_proved':False,'application_proof_proved':False,'native_Linux_release_acceptance':False,'full_T_gate':'Deferred to final T02-W04: ./scripts/check-fast.sh'}
    out=ROOT/'inspection.json';assert not out.exists();out.write_text(json.dumps(result,indent=2)+'\n');print(json.dumps({'status':result['status'],'same':len(same),'changed':len(changed),'added':len(added),'removed':len(removed),'normalized_commands_equal':True,'cross_epoch_binaries_equal':cross_equal,'later_binary':result['later_binary']}),flush=True)


if __name__=='__main__':main()
