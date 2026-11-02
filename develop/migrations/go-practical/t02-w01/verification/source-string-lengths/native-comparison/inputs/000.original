"""Compare original compiler observations after complete Rust verification.

Native observations remain comparison data, never constructor authority.
"""
from pathlib import Path
import datetime,hashlib,json,runpy
ROOT=Path(__file__).resolve().parent
OUT=ROOT/'native-comparison';OUT.mkdir(exist_ok=False)
def seal(p):
 b=Path(p).read_bytes();return {'path':str(p),'bytes':len(b),'sha256':hashlib.sha256(b).hexdigest()}
def identity(domain,body):
 return hashlib.sha256(domain.encode()+b'\0'+json.dumps(body,sort_keys=True,separators=(',',':'),ensure_ascii=False).encode()).hexdigest()
run_path=ROOT/'rust/result.json';run=json.loads(run_path.read_bytes());assert run['status']=='passed' and run['whole_source_inputs_unchanged']
relocations=json.loads((ROOT/'native-original-relocations.json').read_bytes());assert relocations['status']=='passed'
archive=ROOT/'native-output-original/manifest.json';manifest=json.loads(archive.read_bytes());assert manifest['status']=='passed'
inputs=[Path(__file__),ROOT/'read-originals.py',ROOT/'native-original-relocations.json',archive,ROOT/'corpus.json',run_path,ROOT/'rust/inputs.json']
inputs.extend(Path(r['encoded']['path']) for r in manifest['chunks'])
metadata=sorted(p for p in (ROOT/'rust/certificates').glob('source-length-*.sources.json'));assert len(metadata)==12;inputs.extend(metadata)
rows=[]
for i,p in enumerate(inputs):
 target=OUT/'inputs'/f'{i:03d}.original';target.parent.mkdir(exist_ok=True);target.write_bytes(p.read_bytes());mode=p.stat().st_mode&0o777;target.chmod(mode)
 rows.append({**seal(p),'mode':mode,'complete_prelaunch_snapshot':seal(target)})
receipt=OUT/'prelaunch-inputs.json';receipt.write_text(json.dumps(rows,indent=2)+'\n');(OUT/'driver.py').write_bytes(Path(__file__).read_bytes())
result={'status':'running','actual_driver_prelaunch':seal(OUT/'driver.py'),'complete_prelaunch_inputs':seal(receipt),'started_at_utc':datetime.datetime.now(datetime.timezone.utc).isoformat()}
def save():(OUT/'result.json').write_text(json.dumps(result,indent=2)+'\n')
save()
try:
 _,files=runpy.run_path(str(ROOT/'read-originals.py'))['read']('native-output-original')
 calls=json.loads(files['length-call-types.json']);assert len(calls)==4 and {r['operand_type'] for r in calls}=={'string','example.com/policy.Named','example.com/policy.Chained','example.com/policy/helper.Remote'}
 assert all(r['result_type']=='int' and r['constant'] is False for r in calls)
 corpus=json.loads((ROOT/'corpus.json').read_bytes());components=json.loads(bytes.fromhex(corpus['input']['components_hex']))
 assert not components['declarations'] and len(components['types'])==1 and components['types'][0]['name']=='bool'
 packages={p['import_path']:p for p in components['packages']};sources={r['object']['id']:bytes.fromhex(r['raw_hex']) for r in corpus['input']['sources']}
 for call in calls:assert sources[call['source']][call['start']:call['end']].startswith(b'len(')
 strings={identity('MPK-GO-SOURCE-TYPE-2.0',{'schema':'mpk.go.source_type.v2','kind':'builtin','name':'string','width':None,'signed':None}):'string'}
 for name in ['example.com/policy.Named','example.com/policy.Chained','example.com/policy/helper.Remote']:
  pkg,local=name.rsplit('.',1);d=corpus['declarations'][name]
  body={'schema':'mpk.go.type_declaration.v2','package_id':packages[pkg]['id'],'source_object_id':d['source'],'kind':'type','start_byte':d['start'],'end_byte':d['end'],'name':local}
  declaration=identity('MPK-GO-TYPE-DECLARATION-2.0',body)
  tid=identity('MPK-GO-SOURCE-TYPE-2.0',{'schema':'mpk.go.source_type.v2','kind':'nominal','declaration_id':declaration});strings[tid]=name
 units={}
 for p in metadata:
  m=json.loads(p.read_bytes());units.setdefault(m['unit'],[]).append(m)
 for case in corpus['cases']:
  unit=case['input']['name'];group=units[unit]
  assert len(group)==4 and {m['source_string_type_id'] for m in group}==set(strings)
  for m in group:
   assert m['cases']==case['native']['global_cases']
   keys={k['id']:k for k in m['types']};rid=m['source_result_type_id'];k=keys[rid]
   assert k['kind']=='builtin' and k['name']=='int' and k['width']==64 and k['signed'] is True
   assert not any(k.get('name')=='uint8' for k in keys.values())
   assert m['all_prior_string_identities_unchanged'] and m['source_expression_or_context_authority'] is False
 assert sorted(len(c['native']['global_cases']) for c in corpus['cases'])==[0,4,70]
 for r in rows:
  assert seal(r['path'])=={k:r[k] for k in ['path','bytes','sha256']} and Path(r['path']).stat().st_mode&0o777==r['mode']
 result.update(status='passed',whole_inputs_unchanged=True,native_dynamic_len_calls=calls,native_string_type_identities=strings,source_int_width=64,source_int_signed=True,all_12_source_length_rows_compared=True,complete_original_error_case_counts=[0,4,70],native_facts_passed_to_Rust_constructor=False,source_expression_or_context_authority=False,finished_at_utc=datetime.datetime.now(datetime.timezone.utc).isoformat());save()
 print(json.dumps({'status':'passed','dynamic_int_len_calls':4,'string_types':4,'source_length_rows':12,'error_cases':[0,4,70]}))
except BaseException:
 result['status']='failed';save();raise
