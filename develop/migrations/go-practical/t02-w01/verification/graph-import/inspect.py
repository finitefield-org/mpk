"""Inspect independently reconstructed original graphs and coherent controls."""
from pathlib import Path
import base64,hashlib,json,re
ROOT=Path(__file__).resolve().parent
INPUTS={}
def seal(path):
 path=Path(path);data=path.read_bytes();return {'path':str(path),'bytes':len(data),'sha256':hashlib.sha256(data).hexdigest(),'mode':path.stat().st_mode&0o777}
def raw(path):
 path=Path(path);INPUTS[str(path)]=seal(path);return path.read_bytes()
def read(path):return json.loads(raw(path))
def canonical(value):return json.dumps(value,sort_keys=True,ensure_ascii=False,separators=(',',':')).encode()
def typed(domain,value):return hashlib.sha256(domain.encode()+b'\0'+canonical(value)).hexdigest()
def check_record(value,domain):assert typed(domain,{k:v for k,v in value.items() if k!='content_sha256'})==value['content_sha256']
def object_bytes(obj,data):
 assert len(data)==obj['bytes'] and hashlib.sha256(data).hexdigest()==obj['raw_sha256']
 assert typed('MPK-GO-SOURCE-OBJECT-1.0',{k:v for k,v in obj.items() if k!='id'})==obj['id']
