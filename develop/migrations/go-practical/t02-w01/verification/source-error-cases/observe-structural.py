"""Compare every source-specific box in a complete terminal original epoch."""
from pathlib import Path
import importlib.util
import json
import sys

root=Path(__file__).resolve().parent
spec=importlib.util.spec_from_file_location('source_boxing_shapes',root/'structural-oracle.py')
m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m)
inputs,output=map(Path,sys.argv[1:])
files=sorted(inputs.glob('*.mpcert'));assert files
rows=[m.inspect(p) for p in files]
assert len({r['certificate'] for r in rows})==len(rows)
output.write_text(json.dumps({'status':'frozen_error_case_structure_passed','streams':rows,'case_constructors':sum(len(r['cases']) for r in rows),'scope':'Complete source case table, full constructor/payload catalog and nil/copy/comparison statements only; no source-state, validity, dispatch or canonical context authority.'},indent=2)+'\n')
print(json.dumps({'status':'passed','streams':len(rows),'case_constructors':sum(len(r['cases']) for r in rows)}))
