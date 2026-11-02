"""Supplement historical conversion receipts with actual source/output correspondence.
The saved original conversion drivers and their original receipts stay unchanged.
"""
from pathlib import Path
import base64,datetime,hashlib,json,runpy
OUT=Path(__file__).resolve().parent
RUN=OUT/'derived-input-audit';RUN.mkdir(exist_ok=False)
(RUN/'driver.py').write_bytes(Path(__file__).read_bytes())
read=runpy.run_path(str(OUT/'read-originals.py'))['read']
parent=runpy.run_path(str(OUT.parent/'source-closure/read-originals.py'))['read']
def digest(p):
 raw=Path(p).read_bytes();return {'path':str(p),'bytes':len(raw),'sha256':hashlib.sha256(raw).hexdigest()}
def canonical(v):return json.dumps(v,sort_keys=True,separators=(',',':'),ensure_ascii=False).encode()
rows=[]
for driver,corpus,archive,reader in [('derived-native','corpus-parent.json','native-original',parent),('derived-inventory','corpus-inventory31.json','inventory-native-original',read),('derived-inferred','corpus.json','native-original',read)]:
 manifest,files=reader(archive)
 packet=json.loads((OUT/corpus).read_bytes());receipt=json.loads((OUT/driver/'result.json').read_bytes())
 assert hashlib.sha256((OUT/driver/'driver.py').read_bytes()).hexdigest()==receipt['actual_driver_prelaunch_sha256']
 current=digest(OUT/corpus)
 assert {k:current[k]for k in ['bytes','sha256']}=={k:receipt['output'][k]for k in ['bytes','sha256']}
 assert len(packet)==receipt['cases']==json.loads(files['closure-final-controls.json'])['cases']
 modules={}
 for name,raw in files.items():
  if name.startswith('original-source-objects/')and name.endswith('.json'):
   obj=json.loads(raw)
   if obj.get('path')in ['src/go.mod','src/unsafe/unsafe.go']or obj.get('kind')=='go_mod'and obj.get('path')=='go.mod':modules[obj['id']]=(obj,files[name[:-5]+'.raw'])
 for n,row in enumerate(packet):
  before=json.loads(files[f'closure-{n:02}-input.json']);native=json.loads(files[f'closure-{n:02}-actual.json'])
  assert row['native']==native
  assert row['input']['root']==before['root']and row['input']['expected_issue_codes']==before['expected_issue_codes']
  sources={s['object']['id']:(s['object'],base64.b64decode(s['raw'],validate=True))for s in native['sources']};sources.update(modules)
  assert sources=={s['object']['id']:(s['object'],bytes.fromhex(s['raw_hex']))for s in row['input']['sources']}
  c=json.loads(bytes.fromhex(row['input']['components_hex']));assert c['schema']=='mpk.go.type_components.v2'
  assert all(not c[key]for key in ['declarations','underlying','aliases'])and len(c['types'])==1
  assert c['types'][0]['kind']=='builtin'and c['types'][0]['name']=='bool'and c['type_roots']==[c['types'][0]['id']]
  assert {s['id']:s for s in c['source_objects']}=={sid:o for sid,(o,_)in sources.items()}
  for package in c['packages']:
   assert len(package['source_objects'])==1
   source=sources[package['source_objects'][0]][1]
   import re
   assert re.search(rb'(?m)^package\s+(\w+)',source).group(1).decode()==package['package_name']
   assert package['module_source_object_id']in sources
  checksum=c.pop('content_sha256');assert checksum==hashlib.sha256(b'MPK-GO-TYPE-COMPONENTS-2.0\0'+canonical(c)).hexdigest()
 rows.append({'actual_original_directory':manifest['original_directory'],'actual_archive_manifest':digest((OUT.parent/'source-closure'if reader==parent else OUT)/archive/'manifest.json'),'historical_conversion_receipt':digest(OUT/driver/'result.json'),'actual_retained_corpus':current,'cases':len(packet),'all_original_raw_source_objects_exact':True,'only_seed_bool_candidate_type':True})
record={'status':'passed','observed_at_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'actual_driver_prelaunch_sha256':hashlib.sha256((RUN/'driver.py').read_bytes()).hexdigest(),'conversions':rows,'historical_receipt_limit':'The original converter always printed original_parent_manifest and the then-current corpus.json output path. Those original fields are preserved. This supplemental audit identifies the actual fresh native directories and the renamed retained historical corpora; it is an after-run correspondence check, not a prelaunch receipt.','manual_original_reference':digest(OUT/'original-reference.json'),'manual_reference_is_consumer_input':False,'source_packets_or_original_receipts_modified':False}
(RUN/'result.json').write_text(json.dumps(record,indent=2)+'\n');print(json.dumps({'status':'passed','conversions':len(rows),'cases':[r['cases']for r in rows]}))
