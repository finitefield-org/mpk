"""Literal originals audit, including actual failed partial comparison outputs."""
from pathlib import Path
import datetime,hashlib,json,runpy
OUT=Path(__file__).resolve().parent;RUN=OUT/'original-byte-audit';RUN.mkdir(exist_ok=False);(RUN/'driver.py').write_bytes(Path(__file__).read_bytes())
read=runpy.run_path(str(OUT/'read-originals.py'))['read'];originals={};count=size=0;rows=[]
for name in ['native-original','constants-original','rust-initial-partial','rust-minmax-partial','rust-original']:
 manifest,files=read(name);originals[name]=files
 for row in manifest['original_inputs']:
  p=Path(row['path']);raw=p.read_bytes();assert raw==files[row['member']]and p.stat().st_mode&0o777==row['mode'];count+=1;size+=len(raw)
 rows.append({'name':name,'files':len(files),'bytes':sum(map(len,files.values()))})
assert count==49052
cases=json.loads((OUT/'corpus.json').read_bytes());assert len(cases)==4985;native=originals['native-original'];rust=originals['rust-original'];accepted=rejected=outside=0
for n,row in enumerate(cases):
 assert json.loads(native[f'case-{n:05}/input.json'])==row['input'];assert json.loads(native[f'case-{n:05}/native.json'])==row['native'];assert json.loads(rust[f'case-{n:05}-input.json'])==row['input'];actual=json.loads(rust[f'case-{n:05}-actual.json'])
 if row['input']['outside_scalar_typing']and row['native']['accepted']:assert not actual['accepted'];outside+=1
 elif row['native']['accepted']:assert actual==row['native'];accepted+=1
 else:assert not actual['accepted'];rejected+=1
assert(accepted,rejected,outside)==(2775,2205,5)
assert json.loads(rust['summary.json'])=={'accepted':2775,'rejected':2205,'outside':5}
for name,last in [('rust-initial-partial',4681),('rust-minmax-partial',4748)]:
 files=originals[name];assert f'case-{last:05}-actual.json'in files and f'case-{last+1:05}-input.json'not in files;assert json.loads(files[f'case-{last:05}-actual.json'])['accepted']is True and cases[last]['native']['accepted']is False
assert json.loads(rust['deep-context-actual.json'])['accepted']is True
old=json.loads(Path('develop/migrations/go-practical/t02-w01/verification/source-constants/corpus.json').read_bytes());constants=originals['constants-original'];assert len(old)==5117
for n,row in enumerate(old):
 assert json.loads(constants[f'case-{n:05}-input.json'])==row['input'];actual=json.loads(constants[f'case-{n:05}-actual.json'])
 if row['input']['outside_scalar_reader']:assert row['native']['accepted']and not actual['accepted']
 else:
  assert actual['accepted']==row['native']['accepted']
  if actual['accepted']:assert actual==row['native']
record={'status':'passed','observed_at_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'actual_driver_prelaunch_sha256':hashlib.sha256((RUN/'driver.py').read_bytes()).hexdigest(),'archives':rows,'actual_original_files_compared_literally':count,'actual_original_bytes_compared_literally':size,'all_original_modes_compared':True,'actual_scalar_typing_cases':4985,'exact_positive_type_value_addressability_outcomes':2775,'agreeing_actual_rejections':2205,'explicit_valid_Go_unmodeled_forms':5,'previous_constant_cases':5117,'all_original_scalar_case_rows_and_final_outcomes_exact':True,'both_false_partial_acceptances_preserved':True,'partial_missing_outputs_not_reconstructed':True,'not_recreated_or_recompiled':True}
(RUN/'result.json').write_text(json.dumps(record,indent=2)+'\n');print(json.dumps(record))
