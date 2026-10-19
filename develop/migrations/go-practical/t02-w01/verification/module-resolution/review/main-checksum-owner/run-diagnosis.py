from pathlib import Path
import subprocess,os,json,hashlib,time,datetime
root=Path(__file__).resolve().parent.relative_to(Path.cwd().resolve());out=root/'before-fix';out.mkdir(exist_ok=False)
def seal(p):
 b=p.read_bytes();return {'path':str(p),'sha256':hashlib.sha256(b).hexdigest(),'bytes':len(b)}
env_record=json.loads((root.parents[1]/'selected/result.json').read_text())['environment'];env=os.environ.copy();env.update(env_record)
command=[str(Path(env_record['GOROOT'])/'bin/go'),'test','-mod=vendor','-run','^TestPracticalResolutionRejectsChecksumsOwnedByAnotherModuleInput$','-count=1','-json','.']
sources=[]
for name in ['practical_resolution.go','practical_resolution_test.go','practical_resolution_capture_test.go']:
 p=Path('go-tools/go2vir')/name;copy=out/(name+'.txt');copy.write_bytes(p.read_bytes());sources.append({'source':seal(p),'retained':seal(copy)})
record={'status':'running','command':command,'cwd':'go-tools/go2vir','sources':sources,'environment':env_record,'selection_reason':'Reproduce whether an independently captured checksum input can be substituted for the original main module go.sum. The actual discovery caller is unchanged; no public profile or source proof is claimed.','started_at_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'CSharp_tests_executed':False,'full_T02_gate':'deferred to W04'}
started=time.monotonic()
with (out/'stdout.log').open('wb') as stdout,(out/'stderr.log').open('wb') as stderr:
 r=subprocess.run(command,cwd='go-tools/go2vir',env=env,stdout=stdout,stderr=stderr)
record.update(status='failed' if r.returncode else 'passed',exit_code=r.returncode,wall_seconds=time.monotonic()-started,stdout=seal(out/'stdout.log'),stderr=seal(out/'stderr.log'),finished_at_utc=datetime.datetime.now(datetime.timezone.utc).isoformat())
(out/'result.json').write_text(json.dumps(record,indent=2)+'\n');print(json.dumps({k:record[k] for k in ['status','exit_code','wall_seconds']}));raise SystemExit(r.returncode)
