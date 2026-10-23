"""Compare independent original Go type observations after the full Rust run."""
from pathlib import Path
import datetime,hashlib,json,runpy
ROOT=Path(__file__).resolve().parent
OUT=ROOT/'native-comparison';OUT.mkdir(exist_ok=False)
def seal(p):
 b=Path(p).read_bytes();return {'path':str(p),'bytes':len(b),'sha256':hashlib.sha256(b).hexdigest()}
run_path=ROOT/'remaining/result.json';run=json.loads(run_path.read_bytes());assert run['status']=='passed' and run['whole_source_inputs_unchanged']
relocations=json.loads((ROOT/'native-original-relocations.json').read_bytes());assert relocations['status']=='passed'
archive=ROOT/'native-output-original/manifest.json';manifest=json.loads(archive.read_bytes());assert manifest['status']=='passed'
inputs=[Path(__file__),ROOT/'read-originals.py',ROOT/'native-original-relocations.json',archive,ROOT/'corpus.json',run_path,ROOT/'remaining/inputs.json']
inputs.extend(Path(r['encoded']['path']) for r in manifest['chunks'])
metadata=sorted(p for p in (ROOT/'remaining/certificates').glob('source-string-indices-*.sources.json'));assert len(metadata)==56;inputs.extend(metadata)
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
 native_indices=json.loads(files['index-types.json']);native_strings=json.loads(files['string-types.json']);native_cases=json.loads((ROOT/'corpus.json').read_bytes())['cases'][0]['native']['global_cases']
 assert len(native_indices)==13 and len(native_strings)==4 and len(native_cases)==4
 metas=[json.loads(p.read_bytes()) for p in metadata];index_ids={r['source_index_type_id'] for r in metas};string_ids={r['source_string_type_id'] for r in metas}
 assert string_ids==set(native_strings) and set(native_indices)<index_ids and len(index_ids)==14
 for row in metas:
  assert row['cases']==native_cases
  native=native_indices.get(row['source_index_type_id'])
  if native:assert (row['width'],row['signed'])==(native['width'],native['signed'])
  assert row['source_byte_type_id'] in native_indices and native_indices[row['source_byte_type_id']]['go_type']=='uint8'
 extra=index_ids-set(native_indices);assert extra<=set(native_cases) and len(extra)==1
 first=metas[0];keys={r['id']:r for r in first['types']};under=dict(first['underlying'])
 for tid in extra:
  while tid in under:tid=under[tid]
  assert keys[tid]['name']=='uint8'
 for r in rows:
  assert seal(r['path'])=={k:r[k] for k in ['path','bytes','sha256']} and Path(r['path']).stat().st_mode&0o777==r['mode']
 result.update(status='passed',whole_inputs_unchanged=True,native_integer_operand_types=native_indices,native_string_types=sorted(native_strings),native_case_ids=native_cases,original_integer_error_case_also_eligible=sorted(extra),all_56_source_string_integer_rows_compared=True,alias_preserves_original_nominal_index_id=True,native_facts_passed_to_Rust_constructor=False,source_expression_or_context_authority=False,finished_at_utc=datetime.datetime.now(datetime.timezone.utc).isoformat());save()
 print(json.dumps({'status':'passed','native_operand_types':13,'all_eligible_integer_types':14,'complete_string_integer_rows':56}))
except BaseException:
 result['status']='failed';save();raise
