"""Provision the unchanged frozen SDK using SHA-verified retained archive bytes."""
from pathlib import Path
import sys,subprocess,hashlib,json,datetime,time,traceback,base64,shutil
sys.path.insert(0,'scripts')
import rust_build_inputs as rust
ROOT=Path.cwd();OUT=ROOT/'develop/migrations/go-rust-only/verification/provision-rust-case-sensitive';OUT.mkdir(exist_ok=False)
BACKUP=Path('/private/tmp/mpk-go-rust-scope/download-archive-backup')
rows=[];commands=[];inputs=[]
for p in [Path('scripts/rust_build_inputs.py'),Path('develop/specs/vectors/rust-build-inputs-v0.json'),Path('release/build-inputs/rust/build-inputs.json')]:
 if p.is_file():
  raw=p.read_bytes();inputs.append({'path':str(p),'sha256':hashlib.sha256(raw).hexdigest(),'bytes':len(raw),'base64':base64.b64encode(raw).decode()})
(OUT/'inputs.bytes.json').write_text(json.dumps({'schema':'mpk.original_bytes_set.v1','files':inputs},indent=2)+'\n')
actual_download=rust.download
actual_run=subprocess.run
def sha(p):
 h=hashlib.sha256()
 with p.open('rb') as f:
  for block in iter(lambda:f.read(1024*1024),b''):h.update(block)
 return h.hexdigest()
def observed_download(url,expected,destination,maximum=2147483648):
 start=time.time();cached=BACKUP/destination.name;row={'url':url,'expected_sha256':expected,'destination':str(destination),'maximum':maximum}
 try:
  if cached.is_file():
   assert sha(cached)==expected
   shutil.copyfile(cached,destination);row['reuse']='complete original SHA-verified retained archive'
  actual_download(url,expected,destination,maximum)
  assert sha(destination)==expected
  row.update(status='passed',bytes=destination.stat().st_size,sha256=expected)
  if not cached.exists():shutil.copyfile(destination,cached)
 except BaseException as error:row.update(status='failed',error=repr(error));raise
 finally:
  row['seconds']=time.time()-start;rows.append(row);(OUT/'downloads.json').write_text(json.dumps(rows,indent=2)+'\n');print(json.dumps(row),flush=True)
def observed_run(argv,*args,**kwargs):
 start=time.time();r=actual_run(argv,*args,**kwargs);index=len(commands);row={'command':argv,'exit_code':r.returncode,'seconds':time.time()-start}
 for stream,data in [('stdout',r.stdout),('stderr',r.stderr)]:
  if data is not None:
   if isinstance(data,str):data=data.encode()
   p=OUT/f'command-{index:03d}.{stream}.log';p.write_bytes(data);row[stream]={'path':str(p.relative_to(ROOT)),'sha256':hashlib.sha256(data).hexdigest(),'bytes':len(data)}
 commands.append(row);(OUT/'commands.json').write_text(json.dumps(commands,indent=2)+'\n');return r
rust.download=observed_download;subprocess.run=observed_run
try:
 rust.update_build_inputs(provision=True)
 descriptor,cache=rust.check_build_inputs()
 result={'status':'passed','cache':str(cache),'build_inputs_sha256':descriptor['build_inputs_sha256']}
except BaseException as error:
 result={'status':'failed','error':repr(error),'code':getattr(error,'code',None)};(OUT/'exception.log').write_text(traceback.format_exc())
finally:
 result.update(downloads=rows,commands=commands,source_inputs_unchanged=all(hashlib.sha256(Path(r['path']).read_bytes()).hexdigest()==r['sha256'] for r in inputs),finished_at_utc=datetime.datetime.now(datetime.timezone.utc).isoformat(),pins_unchanged=True,native_release_acceptance=False)
 (OUT/'result.json').write_text(json.dumps(result,indent=2)+'\n');(OUT/'runner.py').write_bytes(Path(__file__).read_bytes());print(json.dumps({k:v for k,v in result.items() if k not in {'commands','downloads'}}),flush=True)
 if result['status']!='passed':raise SystemExit(1)
