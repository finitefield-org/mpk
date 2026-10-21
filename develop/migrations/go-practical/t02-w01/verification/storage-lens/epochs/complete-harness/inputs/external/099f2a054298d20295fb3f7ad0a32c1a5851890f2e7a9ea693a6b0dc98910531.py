"""Original pinned Go observations of current-parent views; instrumentation only."""
from pathlib import Path
import datetime,hashlib,json,os,runpy,subprocess,time
OUT=Path(__file__).resolve().parent
PRIOR=OUT.parent/'array-equality/native-go/result.json'
prior=json.loads(PRIOR.read_bytes());assert prior['status']=='passed'
def seal(path):
    raw=Path(path).read_bytes();return {'path':str(path),'sha256':hashlib.sha256(raw).hexdigest(),'bytes':len(raw)}
sdk=json.loads(Path(prior['whole_original_sdk_inputs']['path']).read_bytes())
def verify_sdk():
    assert all(seal(row['path'])==row for row in sdk)
    root=Path(prior['environment']['GOROOT'])
    assert {str(p)for p in root.rglob('*')if p.is_file()}=={row['path']for row in sdk}
verify_sdk()
transport=OUT.parent/'array-equality/toolchain-archive/manifest.json'
raw=runpy.run_path(str(OUT.parent/'array-equality/transport-archive.py'))['decode']()
assert len(raw)==prior['toolchain_archive']['bytes']and hashlib.sha256(raw).hexdigest()==prior['toolchain_archive']['sha256']
run=OUT/'native-go-current';run.mkdir(exist_ok=False);(run/'observe-go.py').write_bytes(Path(__file__).read_bytes())
lengths=[1,2,3,4,5,7,8,9,15,16,17,31,32,33,63,64,65,127,128,129,255,256]
cases=[]
def new_case(kind,n,index,depth,seed):
    return {'id':f'{kind}-{n:03}-{depth:02}-{index:03}-{seed}','kind':kind,'length':n,'index':index,'depth':depth,'bytes':[(i*73+seed*19+n)%256 for i in range(256)],'replacement':[(i*29+seed*11+n+128)%256 for i in range(256)],'tag':(n+seed)%256,'replacement_tag':(n+seed+97)%256,'value1':(n+seed+61)%256,'value2':(n+seed+173)%256}
