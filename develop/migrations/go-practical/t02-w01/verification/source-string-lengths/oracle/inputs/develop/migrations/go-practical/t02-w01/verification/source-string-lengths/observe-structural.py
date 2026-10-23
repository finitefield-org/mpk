from pathlib import Path
import importlib.util,json,sys
root=Path(__file__).resolve().parent
spec=importlib.util.spec_from_file_location('length_oracle',root/'structural-oracle.py');m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m)
inputs,output=map(Path,sys.argv[1:]);files=sorted(p for p in inputs.glob('*.mpcert') if not p.name.startswith('control-'));assert len(files)==12
rows=[m.inspect(p) for p in files]
units={}
for p in files:
 meta=json.loads(p.with_suffix('.sources.json').read_bytes());units.setdefault(meta['unit'],[]).append(meta)
assert len(units)==3
for name,group in units.items():
 keys={k['id']:k for k in group[0]['types']};under=dict(group[0]['underlying'])
 def shape(i):
  while i in under:i=under[i]
  return keys[i]
 strings={i for i in keys if shape(i).get('name')=='string'}
 assert len(group)==len(strings)==4 and {r['source_string_type_id'] for r in group}==strings,name
 assert all(len(r['cases'])=={'source-length-empty':0,'source-length-small':4,'source-length-wide':70}[name] for r in group)
control=inputs/'control-extra.mpcert';assert control.exists();failure=None
try:m.inspect(control)
except AssertionError as error:
 failure=str(error);assert failure=='unexpected source-length export',failure
else:raise AssertionError('extra source export accepted by observer')
result={'status':'passed','streams':rows,'complete_source_string_matrices':{name:len(group) for name,group in units.items()},'accepted_extra_export_rejected':failure,'exact_source_recipe_members':sum(r['source_recipe_members'] for r in rows),'new_source_members':sum(r['new_source_members'] for r in rows),'generic_public_statements':sum(r['generic_public_statements'] for r in rows),'closed_polynomial_checkpoints':sum(r['closed_polynomial_checkpoints'] for r in rows),'full64_bit_observations':sum(r['full64_bit_observations'] for r in rows),'source_uint8_type_required':False,'source_expression_or_context_authority':False}
output.write_text(json.dumps(result,indent=2)+'\n');print(json.dumps({'status':'passed','streams':len(rows),'full64_bit_observations':result['full64_bit_observations']}))
