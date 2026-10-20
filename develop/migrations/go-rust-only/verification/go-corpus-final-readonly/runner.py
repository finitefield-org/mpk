from pathlib import Path
import base64,datetime,hashlib,json,subprocess,sys,time
ROOT=Path.cwd();MODE=sys.argv[1];TAG=sys.argv[2];assert MODE in {'update','check'}
OUT=ROOT/'develop/migrations/go-rust-only/verification'/('go-corpus-'+TAG);OUT.mkdir(exist_ok=False)
sys.path.insert(0,'scripts');import release_bundles as release
inputs=[]
for base in ['go-tools/go2vir','fixtures/go-alpha','fixtures/go-basic','examples']:
 for p in sorted((ROOT/base).rglob('*')):
  if p.is_file() and (p.suffix=='.go' or p.name in {'go.mod','go.sum'}):
   b=p.read_bytes();inputs.append({'path':str(p.relative_to(ROOT)),'sha256':hashlib.sha256(b).hexdigest(),'bytes':len(b),'base64':base64.b64encode(b).decode()})
(OUT/'source-inputs.bytes.json').write_text(json.dumps({'schema':'mpk.original_bytes_set.v1','files':inputs},indent=2)+'\n');(OUT/'runner.py').write_bytes(Path(__file__).read_bytes())
writable=[ROOT/'fixtures/vir-go']
if MODE=='update':
 names={'frontend-envelope.json','vir.json','source-map.json','source-manifest.frontend.json','mpk-semantic-context.json','mpk-selection.json'}
 writable += [p for p in sorted((ROOT/'examples').rglob('*.json')) if p.name in names]
 before=[]
 for base in writable:
  for p in ([base] if base.is_file() else sorted(base.rglob('*'))):
   if p.is_file():
    b=p.read_bytes();before.append({'path':str(p.relative_to(ROOT)),'sha256':hashlib.sha256(b).hexdigest(),'bytes':len(b),'base64':base64.b64encode(b).decode()})
 (OUT/'generated-before.bytes.json').write_text(json.dumps({'schema':'mpk.original_bytes_set.v1','files':before},indent=2)+'\n')
cache=Path('/private/tmp/mpk-go-rust-scope/go-corpus-final-cache');cache.mkdir(exist_ok=True);cache.chmod(0o777)
cmd=[release.docker_path(),'run','--rm','--pull=never','--network=none','--platform=linux/amd64','--read-only','--user=501:501','--cap-drop=ALL','--security-opt=no-new-privileges','--cpus=6','--memory=12g','--pids-limit=256','--tmpfs=/tmp:rw,exec,nosuid,nodev,size=4g,mode=1777',f'--mount=type=bind,src={ROOT},dst=/src,readonly',f'--mount=type=bind,src={cache},dst=/gocache']
if MODE=='update':
 for p in writable:cmd.append(f'--mount=type=bind,src={p},dst=/src/{p.relative_to(ROOT).as_posix()}')
cmd.extend(['--workdir=/src/go-tools/go2vir',release.GO_IMAGE,'/bin/sh','-ceu','test "$(sed -n 1p /usr/local/go/VERSION)" = go1.25.0; mkdir -p /tmp/mpk-home; exec /usr/bin/env -i PATH=/usr/local/go/bin:/usr/bin:/bin HOME=/tmp/mpk-home GOCACHE=/gocache GOMAXPROCS=4 CGO_ENABLED=0 GOOS=linux GOARCH=amd64 GOAMD64=v1 GOENV=off GOFLAGS= GOWORK=off GOPROXY=off GOSUMDB=off GOTOOLCHAIN=local TZ=UTC LANG=C LC_ALL=C'+(' MPK_UPDATE_GO_CORPUS=1' if MODE=='update' else '')+' /usr/local/go/bin/go test -buildvcs=false -mod=vendor -count=1 -timeout=10m -v -run "^TestActiveGoCorpus$" .'])
start=time.time();r=subprocess.run(cmd,capture_output=True);result={'status':'passed' if r.returncode==0 else 'failed','command':cmd,'exit_code':r.returncode,'seconds':time.time()-start,'mode':MODE,'native_release_acceptance':False,'selection_reason':'Final Go/Rust bundle root changed; refresh/verify only the affected canonical Go corpus and examples using the exact pinned Go image. The heavy unchanged standard-library source suite already passed in the earlier aggregate retry.'}
for stream,b in [('stdout',r.stdout),('stderr',r.stderr)]:
 p=OUT/(stream+'.log');p.write_bytes(b);result[stream]={'path':str(p.relative_to(ROOT)),'sha256':hashlib.sha256(b).hexdigest(),'bytes':len(b)}
result['source_inputs_unchanged']=all(hashlib.sha256((ROOT/x['path']).read_bytes()).hexdigest()==x['sha256'] for x in inputs)
if not result['source_inputs_unchanged']:result['status']='failed'
result['finished_at_utc']=datetime.datetime.now(datetime.timezone.utc).isoformat();(OUT/'result.json').write_text(json.dumps(result,indent=2)+'\n');print(json.dumps({k:v for k,v in result.items() if k in {'status','exit_code','seconds','source_inputs_unchanged'}}),flush=True)
raise SystemExit(result['status']!='passed')
