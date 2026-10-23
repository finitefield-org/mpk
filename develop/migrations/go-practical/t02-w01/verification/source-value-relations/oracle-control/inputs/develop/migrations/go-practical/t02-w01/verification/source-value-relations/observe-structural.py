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
output.write_text(json.dumps({'status':'closed_source_value_relation_structure_passed','streams':rows,'case_constructors':sum(len(r['cases']) for r in rows),'scope':'Independently reconstruct every source-specific complete View/Related body and every law statement from all original source types, including every word bit, string byte, array padding, slice owner/path/offset/length/capacity and inactive error field. Observe identical original zeros and a changed value at every encoded scalar leaf in each unit nil row. Preserve prior source operations and both-checker source-free evidence. No Go operator, validity or program/context authority.'},indent=2)+'\n')
print(json.dumps({'status':'passed','streams':len(rows),'case_constructors':sum(len(r['cases']) for r in rows)}))
