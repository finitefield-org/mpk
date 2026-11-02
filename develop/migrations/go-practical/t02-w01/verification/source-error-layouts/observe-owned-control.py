"""A coherent mathematical alternative must still fail the frozen source recipe."""
from pathlib import Path
import importlib.util
import json
import sys

root=Path(__file__).resolve().parent
spec=importlib.util.spec_from_file_location('source_zero_owned_control',root/'structural-oracle.py')
m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m)
inputs,output=map(Path,sys.argv[1:])
p=inputs/'coherent-wrong-bool-zero.mpcert'
s=json.loads(p.with_suffix('.sources.json').read_bytes())
r=json.loads(p.with_suffix('.rust.json').read_bytes());assert r['verdict']=='accepted' and r['axiom_count']==0
boolean=next(t['id'] for t in s['types'] if t['kind']=='builtin' and t['name']=='bool')
try:
    m.inspect(p,p.with_suffix('.sources.json'),p.with_suffix('.rust.json'))
except AssertionError as error:
    assert error.args == ((boolean,'Zero','value'),),error.args
    result={'status':'passed','mathematical_stream_accepted_in_Rust':True,'frozen_source_zero_structural_mismatch_detected':True,'observed_mismatch':error.args,'scope':'Expected source-recipe mismatch for a coherent zero=true mathematical alternative; not a kernel type rejection.'}
else:
    raise AssertionError('coherent wrong canonical source zero was missed')
output.write_text(json.dumps(result,indent=2)+'\n');print(json.dumps(result))
