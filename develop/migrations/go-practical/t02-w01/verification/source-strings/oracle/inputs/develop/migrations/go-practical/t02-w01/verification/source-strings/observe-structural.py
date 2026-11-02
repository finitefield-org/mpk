from pathlib import Path
import importlib.util,json,sys
root=Path(__file__).resolve().parent
spec=importlib.util.spec_from_file_location('source_string_oracle',root/'structural-oracle.py')
m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m)
inputs,output=map(Path,sys.argv[1:]);files=sorted(p for p in inputs.glob('*.mpcert') if not p.name.startswith('control-'));assert files
rows=[m.inspect(p) for p in files]
control=inputs/'control-coherent-length.mpcert'
assert control.exists()
try:m.inspect(control)
except AssertionError as error:
 failure=str(error);assert 'Length' in failure and 'body' in failure,failure
else:raise AssertionError('coherent source mismatch was accepted by independent observer')
output.write_text(json.dumps({'status':'passed','streams':rows,'coherent_wrong_length_mathematics_rejected_by_original_source_body':failure,'complete_bit_observations':sum(r['complete_length_and_slot_bit_observations'] for r in rows),'source_recipe_members':sum(r['exact_source_recipe_members'] for r in rows)},indent=2)+'\n')
print(json.dumps({'status':'passed','streams':len(rows),'complete_bit_observations':sum(r['complete_length_and_slot_bit_observations'] for r in rows)}))
