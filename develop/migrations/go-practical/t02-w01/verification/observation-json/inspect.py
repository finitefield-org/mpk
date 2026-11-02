"""Inspect exact original native-byte transport and full owner consumers."""
from pathlib import Path
import base64,hashlib,json,re
ROOT=Path(__file__).resolve().parent
INPUTS={}
def seal(path):
 p=Path(path);raw=p.read_bytes();return {'path':str(p.resolve()),'bytes':len(raw),'sha256':hashlib.sha256(raw).hexdigest(),'mode':p.stat().st_mode&0o777}
def raw(path):
 p=Path(path);INPUTS[str(p.resolve())]=seal(p);return p.read_bytes()
def read(path):return json.loads(raw(path))
def canonical(v):return json.dumps(v,sort_keys=True,ensure_ascii=False,separators=(',',':')).encode()
def typed(domain,v):return hashlib.sha256(domain.encode()+b'\0'+canonical(v)).hexdigest()
def record(v,domain):assert typed(domain,{k:x for k,x in v.items() if k!='content_sha256'})==v['content_sha256']
def source(obj,data):
 assert len(data)==obj['bytes'] and hashlib.sha256(data).hexdigest()==obj['raw_sha256']
 assert typed('MPK-GO-SOURCE-OBJECT-1.0',{k:x for k,x in obj.items() if k!='id'})==obj['id']
