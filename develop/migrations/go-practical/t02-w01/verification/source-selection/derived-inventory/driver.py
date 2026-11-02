"""Retain exact parent observations and build source-only scope input packets."""
from pathlib import Path
import base64,hashlib,json,re,runpy,sys
OUT=Path(__file__).resolve().parent;ROOT=Path.cwd();RUN=OUT/(sys.argv[1]if len(sys.argv)>1 else 'derived-native');RUN.mkdir(exist_ok=False);(RUN/'driver.py').write_bytes(Path(__file__).read_bytes())
read=runpy.run_path(str(OUT.parent/'source-closure/read-originals.py'))['read'];manifest,files=read('native-original')
if len(sys.argv)>2:
 source=Path(sys.argv[2]);files={str(p.relative_to(source)):p.read_bytes()for p in source.rglob('*')if p.is_file()}

def canonical(x):return json.dumps(x,ensure_ascii=False,sort_keys=True,separators=(',',':')).encode()
def seal(x,domain,field='id'):
 x[field]=hashlib.sha256(domain.encode()+b'\0'+canonical(x)).hexdigest();return x
modules=[]
for name,raw in files.items():
 if name.startswith('original-source-objects/')and name.endswith('.json'):
  obj=json.loads(raw)
  if obj.get('path')in ['src/go.mod','src/unsafe/unsafe.go']or obj.get('kind')=='go_mod'and obj.get('path')=='go.mod':modules.append({'object':obj,'raw_hex':files[name[:-5]+'.raw'].hex()})
packets=[]
for n in range(31):
 before=json.loads(files[f'closure-{n:02}-input.json']);native=json.loads(files[f'closure-{n:02}-actual.json'])
 sources=[{'object':s['object'],'raw_hex':base64.b64decode(s['raw'],validate=True).hex()}for s in native['sources']]
 sources+=modules;sources=sorted({s['object']['id']:s for s in sources}.values(),key=lambda s:s['object']['id'])
 main=next(s['object']['id']for s in sources if s['object']['path']=='go.mod');stdlib=next(s['object']['id']for s in sources if s['object']['path']=='src/go.mod')
 packages=[]
 for path,filename,module in [('example.com/policy','policy.go',main),('example.com/policy/helper','helper/helper.go',main),('unsafe','src/unsafe/unsafe.go',stdlib)]:
  source=next(s for s in sources if s['object']['path']==filename)
  name=re.search(rb'(?m)^package\s+(\w+)',bytes.fromhex(source['raw_hex'])).group(1).decode()
  packages.append(seal({'schema':'mpk.go.type_package.v2','module_source_object_id':module,'import_path':path,'package_name':name,'source_objects':[source['object']['id']]},'MPK-GO-TYPE-PACKAGE-2.0'))
 builtin=seal({'schema':'mpk.go.source_type.v2','kind':'builtin','name':'bool','width':None,'signed':None},'MPK-GO-SOURCE-TYPE-2.0')
 components=seal({'schema':'mpk.go.type_components.v2','source_objects':[s['object']for s in sources],'packages':sorted(packages,key=lambda p:p['id']),'declarations':[],'types':[builtin],'underlying':[],'aliases':[],'package_roots':sorted(p['id']for p in packages),'type_roots':[builtin['id']]},'MPK-GO-TYPE-COMPONENTS-2.0','content_sha256')
 packets.append({'input':{'components_hex':canonical(components).hex(),'sources':sources,'root':before['root'],'expected_issue_codes':before['expected_issue_codes']},'native':native})
raw=canonical(packets)+b'\n';target=OUT/'corpus.json';assert not target.exists();target.write_bytes(raw)
(RUN/'result.json').write_text(json.dumps({'status':'passed','actual_driver_prelaunch_sha256':hashlib.sha256((RUN/'driver.py').read_bytes()).hexdigest(),'original_parent_manifest':str(OUT.parent/'source-closure/native-original/manifest.json'),'original_parent_manifest_sha256':hashlib.sha256((OUT.parent/'source-closure/native-original/manifest.json').read_bytes()).hexdigest(),'cases':31,'complete_original_source_packets_edited':False,'candidate_types_are_seed_bool_only':True,'output':{'path':str(target),'bytes':len(raw),'sha256':hashlib.sha256(raw).hexdigest()}},indent=2)+'\n')
print(json.dumps({'cases':31,'bytes':len(raw)}))
