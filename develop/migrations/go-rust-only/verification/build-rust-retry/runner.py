from pathlib import Path
import sys,subprocess,hashlib,json,time,datetime,traceback,base64
sys.path.insert(0,str(Path.cwd()/'scripts'))
import successor_release_bundles as build
language=sys.argv[1];tag=sys.argv[2];root=Path.cwd();record=root/'develop/migrations/go-rust-only/verification'/f'build-{language}-{tag}';record.mkdir(parents=True,exist_ok=False)
paths=[]
for base in ([root/'go-tools/go2vir'] if language=='go' else [root/'rust-tools/rust2vir']):
 for p in sorted(base.rglob('*')):
  if p.is_file() and not any(x in p.parts for x in ['target','.git','__pycache__']):
   if p.suffix in {'.go','.rs','.toml','.lock'} or p.name in {'go.mod','go.sum','modules.txt','Cargo.lock'}:paths.append(p)
inputs=[{'path':str(p.relative_to(root)),'sha256':hashlib.sha256(p.read_bytes()).hexdigest(),'bytes':p.stat().st_size} for p in paths]
(record/'inputs.json').write_text(json.dumps(inputs,indent=2)+'\n')
(record/'source-preimages.bytes.json').write_text(json.dumps({'schema':'mpk.original_bytes_set.v1','files':[{**row,'base64':base64.b64encode((root/row['path']).read_bytes()).decode()} for row in inputs]},indent=2)+'\n')
(record/'runner.py').write_bytes(Path(__file__).read_bytes())
actual_run=subprocess.run;calls=[]
def observed_run(argv,*args,**kwargs):
 n=len(calls);start=time.time();result=actual_run(argv,*args,**kwargs)
 row={'command':argv,'exit_code':result.returncode,'seconds':time.time()-start}
 for label,data in [('stdout',result.stdout),('stderr',result.stderr)]:
  if data is not None:
   if isinstance(data,str):data=data.encode()
   path=record/f'command-{n:02d}.{label}.log';path.write_bytes(data);row[label]={'path':str(path.relative_to(root)),'sha256':hashlib.sha256(data).hexdigest(),'bytes':len(data)}
 calls.append(row);return result
subprocess.run=observed_run
result={'language':language,'started_at_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'native_release_acceptance':False,'selection_reason':'Language scope requires new compiled registry identity in both supported frontends. Two independent builds use the exact pinned compiler/link flags; no Go/Rust runtime semantics or proof limits are reduced.'}
try:
 outputs=[]
 for attempt in ['first','second']:
  work=Path('/private/tmp/mpk-go-rust-scope')/f'build-{language}-{tag}-{attempt}';work.mkdir()
  if language=='go':
   out=work/'go2vir'
   # Exact frontend build portion of release_bundles.build_release_outputs, with no unrelated checker/SDK copy.
   script='''umask 077
test "$(sed -n '1p' /usr/local/go/VERSION)" = "go1.25.0"
mkdir -p /tmp/mpk-home /tmp/mpk-cache
cd /src/go-tools/go2vir
env -i PATH=/usr/local/go/bin:/usr/bin:/bin HOME=/tmp/mpk-home GOCACHE=/tmp/mpk-cache CGO_ENABLED=0 GOOS=linux GOARCH=amd64 GOAMD64=v1 GOENV=off GOFLAGS= GOWORK=off GOPROXY=off GOSUMDB=off GOTOOLCHAIN=local TZ=UTC LANG=C LC_ALL=C SOURCE_DATE_EPOCH=0 /usr/local/go/bin/go build -mod=vendor -trimpath -buildvcs=false -ldflags=-buildid= -o /out/go2vir .
'''
   cmd=[build.go_release.docker_path(),'run','--rm','--pull=never','--network=none','--platform=linux/amd64','--read-only','--tmpfs=/tmp:rw,nosuid,nodev,noexec,mode=1777',f'--mount=type=bind,src={root},dst=/src,readonly',f'--mount=type=bind,src={work},dst=/out',build.go_release.GO_IMAGE,'/bin/sh','-ceu',script]
   completed=subprocess.run(cmd,capture_output=True);assert completed.returncode==0
   bins={'go2vir':out}
  else:
   target=build.portable_rust_build(work);bins={n:target/'x86_64-unknown-linux-gnu/release'/n for n in ['rust2vir','rust2vir-driver']}
  outputs.append({n:{'path':str(p),'sha256':hashlib.sha256(p.read_bytes()).hexdigest(),'bytes':p.stat().st_size} for n,p in bins.items()})
 assert {n:(v['sha256'],v['bytes']) for n,v in outputs[0].items()}=={n:(v['sha256'],v['bytes']) for n,v in outputs[1].items()}
 result.update(status='passed',outputs=outputs,byte_identical=True)
except BaseException as error:
 result.update(status='failed',error=repr(error));(record/'exception.log').write_text(traceback.format_exc())
finally:
 result['commands']=calls;result['whole_source_inputs_unchanged']=all(hashlib.sha256((root/r['path']).read_bytes()).hexdigest()==r['sha256'] for r in inputs);result['finished_at_utc']=datetime.datetime.now(datetime.timezone.utc).isoformat()
 (record/'result.json').write_text(json.dumps(result,indent=2)+'\n')
 print(json.dumps({k:v for k,v in result.items() if k in {'status','error','byte_identical','whole_source_inputs_unchanged'}}))
 if result['status']!='passed':sys.exit(1)
