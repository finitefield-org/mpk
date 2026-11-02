"""Compare complete original Go operation sites, types and child call origins."""
from pathlib import Path
import datetime,hashlib,json,re,runpy
ROOT=Path(__file__).resolve().parent;OUT=ROOT/'native-comparison';OUT.mkdir(exist_ok=False)
def seal(p):
 b=Path(p).read_bytes();return {'path':str(p),'bytes':len(b),'sha256':hashlib.sha256(b).hexdigest()}
def ident(domain,body):return hashlib.sha256(domain.encode()+b'\0'+json.dumps(body,sort_keys=True,separators=(',',':'),ensure_ascii=False).encode()).hexdigest()
run=ROOT/'final-streams/result.json';r=json.loads(run.read_bytes());assert r['status']=='passed' and r['whole_source_inputs_unchanged']
manifest=ROOT/'native-output-original/manifest.json';archive=json.loads(manifest.read_bytes());assert archive['status']=='passed'
inputs=[Path(__file__),ROOT/'read-originals.py',ROOT/'native-original-relocations.json',manifest,ROOT/'corpus.json',run,ROOT/'final-streams/prelaunch-inputs.json']+[Path(r['encoded']['path']) for r in archive['chunks']]+sorted((ROOT/'final-streams/certificates').glob('*.sources.json'))+sorted((ROOT/'final-streams/certificates').glob('*-inventory.json'))
rows=[]
for i,p in enumerate(inputs):
 q=OUT/'inputs'/f'{i:03d}.original';q.parent.mkdir(exist_ok=True);q.write_bytes(p.read_bytes());q.chmod(p.stat().st_mode&0o777);rows.append({**seal(p),'mode':p.stat().st_mode&0o777,'complete_prelaunch_snapshot':seal(q)})
receipt=OUT/'prelaunch-inputs.json';receipt.write_text(json.dumps(rows,indent=2)+'\n');(OUT/'driver.py').write_bytes(Path(__file__).read_bytes())
result={'status':'running','actual_driver_prelaunch':seal(OUT/'driver.py'),'complete_prelaunch_inputs':seal(receipt),'started_at_utc':datetime.datetime.now(datetime.timezone.utc).isoformat()}
def save():(OUT/'result.json').write_text(json.dumps(result,indent=2)+'\n')
save()
try:
 _,files=runpy.run_path(str(ROOT/'read-originals.py'))['read']('native-output-original');corpus=json.loads((ROOT/'corpus.json').read_bytes());comparisons=[]
 for unit in corpus['units']:
  name=unit['name'];native=json.loads(files[name+'/native-operations.json']);closure=json.loads(files[name+'/native-closure.json']);inventory=json.loads((ROOT/f'final-streams/certificates/{name}-inventory.json').read_bytes());assert len(native)==len(inventory['sites'])==12
  packet=json.loads(bytes.fromhex(unit['input']['components_hex']));assert len(packet['packages'])==3;packages={p['import_path']:p for p in packet['packages']};source={s['object']['id']:bytes.fromhex(s['raw_hex']) for s in unit['input']['sources']}
  alias=unit['declarations']['example.com/policy.Alias'];assert re.fullmatch(rb'Alias\s*=\s*Text',source[alias['source']][alias['start']:alias['end']])
  def type_id(display):
   if display=='example.com/policy.Alias':display='example.com/policy.Text'
   if display=='byte':display='uint8'
   if display=='untyped string':display='string'
   widths={'int':(64,True),'int8':(8,True),'uint8':(8,False),'uint64':(64,False)}
   if display in widths:width,signed=widths[display];body={'schema':'mpk.go.source_type.v2','kind':'builtin','name':display,'width':width,'signed':signed}
   elif display=='string':body={'schema':'mpk.go.source_type.v2','kind':'builtin','name':'string','width':None,'signed':None}
   else:
    pkg,local=display.rsplit('.',1);d=unit['declarations'][display]
    decl=ident('MPK-GO-TYPE-DECLARATION-2.0',{'schema':'mpk.go.type_declaration.v2','package_id':packages[pkg]['id'],'source_object_id':d['source'],'kind':'type','start_byte':d['start'],'end_byte':d['end'],'name':local})
    body={'schema':'mpk.go.source_type.v2','kind':'nominal','declaration_id':decl}
   return ident('MPK-GO-SOURCE-TYPE-2.0',body)
  actual={(r['source'],r['start'],r['end']):r for r in inventory['sites']};assert len(actual)==12
  for observed in native:
   key=tuple(observed[k] for k in ['source','start','end']);derived=actual[key];kind=observed['kind']
   assert derived['caller_span']==[observed['caller_start'],observed['caller_end']]
   assert derived['operand_spans']==[[p['start'],p['end']] for p in observed['operands']]
   assert derived['result']==type_id(observed['result'])
   if kind=='ConstantLength':
    assert derived['kind']==f"ConstantLength({observed['constant']})" and derived['operand_types']==[None]
   else:
    assert derived['kind']==kind and derived['operand_types']==[type_id(p['type']) for p in observed['operands']]
    metadata=next(json.loads(p.read_bytes()) for p in (ROOT/'final-streams/certificates').glob(name+'-*.sources.json') if json.loads(p.read_bytes())['node']==derived['node'])
    assert metadata['cases']==closure['global_cases'] and len(metadata['cases'])==(70 if name=='wide' else 0)
    calls=[c for c in closure['calls'] if c['source']==observed['source'] and c['start']>=observed['start'] and c['end']<=observed['end']]
    expected=sorted((c['source'],c['start'],c['end'],c['callee']['source'],c['callee']['start'],c['callee']['end']) for c in calls)
    assert sorted(tuple(c) for c in metadata['call_origins'])==expected
  comparisons.append({'unit':name,'complete_sites':12,'dynamic_rows':11,'constant_int_length_sites':1,'cases':len(closure['global_cases']),'all_types_origins_and_callees_equal':True})
 for row in rows:assert seal(row['path'])=={k:row[k] for k in ['path','bytes','sha256']} and Path(row['path']).stat().st_mode&0o777==row['mode']
 result.update(status='passed',whole_inputs_unchanged=True,units=comparisons,source_expression_execution_or_context_authority=False,native_facts_used_as_constructor_inputs=False,finished_at_utc=datetime.datetime.now(datetime.timezone.utc).isoformat());save();print(json.dumps({'status':'passed','complete_source_sites':24,'constant_int_sites':2,'cases':[0,70]}))
except BaseException:
 result['status']='failed';save();raise
