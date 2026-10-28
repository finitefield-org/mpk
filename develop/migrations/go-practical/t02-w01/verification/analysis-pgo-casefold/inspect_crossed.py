"""Compare complete original capture and the additional same-byte control."""
from pathlib import Path
import hashlib
import json

ROOT=Path(__file__).resolve().parent

def read(name):
    root=ROOT/name;packet=json.loads((root/'native.json').read_bytes());items={}
    for row in packet['original_sources']:
        obj=row['object'];raw=(root/'original-bytes'/row['raw_file']).read_bytes()
        assert obj['id'] not in items and len(raw)==obj['bytes'] and hashlib.sha256(raw).hexdigest()==obj['raw_sha256']
        items[obj['id']]=(obj,raw)
    return packet,items

folded,first=read('folded-final');crossed,second=read('crossed-final')
assert len(first)==197 and len(second)==198 and all(second[k]==v for k,v in first.items())
extra=set(second)-set(first);assert len(extra)==1
obj,raw=second[next(iter(extra))];assert obj['path']=='other.pgo' and obj['kind']=='auxiliary'
profile=(ROOT/'folded-final/original-profile.raw').read_bytes();assert raw==profile and len(profile)==293923
assert crossed['build_objects']==folded['build_objects'] and len(crossed['build_objects'])==185
result={'status':'passed','selection_reason':'Verify the extra same-byte profile preserves every original source object and build input, without replacing or stubbing any original.','folded_original_objects':197,'crossed_objects':198,'preserved_original_objects':197,'unchanged_build_inputs':185,'additional_path':'other.pgo','additional_bytes':len(raw),'additional_sha256':hashlib.sha256(raw).hexdigest(),'same_original_bytes':True,'additional_file_required_by_go':False,'native_Linux_release_acceptance':False,'full_T_gate':'Deferred to final T02-W04: ./scripts/check-fast.sh'}
out=ROOT/'crossed-inspection.json';assert not out.exists();out.write_text(json.dumps(result,indent=2)+'\n');print(json.dumps({'status':'passed','preserved_objects':197,'additional_original_bytes':len(raw)}),flush=True)
