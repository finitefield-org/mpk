"""Feed exact Rust-observed positive/negative Boolean certificates to current Go."""
from pathlib import Path
import datetime,hashlib,json,subprocess,sys,time
root=Path(__file__).resolve().parent.relative_to(Path.cwd().resolve());attempt=sys.argv[1]
assert attempt in ["final","reviewed"]
directory=root/attempt/"paired";directory.mkdir(parents=True,exist_ok=False)
def seal(p):
 p=Path(p);b=p.read_bytes();return {"path":str(p),"sha256":hashlib.sha256(b).hexdigest(),"bytes":len(b)}
rust_path=root/"final/rust-tests/result.json";rust=json.loads(rust_path.read_text());assert rust["status"]=="passed" and rust["tests_passed"]==3
for row in rust["sources"]:assert seal(row["path"])==row
build_path=root/"initial/go-build/result.json";build=json.loads(build_path.read_text());assert build["status"]=="passed"
binary=build["checker_binary"];assert seal(binary["path"])==binary
inventory=json.loads((root/"checker-inputs.json").read_text())
for row in inventory["inputs"]:assert seal(row["path"])==row
reports=[]
for line in Path(rust["stdout"]["path"]).read_text().splitlines():
 start=line.find("{")
 if start<0:continue
 try:v=json.loads(line[start:])
 except ValueError:continue
 if isinstance(v,dict) and "verdict" in v:reports.append(v)
assert len(reports)==2
rows=[]
for name,expected in [("boolean-foundation",0),("incorrect-boolean-claim",1)]:
 source=root/"final/rust-tests/certificates"/(name+".mpcert");cert=seal(source);assert cert["bytes"]<=2097152
 command=[binary["path"],"verify",str(source)];started=time.monotonic()
 run=subprocess.run(command,stdout=subprocess.PIPE,stderr=subprocess.PIPE,timeout=60,check=False)
 stdout=directory/(name+".stdout.log");stderr=directory/(name+".stderr.log");stdout.write_bytes(run.stdout);stderr.write_bytes(run.stderr)
 go=json.loads(run.stdout);accepted=expected==0
 assert run.returncode==expected and go["verdict"]==("accepted" if accepted else "rejected") and not run.stderr
 corresponding=[v for v in reports if v["verdict"]==go["verdict"]];assert len(corresponding)==1
 if accepted:
  assert go.get("axiom_count",0)==0 and corresponding[0]["axiom_count"]==0
  assert go["hashes"]==corresponding[0]["hashes"]
 else:assert go["error_kind"]=="core_check"
 rows.append({"certificate":cert,"command":command,"exit_code":run.returncode,"wall_seconds":time.monotonic()-started,"stdout":seal(stdout),"stderr":seal(stderr),"rust_report":corresponding[0],"go_report":go,"same_ordinary_certificate_in_both_checkers":True})
record={"schema":"mpk.go_practical.paired_boolean_component.v1","status":"passed","observed_at_utc":datetime.datetime.now(datetime.timezone.utc).isoformat(),"rust_tests":seal(rust_path),"go_checker_build":seal(build_path),"go_checker_binary":binary,"cases":rows,"scope":"Fresh actual zero-axiom ordinary Boolean definitions/laws and incorrect ordinary-claim rejection. Neither complete Go foundation/registry activation nor source/application proof or native Linux acceptance.","full_T02_gate":"deferred to T02-W04: check-fast.sh --skip-csharp","CSharp_tests_executed":False}
(directory/"result.json").write_text(json.dumps(record,indent=2)+"\n")
print(json.dumps({"status":"passed","same_bytes_paired_cases":len(rows),"accepted_zero_axiom":1,"incorrect_ordinary_claim_rejected":1}))
