from pathlib import Path
import importlib.util,json,sys
root=Path(__file__).resolve().parent
spec=importlib.util.spec_from_file_location('operation_oracle',root/'structural-oracle.py');m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m)
inputs,output=map(Path,sys.argv[1:]);files=sorted(p for p in inputs.glob('*.mpcert') if not p.name.startswith('control-'));assert len(files)==25
metadata={json.loads(p.with_suffix('.sources.json').read_bytes())['module']:json.loads(p.with_suffix('.sources.json').read_bytes()) for p in files};rows=[m.inspect(p) for p in files]
units={}
for p in files:
 d=json.loads(p.with_suffix('.sources.json').read_bytes());units.setdefault(d['unit'],[]).append(d)
assert {k:len(v) for k,v in units.items()}=={'pure':11,'wide':11,'length-only':1,'index-only':1,'literal-only':1}
for name,group in units.items():
 assert all(len(r['cases'])==(70 if name=='wide' else 0) for r in group)
 assert len({(r['caller'][0],r['node']) for r in group})==len(group)
control=inputs/'control-extra-theorem.mpcert';assert control.exists();module=m.decode(control)[1];failure=None
try:m.inspect(control,metadata[module])
except AssertionError as error:failure=str(error);assert failure=='unexpected source-operation export',failure
else:raise AssertionError('extra source export accepted')
record={'status':'passed','streams':rows,'complete_dynamic_site_counts':{k:len(v) for k,v in units.items()},'accepted_extra_export_rejected':failure,'exact_source_recipe_members':sum(r['source_recipe_members'] for r in rows),'new_operation_members':sum(r['operation_members'] for r in rows),'source_expression_execution_or_context_authority':False}
output.write_text(json.dumps(record,indent=2)+'\n');print(json.dumps({'status':'passed','streams':len(rows),'new_operation_members':record['new_operation_members'],'source_recipes':record['exact_source_recipe_members']}))
