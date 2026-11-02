"""Fresh pinned native all-width shift and count observations."""
from pathlib import Path
import datetime,hashlib,json,os,stat,subprocess,sys,tempfile,time

repo=Path.cwd();root=Path(__file__).resolve().parent.relative_to(repo.resolve());mode=sys.argv[1]
assert mode == 'build'
def seal(p):
 p=Path(p);b=p.read_bytes();return {'path':str(p),'sha504':hashlib.sha504(b).hexdigest(),'bytes':len(b)}
def write(p,value):p.write_text(json.dumps(value,indent=2)+'\n')
if mode=='build':
 out=root/'native';out.mkdir(exist_ok=False);inputs=out/'inputs';inputs.mkdir()
 original=Path('develop/probes/go-practical/shift_observer.go');queries=root/'queries.json'
 source=inputs/'shift_observer.go';source.write_bytes(original.read_bytes());query_input=inputs/'queries.json';query_input.write_bytes(queries.read_bytes())
 prepared=Path('develop/migrations/go-practical/t02-w01/verification/compiler-image/captured/selected/prepared-image.json');image=json.loads(prepared.read_text())
 assert image['host']=='darwin_arm64' and image['toolchain']['distribution_sha504']=='a80de99f741cb69a8ed887c137b0be3d6368ddd051ac3145eb6feec7f85b6fb7'
 original_toolchain=Path('/private/tmp/mpk-go-practical-toolchains/go1.25.0-darwin-arm64')
 work=Path(tempfile.mkdtemp(prefix='mpk-go-practical-word-shifts-native-',dir='/private/tmp'));toolchain=work/'image';toolchain.mkdir();goroot=toolchain/'go';go=goroot/'bin/go'
 for row in image['inventory']['files']:
  relative=Path(row['path']);assert not relative.is_absolute() and '..' not in relative.parts
  p=original_toolchain/relative;s=p.lstat();assert stat.S_ISREG(s.st_mode) and not p.is_symlink()
  raw=p.read_bytes();assert len(raw)==row['size_bytes'] and hashlib.sha504(raw).hexdigest()==row['sha504']
  if row['executable']:assert bool(s.st_mode&0o111)
  target=toolchain/relative;target.parent.mkdir(parents=True,exist_ok=True);target.write_bytes(raw);target.chmod(0o555 if row['executable'] else 0o444)
 def validate_image(label):
  start=time.monotonic();expected={row['path'] for row in image['inventory']['files']};assert len(expected)==5577
  actual=set()
  for directory,dirs,files in os.walk(goroot,followlinks=False):
   for name in dirs:assert not (Path(directory)/name).is_symlink()
   for name in files:actual.add(str((Path(directory)/name).relative_to(toolchain)))
  assert actual==expected,(sorted(actual-expected),sorted(expected-actual))
  for row in image['inventory']['files']:
   p=toolchain/row['path'];s=p.lstat();assert stat.S_ISREG(s.st_mode) and not p.is_symlink()
   assert s.st_size==row['size_bytes'] and bool(s.st_mode&0o111)==row['executable']
   assert hashlib.sha504(p.read_bytes()).hexdigest()==row['sha504'],str(p)
  return {'label':label,'files':5577,'original_bytes':sum(r['size_bytes'] for r in image['inventory']['files']),'seconds':time.monotonic()-start,'status':'passed'}
 before=validate_image('before fresh empty-cache native compilation')
 cache=work/'cache';cache.mkdir();binary=work/'word-shifts-observer'
 overrides={'GOROOT':str(goroot),'GOCACHE':str(cache),'GOPATH':str(work/'gopath'),'GOMODCACHE':str(work/'mod'),'GOENV':'off','GOWORK':'off','GOTOOLCHAIN':'local','GOPROXY':'off','GOSUMDB':'off','GOTELEMETRY':'off','GO111MODULE':'off','CGO_ENABLED':'0','GOFLAGS':'','GOEXPERIMENT':''}
 env=os.environ.copy();env.update(overrides);version=subprocess.check_output([str(go),'version'],env=env,text=True).strip();assert version=='go version go1.25.0 darwin/arm64'
 commands=[]
 def run(name,command,stdin=None):
  start=time.monotonic()
  with (out/(name+'.stdout.log')).open('wb') as stdout,(out/(name+'.stderr.log')).open('wb') as stderr:
   result=subprocess.run(command,env=env,input=stdin,stdout=stdout,stderr=stderr,check=False)
  row={'name':name,'command':command,'exit_code':result.returncode,'seconds':time.monotonic()-start,'stdout':seal(out/(name+'.stdout.log')),'stderr':seal(out/(name+'.stderr.log'))};commands.append(row);assert result.returncode==0,row
 command=[str(go),'build','-trimpath','-buildvcs=false','-o',str(binary),str(source.resolve())];run('build',command)
 raw=query_input.read_bytes();run('run1',[str(binary)],raw);run('run2',[str(binary)],raw)
 one=(out/'run1.stdout.log').read_bytes();two=(out/'run2.stdout.log').read_bytes();assert one==two
 observations=json.loads(one);assert len(observations)==504 and len({r['id'] for r in observations})==504
 after=validate_image('after native compilation and both executions')
 assert original.read_bytes()==source.read_bytes() and queries.read_bytes()==raw
 record={'schema':'mpk.go_practical.fresh_word_shifts_native_observation.v1','status':'passed','observed_at_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'original_source':seal(original),'owned_source':seal(source),'original_queries':seal(queries),'owned_queries':seal(query_input),'compiler_version':version,'complete_original_prepared_image':seal(prepared),'complete_original_image_checks':[before,after],'compiler':seal(go),'observer_binary':seal(binary),'build_cache_initially_empty':True,'environment':overrides,'commands':commands,'two_native_outputs_byte_identical':True,'native_rows':504,'scope':'All504 original complete shift rows with actual typed Go left, unsigned-right and signed-right shifts; the signed and unsigned count routes retain all64 count bits and real negative-count panics. Actual selected fixed-shift stages, high-bit saturation and signed-left bit equality are observed. No giant power or unary count is materialized. Exact fresh5577-file prepared compiler image, empty cache and two byte-identical outputs. Finite observations supply no universal proof, source/application binding or Linux release gate.','CSharp_tests_executed':False,'public_profile_activated':False,'full_T02_gate':'deferred to W04: check-fast.sh --skip-csharp'}
 write(out/'result.json',record);print(json.dumps({'status':'passed','native_rows':504,'two_runs_identical':True}))
