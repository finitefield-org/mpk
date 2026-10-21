from pathlib import Path
import datetime,hashlib,json,subprocess,time
OUT=Path(__file__).resolve().parent;RUN=OUT/'unicode-check';RUN.mkdir(exist_ok=False)
(RUN/'driver.py').write_bytes(Path(__file__).read_bytes())
def seal(p):
 raw=Path(p).read_bytes();return {'path':str(p),'bytes':len(raw),'sha256':hashlib.sha256(raw).hexdigest()}
input=json.loads((OUT/'unicode-input.json').read_bytes());sources=[OUT/'verify-unicode.py',OUT/'unicode-input.json',Path(input['generated']),Path(input['source']),OUT/'run-checks.py'];rows=[]
for n,p in enumerate(sources):
 row=seal(p);target=RUN/f'input-{n:02}.raw';target.write_bytes(p.read_bytes());row['complete_original']=str(target);rows.append(row)
command=['python3',str(OUT/'verify-unicode.py')];record={'status':'running','command':command,'actual_driver_prelaunch':seal(RUN/'driver.py'),'sources':rows,'started_at_utc':datetime.datetime.now(datetime.timezone.utc).isoformat()};result=RUN/'result.json';result.write_text(json.dumps(record,indent=2)+'\n');start=time.monotonic()
with(RUN/'stdout.log').open('wb')as stdout,(RUN/'stderr.log').open('wb')as stderr:code=subprocess.run(command,stdout=stdout,stderr=stderr,check=False).returncode
record.update(status='passed'if code==0 else'failed',exit_code=code,seconds=time.monotonic()-start,stdout=seal(RUN/'stdout.log'),stderr=seal(RUN/'stderr.log'),whole_inputs_unchanged=all(seal(r['path'])=={k:r[k]for k in ['path','bytes','sha256']}for r in rows));result.write_text(json.dumps(record,indent=2)+'\n');print(json.dumps({k:record[k]for k in ['status','exit_code','seconds','whole_inputs_unchanged']}));raise SystemExit(code)
