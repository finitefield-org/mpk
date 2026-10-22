"""A coherent wrong ordinal or another accepted call site is not exact binding."""
from pathlib import Path
import importlib.util
import json
import sys
root=Path(__file__).resolve().parent
spec=importlib.util.spec_from_file_location('source_selector_owned_oracle',root/'structural-oracle.py')
o=importlib.util.module_from_spec(spec);spec.loader.exec_module(o)
controls,entries,output=map(Path,sys.argv[1:]);p=controls/'wrong-ordinal-is-only-math.mpcert'
d=json.loads(p.with_suffix('.sources.json').read_bytes());report=json.loads(p.with_suffix('.rust.json').read_bytes())
assert report['verdict']=='accepted' and report['axiom_count']==0
first=d['targets'][0];name='Target.C'+first['type_id']+'.S'+first['source']+'.N'+str(first['declaration_node'])
try:o.inspect(p)
except AssertionError as e:assert e.args==((name,'selector body'),),e.args
else:raise AssertionError('wrong selected method ordinal was missed')
id=d['boxed_source_type_id'];expected=entries/f'multiple-roots-call0-{id}.mpcert';actual=entries/f'multiple-roots-call1-{id}.mpcert'
for p in [actual,expected]:assert json.loads(p.with_suffix('.rust.json').read_bytes())['verdict']=='accepted'
work=output.parent/'wrong-site-copy';work.mkdir(exist_ok=False)
copy=work/'substituted.mpcert';copy.write_bytes(actual.read_bytes());copy.with_suffix('.rust.json').write_bytes(actual.with_suffix('.rust.json').read_bytes());copy.with_suffix('.sources.json').write_bytes(expected.with_suffix('.sources.json').read_bytes())
a=json.loads(actual.with_suffix('.sources.json').read_bytes());e=json.loads(expected.with_suffix('.sources.json').read_bytes());assert a['case_tags']==e['case_tags'] and a['targets']==e['targets'] and a['call']!=e['call'] and a['module']!=e['module']
try:o.inspect(copy)
except AssertionError as e:assert e.args==()
else:raise AssertionError('another accepted original call site was substituted')
value={'status':'passed','coherent_wrong_ordinal_is_valid_math_but_fails_exact_target_definition':True,'different_accepted_original_call_site_owner_mismatch_detected':True,'same_complete_U_and_method_targets_preserved_for_wrong_site_control':True,'actual_other_site':str(actual),'expected_owner_site':str(expected),'wrong_site_copy_is_literal_original_accepted_math_bytes':copy.read_bytes()==actual.read_bytes(),'scope':'Private original source/call/target binding only; mathematical target equations do not imply source method execution, nil safety or canonical compiler/context/program authority.'}
output.write_text(json.dumps(value,indent=2)+'\n');print(json.dumps(value))
