"""Run outside sandbox: read every committed blob and compare reviewed SHA256/length."""
from pathlib import Path
import hashlib
import json
import subprocess


def git(*args):
    return subprocess.run(["/usr/bin/git",*args],check=True,stdout=subprocess.PIPE,stderr=subprocess.PIPE).stdout


def seal(path):
    path=Path(path);raw=path.read_bytes()
    return {"path":str(path),"sha256":hashlib.sha256(raw).hexdigest(),"bytes":len(raw)}


root=Path(__file__).resolve().parent.parent.relative_to(Path.cwd().resolve())
base="528b599aacfe439d4abd8f50e046a754c9a24435"
commit=git("rev-parse","HEAD").decode().strip();assert git("rev-parse","HEAD^").decode().strip()==base
assert git("branch","--show-current").decode().strip()=="codex/go-practical"
assert not git("status","--porcelain=v1","--untracked-files=all")
review=root/"input-review/changed-inputs-review.json";record=json.loads(review.read_text())
assert record["status"]=="passed" and not(record["findings"]or record["whitespace_errors"]or record["git_errors"]or record["missing_links"])
expected={row["path"]:row for row in record["inputs"]};assert len(expected)==len(record["inputs"])
expected[str(review)]=seal(review)
pre_path=Path("/private/tmp/mpk-go-native-host-precommit-audit.json");pre=json.loads(pre_path.read_text())
assert pre["status"]=="passed" and pre["staged_files"]==len(expected) and pre["review"]==seal(review)
rows=git("diff-tree","--no-commit-id","--raw","--abbrev=40","-r","-z","HEAD^","HEAD").split(b"\0")
assert rows.pop()==b"" and len(rows)==2*len(expected)
objects=[]
for i in range(0,len(rows),2):
    header,name=rows[i].decode().split(),rows[i+1].decode()
    assert header[4]in{"A","M"} and name in expected
    assert seal(name)==expected[name]
    objects.append((name,header[3]))
assert {name for name,_ in objects}==set(expected)
actual={}
with subprocess.Popen(["/usr/bin/git","cat-file","--batch"],stdin=subprocess.PIPE,stdout=subprocess.PIPE,stderr=subprocess.PIPE)as process:
    assert process.stdin is not None and process.stdout is not None and process.stderr is not None
    for name,oid in objects:
        if oid not in actual:
            process.stdin.write(oid.encode()+b"\n");process.stdin.flush()
            fields=process.stdout.readline().decode().strip().split();assert len(fields)==3 and fields[:2]==[oid,"blob"]
            remaining=size=int(fields[2]);digest=hashlib.sha256()
            while remaining:
                raw=process.stdout.read(min(remaining,1048576));assert raw
                digest.update(raw);remaining-=len(raw)
            assert process.stdout.read(1)==b"\n"
            actual[oid]=(digest.hexdigest(),size)
        assert actual[oid]==(expected[name]["sha256"],expected[name]["bytes"]),name
    process.stdin.close();error=process.stderr.read();assert process.wait()==0 and not error
assert not git("status","--porcelain=v1","--untracked-files=all")
frozen=json.loads((root/"frozen-design-preservation.json").read_text());assert len(frozen["inputs"])==255
for source in frozen["inputs"]:assert seal(source["path"])==source
product=json.loads((root/"verification.json").read_text());assert product["status"]=="scoped_checks_passed" and product["all_actual_validation_handles_terminal"]
archive_path=root.parent/"archive-ownership/verification.json";archive=json.loads(archive_path.read_text());assert archive["status"]=="scoped_checks_passed" and archive["all_related_product_validation_handles_terminal"]
result={"status":"passed","commit":commit,"parent":base,"branch":"codex/go-practical","reviewed_and_committed_files":len(expected),"distinct_committed_blobs_actually_read":len(actual),"all_committed_blobs_equal_reviewed_SHA256_and_length":True,"working_tree_clean":True,"precommit_audit":seal(pre_path),"review":seal(review),"native_host_checkpoint":seal(root/"verification.json"),"archive_checkpoint":seal(archive_path),"frozen_original_inputs_unchanged":255,"all_actual_validation_handles_terminal":True,"CSharp_tests_executed":False,"T02_W01_complete":False,"goal_status":"active","public_profile_activated":False,"whole_source_program_proof_claimed":False,"remaining_application_demands":169,"full_T02_gate_passed":False,"full_T02_gate":"deferred to T02-W04: ./scripts/check-fast.sh --skip-csharp"}
Path("/private/tmp/mpk-go-native-host-postcommit-audit.json").write_text(json.dumps(result,indent=2)+"\n")
print(json.dumps({k:result[k]for k in ["status","commit","reviewed_and_committed_files","distinct_committed_blobs_actually_read","all_committed_blobs_equal_reviewed_SHA256_and_length","working_tree_clean","T02_W01_complete","goal_status"]}))
