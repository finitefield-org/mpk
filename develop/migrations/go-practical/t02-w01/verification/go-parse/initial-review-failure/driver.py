from pathlib import Path
import datetime,hashlib,json,subprocess,sys,time
name=sys.argv[1];root=Path('/private/tmp')/('mpk-go-parse-review-'+name);root.mkdir(exist_ok=False);source=Path('develop/migrations/go-practical/t02-w01/verification/go-parse/review-inputs.py');(root/'driver.py').write_bytes(Path(__file__).read_bytes());(root/'review-inputs.py').write_bytes(source.read_bytes())
record={'status':'running','command':['python3',str(source)],'actual_driver_sha256':hashlib.sha256((root/'driver.py').read_bytes()).hexdigest(),'actual_review_source_sha256':hashlib.sha256(source.read_bytes()).hexdigest(),'started_at_utc':datetime.datetime.now(datetime.timezone.utc).isoformat()};result=root/'result.json';result.write_text(json.dumps(record,indent=2)+'\n');start=time.monotonic()
with(root/'stdout.log').open('wb')as stdout,(root/'stderr.log').open('wb')as stderr:code=subprocess.run(record['command'],stdout=stdout,stderr=stderr,check=False).returncode
record.update(status='passed'if code==0 else'failed',exit_code=code,seconds=time.monotonic()-start);result.write_text(json.dumps(record,indent=2)+'\n');print(json.dumps(record));raise SystemExit(code)
