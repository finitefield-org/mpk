"""A coherent constant-false case test is math, not the frozen operation."""
from pathlib import Path
import importlib.util
import json
import sys
root=Path(__file__).resolve().parent
spec=importlib.util.spec_from_file_location('source_error_operation_owned_shapes',root/'structural-oracle.py')
m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m)
inputs,output=map(Path,sys.argv[1:]);path=inputs/'constant-false-case-test-is-only-math.mpcert'
report=json.loads(path.with_suffix('.rust.json').read_bytes());assert report['verdict']=='accepted' and report['axiom_count']==0
try:
    m.inspect(path)
except AssertionError as error:
    assert error.args==(('HasCase','body'),),error.args
    result={'status':'passed','constant_false_case_test_is_valid_math':True,'exact_frozen_case_comparison_body_mismatch_detected':True,'mismatch':error.args,'scope':'Mathematical nil/copy laws do not supply exact source operation ownership; no semantic-negative core or source runtime claim.'}
else:
    raise AssertionError('constant-false case comparison was missed')
output.write_text(json.dumps(result,indent=2)+'\n');print(json.dumps(result))
