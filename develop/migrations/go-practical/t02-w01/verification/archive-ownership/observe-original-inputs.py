"""Reconstruct exact logged original input bytes; independently check Go h1/ownership IDs."""
from pathlib import Path
import base64
import datetime
import hashlib
import io
import json
import re
import zipfile


def seal(path):
    path=Path(path);raw=path.read_bytes()
    return {"path":str(path),"sha256":hashlib.sha256(raw).hexdigest(),"bytes":len(raw)}


root=Path(__file__).resolve().parent.relative_to(Path.cwd().resolve())
pattern=re.compile(r"complete original ZIP=([0-9a-f]+) main go.mod=([0-9a-f]+) main go.sum=([0-9a-f]+) archive object=\{Schema:(\S+) ID:(\S+) Capability:(\S+) Path:(\S+) Kind:(\S+) RawSHA256:(\S+) Bytes:([0-9]+)\}")
changed_pattern=re.compile(r"complete changed member=([0-9a-f]+) object=")
observations=[]
for suffix in ["reviewed/regression/stdout.log","final/selected/stdout.log"]:
    path=root/suffix;seen=[];changed=[]
    events=[json.loads(line) for line in path.read_text().splitlines()]
    text="".join(event.get("Output", "") for event in events)
    for match in pattern.finditer(text):
            z,m,s=[bytes.fromhex(match.group(i)) for i in [1,2,3]]
            source={"schema":match.group(4),"capability":match.group(6),"path":match.group(7),"kind":match.group(8),"raw_sha256":match.group(9),"bytes":int(match.group(10))}
            assert source["raw_sha256"]==hashlib.sha256(z).hexdigest() and source["bytes"]==len(z)
            canonical=json.dumps(source,sort_keys=True,separators=(",",":"),ensure_ascii=True).encode()
            assert hashlib.sha256(b"MPK-GO-SOURCE-OBJECT-1.0\0"+canonical).hexdigest()==match.group(5)
            source["id"]=match.group(5)
            seen.append((z,m,s,source))
    for match in changed_pattern.finditer(text):
        changed.append(bytes.fromhex(match.group(1)))
    assert len(seen)==len(changed)==4
    for z,m,s,source in seen:
        assert (z,m,s,source)==seen[0]
    assert all(c==changed[0] for c in changed)
    observations.append((path,seen[0],changed[0]))
assert observations[0][1:]==observations[1][1:]
raw,main_mod,main_sum,source=observations[0][1];wrong=observations[0][2]
with zipfile.ZipFile(io.BytesIO(raw)) as archive:
    assert len(archive.infolist())==2 and not any(i.is_dir() for i in archive.infolist())
    files={info.filename:archive.read(info) for info in archive.infolist()}
assert set(files)=={"example.com/pracdep@v1.0.0/dep.go","example.com/pracdep@v1.0.0/go.mod"}
framed=b"".join(hashlib.sha256(data).hexdigest().encode()+b"  "+name.encode()+b"\n" for name,data in sorted(files.items()))
h1="h1:"+base64.b64encode(hashlib.sha256(framed).digest()).decode()
assert main_sum==("example.com/pracdep v1.0.0 "+h1+"\n").encode()
assert h1=="h1:MTsAUxsuzB4cUq0dDw78AH8ovfJ05oBPSyUb+5y8IPA="
assert wrong!=files["example.com/pracdep@v1.0.0/dep.go"]
out=root/"observed-inputs";out.mkdir(exist_ok=False)
for name,data in [("original.zip",raw),("main.go.mod",main_mod),("main.go.sum",main_sum),("dependency.go.mod",files["example.com/pracdep@v1.0.0/go.mod"]),("dependency.go",files["example.com/pracdep@v1.0.0/dep.go"]),("changed-dependency.go",wrong)]:
    (out/name).write_bytes(data)
record={"status":"passed","observed_at_utc":datetime.datetime.now(datetime.timezone.utc).isoformat(),"runner":seal(__file__),"complete_actual_input_logs":[seal(p) for p,_,_ in observations],"full_original_first_and_cached_inputs_identical_before_and_after_fix":True,"actual_original_archive_source_object":source,"independent_original_source_id_matches":True,"independent_complete_Go_h1":h1,"all_original_members":sorted(files),"original_bytes":[seal(p) for p in sorted(out.iterdir())],"scope":"Every complete actual logged ZIP/main metadata/changed-member payload is compared across all4 original failing and4 final passing cases. Complete original ZIP encodings, Go member hashes/checksums and typed source IDs remain exact; no input normalization or source replacement occurs. This independently observes the ownership regression fixtures, not application source/typechecking/context or immutable package replay authority."}
(out/"result.json").write_text(json.dumps(record,indent=2)+"\n")
print(json.dumps({"status":"passed","original_and_final_cases":8,"original_members":2,"independent_h1":h1}))
