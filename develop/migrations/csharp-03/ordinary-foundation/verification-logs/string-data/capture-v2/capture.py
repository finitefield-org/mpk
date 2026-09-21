import sys,json,hashlib
from pathlib import Path
sys.path.insert(0,"/repo/scripts")
import csharp_practical_build_inputs as gate
p=Path("/capture")
request=(p/"requests.json").read_bytes()
first=gate.test_data_phase(requests=request)
second=gate.test_data_phase(requests=request)
assert first==second,"nondeterministic capture"
(p/"responses.json").write_bytes(first)
rows=json.loads(first)
assert len(rows)==2
for row in rows: assert "facts" in row and "reject" not in row,row
(p/"receipt.json").write_text(json.dumps({"request_sha256":hashlib.sha256(request).hexdigest(),"response_sha256":hashlib.sha256(first).hexdigest(),"source_contexts":len(rows),"identical_double_capture":True},indent=2)+"\n")
print("Two native multi-argument string sources accepted; identical offline double capture")
