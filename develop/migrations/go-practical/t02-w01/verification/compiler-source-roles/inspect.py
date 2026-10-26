"""Inspect complete SDK role/source-byte and whole-owner control evidence."""
from pathlib import Path
import hashlib,json,re
ROOT=Path(__file__).resolve().parent
INPUTS={}
def seal(path):
 p=Path(path);b=p.read_bytes();return {'path':str(p.resolve()),'bytes':len(b),'sha256':hashlib.sha256(b).hexdigest(),'mode':p.stat().st_mode&0o777}
def raw(path):
 p=Path(path);INPUTS[str(p.resolve())]=seal(p);return p.read_bytes()
def read(path):return json.loads(raw(path))
def typed(domain,v):return hashlib.sha256(domain.encode()+b'\0'+json.dumps(v,sort_keys=True,ensure_ascii=False,separators=(',',':')).encode()).hexdigest()
def record(v,domain,field='content_sha256'):assert typed(domain,{k:x for k,x in v.items() if k!=field})==v[field]
def main():
 out=ROOT/'inspection.json';assert not out.exists()
 verification=read(ROOT/'verification.json');epochs={p.parent.name:read(p) for p in ROOT.glob('*/result.json')}
 assert sorted(epochs)==verification['source_epochs']
 assert sorted(n for n,r in epochs.items() if r['status']=='failed')==verification['preserved_failed_epochs']
 for n,r in epochs.items():
  assert r['whole_sdk_unchanged'] and (r['consumed_inputs_unchanged'] or r['expected_formatter_mutation'])
  assert r['inputs']==seal(r['inputs']['path'])
  for key in ['stdout','stderr']:assert r[key]==seal(r[key]['path'])
 native=read(ROOT/'native/observations/original-sdk-source-roles.json')
 assert native['go_version']=='go1.25.0' and native['target']=='linux_amd64' and not native['cgo_enabled']
 assert len(native['sdk_sources'])==11004 and len(native['package_paths'])==1017 and len(native['probes'])==388
 directory=ROOT.parent/'compiler-inputs/native/native-originals/versioned-owned'
 packet=read(directory/'complete-original-module-graph-inputs.json');original=json.loads(bytes.fromhex(packet['compiler_inputs_hex']));exports=json.loads(bytes.fromhex(packet['export_inputs_hex']))
 prepared={r['object']['path']:r['object'] for r in packet['snapshot_sources'] if r['object']['capability']=='original-go-sdk'};assert len(prepared)==5480
 transport=read(ROOT/'source-byte-transport.json');assert transport['raw_source_objects']==11004 and transport['new_unique_raw_files']==5415 and transport['complete_original_raw_sdk_unchanged']
 for key in ['native','original_sdk','driver']:assert transport['inputs'][key]==seal(transport['inputs'][key]['path'])
 paths=set();new_files=set();prepared_roles={};package_paths=set()
 for a,b in zip(native['sdk_sources'],transport['sdk_sources']):
  assert a['path']==b['path'] and a['bytes']==b['bytes'] and a['raw_sha256']==b['raw_sha256'] and a['error'] is None
  assert b['raw_file']==b['raw_sha256']+'.raw'
  store=directory/'original-compiler-files' if b['storage']=='prepared-compiler' else ROOT/'original-source-bytes'
  if b['storage']=='raw-sdk':new_files.add(b['raw_file'])
  data=raw(store/b['raw_file']);assert len(data)==a['bytes'] and hashlib.sha256(data).hexdigest()==a['raw_sha256']
  assert a['path'] not in paths;paths.add(a['path'])
  if a['path'] in prepared:
   assert prepared[a['path']]['kind']==a['role'];prepared_roles['go/'+a['path']]=a['role']
   p=a['path'][4:]
   if p.endswith('.go') and '/' in p:package_paths.add(p.rsplit('/',1)[0])
 assert len(new_files)==5415 and new_files=={p.name for p in (ROOT/'original-source-bytes').iterdir()}
 assert len(prepared_roles)==5480 and len(package_paths)==715
 report=read(ROOT/'rust/observations/source-roles/whole-original-sdk-role-comparison.json')
 assert report['status']=='passed' and len(report['sdk_sources'])==11004 and report['package_paths']==native['package_paths'] and len(report['probes'])==388
 for a,b in zip(native['sdk_sources'],report['sdk_sources']):assert all(a[k]==b[k] for k in ['path','bytes','raw_sha256','role'])
 for a,b in zip(native['probes'],report['probes']):
  data=bytes.fromhex(a['raw_hex']);assert len(data)==a['bytes'] and hashlib.sha256(data).hexdigest()==a['raw_sha256']
  assert a['name']==b['name'] and a['bytes']==b['bytes'] and a['path']==b['path']
  assert (a['error'] is None)==(b['error'] is None)
  if a['error'] is None:assert a['role']==b['role']
 long=next(a for a in native['probes'] if a['name']=='long-complete-header-and-body');data=bytes.fromhex(long['raw_hex'])
 assert data.count(b'_ "example.com/unused"')==40000 and data.endswith(b'func OriginalBody(){ go func(){select{}}() }\n') and long['role']=='excluded_source'
 owner=read(ROOT/'rust/observations/compilers/whole-original-compiler-import.json')
 assert owner['files']==5577 and owner['bytes']==163457770 and owner['original_objects']==5697 and owner['original_packages']==182 and owner['sdk_sources']==5480
 assert owner['sdk_source_roles']==prepared_roles and owner['sdk_source_package_paths']==sorted(package_paths) and owner['sdk_go_version']=='go1.25'
 assert owner['sdk_module_source_object_id']==prepared['src/go.mod']['id'] and owner['independently_bound_executables']==9 and owner['compiler_inputs']==original
 original_snapshot=json.loads(bytes.fromhex(packet['snapshot_hex']));controls=[]
 for name in ['active-as-excluded','excluded-as-active','auxiliary-as-metadata']:
  v=read(ROOT/f'rust/observations/compilers/coherent-unused-sdk-role-{name}.json')
  assert v['status']=='rejected' and v['error']=='SourceRole' and v['whole_original_source_bytes_and_Root_contract_unchanged']
  assert v['original_objects']==5697 and v['original_packages']==182 and v['original_compiler_files']==5577
  old=v['original_object'];changed=v['changed_object'];record(changed,'MPK-GO-SOURCE-OBJECT-1.0','id')
  assert old['raw_sha256']==changed['raw_sha256']==v['original_bytes_sha256'] and old['bytes']==changed['bytes'] and old['kind']!=changed['kind']
  assert {k:x for k,x in old.items() if k not in ['id','kind']}=={k:x for k,x in changed.items() if k not in ['id','kind']}
  snapshot=json.loads(bytes.fromhex(v['changed_snapshot_hex']));capture=json.loads(bytes.fromhex(v['changed_capture_hex']));export=json.loads(bytes.fromhex(v['changed_export_hex']));wire=v['changed_compiler_record']
  for value,domain in [(snapshot,'MPK-PRIVATE-GO-SOURCE-SNAPSHOT-INPUTS-1.0'),(capture,'MPK-PRIVATE-GO-SOURCE-CAPTURE-INPUTS-1.0'),(export,'MPK-PRIVATE-GO-SOURCE-EXPORT-INPUTS-1.0'),(wire,'MPK-PRIVATE-GO-COMPILER-IMAGE-INPUTS-1.0')]:record(value,domain)
  expected=sorted([changed if row['id']==old['id'] else row for row in original_snapshot['objects']],key=lambda row:row['id'])
  assert snapshot['objects']==expected and len(expected)==5697
  assert capture['snapshot_inputs_sha256']==snapshot['content_sha256'] and export['source_capture_inputs_sha256']==capture['content_sha256'] and wire['source_export_inputs_sha256']==export['content_sha256']
  for key in ['metadata_hex','diagnostics_hex','packages','origins','go_binary_sha256','compiler_sha256','image_sha256']:assert export[key]==exports[key]
  assert wire['inventory']==original['inventory'] and wire['components']==original['components'] and wire['toolchain']==original['toolchain']
  controls.append({'case':name,'path':old['path'],'error':v['error']})
 old_controls=read(ROOT/'rust/observations/compilers/coherent-original-compiler-controls.json');assert len(old_controls['cases'])==30
 read(ROOT/'rust/observations/compilers/coherent-original-sdk-content-control.json')
 for name in ['release','tool','cgo']:assert read(ROOT/f'rust/observations/compilers/coherent-executable-{name}.json')['error']=='BuildInfo'
 log=raw(ROOT/'rust/stdout.log').decode();counts=[int(n) for n in re.findall(r'test result: ok\. (\d+) passed; 0 failed; 0 ignored;',log)];assert counts==[3,6,1,6,3,2,2],counts
 native_log=[json.loads(line) for line in raw(ROOT/'native/stdout.log').splitlines()]
 assert len([row for row in native_log if row.get('Action')=='pass' and row.get('Test') and '/' not in row['Test']])==1 and not any(row.get('Action')=='fail' for row in native_log)
 result={'status':'passed','affected_rust_roots':23,'native_roots':1,'raw_sdk_sources':11004,'raw_sdk_packages':1017,'prepared_sdk_sources':5480,'prepared_sdk_packages':715,'native_probes':388,'coherent_compiler_claim_controls':30,'coherent_binary_content_controls':3,'coherent_source_role_controls':3,'source_role_controls':controls,'whole_original_owners':[{'objects':5697,'packages':182,'compiler_files':5577,'compiler_bytes':163457770}],'installed_context_authority':False,'proofs_replayed_in_this_checkpoint':False,'inputs':list(INPUTS.values()),'full_T_gate':'Deferred to final T02-W04: ./scripts/check-fast.sh'}
 out.write_text(json.dumps(result,indent=2)+'\n');print(json.dumps({k:v for k,v in result.items() if k!='inputs'}))
if __name__=='__main__':main()
