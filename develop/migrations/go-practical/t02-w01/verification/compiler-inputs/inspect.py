"""Inspect complete original compiler bundle bytes and independent bindings."""
from pathlib import Path
import hashlib,json,re
ROOT=Path(__file__).resolve().parent
INPUTS={}
def seal(path):
 p=Path(path);data=p.read_bytes();return {'path':str(p.resolve()),'bytes':len(data),'sha256':hashlib.sha256(data).hexdigest(),'mode':p.stat().st_mode&0o777}
def raw(path):
 p=Path(path);INPUTS[str(p.resolve())]=seal(p);return p.read_bytes()
def read(path):return json.loads(raw(path))
def canonical(v):return json.dumps(v,sort_keys=True,ensure_ascii=False,separators=(',',':')).encode()
def typed(domain,v):return hashlib.sha256(domain.encode()+b'\0'+canonical(v)).hexdigest()
def record(v,domain):assert typed(domain,{k:x for k,x in v.items() if k!='content_sha256'})==v['content_sha256']
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
 native=[json.loads(s) for s in raw(ROOT/'native/stdout.log').splitlines()]
 roots=[r for r in native if r.get('Action')=='pass' and r.get('Test') and '/' not in r['Test']]
 assert len(roots)==2 and not any(r.get('Action')=='fail' for r in native)
 log=raw(ROOT/'rust/stdout.log').decode();assert [int(n) for n in re.findall(r'test result: ok\. (\d+) passed; 0 failed; 0 ignored;',log)]==[4,6]
 directory=ROOT/'native/native-originals/versioned-owned'
 packet=read(directory/'complete-original-module-graph-inputs.json')
 compiler=json.loads(bytes.fromhex(packet['compiler_inputs_hex']));record(compiler,'MPK-PRIVATE-GO-COMPILER-IMAGE-INPUTS-1.0')
 exports=json.loads(bytes.fromhex(packet['export_inputs_hex']));record(exports,'MPK-PRIVATE-GO-SOURCE-EXPORT-INPUTS-1.0')
 assert compiler['source_export_inputs_sha256']==exports['content_sha256'] and compiler['sdk_capability']==exports['sdk_capability']
 assert compiler['toolchain']['distribution_sha256']==exports['compiler_sha256']==typed('MPK-BUNDLE-CONTENT-0.1',compiler['inventory'])
 assert len(exports['packages'])==182 and len(packet['snapshot_sources'])==5697 and any(r['import_path']=='net/http' for r in exports['packages'])
 files=compiler['inventory']['files'];actual=packet['compiler_files'];assert len(files)==len(actual)==5577
 assert len(compiler['components'])==len(compiler['toolchain']['components'])==10
 total=0;large=[];by_path={}
 for entry,row in zip(files,actual):
  assert entry==row['entry']
  data=raw(directory/'original-compiler-files'/row['raw_file'])
  assert len(data)==entry['size_bytes'] and hashlib.sha256(data).hexdigest()==entry['sha256'] and row['raw_file']==entry['sha256']+'.raw'
  total+=len(data);by_path[entry['path']]=entry
  if len(data)>16777216:large.append(entry)
 assert total==163457770 and len(large)==1
 components={};partition={}
 for component in compiler['components']:
  identity=component['identity'];inventory=component['inventory'];name=identity['name'];assert name not in components;components[name]=identity
  assert identity['release']=='go1.25.0' and inventory['scope']=={'kind':'component','bundle_id':compiler['toolchain']['bundle_id'],'component_name':name}
  if identity['kind']=='content':assert identity['content_sha256']==typed('MPK-BUNDLE-CONTENT-0.1',inventory)
  else:assert len(inventory['files'])==1 and inventory['files'][0]['executable'] and inventory['files'][0]['sha256']==identity['binary_sha256']
  for entry in inventory['files']:assert entry['path'] not in partition;partition[entry['path']]=entry
 assert partition==by_path and compiler['toolchain']['components']==[components[n] for n in sorted(components)]
 assert exports['go_binary_sha256']==by_path['go/bin/go']['sha256']
 sdk_sources={}
 for row in packet['snapshot_sources']:
  obj=row['object'];data=raw(directory/'original-source-objects'/row['raw_file'])
  assert len(data)==obj['bytes'] and hashlib.sha256(data).hexdigest()==obj['raw_sha256']
  assert typed('MPK-GO-SOURCE-OBJECT-1.0',{k:v for k,v in obj.items() if k!='id'})==obj['id']
  if obj['capability']==compiler['sdk_capability']:
   entry=by_path['go/'+obj['path']];assert entry['sha256']==obj['raw_sha256'] and entry['size_bytes']==obj['bytes'];sdk_sources['go/'+obj['path']]=obj
 expected={e['path'] for e in files if e['path'].startswith('go/src/')};assert set(sdk_sources)==expected
 ownership=read(directory/'compiler-original-ownership.json');assert ownership['files']==5577 and ownership['bytes']==total and ownership['files_larger_than_source_limit']==1 and ownership['whole_compiler_survives_original_and_image_retirement']
 assert ownership['source_file_limit_unchanged']==16777216 and ownership['source_aggregate_limit_unchanged']==268435456
 imported=read(ROOT/'rust/observations/compilers/whole-original-compiler-import.json')
 assert imported['status']=='passed' and imported['compiler_inputs']==compiler and imported['compiler_file_hashes']==files
 assert imported['files']==5577 and imported['bytes']==total and imported['sdk_sources']==len(sdk_sources) and imported['original_objects']==5697 and imported['original_packages']==182
 controls=read(ROOT/'rust/observations/compilers/coherent-original-compiler-controls.json')['cases'];assert len(controls)==30
 for row in controls:
  record(row['complete_changed_record'],'MPK-PRIVATE-GO-COMPILER-IMAGE-INPUTS-1.0')
  assert row['error'] and row['whole_original_capture_and_Root_contract_preserved'] and row['original_compiler_files']==5577
 changed=read(ROOT/'rust/observations/compilers/coherent-original-sdk-content-control.json')
 data=bytes.fromhex(changed['changed_raw_hex']);obj=changed['changed_object']
 assert len(data)==obj['bytes'] and hashlib.sha256(data).hexdigest()==obj['raw_sha256']
 assert typed('MPK-GO-SOURCE-OBJECT-1.0',{k:v for k,v in obj.items() if k!='id'})==obj['id']
 assert obj['path']=='src/cmd/compile/internal/ssa/poset.go' and changed['sdk_source_was_unused_by_application_packages'] and changed['original_application_source_and_Root_contract_unchanged']
 assert changed['error']=='Go original compiler inputs: Source' and changed['original_objects']==5697
 docs={}
 for field,domain in [('changed_snapshot_hex','MPK-PRIVATE-GO-SOURCE-SNAPSHOT-INPUTS-1.0'),('changed_capture_hex','MPK-PRIVATE-GO-SOURCE-CAPTURE-INPUTS-1.0'),('changed_export_hex','MPK-PRIVATE-GO-SOURCE-EXPORT-INPUTS-1.0')]:
  docs[field]=json.loads(bytes.fromhex(changed[field]));record(docs[field],domain)
 assert docs['changed_capture_hex']['snapshot_inputs_sha256']==docs['changed_snapshot_hex']['content_sha256']
 assert docs['changed_export_hex']['source_capture_inputs_sha256']==docs['changed_capture_hex']['content_sha256']
 record(changed['changed_compiler_record'],'MPK-PRIVATE-GO-COMPILER-IMAGE-INPUTS-1.0')
 assert changed['changed_compiler_record']['source_export_inputs_sha256']==docs['changed_export_hex']['content_sha256']
 assert obj in docs['changed_snapshot_hex']['objects'] and len(docs['changed_snapshot_hex']['objects'])==5697
 assert changed['changed_compiler_record']['inventory']==compiler['inventory'] and hashlib.sha256(data).hexdigest()!=by_path['go/'+obj['path']]['sha256']
 previous=read(ROOT.parent/'observation-json/inspection.json');assert previous['status']=='passed' and not previous['proofs_replayed_in_this_checkpoint']
 proofs=read(ROOT.parent/'module-data/controls-remaining/observations/sources/original-proof-replay.json')['complete_original_call_proofs'];assert len(proofs)==32
 for row in proofs:
  data=raw(row['original_certificate']);assert len(data)==row['bytes'] and hashlib.sha256(data).hexdigest()==row['raw_sha256']
 assert all(row==seal(path) for path,row in INPUTS.items())
 result={'status':'passed','driver':seal(__file__),'inputs':list(INPUTS.values()),'source_epochs':len(epochs),'affected_rust_roots':10,'native_roots':2,'whole_original_owners':[{'name':'versioned','packages':182,'objects':5697,'compiler_files':5577,'compiler_bytes':total,'sdk_sources':len(sdk_sources),'actual_compiler_files_above_source_limit':large}],'coherent_compiler_claim_controls':30,'coherent_unused_sdk_source_content_controls':1,'original_proof_streams_unchanged':32,'proofs_replayed_in_this_checkpoint':False,'binary_build_info_checked_by_rust':False,'all_sdk_source_roles_derived_by_rust':False,'installed_context_authority':False,'bundle_source_kernel_limits_changed':False,'whole_repository_gate_run':False,'full_T_gate':'Deferred to final T02-W04: ./scripts/check-fast.sh'}
 out.write_text(json.dumps(result,indent=2)+'\n');print(json.dumps({k:result[k] for k in ['status','source_epochs','affected_rust_roots','native_roots','coherent_compiler_claim_controls']}))
if __name__=='__main__':main()