def main():
 out=ROOT/'inspection.json';assert not out.exists()
 verification=read(ROOT/'verification.json')
 epochs={p.parent.name:read(p) for p in ROOT.glob('*/result.json')}
 assert sorted(epochs)==verification['source_epochs']
 assert sorted(n for n,r in epochs.items() if r['status']=='failed')==['clippy-final','clippy-initial']
 for name,row in epochs.items():
  assert row['status'] in ['passed','failed'] and row['whole_sdk_unchanged']
  assert row['consumed_inputs_unchanged'] or row['expected_formatter_mutation']
  assert row['inputs']==seal(row['inputs']['path'])
  for field in ['stdout','stderr']:assert row[field]==seal(row[field]['path'])
 assert 'expected `&[_; 0]`' in raw(ROOT/'clippy-initial/stderr.log').decode()
 assert 'called `.err().expect()`' in raw(ROOT/'clippy-final/stderr.log').decode()
 assert epochs['clippy-controls']['status']=='passed'
 log=raw(ROOT/'rust-initial/stdout.log').decode()
 assert [int(n) for n in re.findall(r'test result: ok\. (\d+) passed; 0 failed; 0 ignored;',log)]==[6,3,3,2]
 assert 'test result: ok. 1 passed; 0 failed;' in raw(ROOT/'checksum/stdout.log').decode()
 owners=[]
 for name,directory in [('local',ROOT.parent/'graph-inputs/native/native-originals/graph-owned'),('versioned',ROOT/'native/native-originals/versioned-owned')]:
  packet=read(directory/'complete-original-module-graph-inputs.json')
  graph=json.loads(bytes.fromhex(packet['module_graph_inputs_hex']));check_record(graph,'MPK-PRIVATE-GO-SOURCE-MODULE-GRAPH-INPUTS-2.0')
  objects={};total=0
  for row in packet['snapshot_sources']:
   obj=row['object'];data=raw(directory/'original-source-objects'/row['raw_file']);object_bytes(obj,data);objects[obj['id']]=(obj,data);total+=len(data)
  export=json.loads(bytes.fromhex(packet['export_inputs_hex']));packages=len(export['packages'])
  assert any(r['import_path']=='net/http' for r in export['packages'])
  if name=='local':
   imported=read(ROOT/'rust-initial/observations/graphs/whole-original-local-graph-import.json')
   assert imported['graph']==graph and imported['original_objects']==len(objects)==5667 and packages==181
   assert imported['unchanged_original_source_universe']
   suggestion=b'go1.25.0-original-'+bytes([255]);assert suggestion in bytes.fromhex(graph['metadata_hex'])
  else:
   imported=read(ROOT/'rust-initial/observations/graphs/whole-original-versioned-graph-import.json')
   assert imported['original_objects']==len(objects)==5697 and imported['original_packages']==packages==182
   assert imported['metadata_hex']==graph['metadata_hex'] and imported['diagnostics_hex']==graph['diagnostics_hex']
   for phase in ['initial_sources','final_sources']:
    expected=graph['initial_summaries' if phase=='initial_sources' else 'final_summaries']
    expected={(r['module']['path'],r['module']['version_hex']):r for r in expected if r['manifest_source_object_id'] is not None}
    assert len(imported[phase])==len(expected)
    for row in imported[phase]:
     original=expected.pop((row['path'],row['version_hex']));assert objects[original['manifest_source_object_id']][0]==row['source']
    assert not expected
   assert any(r['path']=='example.com/leaf' and bytes.fromhex(r['version_hex'])==b'v1.0.0' for r in imported['final_sources'])
  owners.append({'name':name,'packages':packages,'objects':len(objects),'bytes':total})
 controls=read(ROOT/'rust-initial/observations/graphs/coherent-original-graph-controls.json')['cases'];assert len(controls)==20
 for row in controls:
  check_record(row['complete_changed_graph'],'MPK-PRIVATE-GO-SOURCE-MODULE-GRAPH-INPUTS-2.0')
  assert row['error'] and row['original_capture_and_Root_contract_unchanged'] and row['original_object_count']==5697
 changed=read(ROOT/'checksum/observations/graphs/coherent-original-unselected-manifest-control.json')
 data=bytes.fromhex(changed['complete_changed_raw_hex']);obj=changed['changed_object'];object_bytes(obj,data)
 assert changed['error']=='Go original module graph: Checksum' and changed['same_original_package_source_and_Root_contract_universe']
 assert changed['original_object_count']==5697
 docs={}
 for field,domain in [('changed_snapshot_hex','MPK-PRIVATE-GO-SOURCE-SNAPSHOT-INPUTS-1.0'),('changed_capture_hex','MPK-PRIVATE-GO-SOURCE-CAPTURE-INPUTS-1.0'),('changed_export_hex','MPK-PRIVATE-GO-SOURCE-EXPORT-INPUTS-1.0'),('changed_module_hex','MPK-PRIVATE-GO-SOURCE-MODULE-OBSERVATIONS-1.0')]:
  docs[field]=json.loads(bytes.fromhex(changed[field]));check_record(docs[field],domain)
 check_record(changed['changed_graph'],'MPK-PRIVATE-GO-SOURCE-MODULE-GRAPH-INPUTS-2.0')
 assert docs['changed_capture_hex']['snapshot_inputs_sha256']==docs['changed_snapshot_hex']['content_sha256']
 assert docs['changed_export_hex']['source_capture_inputs_sha256']==docs['changed_capture_hex']['content_sha256']
 assert docs['changed_module_hex']['source_export_inputs_sha256']==docs['changed_export_hex']['content_sha256']
 assert changed['changed_graph']['source_module_observations_sha256']==docs['changed_module_hex']['content_sha256']
 assert obj in docs['changed_snapshot_hex']['objects']
 assert len(docs['changed_snapshot_hex']['objects'])==5697
 # Hash1 is independently recomputed for the changed single go.mod member;
 # its original main go.sum remains the same full owned checksum universe.
 directory=ROOT/'native/native-originals/versioned-owned';packet=read(directory/'complete-original-module-graph-inputs.json')
 checksum_source=next(r for r in packet['snapshot_sources'] if r['object']['kind']=='go_sum' and r['object']['path']=='go.sum')
 sums=raw(directory/'original-source-objects'/checksum_source['raw_file']).decode()
 original_sum=next(line.split()[2] for line in sums.splitlines() if line.startswith('example.com/c v1.0.0/go.mod '))
 changed_sum='h1:'+base64.b64encode(hashlib.sha256((obj['raw_sha256']+'  go.mod\n').encode()).digest()).decode()
 assert changed_sum!=original_sum
 previous=read(ROOT.parent/'graph-inputs/inspection.json');assert previous['status']=='passed' and not previous['proofs_replayed_in_this_checkpoint']
 proofs=read(ROOT.parent/'module-data/controls-remaining/observations/sources/original-proof-replay.json')['complete_original_call_proofs'];assert len(proofs)==32
 for row in proofs:
  data=raw(row['original_certificate']);assert len(data)==row['bytes'] and hashlib.sha256(data).hexdigest()==row['raw_sha256']
 assert not raw(ROOT/'go-format-final/stdout.log')
 assert all(row==seal(path) for path,row in INPUTS.items())
 result={'status':'passed','driver':seal(Path(__file__).resolve()),'inputs':list(INPUTS.values()),'source_epochs':len(epochs),'affected_rust_roots':15,'native_roots':1,'whole_original_owners':owners,'coherent_graph_claim_controls':20,'coherent_unselected_mod_content_controls':1,'original_proof_streams_unchanged':32,'proofs_replayed_in_this_checkpoint':False,'previous_original_roots_per_unit':43,'previous_original_error_universes':[0,70],'independent_root_expansion_or_installed_context_authority':False,'full_T_gate':'Deferred to final T02-W04: ./scripts/check-fast.sh'}
 out.write_text(json.dumps(result,indent=2)+'\n');print(json.dumps({k:result[k] for k in ['status','source_epochs','affected_rust_roots','coherent_graph_claim_controls','coherent_unselected_mod_content_controls']}))
if __name__=='__main__':main()
