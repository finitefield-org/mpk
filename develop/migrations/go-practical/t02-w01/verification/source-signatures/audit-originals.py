"""Compare every preserved execution original literally; never reconstruct output."""
from pathlib import Path
import base64,datetime,hashlib,json,runpy
OUT=Path(__file__).resolve().parent;RUN=OUT/'original-byte-audit';RUN.mkdir(exist_ok=False);(RUN/'driver.py').write_bytes(Path(__file__).read_bytes())
read=runpy.run_path(str(OUT/'read-originals.py'))['read'];originals={};rows=[];count=size=0
for name in ['native-span-failure','native-original','native-binders','rust-partial','types-original','rust-original']:
 manifest,files=read(name);originals[name]=files
 for row in manifest['original_inputs']:
  p=Path(row['path']);raw=p.read_bytes();assert raw==files[row['member']]and p.stat().st_mode&0o777==row['mode'];count+=1;size+=len(raw)
 rows.append({'name':name,'files':len(files),'bytes':sum(map(len,files.values()))})
assert count==33373
old=originals['native-original'];new=originals['native-binders'];rust=originals['rust-original'];partial=originals['rust-partial']
assert old['signature-components.json']==new['signature-components.json']
initial=json.loads((OUT/'corpus.json').read_bytes());corpus=json.loads((OUT/'corpus-binders.json').read_bytes());assert json.loads(partial['original-before.json'])==initial and json.loads(rust['original-before.json'])==corpus
observed=json.loads(new['signature-observations.json']);assert observed==corpus['observations'];components=json.loads(new['signature-components.json']);assert json.loads(base64.b64decode(components['canonical_components'],validate=True))==corpus['components']
for source in corpus['sources']:
 obj=source['object'];raw=bytes(source['whole_original_bytes']);matches=[row for row in components['sources']if row['object']==obj];assert len(matches)==1 and base64.b64decode(matches[0]['raw'],validate=True)==raw
prior=json.loads(old['signature-observations.json']);assert prior['functions']==observed['functions']
origin=lambda row:{key:row[key]for key in ['source_object_id','start_byte','end_byte','name']}
assert list(map(origin,prior['variables']))==list(map(origin,observed['variables']))
assert len(observed['functions'])==31 and len(observed['variables'])==259
ok=outside=declared=inferred=unsupported=type_only=0
for n,row in enumerate(observed['functions']):
 actual=json.loads(rust[f'function-{n:03}-actual.json'])
 if row['expected_reader_error']:
  assert actual=={'error':row['expected_reader_error']};outside+=1
 else:
  assert actual['slots']==row['slots']and actual['whole_body_or_program_authority']is False;ok+=1
for n,row in enumerate(observed['variables']):
 actual=json.loads(rust[f'variable-{n:03}-actual.json'])
 if not row['lexical_binding']:
  assert actual=={'type_only_function_parameter':True,'lexical_value_binding':False};type_only+=1
 elif row['expected_reader_error']:
  assert actual=={'error':row['expected_reader_error']}
  if row['expected_reader_error']=='InferenceRequired':inferred+=1
  else:unsupported+=1
 else:assert actual['type_id']==row['type_id']and actual['initializer_or_body_or_program_authority']is False;declared+=1
assert(ok,outside,declared,inferred,unsupported,type_only)==(27,4,250,5,3,1)
assert json.loads(rust['summary.json'])=={'accepted_function_signatures':27,'declared_bindings':250,'explicit_valid_Go_outside_signature_domain':4,'explicit_valid_Go_outside_value_type_domain':3,'inferred_bindings_require_expression_typing':5,'type_only_parameter_names':1}
assert'variable-003-actual.json'not in partial and'variable-002-actual.json'in partial
record={'status':'passed','observed_at_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'actual_driver_prelaunch_sha256':hashlib.sha256((RUN/'driver.py').read_bytes()).hexdigest(),'archives':rows,'actual_original_files_compared_literally':count,'actual_original_bytes_compared_literally':size,'all_original_modes_compared':True,'both_successful_native_component_streams_literal_equal':True,'all31_original_function_observations_equal_across_successful_runs':True,'all259_variable_physical_origins_equal_across_successful_runs':True,'final_original_corpus_and_all_actual_results_exact':True,'partial_namespace_missing_outputs_not_reconstructed':True,'not_recreated_or_recompiled':True}
(RUN/'result.json').write_text(json.dumps(record,indent=2)+'\n');print(json.dumps(record))
