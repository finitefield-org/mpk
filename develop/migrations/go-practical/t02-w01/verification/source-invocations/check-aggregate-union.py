"""Independently compare every whole named type/body/proof in both exact unions."""
from pathlib import Path
import datetime,hashlib,importlib.util,json
ROOT=Path(__file__).resolve().parent;OUT=ROOT/'aggregate-union';OUT.mkdir(exist_ok=False)
def seal(p):
 b=Path(p).read_bytes();return {'path':str(p),'bytes':len(b),'sha256':hashlib.sha256(b).hexdigest()}
inputs=[Path(__file__),ROOT/'structural-oracle.py',ROOT.parent/'source-error-layouts/structural-oracle.py',Path('develop/probes/go-practical/carrier_terms.py'),ROOT/'aggregation-interned/result.json',ROOT/'aggregation-interned/prelaunch-inputs.json',ROOT/'generation-linked/result.json',ROOT/'generation-linked/prelaunch-inputs.json']+sorted((ROOT/'aggregation-interned/certificates').iterdir())+sorted((ROOT/'generation-linked/certificates').iterdir());rows=[]
for i,p in enumerate(inputs):
 target=OUT/'inputs'/f'{i:03d}.original';target.parent.mkdir(exist_ok=True);target.write_bytes(p.read_bytes());target.chmod(p.stat().st_mode&0o777);rows.append({**seal(p),'mode':p.stat().st_mode&0o777,'complete_prelaunch_snapshot':seal(target)})
receipt=OUT/'prelaunch-inputs.json';receipt.write_text(json.dumps(rows,indent=2)+'\n');(OUT/'driver.py').write_bytes(Path(__file__).read_bytes());record={'status':'running','actual_driver_prelaunch':seal(OUT/'driver.py'),'complete_prelaunch_inputs':seal(receipt),'started_at_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'kernel_proofs_checked':False}
def save():(OUT/'result.json').write_text(json.dumps(record,indent=2)+'\n')
save()
try:
 spec=importlib.util.spec_from_file_location('full_named_union',ROOT/'structural-oracle.py');m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m)
 def named(table,term):
  names=list(table)
  def signature(d):
   _,kind,ty,*rest=d
   if kind==1:return (kind,term(ty),term(rest[0]),rest[1])
   if kind==2:return (kind,term(ty),term(rest[0]))
   if kind in [4,5]:return (kind,term(ty),names[rest[0]],rest[1])
   assert kind==3,'nonordinary original declaration';return (kind,term(ty))
  return {name:signature(d) for name,d in table.items()}
 units=[]
 for unit in ['pure','wide']:
  p=ROOT/'aggregation-interned/certificates'/f'{unit}-all.mpcert';raw,module,table,term=m.decode(p);actual=named(table,term);expected={};checks=0
  originals=sorted((ROOT/'generation-linked/certificates').glob(unit+'-*.mpcert'))
  for original in originals:
   _,_,t,fn=m.decode(original)
   for name,value in named(t,fn).items():
    assert actual[name]==value,(unit,original.name,name,'whole type/body/proof changed')
    if name in expected:assert expected[name]==value
    else:expected[name]=value
    checks+=1
  assert set(actual)==set(expected),'missing or extra original declaration'
  metadata=json.loads(p.with_suffix('.sources.json').read_bytes());assert len(metadata['components'])==len(originals) and metadata['declarations']==len(actual)
  for component,original in zip(metadata['components'],originals):
   assert component['name']==original.stem and component['sha256']==seal(original)['sha256'] and component['bytes']==original.stat().st_size
  units.append({'unit':unit,'complete_original_streams':len(originals),'complete_original_named_declarations':len(actual),'whole_original_declaration_comparisons':checks,'certificate':seal(p),'all_named_types_bodies_proofs_and_recursor_ownership_equal':True,'no_missing_or_extra_declarations':True,'source_U_or_type_pruning':False})
 for row in rows:assert seal(row['path'])=={k:row[k] for k in ['path','bytes','sha256']} and Path(row['path']).stat().st_mode&0o777==row['mode']
 record.update(status='passed',whole_inputs_unchanged=True,units=units,complete_original_streams=sum(u['complete_original_streams'] for u in units),whole_original_declaration_comparisons=sum(u['whole_original_declaration_comparisons'] for u in units),finished_at_utc=datetime.datetime.now(datetime.timezone.utc).isoformat());save();print(json.dumps({'status':'passed','complete_original_streams':record['complete_original_streams'],'whole_original_declaration_comparisons':record['whole_original_declaration_comparisons']}))
except BaseException:
 record['status']='failed';save();raise
