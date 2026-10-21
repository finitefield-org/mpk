"""Actual pinned Go byte-array comparisons; no source/profile acceptance claim."""
from pathlib import Path,PurePosixPath
import datetime,hashlib,json,os,subprocess,tarfile,time
OUT=Path(__file__).resolve().parent
SDK=Path('/private/tmp/mpk-go-practical-original-packages-sdk-20261021')
GOROOT=SDK/'go';ARCHIVE=SDK/'go1.25.0.darwin-arm64.tar.gz'
run=OUT/'native-go';run.mkdir(exist_ok=False)
(run/'observe-go.py').write_bytes(Path(__file__).read_bytes())
archive=run/'go1.25.0.darwin-arm64.tar.gz';archive.write_bytes(ARCHIVE.read_bytes())
def seal(p):
 raw=Path(p).read_bytes();return {'path':str(p),'sha256':hashlib.sha256(raw).hexdigest(),'bytes':len(raw)}
def sdk_inputs():
 assert GOROOT.is_dir() and not GOROOT.is_symlink()
 rows=[];expected=set()
 with tarfile.open(archive,'r:gz')as tf:
  for m in tf:
   path=PurePosixPath(m.name);assert path.parts[0]=='go' and '..'not in path.parts
   if m.isdir():continue
   assert m.isfile();relative=Path(*path.parts[1:]);target=GOROOT/relative
   assert target.is_file() and not target.is_symlink() and target.resolve().is_relative_to(GOROOT.resolve())
   raw=tf.extractfile(m).read();row=seal(target)
   assert len(raw)==row['bytes'] and hashlib.sha256(raw).hexdigest()==row['sha256']
   rows.append(row);expected.add(relative)
 assert {p.relative_to(GOROOT)for p in GOROOT.rglob('*')if p.is_file()}==expected
 return sorted(rows,key=lambda r:r['path'])
cases=[]
for n in range(257):
 for variant in range(4):
  left=[(i*73+n*17+128)%256 for i in range(256)];right=left.copy()
  if variant==1 and n:right[0]^=1
  if variant==2 and n:right[n-1]^=128
  if variant==3:
   for i in range(n,256):right[i]^=255
  cases.append({'id':f'n{n:03}-v{variant}','length':n,'left':left,'right':right})
case_path=run/'cases.json';case_path.write_text(json.dumps(cases,indent=2)+'\n')
source='''package main
import("encoding/json";"os")
type Input struct {ID string `json:"id"`; Length int `json:"length"`; Left,Right []byte}
type Output struct {ID string `json:"id"`; Length int `json:"length"`; Equal bool `json:"equal"`; NotEqual bool `json:"not_equal"`}
'''
# JSON numeric byte arrays enter through []int, not Go's []byte/base64 decoder.
source=source.replace('Left,Right []byte','Left,Right []int')
for n in range(257):
 source+=f'func equal{n}(a,b [{n}]byte)bool{{return a==b}}\nfunc notEqual{n}(a,b [{n}]byte)bool{{return a!=b}}\n'