for n in lengths:
    for i in sorted({0,n//2,n-1}):
        for seed in [0,1]:cases.append(new_case('array',n,i,0,seed))
for d in range(9):
    for i in [0,1,127,128,255]:cases.append(new_case('nested',256,i,d,2))
for d in range(1,17):
    for seed in [3,4]:cases.append(new_case('chain',d+1,0,d,seed))
case_path=run/'cases.json';case_path.write_text(json.dumps(cases,indent=2)+'\n')
source=[r'''package main
import("encoding/json";"os")
type Input struct {ID string `json:"id"`;Kind string `json:"kind"`;Length,Index,Depth int;Bytes,Replacement []int;Tag byte;ReplacementTag byte `json:"replacement_tag"`;Value1,Value2 byte}
type Snapshot struct {Bytes [256]byte `json:"bytes"`;Tag byte `json:"tag"`}
type Output struct {ID string `json:"id"`;Snapshots []Snapshot `json:"snapshots"`;AliasBefore byte `json:"alias_before"`;AliasAfter byte `json:"alias_after"`}
func buffer(v []int)[256]byte{if len(v)!=256{panic("whole input")};var a [256]byte;for i,x:=range v{if x<0||x>255{panic("byte")};a[i]=byte(x)};return a}
''']
for n in lengths:
    source.append(f'''type Root{n} struct {{Rows [{n}]byte;Tag byte}}
func snap{n}(r Root{n})Snapshot{{var s Snapshot;copy(s.Bytes[:],r.Rows[:]);s.Tag=r.Tag;return s}}
func array{n}(in Input)Output{{b,c:=buffer(in.Bytes),buffer(in.Replacement);var r,t Root{n};copy(r.Rows[:],b[:]);r.Tag=in.Tag;copy(t.Rows[:],c[:]);t.Tag=in.ReplacementTag;saved:=r;child:=r.Rows[:];shared:=child;child[in.Index]=in.Value1;a:=snap{n}(r);before:=shared[in.Index];r=t;parent:=snap{n}(r);after:=shared[in.Index];shared[in.Index]=in.Value2;return Output{{in.ID,[]Snapshot{{snap{n}(saved),a,parent,snap{n}(r),snap{n}(saved)}},before,after}}}}
''')
for d in range(9):
    outer,inner=1<<d,1<<(8-d)
    source.append(f'''type Nested{d} struct {{Rows [{outer}][{inner}]byte;Tag byte}}
func snest{d}(r Nested{d})Snapshot{{var s Snapshot;for i:=range r.Rows{{copy(s.Bytes[i*{inner}:(i+1)*{inner}],r.Rows[i][:])}};s.Tag=r.Tag;return s}}
func nested{d}(in Input)Output{{b,c:=buffer(in.Bytes),buffer(in.Replacement);var r,t Nested{d};for i:=range r.Rows{{copy(r.Rows[i][:],b[i*{inner}:(i+1)*{inner}]);copy(t.Rows[i][:],c[i*{inner}:(i+1)*{inner}])}};r.Tag=in.Tag;t.Tag=in.ReplacementTag;saved:=r;child:=r.Rows[in.Index/{inner}][:];shared:=child;child[in.Index%{inner}]=in.Value1;a:=snest{d}(r);before:=shared[in.Index%{inner}];r=t;parent:=snest{d}(r);after:=shared[in.Index%{inner}];shared[in.Index%{inner}]=in.Value2;return Output{{in.ID,[]Snapshot{{snest{d}(saved),a,parent,snest{d}(r),snest{d}(saved)}},before,after}}}}
''')
source.append('type Chain0 byte\n')
for d in range(1,17):
    source.append(f'type Chain{d} struct {{Child Chain{d-1};Mark byte}}\n')
    child='r'+'.Child'*d
    make='Chain0(b[0])';replace='Chain0(c[0])'
    for k in range(1,d+1):
        make=f'Chain{k}{{Child:{make},Mark:b[{k}]}}'
        replace=f'Chain{k}{{Child:{replace},Mark:c[{k}]}}'
    snap='s.Bytes[0]=byte('+child+');'+''.join(f's.Bytes[{k}]='+('r'+'.Child'*(d-k))+'.Mark;' for k in range(1,d+1))
    source.append(f'''func schain{d}(r Chain{d})Snapshot{{var s Snapshot;{snap}return s}}
func chain{d}(in Input)Output{{b,c:=buffer(in.Bytes),buffer(in.Replacement);r:={make};t:={replace};saved:=r;child:=&{child};shared:=child;*child=Chain0(in.Value1);a:=schain{d}(r);before:=byte(*shared);r=t;parent:=schain{d}(r);after:=byte(*shared);*shared=Chain0(in.Value2);return Output{{in.ID,[]Snapshot{{schain{d}(saved),a,parent,schain{d}(r),schain{d}(saved)}},before,after}}}}
''')
source.append('func observe(in Input)Output{switch in.Kind{case "array":switch in.Length{'+''.join(f'case {n}:return array{n}(in);'for n in lengths)+'};case "nested":switch in.Depth{'+''.join(f'case {d}:return nested{d}(in);'for d in range(9))+'};case "chain":switch in.Depth{'+''.join(f'case {d}:return chain{d}(in);'for d in range(1,17))+'}};panic("case")}\n')
source.append('func main(){var rows []Input;if err:=json.NewDecoder(os.Stdin).Decode(&rows);err!=nil{panic(err)};out:=make([]Output,0,len(rows));for _,r:=range rows{out=append(out,observe(r))};if err:=json.NewEncoder(os.Stdout).Encode(out);err!=nil{panic(err)}}\n')
source_path=run/'main.go';source_path.write_text(''.join(source))
env=os.environ.copy();updates={**prior['environment'],'GOCACHE':'/private/tmp/mpk-go-storage-lens-go-cache'};env.update(updates)
record={'status':'running','started_at_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'actual_driver_prelaunch':seal(run/'observe-go.py'),'pinned_original_sdk_receipt':seal(PRIOR),'whole_original_sdk_inputs':prior['whole_original_sdk_inputs'],'lossless_original_sdk_transport':seal(transport),'compiler_binary':prior['compiler_binary'],'sources':[seal(source_path)],'cases_input':seal(case_path),'environment':updates,'commands':[],'source_or_registered_context_proof':False,'native_Linux_release_acceptance':False,'closed_profile_source_admission_claimed':False}
def save():(run/'result.json').write_text(json.dumps(record,indent=2)+'\n')
def execute(name,cmd,input_data=None):
    save();start=time.monotonic();p=subprocess.run(cmd,input=input_data,env=env,capture_output=True,check=False)
    stdout=run/(name+'.stdout.log');stderr=run/(name+'.stderr.log');stdout.write_bytes(p.stdout);stderr.write_bytes(p.stderr)
    record['commands'].append({'name':name,'command':cmd,'exit_code':p.returncode,'seconds':time.monotonic()-start,'stdout':seal(stdout),'stderr':seal(stderr)});save();assert p.returncode==0 and not p.stderr
    return p
save()
try:
    version=execute('version',[prior['compiler_binary']['path'],'version']);assert version.stdout.decode().strip()=='go version go1.25.0 darwin/arm64'
    binary=Path('/private/tmp/mpk-go-storage-lens-native-oracle');execute('build',[prior['compiler_binary']['path'],'build','-o',str(binary),str(source_path)]);record['oracle_binary']=seal(binary)
    output=json.loads(execute('views',[str(binary)],case_path.read_bytes()).stdout);assert len(output)==len(cases)
    for case,row in zip(cases,output):
        n=case['length'];i=case['index'];tag=case['tag']if case['kind']!='chain'else 0;rtag=case['replacement_tag']if case['kind']!='chain'else 0
        original=case['bytes'][:n]+[0]*(256-n);replacement=case['replacement'][:n]+[0]*(256-n)
        changed=original.copy();changed[i]=case['value1'];last=replacement.copy();last[i]=case['value2']
        expected=[{'bytes':original,'tag':tag},{'bytes':changed,'tag':tag},{'bytes':replacement,'tag':rtag},{'bytes':last,'tag':rtag},{'bytes':original,'tag':tag}]
        assert row=={'id':case['id'],'snapshots':expected,'alias_before':case['value1'],'alias_after':case['replacement'][i]}
    verify_sdk();assert seal(binary)==record['oracle_binary']
    record.update(status='passed',whole_input_cases=len(cases),full_root_snapshots=len(cases)*5,array_lengths=len(lengths),nested_array_splits=9,path_lengths=16,finished_at_utc=datetime.datetime.now(datetime.timezone.utc).isoformat());save()
    print(json.dumps({k:record[k]for k in ['status','whole_input_cases','full_root_snapshots','array_lengths','nested_array_splits','path_lengths']}))
except BaseException:
    record['status']='failed';save();raise
