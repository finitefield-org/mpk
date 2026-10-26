"""Inspect complete strict-module observations and fully rekeyed SDK controls."""
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
 native=read(ROOT/'native/observations/complete-original-strict-modules.json')
 assert native['go_version']=='go1.25.0' and native['prior_original_cases']==656 and len(native['cases'])==1427
 report=read(ROOT/verification['rust_epoch']/'observations/strict-modules/complete-original-strict-module-comparison.json')
 assert report['status']=='passed' and report['sdk_manifests']==18 and len(report['observations'])==1427
 for a,b in zip(native['cases'],report['observations']):
  assert a['name']==b['name'] and a['source_object']==b['source_object'] and a['parse_accepted']==b['parse_accepted'] and a['standard_accepted']==b['standard_accepted'] and a['facts']==b['facts']
  data=bytes.fromhex(a['raw_hex']);obj=a['source_object'];assert len(data)==obj['bytes'] and hashlib.sha256(data).hexdigest()==obj['raw_sha256'];record(obj,'MPK-GO-SOURCE-OBJECT-1.0','id')
 long=next(r for r in native['cases'] if r['name']=='long-complete-directives')
 assert bytes.fromhex(long['raw_hex']).count(b'godebug key=value\n')==40000 and len(long['facts']['debug_settings'])==40000
 directory=ROOT.parent/'compiler-inputs/native/native-originals/versioned-owned';packet=read(directory/'complete-original-module-graph-inputs.json')
 base=json.loads(bytes.fromhex(packet['compiler_inputs_hex']));components=json.loads(bytes.fromhex(packet['components_hex']));original_export=json.loads(bytes.fromhex(packet['export_inputs_hex']))
 original_std=next(r['object'] for r in packet['snapshot_sources'] if r['object']['capability']=='original-go-sdk' and r['object']['path']=='src/go.mod')
 original_raw=raw(directory/'original-compiler-files'/(original_std['raw_sha256']+'.raw'))
 controls=[]
 for name in ['unknown-directive','malformed-tool','malformed-debug','complete-extra-directives']:
  v=read(ROOT/verification['rust_epoch']/f'observations/compilers/coherent-standard-module-{name}.json');accepted=name=='complete-extra-directives'
  assert v['accepted']==accepted and v['error']==(None if accepted else 'StandardModule')
  assert v['all_original_application_source_and_Root_contract_bytes_unchanged'] and v['original_objects']==5697 and v['original_packages']==182 and v['original_compiler_files']==5577
  data=bytes.fromhex(v['changed_raw_hex']);obj=v['changed_object'];assert data.startswith(original_raw) and v['original_object']==original_std
  assert len(data)==obj['bytes'] and hashlib.sha256(data).hexdigest()==obj['raw_sha256'];record(obj,'MPK-GO-SOURCE-OBJECT-1.0','id')
  ids=v['rekeyed_ids'];assert ids[original_std['id']]==obj['id']
  def remap(value):
   if isinstance(value,str):return ids.get(value,value)
   if isinstance(value,list):return [remap(x) for x in value]
   if isinstance(value,dict):return {k:remap(x) for k,x in value.items()}
   return value
  changed=v['components'];record(changed,'MPK-GO-TYPE-COMPONENTS-2.0')
  expected_sources=sorted([obj if r['id']==original_std['id'] else r for r in components['source_objects']],key=lambda r:r['id']);assert changed['source_objects']==expected_sources
  for key,sort_key,domain in [('packages','id','MPK-GO-TYPE-PACKAGE-2.0'),('declarations','id','MPK-GO-TYPE-DECLARATION-2.0'),('types','id','MPK-GO-SOURCE-TYPE-2.0'),('underlying','type_id',None),('aliases','declaration_id',None)]:
   expected=sorted(remap(components[key]),key=lambda r:r[sort_key]);assert changed[key]==expected and len(changed[key])==len(components[key])
   if domain:
    for r in changed[key]:record(r,domain,'id')
  for key in ['package_roots','type_roots']:assert changed[key]==sorted(remap(components[key]))
  wire=v['compiler_inputs'];record(wire,'MPK-PRIVATE-GO-COMPILER-IMAGE-INPUTS-1.0')
  entries=wire['inventory']['files'];assert len(entries)==5577
  for old,new in zip(base['inventory']['files'],entries):
   if old['path']=='go/src/go.mod':assert new['path']==old['path'] and new['sha256']==obj['raw_sha256'] and new['size_bytes']==len(data) and not new['executable']
   else:assert new==old
  assert wire['toolchain']['distribution_sha256']==typed('MPK-BUNDLE-CONTENT-0.1',wire['inventory'])
  for component in wire['components']:
   inventory=component['inventory'];identity=component['identity']
   if identity['kind']=='content':assert identity['content_sha256']==typed('MPK-BUNDLE-CONTENT-0.1',inventory)
   assert identity in wire['toolchain']['components']
  for key,domain in [('envelope','MPK-PRIVATE-GO-SOURCE-CONTRACT-INPUTS-1.0'),('selection','MPK-PRIVATE-GO-SOURCE-SELECTION-INPUTS-1.0'),('snapshot','MPK-PRIVATE-GO-SOURCE-SNAPSHOT-INPUTS-1.0'),('capture','MPK-PRIVATE-GO-SOURCE-CAPTURE-INPUTS-1.0'),('exports','MPK-PRIVATE-GO-SOURCE-EXPORT-INPUTS-1.0')]:record(v[key],domain)
  assert v['envelope']['source_components_sha256']==changed['content_sha256'] and v['envelope']['compiler_sha256']==wire['toolchain']['distribution_sha256']
  assert v['selection']['source_contract_inputs_sha256']==v['envelope']['content_sha256'] and v['capture']['selection_inputs_sha256']==v['selection']['content_sha256'] and v['capture']['snapshot_inputs_sha256']==v['snapshot']['content_sha256']
  assert v['exports']['source_capture_inputs_sha256']==v['capture']['content_sha256'] and v['exports']['compiler_sha256']==wire['toolchain']['distribution_sha256'] and wire['source_export_inputs_sha256']==v['exports']['content_sha256']
  for key in ['metadata_hex','diagnostics_hex','go_binary_sha256','image_sha256','origins']:assert v['exports'][key]==original_export[key]
  assert len(v['snapshot']['objects'])==5697 and len(v['exports']['packages'])==182
  controls.append({'case':name,'accepted':accepted,'error':v['error'],'complete_component_counts':{k:len(changed[k]) for k in ['source_objects','packages','declarations','types','underlying','aliases','package_roots','type_roots']}})
 owner=read(ROOT/verification['rust_epoch']/'observations/compilers/whole-original-compiler-import.json');assert owner['files']==5577 and owner['bytes']==163457770 and owner['original_objects']==5697 and owner['original_packages']==182 and owner['sdk_sources']==5480 and len(owner['sdk_source_package_paths'])==715
 assert len(read(ROOT/verification['rust_epoch']/'observations/compilers/coherent-original-compiler-controls.json')['cases'])==30
 for name in ['active-as-excluded','excluded-as-active','auxiliary-as-metadata']:assert read(ROOT/verification['rust_epoch']/f'observations/compilers/coherent-unused-sdk-role-{name}.json')['error']=='SourceRole'
 for name in ['release','tool','cgo']:assert read(ROOT/verification['rust_epoch']/f'observations/compilers/coherent-executable-{name}.json')['error']=='BuildInfo'
 read(ROOT/verification['rust_epoch']/'observations/compilers/coherent-original-sdk-content-control.json')
 counts=[int(n) for n in re.findall(r'test result: ok\. (\d+) passed; 0 failed; 0 ignored;',raw(ROOT/verification['rust_epoch']/'stdout.log').decode())];assert counts==[7,6,3,2,2],counts
 native_roots=0
 for epoch in ['native',verification['native_consumer_epoch']]:
  rows=[json.loads(line) for line in raw(ROOT/epoch/'stdout.log').splitlines()]
  assert not any(r.get('Action')=='fail' for r in rows)
  roots=[r for r in rows if r.get('Action')=='pass' and r.get('Test') and '/' not in r['Test']];assert len(roots)==1;native_roots+=len(roots)
 result={'status':'passed','affected_rust_roots':20,'native_roots':native_roots,'native_strict_cases':1427,'complete_prior_cases':656,'sdk_manifests':18,'coherent_standard_module_controls':4,'standard_module_controls':controls,'coherent_compiler_claim_controls':30,'coherent_binary_content_controls':3,'coherent_source_role_controls':3,'whole_original_owners':[{'objects':5697,'packages':182,'compiler_files':5577,'compiler_bytes':163457770}],'installed_context_authority':False,'proofs_replayed_in_this_checkpoint':False,'inputs':list(INPUTS.values()),'full_T_gate':'Deferred to final T02-W04: ./scripts/check-fast.sh'}
 out.write_text(json.dumps(result,indent=2)+'\n');print(json.dumps({k:v for k,v in result.items() if k!='inputs'}))
if __name__=='__main__':main()