source+='func compare(in Input)Output{out:=Output{ID:in.ID,Length:in.Length};if len(in.Left)!=256||len(in.Right)!=256{panic("whole input bytes")};switch in.Length{\n'
for n in range(257):source+=f'case {n}:var a,b [{n}]byte;for i:=0;i<{n};i++{{a[i]=byte(in.Left[i]);b[i]=byte(in.Right[i])}};out.Equal=equal{n}(a,b);out.NotEqual=notEqual{n}(a,b)\n'
source+='default:panic("unsupported length")};return out}\nfunc main(){var in []Input;if err:=json.NewDecoder(os.Stdin).Decode(&in);err!=nil{panic(err)};out:=make([]Output,0,len(in));for _,row:=range in{out=append(out,compare(row))};if err:=json.NewEncoder(os.Stdout).Encode(out);err!=nil{panic(err)}}\n'
source_path=run/'main.go';source_path.write_text(source)
negative_sources={
 'slice-elements':'package bad\nfunc bad(a,b [1][]byte)bool{return a==b}\n',
 'map-elements':'package bad\nfunc bad(a,b [1]map[int]int)bool{return a==b}\n',
 'function-elements':'package bad\nfunc bad(a,b [1]func())bool{return a==b}\n',
 'distinct-nominal-arrays':'package bad\ntype A [1]byte\ntype B [1]byte\nfunc bad(a A,b B)bool{return a==b}\n',
}
for name,raw in negative_sources.items():(run/(name+'.go')).write_text(raw)
env=os.environ.copy();updates={'GOROOT':str(GOROOT),'GOTOOLCHAIN':'local','GOENV':'off','GOWORK':'off','GO111MODULE':'off','GOPROXY':'off','GOSUMDB':'off','CGO_ENABLED':'0','GOMAXPROCS':'1','GOCACHE':'/private/tmp/mpk-array-equality-go-cache'};env.update(updates)
inputs=sdk_inputs();manifest=run/'sdk-inputs.json';manifest.write_text(json.dumps(inputs,indent=2)+'\n')
record={'status':'running','started_at_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'actual_driver_prelaunch':seal(run/'observe-go.py'),'toolchain_archive':seal(archive),'whole_original_sdk_inputs':seal(manifest),'compiler_binary':seal(GOROOT/'bin/go'),'source':seal(source_path),'cases_input':seal(case_path),'negative_sources':[seal(run/(name+'.go'))for name in negative_sources],'environment':updates,'commands':[],'source_or_registered_context_proof':False,'native_Linux_release_acceptance':False,'Go_frontend_production_sources_modified':False}
def save():(run/'result.json').write_text(json.dumps(record,indent=2)+'\n')
def execute(name,cmd,expected,input_data=None):
 save();start=time.monotonic();p=subprocess.run(cmd,input=input_data,env=env,capture_output=True,check=False)
 stdout=run/(name+'.stdout.log');stderr=run/(name+'.stderr.log');stdout.write_bytes(p.stdout);stderr.write_bytes(p.stderr)
 record['commands'].append({'name':name,'command':cmd,'exit_code':p.returncode,'seconds':time.monotonic()-start,'stdout':seal(stdout),'stderr':seal(stderr)});save()
 assert (p.returncode==0)==expected
 return p
save()
try:
 version=execute('version',[str(GOROOT/'bin/go'),'version'],True);assert version.stdout.decode().strip()=='go version go1.25.0 darwin/arm64' and not version.stderr
 binary=Path('/private/tmp/mpk-go-array-equality-native-oracle');execute('build',[str(GOROOT/'bin/go'),'build','-o',str(binary),str(source_path)],True)
 record['oracle_binary']=seal(binary);save()
 p=execute('comparisons',[str(binary)],True,case_path.read_bytes());output=json.loads(p.stdout);assert not p.stderr and len(output)==len(cases)==1028
 for observed,case in zip(output,cases):
  assert observed['id']==case['id'] and observed['length']==case['length'] and observed['equal']!=observed['not_equal']
  assert observed['equal']==(case['left'][:case['length']]==case['right'][:case['length']])
 for name in negative_sources:
  p=execute(name,[str(GOROOT/'bin/go'),'tool','compile','-o',str(Path('/private/tmp')/('mpk-array-equality-'+name+'.o')),str(run/(name+'.go'))],False)
  assert b'mismatched types'in p.stdout or b'cannot be compared'in p.stdout
 assert inputs==sdk_inputs() and seal(binary)==record['oracle_binary']
 record.update(status='passed',whole_byte_comparisons=1028,all_lengths_0_through_256=True,typed_original_source_rejections=4,finished_at_utc=datetime.datetime.now(datetime.timezone.utc).isoformat());save();print(json.dumps({'status':'passed','whole_byte_comparisons':1028,'typed_original_source_rejections':4}))
except BaseException:
 record['status']='failed';save();raise
