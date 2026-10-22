"""A coherent tag collision is mathematical acceptance but source-table failure."""
from pathlib import Path
import importlib.util
import json
import sys

root=Path(__file__).resolve().parent
spec=importlib.util.spec_from_file_location('source_boxing_owned_shapes',root/'structural-oracle.py')
m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m)
inputs,output=map(Path,sys.argv[1:]);path=inputs/'colliding-source-tags-are-only-math.mpcert'
source=json.loads(path.with_suffix('.sources.json').read_bytes());report=json.loads(path.with_suffix('.rust.json').read_bytes())
assert report['verdict']=='accepted' and report['axiom_count']==0
expected=source['case_tags'][2][0]
try:
    m.inspect(path)
except AssertionError as error:
    assert error.args==((expected,'Tag','body'),),error.args
    value={'status':'passed','coherent_tag_collision_is_valid_math':True,'exact_source_tag_body_mismatch_detected':True,'mismatch':error.args,'scope':'Exact source table ownership and frozen tag construction; this is not a kernel type rejection or source runtime proof.'}
else:
    raise AssertionError('source tag collision was missed')
output.write_text(json.dumps(value,indent=2)+'\n');print(json.dumps(value))
