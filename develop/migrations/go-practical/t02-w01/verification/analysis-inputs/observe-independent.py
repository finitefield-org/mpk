"""Independently hash original inputs/archives and rebuild the observed frontend."""
from pathlib import Path
import base64,datetime,hashlib,json,os,stat,subprocess,time,zipfile
root=Path(__file__).resolve().parent.relative_to(Path.cwd().resolve());out=root/'independent-build';out.mkdir(exist_ok=False)
work=Path('/private/tmp/mpk-go-practical-analysis-independent-build');work.mkdir(exist_ok=False)
def seal(p):
 raw=p.read_bytes();return {'path':str(p),'sha256':hashlib.sha256(raw).hexdigest(),'bytes':len(raw)}
def canonical(x):return json.dumps(x,sort_keys=True,separators=(',',':'),ensure_ascii=False).encode()
def framed(domain,x):return hashlib.sha256(domain.encode()+b'\0'+canonical(x)).hexdigest()
def h1(names,read):
 data=b''.join((hashlib.sha256(read(name)).hexdigest()+'  '+name+'\n').encode() for name in sorted(names))
 return 'h1:'+base64.b64encode(hashlib.sha256(data).digest()).decode()
def readonly_tree(folder):
 for current,dirs,files in os.walk(folder,topdown=False):os.chmod(current,0o555)
