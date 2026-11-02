"""Compare complete capture before and after temporary-root normalization."""
from pathlib import Path
import hashlib
import json

ROOT=Path(__file__).resolve().parent
runs=[ROOT/'native-fixed',ROOT/'native-after-path']
for run in runs:assert json.loads((run/'result.json').read_bytes())['status']=='passed'
packets=[(run/'native.json').read_bytes() for run in runs];assert packets[0]==packets[1]
files=[{p.name:p for p in (run/'original-bytes').iterdir() if p.is_file()} for run in runs]
assert files[0].keys()==files[1].keys()
rows=[]
for name in sorted(files[0]):
    first=files[0][name].read_bytes();second=files[1][name].read_bytes();assert first==second
    rows.append({'name':name,'bytes':len(first),'sha256':hashlib.sha256(first).hexdigest()})
packet=json.loads(packets[0]);assert len(packet['original_sources'])==196 and len(packet['build_objects'])==184
result={'status':'passed','selection_reason':'Verify the corrected platform path handling reproduces the complete prior Go snapshot, all original source buffers, and all build inputs exactly; unchanged Rust and held-root consumers need not be repeated.','whole_native_packet_bytes_equal':True,'native_packet_sha256':hashlib.sha256(packets[0]).hexdigest(),'all_original_source_buffers_equal':True,'unique_raw_files':len(rows),'original_objects':196,'build_objects':184,'files':rows,'full_T_gate':'Deferred to final T02-W04: ./scripts/check-fast.sh'}
out=ROOT/'native-epochs-equal.json';assert not out.exists();out.write_text(json.dumps(result,indent=2)+'\n');print(json.dumps({k:result[k] for k in ['status','whole_native_packet_bytes_equal','all_original_source_buffers_equal','unique_raw_files']}))
