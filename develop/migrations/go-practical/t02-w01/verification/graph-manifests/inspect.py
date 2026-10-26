"""Inspect all native/Rust dependency-manifest facts and intact original owners."""
from pathlib import Path
import base64, hashlib, json, re

ROOT=Path(__file__).resolve().parent
INPUTS={}
def seal(path):
 path=Path(path);raw=path.read_bytes()
 return {'path':str(path),'bytes':len(raw),'sha256':hashlib.sha256(raw).hexdigest(),'mode':path.stat().st_mode&0o777}
def raw(path):
 path=Path(path);INPUTS[str(path)]=seal(path);return path.read_bytes()
def read(path):return json.loads(raw(path))
def object_bytes(obj,data):
 assert len(data)==obj['bytes'] and hashlib.sha256(data).hexdigest()==obj['raw_sha256']
 body={k:v for k,v in obj.items() if k!='id'}
 encoded=json.dumps(body,sort_keys=True,ensure_ascii=False,separators=(',',':')).encode()
 assert hashlib.sha256(b'MPK-GO-SOURCE-OBJECT-1.0\0'+encoded).hexdigest()==obj['id']
def main():
 out=ROOT/'inspection.json';assert not out.exists()
 verification=read(ROOT/'verification.json')
 epochs={p.parent.name:read(p) for p in ROOT.glob('*/result.json')}
 assert sorted(epochs)==verification['source_epochs']
 for name,row in epochs.items():
  assert row['status']=='passed' and row['whole_sdk_unchanged']
  assert row['consumed_inputs_unchanged'] or row['expected_formatter_mutation']
  assert row['inputs']==seal(row['inputs']['path'])
  for field in ['stdout','stderr']:assert row[field]==seal(row[field]['path'])
 native=read(ROOT/'native/native-originals/graph-manifests/complete-original-graph-manifests.json')['cases']
 rust=read(ROOT/'rust-initial/observations/graph-manifests/complete-original-graph-manifest-observations.json')['observations']
 assert len(native)==len(rust)==779
 previous=read(ROOT.parent/'module-manifests/native-toolchain/native-originals/module-manifests/complete-original-module-manifests.json')['cases']
 old={(r['case'],r['main']):r for r in previous};assert len(old)==656
 kept=0;accepted=0
 for original,observed in zip(native,rust):
  data=bytes.fromhex(original['raw_hex']);object_bytes(original['object'],data)
  for field in ['case','origin_main','object','accepted','facts']:assert original[field]==observed[field],(original['case'],field)
  if original['origin_main'] is not None:
   prior=old[original['case'],original['origin_main']]
   assert original['object']==prior['object'] and original['raw_hex']==prior['raw_hex'];kept+=1
  accepted+=original['accepted']
 assert kept==656 and accepted==376
 cases={r['case']:r for r in native if r['origin_main'] is None}
 assert cases['lax-go-'+b'1.25.00'.hex()]['facts']['go_requirement']=='1.25'
 assert cases['lax-go-'+b'1.25.0rc1'.hex()]['facts']['go_requirement']=='1.25.0rc1'
 assert not cases['lax-go-'+b'1.25.0rc1'.hex()]['facts']['pruned']
 assert cases['lax-go-'+b'1.99'.hex()]['facts']['requires_newer_compiler']
 assert cases['lax-go-'+(b'1.25'+bytes([255])).hex()]['facts']['go_requirement']=='1.25'
 assert cases['lax-retract-'+b'[v1,v2] "bad\\q"'.hex()]['facts']['retractions']==[{'low_hex':b'v1'.hex(),'high_hex':b'v2'.hex()}]
 # All original strict cases still match after sharing the syntax engine.
 strict=read(ROOT/'rust-initial/observations/manifests/complete-original-module-manifest-observations.json')['observations']
 assert len(strict)==len(previous)
 for native_row,rust_row in zip(previous,strict):
  for field in ['case','main','object','accepted','facts']:assert native_row[field]==rust_row[field]
 logs=raw(ROOT/'rust-initial/stdout.log').decode()
 assert [int(n) for n in re.findall(r'test result: ok\. (\d+) passed; 0 failed; 0 ignored;',logs)]==[6,3,2]
 assert [int(n) for n in re.findall(r'test result: ok\. (\d+) passed; 0 failed; 0 ignored;',raw(ROOT/'links-final/stdout.log').decode())]==[2]
 # The exact native parser/library source is present in the full pinned SDK,
 # not a substituted external modfile implementation.
 libraries=read(ROOT/'library-correspondence.json')
 assert libraries['status']=='passed' and len(libraries['original_library_files'])==8
 for row in libraries['original_library_files']:
  sdk=raw(row['sdk_path']);producer=raw(row['producer_path'])
  assert sdk==producer and len(sdk)==row['bytes'] and hashlib.sha256(sdk).hexdigest()==row['sha256']
 # Compare complete native initial/final real-manifest inventories and actual
 # ordered requirements (including main excludes and the implicit go edge).
 expected={}
 for epoch in ['edges-final','replacements-final','toolchains-final']:
  directory=ROOT.parent/'module-graphs'/epoch/'native-originals'
  for path in sorted(directory.glob('*.json')):
   if path.name.endswith(('.graph-whole-original-observation.json','.graph-virtual-observation.json')) or path.name=='graph-rejected-native-root-expansion.json':
    packet=read(path)
    for phase in ['initial','final']:
     for summary in packet.get(phase,[]):
      if summary['source'] is None:continue
      obj=summary['source']['object'];data=base64.b64decode(summary['source']['raw'],validate=True);object_bytes(obj,data)
      key=(epoch,path.name,phase,summary['identity']['Path'],summary['identity']['Version'])
      expected[key]=summary
 linked=read(ROOT/'links-final/observations/graph-manifests/whole-original-initial-final-graph-manifests.json')['observations']
 assert len(linked)==len(expected)
 for row in linked:
  key=(row['epoch'],row['observation'],row['phase'],row['module']['Path'],row['module']['Version']);original=expected.pop(key)
  assert row['original_object']==original['source']['object']
  assert (row['facts']['go_requirement'] or '')==original['go_version'] and row['facts']['pruned']==original['pruned']
  assert row['graph_requirements']==(original['requirements'] or [])
 assert not expected
 assert any(r['module']=={'Path':'example.com/leaf','Version':'v1.0.0'} and r['facts']['go_requirement'] is None for r in linked)
 # Import and keep the complete original owner, including unused HTTP and all
 # source/export/cache bytes. Its source-only facts do not select graph roots.
 directory=ROOT.parent/'module-graphs/native-initial/native-originals/graph-owned'
 packet=read(directory/'complete-original-module-graph-inputs.json')
 objects={}
 for row in packet['snapshot_sources']:
  obj=row['object'];data=raw(directory/'original-source-objects'/row['raw_file']);object_bytes(obj,data);objects[obj['id']]=obj
 assert len(objects)==5667
 owner=read(ROOT/'links-final/observations/graph-manifests/complete-original-graph-owner-manifests.json')
 assert owner['original_objects']==5667 and owner['original_packages']==181 and not owner['whole_source_or_package_universe_reduced']
 graph=json.loads(bytes.fromhex(packet['module_graph_inputs_hex']))
 for row in owner['observations']:
  candidates=graph[row['phase']];original=next(s for s in candidates if s['module']==row['module'])
  assert objects[original['manifest_source_object_id']]==row['original_object']
  assert row['graph_requirements']==original['requirements']
 previous_inspection=read(ROOT.parent/'module-graphs/inspection.json')
 assert previous_inspection['status']=='passed' and not previous_inspection['proofs_replayed_in_this_checkpoint']
 proofs=read(ROOT.parent/'module-data/controls-remaining/observations/sources/original-proof-replay.json')['complete_original_call_proofs'];assert len(proofs)==32
 for row in proofs:
  data=raw(row['original_certificate']);assert len(data)==row['bytes'] and hashlib.sha256(data).hexdigest()==row['raw_sha256']
 assert not raw(ROOT/'go-format-final/stdout.log')
 assert all(row==seal(path) for path,row in INPUTS.items())
 result={'status':'passed','driver':seal(Path(__file__).resolve()),'inputs':list(INPUTS.values()),'source_epochs':len(epochs),'native_cases':779,'accepted_cases':376,'rejected_cases':403,'previous_original_mode_cases_retained':656,'new_lax_cases':123,'affected_rust_roots':11,'native_roots':1,'linked_native_real_manifests':len(linked),'actual_owner_objects':5667,'actual_owner_packages':181,'original_library_files_match_pinned_SDK':8,'unchanged_original_proof_streams':32,'proofs_replayed_in_this_checkpoint':False,'previous_original_roots_per_unit':43,'previous_original_error_universes':[0,70],'independent_MVS_or_graph_transport_import':False,'full_T_gate':'Deferred to final T02-W04: ./scripts/check-fast.sh'}
 out.write_text(json.dumps(result,indent=2)+'\n');print(json.dumps({k:result[k] for k in ['status','source_epochs','native_cases','linked_native_real_manifests','affected_rust_roots']}))
if __name__=='__main__':main()