record={'status':'running','started_at_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'work':str(work),'CSharp_tests_executed':False,'registered_practical_context':False,'native_Linux_acceptance_claimed':False,'T02_W01_complete':False,'full_T02_gate':'deferred to W04: check-fast.sh --skip-csharp'}
(out/'result.json').write_text(json.dumps(record,indent=2)+'\n');started=time.monotonic()
try:
 captured=root/'final/selected/captured-inputs.json';reproduced=root/'final/selected/reproduced-builds.json'
 inputs=json.loads(captured.read_text());paired=json.loads(reproduced.read_text());objects=inputs['build_inputs'];frontend=Path('go-tools/go2vir')
 buildroot=work/'original-frontend';buildroot.mkdir()
 actual_vendor=set();go_vendor=set();source_kinds={}
 for obj in objects:
  assert obj['schema']=='mpk.go.source_object.v1' and obj['capability']=='analysis-frontend'
  preimage={k:v for k,v in obj.items() if k!='id'};assert framed('MPK-GO-SOURCE-OBJECT-1.0',preimage)==obj['id']
  p=frontend/obj['path'];assert stat.S_ISREG(p.lstat().st_mode);raw=p.read_bytes()
  assert len(raw)==obj['bytes'] and hashlib.sha256(raw).hexdigest()==obj['raw_sha256']
  dest=buildroot/obj['path'];dest.parent.mkdir(parents=True,exist_ok=True);dest.write_bytes(raw);dest.chmod(0o444)
  source_kinds[obj['path']]=obj['kind']
  if obj['path'].startswith('vendor/'):
   actual_vendor.add(obj['path'])
   if obj['kind']=='go_file':go_vendor.add(obj['path'])
 expected_sources={str(p.relative_to(frontend)) for p in frontend.glob('*.go') if not p.name.endswith('_test.go')}
 expected_sources|={str(p.relative_to(frontend)) for p in (frontend/'vendor').rglob('*') if p.is_file()}
 expected_sources|={'go.mod','go.sum'}
 assert set(source_kinds)==expected_sources
 sums={tuple(line.split()[:2]):line.split()[2] for line in (frontend/'go.sum').read_text().splitlines()}
 archive_root=frontend/'testdata/practical-analysis-cache';module_observations=[];differences=[]
 for name,version in [('mod','v0.27.0'),('sync','v0.16.0'),('tools','v0.36.0')]:
  module='golang.org/x/'+name;prefix=archive_root/module/'@v'/version
  archive=Path(str(prefix)+'.zip');mod=Path(str(prefix)+'.mod');info=Path(str(prefix)+'.info');ziphash=Path(str(prefix)+'.ziphash')
  assert json.loads(info.read_text())['Version']==version
  with zipfile.ZipFile(archive) as z:
   assert h1(z.namelist(),z.read)==sums[(module,version)]==ziphash.read_text().strip()
   assert h1(['go.mod'],lambda _:mod.read_bytes())==sums[(module,version+'/go.mod')]
   assert z.read(module+'@'+version+'/go.mod')==mod.read_bytes()
   for p in (frontend/'vendor'/module).rglob('*'):
    if not p.is_file():continue
    member=str(p.relative_to(frontend/'vendor'/module));original=z.read(module+'@'+version+'/'+member);raw=p.read_bytes()
    if raw!=original:
     assert p.name=='TODO' and source_kinds[str(p.relative_to(frontend))]=='auxiliary'
     differences.append({'path':str(p.relative_to(frontend)),'original_sha256':hashlib.sha256(original).hexdigest(),'original_bytes':len(original),'actual_sha256':hashlib.sha256(raw).hexdigest(),'actual_bytes':len(raw)})
  module_observations.append({'path':module,'version':version,'archive':seal(archive),'manifest':seal(mod),'checksum':sums[(module,version)]})
 assert len(differences)==1 and len(inputs['auxiliary_differences'])==1
 for difference in differences:
  d=inputs['auxiliary_differences'][0]
  assert all(d[k]==difference[k] for k in ['original_sha256','original_bytes','actual_sha256','actual_bytes'])
 readonly_tree(buildroot)
 sdk_record=Path('develop/migrations/go-practical/t02-w01/verification/compiler-image/captured/selected/prepared-image.json');sdk=json.loads(sdk_record.read_text());sdk_root=work/'sdk';sdk_root.mkdir()
 original_sdk=Path('/private/tmp/mpk-go-practical-toolchains/go1.25.0-darwin-arm64/go')
 assert framed('MPK-BUNDLE-CONTENT-0.1',sdk['inventory'])==sdk['toolchain']['distribution_sha256']
 for entry in sdk['inventory']['files']:
  relative=entry['path'].removeprefix('go/');raw=(original_sdk/relative).read_bytes()
  assert len(raw)==entry['size_bytes'] and hashlib.sha256(raw).hexdigest()==entry['sha256']
  dest=sdk_root/relative;dest.parent.mkdir(parents=True,exist_ok=True);dest.write_bytes(raw);dest.chmod(0o555 if entry['executable'] else 0o444)
 readonly_tree(sdk_root)
 for directory in ['cache','mod-cache','gopath','home','tmp','built']:(work/directory).mkdir()
 for directory in ['mod-cache','gopath','home']:(work/directory).chmod(0o555)
 environment={'CGO_ENABLED':'0','GO111MODULE':'on','GOAMD64':'v1','GOARCH':'arm64','GOCACHE':str(work/'cache'),'GODEBUG':'','GOENV':'off','GOEXPERIMENT':'','GOFLAGS':'','GOMAXPROCS':'1','GOMODCACHE':str(work/'mod-cache'),'GONOPROXY':'','GONOSUMDB':'','GOOS':'darwin','GOPACKAGESDRIVER':'off','GOPATH':str(work/'gopath'),'GOPRIVATE':'','GOPROXY':'off','GOROOT':str(sdk_root),'GOSUMDB':'off','GOTELEMETRY':'off','GOTOOLCHAIN':'local','GOVCS':'*:off','GOWORK':'off','HOME':str(work/'home'),'LANG':'C','LC_ALL':'C','PATH':str(sdk_root/'bin'),'TMPDIR':str(work/'tmp'),'TZ':'UTC'}
 binary=work/'built/go2vir';command=[str(sdk_root/'bin/go'),'build','-mod=vendor','-trimpath','-buildvcs=false','-pgo=off','-o',str(binary),'.']
 record.update(command=command,environment=environment,source_inventory=seal(captured),paired_result=seal(reproduced),prepared_compiler_inventory=seal(sdk_record),prepared_compiler_distribution_sha256=sdk['toolchain']['distribution_sha256'],build_sources=len(objects),vendored_go_sources=len(go_vendor),vendor_files=len(actual_vendor),modules=module_observations,auxiliary_differences=differences)
 (out/'result.json').write_text(json.dumps(record,indent=2)+'\n')
 with (out/'stdout.log').open('wb') as stdout,(out/'stderr.log').open('wb') as stderr:
  r=subprocess.run(command,cwd=buildroot,env=environment,stdout=stdout,stderr=stderr,timeout=240)
 record.update(exit_code=r.returncode,stdout=seal(out/'stdout.log'),stderr=seal(out/'stderr.log'))
 assert r.returncode==0
 built=seal(binary);assert all(b['binary_sha256']==built['sha256'] and b['bytes']==built['bytes'] for b in paired['builds'])
 with (out/'binary-build-info.log').open('wb') as stdout,(out/'binary-build-info.stderr.log').open('wb') as stderr:
  r=subprocess.run([str(sdk_root/'bin/go'),'version','-m',str(binary)],env=environment,stdout=stdout,stderr=stderr,timeout=30)
 assert r.returncode==0
 record.update(status='passed',binary=built,byte_exact_equal_to_both_primary_builds=True,build_info_observation=seal(out/'binary-build-info.log'),build_info_stderr=seal(out/'binary-build-info.stderr.log'),scope='Independent fresh build from individually hash-checked original sources and the exact copied prepared compiler inventory, with a third empty cache and closed offline environment. Proves this development source/build correspondence, not a registered practical semantic context, application proof or native Linux release.')
except Exception as error:
 record.update(status='failed',error=repr(error))
record.update(wall_seconds=time.monotonic()-started,finished_at_utc=datetime.datetime.now(datetime.timezone.utc).isoformat())
(out/'result.json').write_text(json.dumps(record,indent=2)+'\n');print(json.dumps({k:record[k] for k in ['status','build_sources','vendored_go_sources','binary','wall_seconds','error'] if k in record}))
if record['status']!='passed':raise SystemExit(1)
