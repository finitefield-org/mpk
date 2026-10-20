"""Preserve only exact actual terminal cargo libtest stdout EOF observations."""
from pathlib import Path
import datetime
import hashlib
import json


def seal(path):
    path=Path(path);raw=path.read_bytes()
    return {"path":str(path),"sha256":hashlib.sha256(raw).hexdigest(),"bytes":len(raw)}


out=Path(__file__).resolve().parent.relative_to(Path.cwd().resolve())
roots=[out.parent,out.parent.parent/"archive-ownership"]
observations={};receipts=[]
for root in roots:
    for path in sorted(root.rglob("result.json")):
        d=json.loads(path.read_text())
        command=d.get("command",[])
        if len(command)<2 or command[:2]!=["cargo","test"]:continue
        assert d["status"] in {"passed","failed"},str(path)
        stdout=d["stdout"];assert seal(stdout["path"])==stdout
        assert seal(d["stderr"]["path"])==d["stderr"]
        receipts.append(seal(path))
        raw=Path(stdout["path"]).read_bytes()
        if raw.endswith(b"\n\n"):
            assert b"test result:" in raw or b"tests, 0 benchmarks" in raw
            observations[stdout["path"]]={**stdout,"kind":"exact_original_libtest_stdout_EOF"}
assert observations and receipts
record={"observed_at_utc":datetime.datetime.now(datetime.timezone.utc).isoformat(),"runner":seal(__file__),"actual_terminal_execution_receipts":receipts,"observations":observations,"scope":"Only these complete actual terminal cargo libtest stdout files, each with exact original SHA256 and length and actual terminal command/output binding. Final Git whitespace review accepts only their actual EOF-only messages. No source, JSON, directory, generic suffix, all-log or blanket whitespace exclusion is added."}
path=out/"raw-formatter-observations.json";assert not path.exists();path.write_text(json.dumps(record,indent=2)+"\n")
print(json.dumps({"actual_terminal_cargo_receipts":len(receipts),"exact_original_stdout_EOF_observations":len(observations)}))
