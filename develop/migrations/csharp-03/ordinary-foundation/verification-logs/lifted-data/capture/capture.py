import sys,json,hashlib
from pathlib import Path
sys.path.insert(0,"/repo/scripts")
import csharp_practical_build_inputs as gate
p=Path("/capture")
original_execute=gate.active.execute_isolated
def diagnostic_execute(*args,**kwargs):
    result=original_execute(*args,**kwargs)
    if result.returncode or result.stdout or result.stderr:
        (p/"capture-process-diagnostic.json").write_text(json.dumps({"command":args[0],"exit_code":result.returncode,"stdout":result.stdout.decode(errors="replace") if isinstance(result.stdout,bytes) else result.stdout,"stderr":result.stderr.decode(errors="replace") if isinstance(result.stderr,bytes) else result.stderr},indent=2)+"\n")
    return result
gate.active.execute_isolated=diagnostic_execute
request=(p/"requests.json").read_bytes()
first=gate.test_data_phase(requests=request)
second=gate.test_data_phase(requests=request)
assert first==second,"nondeterministic capture"
(p/"responses.json").write_bytes(first)
rows=json.loads(first)
assert len(rows)==7
for row in rows: assert "facts" in row and "reject" not in row,row
(p/"receipt.json").write_text(json.dumps({"request_sha256":hashlib.sha256(request).hexdigest(),"response_sha256":hashlib.sha256(first).hexdigest(),"source_contexts":len(rows),"identical_double_capture":True},indent=2)+"\n")
print("Seven native lifted nullable sources accepted; identical offline double capture")
