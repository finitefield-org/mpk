"""Literal original byte/mode audit; never recreate missing failed outputs."""
from pathlib import Path
import datetime,hashlib,json,runpy
OUT=Path(__file__).resolve().parent;RUN=OUT/'final-alias-original-byte-audit';RUN.mkdir(exist_ok=False);(RUN/'driver.py').write_bytes(Path(__file__).read_bytes())
read=runpy.run_path(str(OUT/'read-originals.py'))['read'];originals={};count=size=0;rows=[]
names=['native-original','rust-element-partial','rust-context-partial','native-context-original','rust-context-original','native-methods-partial','scalar-original','native-methods-original','rust-original','native-index-original','rust-index-original','scalar-index-original','native-alias-original','rust-alias-original','scalar-alias-original']
for name in names:
 manifest,files=read(name);originals[name]=files
 for row in manifest['original_inputs']:
  p=Path(row['path']);raw=p.read_bytes();assert raw==files[row['member']]and p.stat().st_mode&0o777==row['mode'];count+=1;size+=len(raw)
 rows.append({'name':name,'files':len(files),'bytes':sum(map(len,files.values()))})
assert count==50550
for archive,corpus in [('native-original','corpus.json'),('native-context-original','corpus-context.json'),('native-methods-original','corpus-methods.json'),('native-index-original','corpus-indices.json'),('native-alias-original','corpus-aliases.json')]:
 cases=json.loads((OUT/corpus).read_bytes());assert len(cases)==(950 if corpus=='corpus-aliases.json' else 944 if corpus=='corpus-indices.json' else 848)
 for n,row in enumerate(cases):
  assert json.loads(originals[archive][f'case-{n:05}/input.json'])==row['input'];assert json.loads(originals[archive][f'case-{n:05}/native.json'])==row['native']
cases=json.loads((OUT/'corpus-aliases.json').read_bytes());native=originals['native-alias-original'];rust=originals['rust-alias-original'];assert cases[:944]==json.loads((OUT/'corpus-indices.json').read_bytes());accepted=rejected=outside=0
for n,row in enumerate(cases):
 assert json.loads(rust[f'case-{n:05}-input.json'])==row['input'];actual=json.loads(rust[f'case-{n:05}-actual.json'])
 if row['input']['outside_value_typing']and row['native']['accepted']:assert not actual['accepted'];outside+=1
 elif row['native']['accepted']:assert actual==row['native'];accepted+=1
 else:assert not actual['accepted'];rejected+=1
assert(accepted,rejected,outside)==(474,463,13)
assert json.loads(rust['summary.json'])=={'accepted':474,'rejected':463,'outside':13,'failures':[]}
assert json.loads(rust['aggregate-owned-actual.json'])['accepted']is True
partial=originals['rust-element-partial'];assert json.loads(partial['case-00573-actual.json'])['accepted']is True and json.loads((OUT/'corpus.json').read_bytes())[573]['native']['accepted']is False and 'case-00574-input.json'not in partial
partial=originals['rust-context-partial'];assert len(json.loads(partial['summary.json'])['failures'])==6
partial=originals['native-methods-partial'];assert 'case-00845/input.json'in partial and 'case-00845/native.json'not in partial and 'case-00846/input.json'not in partial
old=json.loads((OUT.parent/'scalar-typing/corpus.json').read_bytes());scalar=originals['scalar-index-original','native-alias-original','rust-alias-original','scalar-alias-original'];assert len(old)==4985
sa=sr=so=0
for n,row in enumerate(old):
 assert json.loads(scalar[f'case-{n:05}-input.json'])==row['input'];actual=json.loads(scalar[f'case-{n:05}-actual.json'])
 if row['native']['accepted']and actual['accepted']:assert actual==row['native'];sa+=1
 elif row['native']['accepted']:assert row['input']['outside_scalar_typing']and'1.0'in row['input']['label'];so+=1
 else:assert not actual['accepted'];sr+=1
assert(sa,sr,so)==(2776,2205,4)
assert json.loads(scalar['summary.json'])=={'accepted':2776,'rejected':2205,'outside':4}
assert json.loads(scalar['deep-context-actual.json'])['accepted']is True
record={'status':'passed','observed_at_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'actual_driver_prelaunch_sha256':hashlib.sha256((RUN/'driver.py').read_bytes()).hexdigest(),'archives':rows,'actual_original_files_compared_literally':count,'actual_original_bytes_compared_literally':size,'all_original_modes_compared':True,'actual_value_typing_cases':950,'exact_positive_type_value_addressability_and_nil_context_outcomes':474,'agreeing_actual_rejections':463,'explicit_valid_Go_unmodeled_forms':13,'previous_scalar_cases':4985,'previous_scalar_exact_positives':2776,'previous_scalar_agreeing_rejections':2205,'previous_scalar_remaining_unmodeled':4,'one_previously_unmodeled_aggregate_len_now_exact':True,'index_context_and_untyped_rune_materialization_regressions_checked':True,'original_alias_receiver_denotation_and_pointer_method_barriers_checked':True,'first944_actual_native_cases_unchanged':True,'original_false_partial_acceptance_preserved':True,'original_six_context_differences_preserved':True,'failed_native_observer_partial_preserved':True,'partial_missing_outputs_not_reconstructed':True,'not_recreated_or_recompiled':True}
(RUN/'result.json').write_text(json.dumps(record,indent=2)+'\n');print(json.dumps({k:record[k]for k in ['status','actual_original_files_compared_literally','actual_original_bytes_compared_literally']}))
