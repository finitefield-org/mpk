"""Independently compare both full native casefold captures and original inputs."""
from pathlib import Path
import hashlib
import json

ROOT=Path(__file__).resolve().parent;VERIFY=ROOT.parent


def sha(data):return hashlib.sha256(data).hexdigest()
def read(root):
    packet=json.loads((root/'native.json').read_bytes());items={}
    for row in packet['original_sources']:
        obj=row['object'];raw=(root/'original-bytes'/row['raw_file']).read_bytes()
        assert obj['id'] not in items and len(raw)==obj['bytes'] and sha(raw)==obj['raw_sha256']
        items[obj['id']]=(obj,raw)
    assert set(items)==set(packet['source_ids'])
    return packet,items,json.loads(bytes.fromhex(packet['snapshot_hex']))


def main():
    initial=json.loads((ROOT/'casefold-original/result.json').read_bytes());assert initial['status']=='failed'
    assert initial['observation']['go_plan_uses_profile'] and not initial['observation']['profile_captured']
    fixed=json.loads((ROOT/'casefold-fixed/result.json').read_bytes());assert fixed['status']=='passed' and fixed['observation']['profile_captured']
    base=VERIFY/'registered-analysis/native-fixed';original,base_items,_=read(base)
    folded,folded_items,folded_snapshot=read(ROOT/'folded-final');sensitive,sensitive_items,sensitive_snapshot=read(ROOT/'sensitive-final')
    assert len(base_items)==196 and len(original['build_objects'])==184 and sensitive_items==base_items
    assert folded_snapshot['directories']==sensitive_snapshot['directories']
    extra=set(folded_items)-set(base_items);assert len(extra)==1 and all(folded_items[id]==value for id,value in base_items.items())
    obj,profile=folded_items[next(iter(extra))];assert obj['path']=='DEFAULT.PGO' and obj['kind']=='auxiliary'
    sdk=Path('/private/tmp/mpk-installed-compiler-205-original-go-output/toolchain/go/src/cmd/compile/default.pgo').read_bytes()
    assert profile==sdk==(ROOT/'folded-final/original-profile.raw').read_bytes()==(ROOT/'sensitive-final/original-profile.raw').read_bytes()
    assert len(profile)==293923 and sha(profile)=='d7355ec9e3c8bf05e214b202174ede4205092f852124e6f7f7287e22f6b5d980'
    prior={row['id']:row for row in original['build_objects']};plain={row['id']:row for row in sensitive['build_objects']};added={row['id']:row for row in folded['build_objects']}
    assert prior==plain and set(added)==set(prior)|extra and all(added[k]==v for k,v in prior.items())
    assert folded['go_lookup_resolves'] is True and sensitive['go_lookup_resolves'] is False
    assert 'default.pgo' in (ROOT/'folded-final/plan.log').read_text() and 'default.pgo' not in (ROOT/'sensitive-final/plan.log').read_text()
    result={'status':'passed','selection_reason':'Rehash and compare all captured objects, complete source bytes, directories and build inputs on both actual filesystem behaviors against the full registered source and SDK.','original_source_objects':196,'original_build_inputs':184,'case_sensitive_objects':len(sensitive_items),'case_sensitive_build_inputs':len(plain),'casefold_objects':len(folded_items),'casefold_build_inputs':len(added),'directories_identical':True,'every_original_object_and_byte_equal':True,'every_original_build_input_equal':True,'actual_profile_path':obj['path'],'actual_profile_bytes':len(profile),'actual_profile_sha256':sha(profile),'initial_real_omission_reproduced':True,'real_go_plans_agree_with_lookup':True,'trusted_launcher_authority':False,'native_Linux_release_acceptance':False,'full_T_gate':'Deferred to final T02-W04: ./scripts/check-fast.sh'}
    out=ROOT/'inspection.json';assert not out.exists();out.write_text(json.dumps(result,indent=2)+'\n');print(json.dumps({'status':'passed','original_objects':len(base_items),'casefold_extra_objects':len(extra)}),flush=True)

if __name__=='__main__':main()
