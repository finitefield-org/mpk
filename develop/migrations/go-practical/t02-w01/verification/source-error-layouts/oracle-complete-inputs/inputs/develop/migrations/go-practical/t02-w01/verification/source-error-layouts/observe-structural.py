"""Compare immutable original source recipes with the frozen structural oracle."""
from pathlib import Path
import importlib.util
import json
import sys

root=Path(__file__).resolve().parent
spec=importlib.util.spec_from_file_location('source_zero_oracle',root/'structural-oracle.py')
module=importlib.util.module_from_spec(spec)
spec.loader.exec_module(module)
inputs,output=map(Path,sys.argv[1:])
paths=sorted(inputs.glob('*.mpcert'))
assert paths
rows=[]
for path in paths:
    rows.append(module.inspect(path,path.with_suffix('.sources.json'),path.with_suffix('.rust.json')))
output.write_text(json.dumps({'status':'frozen_structural_comparison_passed','cases':rows,
    'scope':'Every complete source-key Carrier/Zero/ZeroRecipe structure, including empty and otherwise-unobserved dependencies; no source-program/capture/context or arbitrary-input validity claim.'},indent=2)+'\n')
print(json.dumps({'status':'passed','streams':len(rows),'source_type_recipes':sum(len(r['types']) for r in rows)}))
