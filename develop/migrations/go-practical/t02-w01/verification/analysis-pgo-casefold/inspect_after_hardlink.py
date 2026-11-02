"""Compare complete original native observations after spelling selection repair."""
from pathlib import Path
import hashlib
import json

ROOT=Path(__file__).resolve().parent;PRIOR=ROOT.parent/'analysis-pgo'

def read(path):
    packet=json.loads((path/'native.json').read_bytes());sources={}
    for row in packet['original_sources']:
        obj=row['object'];raw=(path/'original-bytes'/row['raw_file']).read_bytes()
        assert len(raw)==obj['bytes'] and hashlib.sha256(raw).hexdigest()==obj['raw_sha256'] and obj['id'] not in sources
        sources[obj['id']]=(obj,raw)
    snapshot=json.loads(bytes.fromhex(packet['snapshot_hex']))
    return packet,sources,snapshot

pairs=[(ROOT/'sensitive-final',ROOT/'sensitive-hardlink-final'),(ROOT/'folded-final',ROOT/'folded-hardlink-final'),(ROOT/'crossed-final',ROOT/'crossed-hardlink-final'),(PRIOR/'native-casefold-final',PRIOR/'native-hardlink-final')]
for name in ['regular','directory','symlink','oversized','absent']:pairs.append((PRIOR/'native-casefold-final/controls'/name,PRIOR/'native-hardlink-final/controls'/name))
rows=[]
for earlier,later in pairs:
    old,old_sources,old_snapshot=read(earlier);new,new_sources,new_snapshot=read(later)
    assert old_sources==new_sources and old_snapshot==new_snapshot and old['build_objects']==new['build_objects']
    rows.append({'before':str(earlier),'after':str(later),'complete_original_objects':len(old_sources),'build_inputs':len(old['build_objects']),'all_objects_bytes_and_namespace_equal':True})
before=json.loads((ROOT/'hardlink-before-fix/result.json').read_bytes());after=json.loads((ROOT/'hardlink-after-fix/result.json').read_bytes())
assert before['status']=='failed' and before['observation']['captured_native_profile_name']=='AAA.PGO'
assert after['status']=='passed' and after['observation']['captured_native_profile_name']=='DEFAULT.PGO'
assert before['observation']['profile_sha256']==after['observation']['profile_sha256'] and before['observation']['source_objects']==after['observation']['source_objects']==197
out=ROOT/'hardlink-inspection.json';assert not out.exists();out.write_text(json.dumps({'status':'passed','selection_reason':'Compare every original buffer, source identity, complete namespace and build input after fixing native spelling preference; preserve the unchanged actual inputs independently consumed by Rust.','comparisons':rows,'hardlink_before':before['observation'],'hardlink_after':after['observation'],'unrelated_inputs_narrowed':False,'rust_consumers_can_reuse_exact_unchanged_inputs':True,'native_Linux_release_acceptance':False,'full_T_gate':'Deferred to final T02-W04: ./scripts/check-fast.sh'},indent=2)+'\n');print(json.dumps({'status':'passed','complete_capture_pairs':len(rows),'hardlink_name_fixed':True}),flush=True)
