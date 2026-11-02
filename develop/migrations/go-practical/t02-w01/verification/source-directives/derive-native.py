"""Derive source-only packets from the actual terminal native output directory."""
from pathlib import Path
import base64,datetime,hashlib,json,re,sys
OUT=Path(__file__).resolve().parent;RUN=OUT/'derived';RUN.mkdir(exist_ok=False)
(RUN/'driver.py').write_bytes(Path(__file__).read_bytes());SOURCE=Path(sys.argv[1]);target=OUT/'corpus.json';assert not target.exists()
def canonical(x):return json.dumps(x,ensure_ascii=False,sort_keys=True,separators=(',',':')).encode()
def hashed(x,domain,field='id'):
 x[field]=hashlib.sha256(domain.encode()+b'\0'+canonical(x)).hexdigest();return x
def seal(p):
 raw=p.read_bytes();return {'path':str(p),'bytes':len(raw),'sha256':hashlib.sha256(raw).hexdigest(),'mode':p.stat().st_mode&0o777}
paths=sorted(p for p in SOURCE.rglob('*')if p.is_file());inputs=[seal(p)for p in paths]
(RUN/'inputs.json').write_text(json.dumps({'actual_original_directory':str(SOURCE),'actual_driver_prelaunch':seal(RUN/'driver.py'),'files':inputs},indent=2)+'\n')
modules=[]
for path in sorted((SOURCE/'original-source-objects').glob('*.json')):
 obj=json.loads(path.read_bytes())
 if obj.get('path')in ['src/go.mod','src/unsafe/unsafe.go']or obj.get('kind')=='go_mod'and obj.get('path')=='go.mod':modules.append({'object':obj,'raw_hex':path.with_suffix('.raw').read_bytes().hex()})
packets=[];cases=json.loads((SOURCE/'closure-final-controls.json').read_bytes())['cases']
for n in range(cases):
 before=json.loads((SOURCE/f'closure-{n:02}-input.json').read_bytes());native=json.loads((SOURCE/f'closure-{n:02}-actual.json').read_bytes())
 sources=[{'object':s['object'],'raw_hex':base64.b64decode(s['raw'],validate=True).hex()}for s in native['sources']]+modules
 sources=sorted({s['object']['id']:s for s in sources}.values(),key=lambda s:s['object']['id'])
 main=next(s['object']['id']for s in sources if s['object']['path']=='go.mod');stdlib=next(s['object']['id']for s in sources if s['object']['path']=='src/go.mod');packages=[]
 for path,filename,module in [('example.com/policy','policy.go',main),('example.com/policy/helper','helper/helper.go',main),('unsafe','src/unsafe/unsafe.go',stdlib)]:
  source=next(s for s in sources if s['object']['path']==filename);name=re.search(rb'(?m)^package\s+(\w+)',bytes.fromhex(source['raw_hex'])).group(1).decode()
  packages.append(hashed({'schema':'mpk.go.type_package.v2','module_source_object_id':module,'import_path':path,'package_name':name,'source_objects':[source['object']['id']]},'MPK-GO-TYPE-PACKAGE-2.0'))
 builtin=hashed({'schema':'mpk.go.source_type.v2','kind':'builtin','name':'bool','width':None,'signed':None},'MPK-GO-SOURCE-TYPE-2.0')
 components=hashed({'schema':'mpk.go.type_components.v2','source_objects':[s['object']for s in sources],'packages':sorted(packages,key=lambda p:p['id']),'declarations':[],'types':[builtin],'underlying':[],'aliases':[],'package_roots':sorted(p['id']for p in packages),'type_roots':[builtin['id']]},'MPK-GO-TYPE-COMPONENTS-2.0','content_sha256')
 packets.append({'input':{'components_hex':canonical(components).hex(),'sources':sources,'root':before['root'],'expected_issue_codes':before['expected_issue_codes']},'native':native})
target.write_bytes(canonical(packets)+b'\n');assert inputs==[seal(p)for p in paths]
record={'status':'passed','actual_original_directory':str(SOURCE),'actual_driver_prelaunch':seal(RUN/'driver.py'),'original_input_receipt':seal(RUN/'inputs.json'),'cases':cases,'output':seal(target),'candidate_types_are_seed_bool_only':True,'complete_original_source_packets_edited':False,'all_original_bytes_and_modes_unchanged':True,'native_AST_type_or_call_facts_passed_to_constructor':False,'finished_at_utc':datetime.datetime.now(datetime.timezone.utc).isoformat()}
(RUN/'result.json').write_text(json.dumps(record,indent=2)+'\n');print(json.dumps({'status':'passed','cases':cases,'bytes':record['output']['bytes']}))
