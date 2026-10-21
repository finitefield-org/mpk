from pathlib import Path
import datetime,hashlib,json,time
OUT=Path(__file__).resolve().parent;RUN=OUT/'sdk-check';RUN.mkdir(exist_ok=False);(RUN/'driver.py').write_bytes(Path(__file__).read_bytes())
source=Path('develop/migrations/go-practical/t02-w01/verification/array-equality/native-go/sdk-inputs.json');rows=json.loads(source.read_bytes());record={'status':'running','actual_driver_prelaunch_sha256':hashlib.sha256((RUN/'driver.py').read_bytes()).hexdigest(),'original_sdk_manifest':str(source),'original_sdk_manifest_sha256':hashlib.sha256(source.read_bytes()).hexdigest(),'started_at_utc':datetime.datetime.now(datetime.timezone.utc).isoformat()};result=RUN/'result.json';result.write_text(json.dumps(record,indent=2)+'\n');start=time.monotonic();total=0
for row in rows:
 raw=Path(row['path']).read_bytes();assert len(raw)==row['bytes']and hashlib.sha256(raw).hexdigest()==row['sha256'];total+=len(raw)
record.update(status='passed',seconds=time.monotonic()-start,original_files_unchanged=len(rows),original_bytes_unchanged=total,compiler_stdlib_and_parser_testdata_unchanged=True,scope='Current exact byte correspondence to the complete parent pinnedSDK inventory; no earlier process or image reconstructed.');result.write_text(json.dumps(record,indent=2)+'\n');print(json.dumps({'status':'passed','files':len(rows),'bytes':total}))
