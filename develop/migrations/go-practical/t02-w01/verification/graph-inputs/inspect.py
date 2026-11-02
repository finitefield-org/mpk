"""Inspect original cache ownership and byte-exact private v2 graph identities."""
from pathlib import Path
import base64, hashlib, json, posixpath

ROOT=Path(__file__).resolve().parent
INPUTS={}
def seal(path):
 path=Path(path);data=path.read_bytes()
 return {'path':str(path),'bytes':len(data),'sha256':hashlib.sha256(data).hexdigest(),'mode':path.stat().st_mode&0o777}
def raw(path):
 path=Path(path);INPUTS[str(path)]=seal(path);return path.read_bytes()
def read(path):return json.loads(raw(path))
def canonical(value):return json.dumps(value,sort_keys=True,ensure_ascii=False,separators=(',',':')).encode()
def typed(domain,value):return hashlib.sha256(domain.encode()+b'\0'+canonical(value)).hexdigest()
def object_bytes(obj,data):
 assert len(data)==obj['bytes'] and hashlib.sha256(data).hexdigest()==obj['raw_sha256']
 assert typed('MPK-GO-SOURCE-OBJECT-1.0',{k:v for k,v in obj.items() if k!='id'})==obj['id']
def part(packet,name,domain):
 data=bytes.fromhex(packet[name]);value=json.loads(data)
 assert canonical(value)==data
 assert typed(domain,{k:v for k,v in value.items() if k!='content_sha256'})==value['content_sha256']
 return value
def main():
 output=ROOT/'inspection.json';assert not output.exists()
 verification=read(ROOT/'verification.json')
 epochs={p.parent.name:read(p) for p in ROOT.glob('*/result.json')}
 assert sorted(epochs)==verification['source_epochs']
 for name,row in epochs.items():
  assert row['status']=='passed' and row['whole_sdk_unchanged']
  assert row['consumed_inputs_unchanged'] or row['expected_formatter_mutation']
  assert row['inputs']==seal(row['inputs']['path'])
  for field in ['stdout','stderr']:assert row[field]==seal(row[field]['path'])
 events=[json.loads(line) for line in raw(ROOT/'native/stdout.log').splitlines()]
 roots=[r for r in events if r.get('Test') and '/' not in r['Test'] and r['Action'] in ['pass','fail']]
 assert len(roots)==5 and all(row['Action']=='pass' for row in roots)
 native=ROOT/'native/native-originals'
 wrong=read(native/'graph-wrong-original-cache-capability.json')
 assert 'original cache capability' in wrong['error']
 for path in native.glob('*.graph-*-inputs.json'):
  snapshot=read(path)
  for obj in snapshot['objects']:object_bytes(obj,base64.b64decode(snapshot['raw'][obj['id']],validate=True))
 directory=native/'graph-owned'
 packet=read(directory/'complete-original-module-graph-inputs.json')
 objects={};total=0
 for row in packet['snapshot_sources']:
  obj=row['object'];data=raw(directory/'original-source-objects'/row['raw_file']);object_bytes(obj,data)
  objects[obj['id']]=(obj,data);total+=len(data)
 assert len(objects)==5667
 snapshot=part(packet,'snapshot_hex','MPK-PRIVATE-GO-SOURCE-SNAPSHOT-INPUTS-1.0')
 capture=part(packet,'capture_hex','MPK-PRIVATE-GO-SOURCE-CAPTURE-INPUTS-1.0')
 exports=part(packet,'export_inputs_hex','MPK-PRIVATE-GO-SOURCE-EXPORT-INPUTS-1.0')
 modules=part(packet,'module_observations_hex','MPK-PRIVATE-GO-SOURCE-MODULE-OBSERVATIONS-1.0')
 graph=part(packet,'module_graph_inputs_hex','MPK-PRIVATE-GO-SOURCE-MODULE-GRAPH-INPUTS-2.0')
 assert graph['schema']=='mpk.go.practical.source_module_graph_inputs.v2'
 assert capture['snapshot_inputs_sha256']==snapshot['content_sha256']
 assert exports['source_capture_inputs_sha256']==capture['content_sha256']
 assert modules['source_export_inputs_sha256']==exports['content_sha256']
 assert graph['source_module_observations_sha256']==modules['content_sha256']
 cache=graph['module_cache'];origins={r['capability']:r['path'] for r in exports['origins']}
 assert cache['capability'] in snapshot['capabilities']
 assert cache['original_path']==posixpath.normpath(posixpath.join(origins[cache['capability']],cache['directory']))
 assert any(d['capability']==cache['capability'] and d['path']==cache['directory'] for d in snapshot['directories'])
 source_main=next(row for row in modules['modules'] if row['version']=='')
 main=objects[source_main['manifest_source_object_id']][1]
 suggestion=b'go1.25.0-original-'+bytes([255])
 assert b'toolchain '+suggestion+b'\n' in main
 assert suggestion in bytes.fromhex(graph['metadata_hex'])
 for phase in ['initial_summaries','final_summaries']:
  labels=[]
  for row in graph[phase]:
   mv=row['module'];assert set(mv)=={'path','version_hex'}
   version=bytes.fromhex(mv['version_hex']);assert version.hex()==mv['version_hex']
   for req in row['requirements']:assert set(req)=={'path','version_hex'} and bytes.fromhex(req['version_hex']).hex()==req['version_hex']
   sid=row['manifest_source_object_id']
   if sid is None:assert mv['path'] in ['go','toolchain']
   else:assert objects[sid][0]['kind']=='go_mod'
   if mv['path']=='toolchain':labels.append(version)
  assert labels==[b'go1.25.0',suggestion]
 assert len(exports['packages'])==181 and any(p['import_path']=='net/http' for p in exports['packages'])
 ownership=read(directory/'graph-original-ownership.json')
 assert ownership['detached_input_buffers'] and ownership['immutable_image_and_original_roots_retired'] and ownership['unused_http_retained']
 previous=read(ROOT.parent/'graph-manifests/inspection.json')
 assert previous['status']=='passed' and not previous['proofs_replayed_in_this_checkpoint']
 # Inspect unchanged proof bytes without claiming another kernel execution.
 proofs=read(ROOT.parent/'module-data/controls-remaining/observations/sources/original-proof-replay.json')['complete_original_call_proofs']
 assert len(proofs)==32
 for row in proofs:
  data=raw(row['original_certificate']);assert len(data)==row['bytes'] and hashlib.sha256(data).hexdigest()==row['raw_sha256']
 assert not raw(ROOT/'format-final/stdout.log')
 assert all(row==seal(path) for path,row in INPUTS.items())
 result={'status':'passed','driver':seal(Path(__file__).resolve()),'inputs':list(INPUTS.values()),'source_epochs':len(epochs),'native_test_roots':5,'original_owner_packages':181,'original_owner_objects':5667,'original_owner_bytes':total,'original_toolchain_suffix_hex':suggestion.hex(),'cache_capability':cache['capability'],'version_identity_uses_original_bytes':True,'equal_version_labels_deterministic':True,'proofs_replayed_in_this_checkpoint':False,'original_proof_streams_unchanged':32,'previous_original_roots_per_unit':43,'previous_original_error_universes':[0,70],'independent_Rust_graph_import_or_MVS':False,'full_T_gate':'Deferred to final T02-W04: ./scripts/check-fast.sh'}
 output.write_text(json.dumps(result,indent=2)+'\n');print(json.dumps({k:result[k] for k in ['status','source_epochs','native_test_roots','original_owner_objects','original_owner_bytes']}))
if __name__=='__main__':main()
