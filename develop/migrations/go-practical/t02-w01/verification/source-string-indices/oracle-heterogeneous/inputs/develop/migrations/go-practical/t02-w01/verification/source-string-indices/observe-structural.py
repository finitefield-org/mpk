from pathlib import Path
import importlib.util,json,sys
root=Path(__file__).resolve().parent
spec=importlib.util.spec_from_file_location('index_oracle',root/'structural-oracle.py');m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m)
inputs,output=map(Path,sys.argv[1:]);files=sorted(p for p in inputs.glob('*.mpcert') if not p.name.startswith('control-'));assert files
rows=[m.inspect(p) for p in files]
units={}
for p in files:
 meta=json.loads(p.with_suffix('.sources.json').read_bytes());units.setdefault(meta['unit'],[]).append(meta)
for name,group in units.items():
 keys={k['id']:k for k in group[0]['types']};under=dict(group[0]['underlying'])
 def shape(i):
  while i in under:i=under[i]
  return keys[i]
 strings={i for i in keys if shape(i).get('name')=='string'};indices={i for i in keys if shape(i).get('width') in [8,16,32,64] and type(shape(i).get('signed')) is bool}
 actual={(r['source_string_type_id'],r['source_index_type_id']) for r in group}
 assert len(actual)==len(group) and actual=={(s,i) for s in strings for i in indices},name
control=inputs/'control-coherent-read.mpcert';failure=None
if control.exists():
 try:m.inspect(control)
 except AssertionError as error:
  failure=str(error);assert 'Read' in failure and 'body' in failure,failure
 else:raise AssertionError('coherent wrong source read accepted by observer')
output.write_text(json.dumps({'status':'passed','streams':rows,'complete_source_string_integer_matrices':{name:len(rows) for name,rows in units.items()},'coherent_wrong_read_rejected':failure,'exact_source_recipe_members':sum(r['source_recipe_members'] for r in rows),'new_exact_members':sum(r['new_exact_members'] for r in rows),'full64_bit_observations':sum(r['full64_bit_observations'] for r in rows),'complete_byte_bit_observations':sum(r['complete_byte_bit_observations'] for r in rows)},indent=2)+'\n');print(json.dumps({'status':'passed','streams':len(rows),'full64_bit_observations':sum(r['full64_bit_observations'] for r in rows)}))
