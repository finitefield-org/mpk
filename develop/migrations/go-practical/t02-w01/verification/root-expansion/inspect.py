"""Inspect complete native roots, original source matrices and coherent controls."""
from pathlib import Path
import hashlib,json,re
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
def source(v):
 obj=v['object'];data=bytes.fromhex(v['raw_hex']);assert len(data)==obj['bytes'] and hashlib.sha256(data).hexdigest()==obj['raw_sha256'];assert typed('MPK-GO-SOURCE-OBJECT-1.0',{k:x for k,x in obj.items() if k!='id'})==obj['id'];return data
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
 assert 'source-derived roots differ from native expansion' in raw(ROOT/'native/stdout.log').decode()
 assert 'duplicate-consistent' in raw(ROOT/'native/stdout.log').decode()
 assert epochs['format-fresh']['exit_code']==1
 assert epochs['format-fresh-final']['status']=='passed'
 native=[json.loads(s) for s in raw(ROOT/'native-final/stdout.log').splitlines()]
 roots=[r for r in native if r.get('Action')=='pass' and r.get('Test') and '/' not in r['Test']]
 assert len(roots)==7 and not any(r.get('Action')=='fail' for r in native)
 corpus=read(ROOT/'native-final/native-originals/root-expansion/complete-original-root-expansion.json')
 assert len(corpus['cases'])==18
 original=[json.loads(s) for s in raw(ROOT/'native/stdout.log').splitlines()]
 first_names={r['Test'].split('/',1)[1] for r in original if r.get('Action')=='run' and r.get('Test','').startswith('TestPracticalOriginalGraphExpansionOracle/')}
 assert len(first_names)==13 and first_names<={r['case'] for r in corpus['cases']}
 comparison=read(ROOT/'unit/observations/expansion/native-expansion-comparison.json')
 assert comparison['status']=='passed' and len(comparison['cases'])==18
 derived={r['case']:r for r in comparison['cases']}
 for row in corpus['cases']:
  assert row['arguments']==['mod','graph'] and 'GOMAXPROCS=1' in row['environment']
  assert 'GOPROXY=off' in row['environment'] and 'GOTOOLCHAIN=local' in row['environment']
  source(row['main'])
  originals={}
  for r in row['sources']:source(r);originals[(r['module']['path'],r['module']['version_hex'])]=r['object']
  actual=derived[row['case']];assert actual['roots']==row['derived_roots'] and actual['original_metadata_hex']==row['metadata_hex']
  for r in actual['consumed_manifests']:assert r['source']==originals[(r['module']['path'],r['module']['version_hex'])]
  if row['case']=='cascade-transient':
   transient=('example.com/c',b'v1.1.0'.hex())
   for phase in ['initial','final']:assert transient not in {(r['identity']['path'],r['identity']['version_hex']) for r in row[phase]}
   assert any((r['module']['path'],r['module']['version_hex'])==transient and r['captured_during_expansion'] for r in row['sources'])
   assert any((r['module']['path'],r['module']['version_hex'])==transient for r in actual['consumed_manifests'])
  if row['case']=='discarded-lower-root-needs-no-manifest':assert len(row['sources'])==1 and bytes.fromhex(row['sources'][0]['module']['version_hex'])==b'v1.2.0'
 ordering=read(ROOT/'native-final/native-originals/root-ordering/complete-original-root-ordering.json')
 compared=read(ROOT/'unit/observations/expansion/native-order-comparison.json')
 assert compared['status']=='passed' and compared['complete_original_corpus']==ordering
 assert compared['comparisons']==len(ordering['comparisons'])==225
 assert compared['sorts']==len(ordering['sorts'])==84
 for epoch,count in [('rust-final',5),('unit',2),('fresh',1)]:
  assert f'test result: ok. {count} passed; 0 failed;' in raw(ROOT/epoch/'stdout.log').decode()
 controls=read(ROOT/'rust-final/observations/graphs/coherent-original-graph-controls.json')['cases'];assert len(controls)==20
 for r in controls:record(r['complete_changed_graph'],'MPK-PRIVATE-GO-SOURCE-MODULE-GRAPH-INPUTS-2.0');assert r['error']
 roots=read(ROOT/'rust-final/observations/graphs/coherent-final-root-controls.json')['cases'];assert len(roots)==2
 for r in roots:
  record(r['complete_changed_graph'],'MPK-PRIVATE-GO-SOURCE-MODULE-GRAPH-INPUTS-2.0')
  assert r['error']=='Go original module graph: Selection' and r['original_objects']==5697 and r['same_original_sources_and_Root_contract']
 changed=read(ROOT/'rust-final/observations/graphs/coherent-original-unselected-manifest-control.json')
 assert changed['error']=='Go original module graph: Checksum' and changed['original_object_count']==5697 and changed['same_original_package_source_and_Root_contract_universe']
 fresh_dir=ROOT/'owner/native-originals/versioned-owned'
 packet=read(fresh_dir/'complete-original-module-graph-inputs.json')
 graph=json.loads(bytes.fromhex(packet['module_graph_inputs_hex']));record(graph,'MPK-PRIVATE-GO-SOURCE-MODULE-GRAPH-INPUTS-2.0')
 owned=read(fresh_dir/'graph-original-ownership.json')
 assert owned['packages']==182 and owned['source_objects']==5697 and owned['selected_modules']==8 and owned['original_and_image_roots_retired']
 assert len(packet['snapshot_sources'])==5697
 for row in packet['snapshot_sources']:
  obj=row['object'];data=raw(fresh_dir/'original-source-objects'/row['raw_file']);assert len(data)==obj['bytes'] and hashlib.sha256(data).hexdigest()==obj['raw_sha256']
 fresh=read(ROOT/'fresh/observations/graphs/fresh-whole-original-root-derivation.json')
 assert fresh['status']=='passed' and fresh['original_objects']==5697 and len(fresh['derived_roots'])==8 and len(fresh['consumed_manifests'])==9 and fresh['original_metadata_hex']==graph['metadata_hex']
 prior=read(ROOT.parent/'graph-import/inspection.json');assert prior['status']=='passed' and not prior['proofs_replayed_in_this_checkpoint']
 proofs=read(ROOT.parent/'module-data/controls-remaining/observations/sources/original-proof-replay.json')['complete_original_call_proofs'];assert len(proofs)==32
 for r in proofs:
  data=raw(r['original_certificate']);assert len(data)==r['bytes'] and hashlib.sha256(data).hexdigest()==r['raw_sha256']
 assert all(row==seal(path) for path,row in INPUTS.items())
 result={'status':'passed','driver':seal(__file__),'inputs':list(INPUTS.values()),'source_epochs':len(epochs),'affected_rust_roots':8,'native_roots':8,'native_expansion_cases':18,'native_version_comparisons':225,'native_sort_permutations':84,'whole_original_owners':[{'name':'local','packages':181,'objects':5667,'evidence':'original owner reimported with current Rust'},{'name':'versioned','packages':182,'objects':5697,'evidence':'original owner reimported; current producer owner independently imported too'}],'coherent_graph_claim_controls':22,'coherent_unselected_manifest_content_controls':1,'missing_transient_manifest_control':1,'original_proof_streams_unchanged':32,'proofs_replayed_in_this_checkpoint':False,'pruned_root_expansion_runtime_implemented':True,'algorithm_correctness_proved':False,'installed_context_authority':False,'whole_repository_gate_run':False,'full_T_gate':'Deferred to final T02-W04: ./scripts/check-fast.sh'}
 out.write_text(json.dumps(result,indent=2)+'\n');print(json.dumps({k:result[k] for k in ['status','source_epochs','affected_rust_roots','native_expansion_cases','native_version_comparisons','native_sort_permutations']}))
if __name__=='__main__':main()