def main():
 out=ROOT/'inspection.json';assert not out.exists()
 verification=read(ROOT/'verification.json')
 epochs={p.parent.name:read(p) for p in ROOT.glob('*/result.json')}
 assert sorted(epochs)==verification['source_epochs']
 assert sorted(n for n,r in epochs.items() if r['status']=='failed')==verification['preserved_failed_epochs']
 for name,r in epochs.items():
  assert r['whole_sdk_unchanged'] and (r['consumed_inputs_unchanged'] or r['expected_formatter_mutation'])
  assert r['inputs']==seal(r['inputs']['path'])
  for key in ['stdout','stderr']:assert r[key]==seal(r[key]['path'])
 assert 'permission denied' in raw(ROOT/'owner/stdout.log').decode()
 assert 'original native output did not cross the former JSON limit 630780 510544' in raw(ROOT/'owner-final/stdout.log').decode()
 assert 'diagnostics-over' in raw(ROOT/'unit/stderr.log').decode()
 native=[json.loads(s) for s in raw(ROOT/'native/stdout.log').splitlines()]
 native_roots=[r for r in native if r.get('Action')=='pass' and r.get('Test') and '/' not in r['Test']]
 assert len(native_roots)==6 and not any(r.get('Action')=='fail' for r in native)
 directory=ROOT/'native/native-originals/observation-json'
 corpus=read(directory/'complete-observation-json-budgets.json')
 assert corpus['common_string_bytes_unchanged']==1048576 and corpus['native_stdout_bytes_unchanged']==16777216 and corpus['native_stderr_bytes_unchanged']==1048576
 assert len(corpus['cases'])==30
 rust=read(ROOT/'unit-final/observations/bytes/original-byte-boundary-comparison.json')
 assert rust['status']=='passed' and rust['native_stdout_bytes']==16777216 and rust['native_stderr_bytes']==1048576
 compared={r['case']:r for r in rust['cases']};assert len(compared)==30
 domains={'mpk.go.practical.source_export_inputs.v1':'MPK-PRIVATE-GO-SOURCE-EXPORT-INPUTS-1.0','mpk.go.practical.source_module_observations.v1':'MPK-PRIVATE-GO-SOURCE-MODULE-OBSERVATIONS-1.0','mpk.go.practical.source_module_graph_inputs.v2':'MPK-PRIVATE-GO-SOURCE-MODULE-GRAPH-INPUTS-2.0'}
 budget_witnesses=[]
 for row in corpus['cases']:
  original=raw(directory/row['original_file']);encoded=raw(directory/row['canonical_file']);wire=json.loads(encoded)
  assert len(original)==row['raw_bytes'] and hashlib.sha256(original).hexdigest()==row['raw_sha256']
  assert len(wire[row['field']])==2*len(original) and bytes.fromhex(wire[row['field']])==original
  assert hashlib.sha256(encoded).hexdigest()==row['canonical_sha256'] and canonical(wire)==encoded
  record(wire,domains[row['schema']]);assert wire['content_sha256']==row['content_sha256']
  actual=compared[row['case']];assert actual['same_original_bytes'] and actual['original_bytes']==len(original)
  for key in ['raw_sha256','canonical_sha256','content_sha256']:assert actual[key]==row[key]
  budget_witnesses.append({'case':row['case'],'bytes':len(original),'raw_sha256':row['raw_sha256'],'canonical_sha256':row['canonical_sha256'],'content_sha256':row['content_sha256']})
 controls_dir=ROOT/'native/native-originals/observation-json-controls'
 controls=read(controls_dir/'complete-observation-json-controls.json');assert len(controls['cases'])==19 and controls['cross_domain_rejected'] and controls['common_codec_unchanged']
 controls_rust=read(ROOT/'unit-final/observations/bytes/original-byte-control-comparison.json');assert controls_rust['status']=='passed' and controls_rust['ordinary_rust_limit_unchanged']==16777216 and controls_rust['ordinary_go_limit_unchanged']==1048576
 compared={r['case']:r for r in controls_rust['cases']};assert len(compared)==17
 for row in controls['cases']:
  if not row['serialized_input_retained']:continue
  data=raw(controls_dir/row['complete_input_file']);actual=compared[row['case']]
  assert actual['bytes']==len(data) and actual['raw_sha256']==hashlib.sha256(data).hexdigest()
  assert actual['rejected']==(row['case'] not in ['ordinary-over','nested-metadata-over'])
 assert epochs['unit-final']['status']=='passed' and 'test result: ok. 2 passed; 0 failed;' in raw(ROOT/'unit-final/stdout.log').decode()
 log=raw(ROOT/'rust/stdout.log').decode();assert [int(n) for n in re.findall(r'test result: ok\. (\d+) passed; 0 failed; 0 ignored;',log)]==[6,7]
 assert epochs['clippy-final']['status']=='passed' and epochs['vet']['status']=='passed'
 full=ROOT/'owner-large/native-originals/versioned-owned'
 packet=read(full/'complete-original-module-graph-inputs.json')
 graph=json.loads(bytes.fromhex(packet['module_graph_inputs_hex']));exports=json.loads(bytes.fromhex(packet['export_inputs_hex']));module=json.loads(bytes.fromhex(packet['module_observations_hex']))
 for v in [graph,exports,module]:record(v,domains[v['schema']])
 assert graph['source_module_observations_sha256']==module['content_sha256'] and module['source_export_inputs_sha256']==exports['content_sha256']
 assert len(exports['packages'])==182 and any(r['import_path']=='net/http' for r in exports['packages'])
 assert len(packet['snapshot_sources'])==5697
 for row in packet['snapshot_sources']:
  obj=row['object'];data=raw(full/'original-source-objects'/row['raw_file']);source(obj,data)
 metadata=bytes.fromhex(graph['metadata_hex']);export_metadata=bytes.fromhex(exports['metadata_hex'])
 assert len(metadata)>524288 and len(export_metadata)>524288 and len(metadata)<=16777216 and len(export_metadata)<=16777216
 assert metadata.count(b'example.com/a@v1.0.0 example.com/c@v1.0.0\n')==15000
 observation=read(ROOT/'rust/observations/graphs/large-original-native-streams.json')
 assert observation['status']=='passed' and observation['original_objects']==5697 and observation['original_packages']==182 and observation['original_duplicate_requirements']==15000
 assert observation['original_export_metadata_bytes']==len(export_metadata) and observation['original_graph_metadata_bytes']==len(metadata) and observation['original_graph_sha256']==hashlib.sha256(metadata).hexdigest()
 owned=read(full/'graph-original-ownership.json');assert owned['original_and_image_roots_retired'] and owned['packages']==182 and owned['source_objects']==5697
 old=read(ROOT/'rust/observations/graphs/coherent-original-graph-controls.json')['cases'];assert len(old)==20
 for row in old:record(row['complete_changed_graph'],domains['mpk.go.practical.source_module_graph_inputs.v2']);assert row['error']
 invented=read(ROOT/'rust/observations/graphs/coherent-final-root-controls.json')['cases'];assert len(invented)==2
 for row in invented:assert row['error']=='Go original module graph: Selection' and row['same_original_sources_and_Root_contract']
 changed=read(ROOT/'rust/observations/graphs/coherent-original-unselected-manifest-control.json');assert changed['error']=='Go original module graph: Checksum' and changed['original_object_count']==5697
 previous=read(ROOT.parent/'root-expansion/inspection.json');assert previous['status']=='passed' and not previous['proofs_replayed_in_this_checkpoint']
 proofs=read(ROOT.parent/'module-data/controls-remaining/observations/sources/original-proof-replay.json')['complete_original_call_proofs'];assert len(proofs)==32
 for row in proofs:
  data=raw(row['original_certificate']);assert len(data)==row['bytes'] and hashlib.sha256(data).hexdigest()==row['raw_sha256']
 assert all(row==seal(path) for path,row in INPUTS.items())
 result={'status':'passed','driver':seal(__file__),'inputs':list(INPUTS.values()),'source_epochs':len(epochs),'affected_rust_roots':15,'native_roots':7,'native_transport_witnesses':30,'native_invalid_cases':19,'rust_compared_serialized_controls':17,'ordinary_rust_policy_cases_retained':2,'budget_witnesses':budget_witnesses,'whole_original_owners':[{'name':'local','packages':181,'objects':5667},{'name':'versioned','packages':182,'objects':5697},{'name':'large-versioned','packages':182,'objects':5697,'native_export_bytes':len(export_metadata),'native_graph_bytes':len(metadata)}],'coherent_graph_claim_controls':22,'coherent_unselected_manifest_content_controls':1,'original_proof_streams_unchanged':32,'proofs_replayed_in_this_checkpoint':False,'installed_context_authority':False,'common_json_and_kernel_limits_changed':False,'whole_repository_gate_run':False,'full_T_gate':'Deferred to final T02-W04: ./scripts/check-fast.sh'}
 out.write_text(json.dumps(result,indent=2)+'\n');print(json.dumps({k:result[k] for k in ['status','source_epochs','affected_rust_roots','native_roots','native_transport_witnesses']}))
if __name__=='__main__':main()
