"""Actual locked local Linux observer build with complete current input bytes."""
from pathlib import Path
import datetime,hashlib,json,os,sys,tarfile,time,subprocess
REPO=Path.cwd();HERE=REPO/'develop/probes/go-practical';ROOT=Path(__file__).resolve().parent
sys.dont_write_bytecode=True;sys.path.insert(0,str(HERE))
import release_native_build

def seal(p):
 p=Path(p);b=p.read_bytes();return {'path':str(p.resolve()),'bytes':len(b),'sha256':hashlib.sha256(b).hexdigest(),'mode':p.stat().st_mode&0o777}
def main():
 out=ROOT/'native-linux';out.mkdir(exist_ok=False)
 paths={Path(__file__).resolve(),REPO/'Cargo.toml',REPO/'Cargo.lock'}
 paths.update(Path(r['path']) if Path(r['path']).is_absolute() else REPO/r['path'] for r in release_native_build.sources())
 paths.update(HERE/name for name in ['release_native_build.py','policy_native_build.py','frontend_native_build.py','run_proof_pilot.py','check_scope_report.py'])
 paths.update(p for p in (REPO/'proofs/std').rglob('*') if p.is_file())
 inputs=[seal(p) for p in sorted(paths)]
 with tarfile.open(out/'inputs.tar.gz','x:gz',dereference=True) as packed:
  for p in sorted(paths):packed.add(p,arcname=str(p.relative_to(REPO)),recursive=False)
 (out/'inputs.json').write_text(json.dumps({'inputs':inputs,'complete_archive':seal(out/'inputs.tar.gz'),'selection_reason':'Compile the real existing public release observer offline/locked inside its pinned local Linux amd64 image, with every linked source/include/resource and complete private codec source package. No proof replay, source adapter, rewritten original SDK/application or native release acceptance is inferred.'},indent=2)+'\n')
 report={'status':'running','started_at_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'inputs':seal(out/'inputs.json')}
 (out/'result.json').write_text(json.dumps(report,indent=2)+'\n');start=time.monotonic()
 calls=[];original_run=subprocess.run
 def observed_run(*args,**kwargs):
  number=len(calls);command=list(map(str,args[0]));entry={'command':command};calls.append(entry)
  try:
   completed=original_run(*args,**kwargs);entry['exit_code']=completed.returncode
   stdout,stderr=completed.stdout or b'',completed.stderr or b''
  except subprocess.TimeoutExpired as error:
   stdout,stderr=error.stdout or b'',error.stderr or b'';entry.update(timeout_seconds=error.timeout,terminal='timeout')
   for name,data in [('stdout',stdout),('stderr',stderr)]:
    path=out/f'command-{number:02d}-{name}.log';path.write_bytes(data);entry[name]=seal(path)
   (out/'commands.json').write_text(json.dumps(calls,indent=2)+'\n');raise
  for name,data in [('stdout',stdout),('stderr',stderr)]:
   path=out/f'command-{number:02d}-{name}.log';path.write_bytes(data);entry[name]=seal(path)
  (out/'commands.json').write_text(json.dumps(calls,indent=2)+'\n')
  return completed
 subprocess.run=observed_run
 try:
  command,audit=release_native_build.build()
  assert audit['binary_sha256'] and audit['source_mount']=='read-only'
  report.update(status='passed',observer_command=command,audit=audit)
 except BaseException as e:
  report.update(status='failed',error=repr(e));raise
 finally:
  subprocess.run=original_run
  report.update(seconds=time.monotonic()-start,inputs_unchanged=inputs==[seal(p) for p in sorted(paths)],native_Linux_release_acceptance=False)
  (out/'result.json').write_text(json.dumps(report,indent=2)+'\n')
 print(json.dumps({'status':report['status'],'seconds':report['seconds'],'inputs_unchanged':report['inputs_unchanged'],'binary_sha256':report['audit']['binary_sha256'],'native_Linux_release_acceptance':False}))
if __name__=='__main__':main()
