"""Require precise independent rejection of a mathematically valid wrong source view."""
from pathlib import Path
import importlib.util,json,sys
root=Path(__file__).resolve().parent
spec=importlib.util.spec_from_file_location('source_value_relations_control',root/'structural-oracle.py')
m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m)
path,output=map(Path,sys.argv[1:])
try:m.inspect(path)
except AssertionError as e:
    assert len(e.args)==1 and isinstance(e.args[0],tuple) and e.args[0][1:]==('View','closed body'),e.args
    output.write_text(json.dumps({'status':'coherent_wrong_source_view_rejected','certificate':str(path),'reason':e.args[0],'mathematics_acceptance_does_not_supply_source_identity':True},indent=2)+'\n')
else:raise AssertionError('wrong source identity accepted')
