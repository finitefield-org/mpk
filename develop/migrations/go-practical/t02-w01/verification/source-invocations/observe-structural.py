from pathlib import Path
import importlib.util,json,sys
ROOT=Path(__file__).resolve().parent;spec=importlib.util.spec_from_file_location('invocation_oracle',ROOT/'structural-oracle.py');m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m)
inputs,output=map(Path,sys.argv[1:3]);generation_only=sys.argv[3:]==['--generation-only'];files=sorted(p for p in inputs.glob('*.mpcert') if not p.name.startswith('control-'));assert files
rows=[m.inspect(p,require_kernel=not generation_only) for p in files];metadata={json.loads(p.with_suffix('.sources.json').read_bytes())['module']:json.loads(p.with_suffix('.sources.json').read_bytes()) for p in files}
units={}
for p in files:
 d=json.loads(p.with_suffix('.sources.json').read_bytes());units.setdefault(d['unit'],[]).append(d)
assert set(units)=={'pure','wide'} and len(units['pure'])==len(units['wide'])
for name,group in units.items():
 assert max(len(r['site']['inputs']) for r in group)==65 and max(len(r['site']['results']) for r in group)==16
 assert min(len(r['site']['results']) for r in group)==0
 assert all(len(r['cases'])==(70 if name=='wide' else 0) for r in group)
control=(ROOT/'rust-entries/certificates' if generation_only else inputs)/'control-extra-theorem.mpcert';module=m.decode(control)[1]
try:m.inspect(control,metadata[module])
except AssertionError as error:assert str(error)=='unexpected source-invocation export';failure=str(error)
else:raise AssertionError('extra export accepted')
r={'status':'passed','streams':rows,'normal_streams':len(rows),'source_recipe_members':sum(r['source_recipe_members'] for r in rows),'invocation_members':sum(r['invocation_members'] for r in rows),'unit_counts':{k:len(v) for k,v in units.items()},'accepted_extra_export_rejected':failure,'original_Rust_kernel_reports_required':not generation_only,'source_execution_callee_or_context_proof':False};output.write_text(json.dumps(r,indent=2)+'\n');print(json.dumps({k:r[k] for k in ['status','normal_streams','source_recipe_members','invocation_members','unit_counts']}))
