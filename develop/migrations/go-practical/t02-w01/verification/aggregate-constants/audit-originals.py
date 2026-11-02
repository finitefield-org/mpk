"""Compare terminal original buffers literally and audit compiler correspondence."""
from pathlib import Path
import datetime,hashlib,json,runpy
OUT=Path(__file__).resolve().parent;RUN=OUT/'original-byte-audit';RUN.mkdir(exist_ok=False);(RUN/'driver.py').write_bytes(Path(__file__).read_bytes())
read=runpy.run_path(str(OUT/'read-originals.py'))['read'];originals={};count=size=0;rows=[]
for p in sorted(OUT.glob('*/manifest.json')):
 name=p.parent.name;manifest,files=read(name);originals[name]=files
 for row in manifest['original_inputs']:
  path=Path(row['path']);raw=path.read_bytes();assert raw==files[row['member']]and path.stat().st_mode&0o777==row['mode'];count+=1;size+=len(raw)
 rows.append({'name':name,'files':len(files),'bytes':sum(map(len,files.values()))})
assert len(rows)==15
cases=json.loads((OUT/'corpus.json').read_bytes());assert len(cases)==406
counts={'supported':0,'rejected':0,'outside':0}
for n,row in enumerate(cases):
 assert json.loads(originals['native-original'][f'case-{n:05}/input.json'])==row['input'];assert json.loads(originals['native-original'][f'case-{n:05}/native.json'])==row['native']
 files=originals['rust-original'];assert json.loads(files[f'case-{n:05}-input.json'])==row['input'];actual=json.loads(files[f'case-{n:05}-actual.json'])
 if row['native']['accepted']and not row['input']['outside_aggregate_reader']:assert actual==row['native'];counts['supported']+=1
 elif row['native']['accepted']:assert not actual['accepted'];counts['outside']+=1
 else:assert not actual['accepted'];counts['rejected']+=1
assert counts=={'supported':345,'rejected':56,'outside':5}
assert json.loads(originals['rust-original']['summary.json'])=={'accepted':345,'rejected':56,'outside':5,'failures':[]}
deep=json.loads(originals['rust-original']['aggregate-deep-actual.json']);assert deep['cross_reader_links']==512 and deep['result']['integer_decimal']=='1'
for n in range(4):assert json.loads(originals['rust-original'][f'aggregate-thread-{n}-actual.json'])['integer_decimal']==str(n+1)
assert json.loads(originals['rust-original']['aggregate-local-actual.json'])['locals']==1
legacy={}
for name,corpus,expected,flag in [('scalar-original','scalar-typing/corpus.json',(2776,2205,4),'outside_scalar_typing'),('aggregate-original','value-typing/corpus-aliases.json',(474,463,13),'outside_value_typing'),('method-original','method-typing/corpus-calls.json',(143,300,18),'outside_value_typing'),('assignment-original','assignment-typing/corpus.json',(475,951,7),'outside_assignment_typing'),('return-original','return-typing/corpus-import.json',(150,470,10),'outside_return_typing'),('constant-original','source-constants/corpus.json',(4833,281,3),'outside_scalar_reader'),('body-original','body-typing/corpus.json',(176,299,20),'outside_body_typing')]:
 files=originals[name];a=r=o=0
 for n,row in enumerate(json.loads((OUT.parent/corpus).read_bytes())):
  assert json.loads(files[f'case-{n:05}-input.json'])==row['input'];actual=json.loads(files[f'case-{n:05}-actual.json'])
  if row['native']['accepted']and actual['accepted']:assert actual==row['native'];a+=1
  elif row['native']['accepted']:assert row['input'].get(flag,row['input'].get('outside_value_typing',False));o+=1
  else:assert not actual['accepted'];r+=1
 assert(a,r,o)==expected;legacy[name]={'accepted':a,'rejected':r,'outside':o}
files=originals['builtin-original'];a=r=o=p=0
for n,row in enumerate(json.loads((OUT.parent/'slice-builtins/corpus.json').read_bytes())):
 assert json.loads(files[f'case-{n:05}-input.json'])==row['input'];actual=json.loads(files[f'case-{n:05}-actual.json'])
 if 'native_panic'in row['native']:assert not actual['accepted'];p+=1
 elif row['native']['accepted']and actual['accepted']:assert actual==row['native'];a+=1
 elif row['native']['accepted']:assert row['input']['outside_value_typing'];o+=1
 else:assert not actual['accepted'];r+=1
assert(a,r,o,p)==(510,1143,3,3);legacy['builtin-original']={'accepted':a,'rejected':r,'outside':o,'original_native_panics':p}
assert originals['scope-original']==originals['complete-scope-original']
assert originals['signature-original']==originals['complete-signature-original']
record={'status':'passed','observed_at_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'actual_driver_prelaunch_sha256':hashlib.sha256((RUN/'driver.py').read_bytes()).hexdigest(),'archives':rows,'actual_original_files_compared_literally':count,'actual_original_bytes_compared_literally':size,'all_original_modes_compared':True,'new_original_cases':406,'independent_observations':counts,'legacy_correspondence':legacy,'flat_cross_reader_links':512,'thread_sessions':4,'historical_body_gaps_now_exact':2,'historical_constant_gap_now_exact':1,'scope_signature_complete_input_reruns_equal_initial_outputs':True,'applications_executed':False,'source_state_or_program_proof':False,'not_recreated_or_recompiled':True}
(RUN/'result.json').write_text(json.dumps(record,indent=2)+'\n');print(json.dumps({k:record[k]for k in ['status','actual_original_files_compared_literally','actual_original_bytes_compared_literally','independent_observations']}))
