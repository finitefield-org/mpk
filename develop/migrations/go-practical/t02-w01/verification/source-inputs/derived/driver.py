"""Use actual production source-only packets, retaining every original byte/mode."""
from pathlib import Path
import base64,datetime,hashlib,json
ROOT=Path(__file__).resolve().parent;OUT=ROOT/'derived';OUT.mkdir(exist_ok=False)
def seal(p):
 b=Path(p).read_bytes();return {'path':str(p),'bytes':len(b),'sha256':hashlib.sha256(b).hexdigest()}
parents=[ROOT/'native/result.json',ROOT/'native-replacement-corrected/result.json']
for p in parents:
 r=json.loads(p.read_bytes());assert r['status']=='passed' and r['whole_source_inputs_unchanged']
folders=[ROOT/f'native/native-originals/{unit}' for unit in ['pure','wide']]+[ROOT/'native-replacement-corrected/native-originals']
inputs=[Path(__file__),*parents,ROOT.parent/'source-invocations/corpus.json']
for i,folder in enumerate(folders):
 inputs.extend(folder/name for name in ['unit-inputs.json','source-input-provenance.json','storage-original-exports.json'])
 if i==2:inputs.append(folder/'replacement-root.json')
rows=[]
for i,p in enumerate(inputs):
 target=OUT/f'{i:03d}.original.hex';target.write_text(p.read_bytes().hex()+'\n');target.chmod(0o644);rows.append({**seal(p),'mode':p.stat().st_mode&0o777,'complete_prelaunch_snapshot':seal(target),'snapshot_encoding':'hex','snapshot_mode':0o644})
receipt=OUT/'prelaunch-inputs.json';receipt.write_text(json.dumps(rows,indent=2)+'\n');(OUT/'driver.py').write_bytes(Path(__file__).read_bytes());record={'status':'running','actual_driver_prelaunch':seal(OUT/'driver.py'),'complete_prelaunch_inputs':seal(receipt),'native_type_facts_used_as_constructor_inputs':False,'started_at_utc':datetime.datetime.now(datetime.timezone.utc).isoformat()}
def save():(OUT/'result.json').write_text(json.dumps(record,indent=2)+'\n')
save()
try:
 def input_packet(folder):
  exports=json.loads((folder/'storage-original-exports.json').read_bytes());assert exports['error']=='<nil>'
  packet=json.loads((folder/'unit-inputs.json').read_bytes());raw=base64.b64decode(packet['canonical_components'],validate=True);component=json.loads(raw)
  assert not component['declarations'] and not component['underlying'] and not component['aliases'] and len(component['types'])==1 and component['types'][0]['name']=='bool'
  provenance=json.loads((folder/'source-input-provenance.json').read_bytes());assert provenance['registered_context_or_program_proof'] is False
  original={'components_hex':raw.hex(),'sources':[{'object':s['object'],'raw_hex':base64.b64decode(s['raw'],validate=True).hex()} for s in packet['sources']]}
  return original,component,provenance
 original=json.loads((ROOT.parent/'source-invocations/corpus.json').read_bytes());units=[]
 for name,folder in zip(['pure','wide'],folders[:2]):
  data,component,provenance=input_packet(folder);assert len(component['packages'])==3
  prior=next(u for u in original['units'] if u['name']==name);assert data==prior['input'],'production handoff changed complete original bytes'
  units.append({'name':name,'input':data,'private_owner_provenance':provenance})
 data,component,provenance=input_packet(folders[2]);replacement=json.loads((folders[2]/'replacement-root.json').read_bytes())
 assert len(component['packages'])==1 and replacement['main_source_object_id']!=replacement['requested_module_source_object_id']
 assert component['packages'][0]['module_source_object_id']==replacement['requested_module_source_object_id']
 replacement.update(input=data,private_owner_provenance=provenance)
 out=ROOT/'corpus.json';assert not out.exists();out.write_text(json.dumps({'units':units,'replacement':replacement},sort_keys=True,separators=(',',':'))+'\n')
 for row in rows:assert seal(row['path'])=={k:row[k] for k in ['path','bytes','sha256']} and Path(row['path']).stat().st_mode&0o777==row['mode']
 record.update(status='passed',whole_inputs_unchanged=True,complete_original_source_packet=seal(out),original_units_byte_identical=['pure','wide'],source_packages_per_original_unit=3,replacement_main_and_requested_modules_distinct=True,source_program_or_context_proof=False,finished_at_utc=datetime.datetime.now(datetime.timezone.utc).isoformat());save();print(json.dumps({'status':'passed','original_units_byte_identical':['pure','wide'],'replacement_module_retained':True,'native_type_facts_used_as_constructor_inputs':False}))
except BaseException:
 record['status']='failed';save();raise
