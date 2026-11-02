"""Actual full-width Go slice bounds, nil metadata and shared-view observations."""
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
run=OUT/'native-go';run.mkdir(exist_ok=False);(run/'observe-go.py').write_bytes(Path(__file__).read_bytes())
MAX=(1<<64)-1
cases=[]
descriptors=[(True,0,0,0)]
for cap in [0,1,3,16,128,256]:
    for n in sorted({0,cap//2,cap}):
        descriptors.append((False,0 if cap==256 else 1,n,cap))
for descriptor,(nil,offset,n,cap)in enumerate(descriptors):
    bounds=[(0,0,0,0),(0,0,n,cap),(max(0,n-1),0,cap,cap),(n,0,n,cap),(n+1,0,cap,cap),(0,1,0,cap),(0,0,cap+1,cap+1),(0,0,0,cap+1),(0,cap,cap,cap)]
    for high in [1<<32,(1<<63)-1,1<<63,MAX]:
        bounds.extend([(high,0,n,cap),(0,high,high,high),(0,0,high,high),(0,0,0,high)])
    for j,(index,low,high,maximum)in enumerate(bounds):
        cases.append({'id':f'd{descriptor:02}-b{j:02}','nil':nil,'offset':offset,'length':n,'capacity':cap,'index':index,'low':low,'high':high,'max':maximum,'bytes':[(k*73+descriptor*19+128)%256 for k in range(256)]})
case_path=run/'cases.json';case_path.write_text(json.dumps(cases,indent=2)+'\n')
source=r'''package main
import("encoding/json";"os")
type Input struct {ID string `json:"id"`;Nil bool `json:"nil"`;Offset,Length,Capacity,Index,Low,High,Max uint64;Bytes []int}
type Output struct {ID string `json:"id"`;Nil bool `json:"nil"`;Length int `json:"length"`;Capacity int `json:"capacity"`;IndexUnsigned bool `json:"index_unsigned"`;IndexSigned bool `json:"index_signed"`;TwoUnsigned bool `json:"two_unsigned"`;TwoSigned bool `json:"two_signed"`;ThreeUnsigned bool `json:"three_unsigned"`;ThreeSigned bool `json:"three_signed"`}
func capture(f func())(ok bool){defer func(){if recover()!=nil{ok=false}}();f();return true}
func observe(in Input)Output{
 var a [256]byte;if len(in.Bytes)!=256{panic("whole input")};for i,v:=range in.Bytes{if v<0||v>255{panic("byte")};a[i]=byte(v)}
 var s []byte;if in.Nil{if in.Offset!=0||in.Length!=0||in.Capacity!=0{panic("nil input")}}else{s=a[in.Offset:in.Offset+in.Length:in.Offset+in.Capacity]}
 out:=Output{ID:in.ID,Nil:s==nil,Length:len(s),Capacity:cap(s)}
 out.IndexUnsigned=capture(func(){_=s[in.Index]});out.IndexSigned=capture(func(){_=s[int64(in.Index)]})
 out.TwoUnsigned=capture(func(){_=s[in.Low:in.High]});out.TwoSigned=capture(func(){_=s[int64(in.Low):int64(in.High)]})
 out.ThreeUnsigned=capture(func(){_=s[in.Low:in.High:in.Max]});out.ThreeSigned=capture(func(){_=s[int64(in.Low):int64(in.High):int64(in.Max)]})
 return out
}
func main(){var rows []Input;if err:=json.NewDecoder(os.Stdin).Decode(&rows);err!=nil{panic(err)};out:=make([]Output,0,len(rows));for _,r:=range rows{out=append(out,observe(r))};if err:=json.NewEncoder(os.Stdout).Encode(out);err!=nil{panic(err)}}
'''
source_path=run/'main.go';source_path.write_text(source)
alias=r'''package main
import("encoding/json";"os")
func main(){rows:=make([][4]uint64,2);rows[0]=[4]uint64{1,2,3,4};child:=rows[0][:];copyOfChild:=child;copyOfChild[1]=9;afterChild:=rows[0];rows[0]=[4]uint64{5,6,7,8};afterParent:=append([]uint64(nil),child...);empty:=child[:0];tail:=empty[:cap(empty)];tail[3]=17;var nilSlice []uint64;nonNil:=make([]uint64,0);if err:=json.NewEncoder(os.Stdout).Encode(struct{AfterChild [4]uint64;AfterParent []uint64;AfterTail [4]uint64;CopyCap int;EmptyLen int;EmptyCap int;NilSlice bool;NonNil bool;NilReslice bool;NonNilReslice bool}{afterChild,afterParent,rows[0],cap(copyOfChild),len(empty),cap(empty),nilSlice==nil,nonNil==nil,nilSlice[:0:0]==nil,nonNil[:0:0]==nil});err!=nil{panic(err)}}
'''
alias_path=run/'aliases.go';alias_path.write_text(alias)
env=os.environ.copy();updates={**prior['environment'],'GOCACHE':'/private/tmp/mpk-go-slice-descriptor-go-cache'};env.update(updates)
record={'status':'running','started_at_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'actual_driver_prelaunch':seal(run/'observe-go.py'),'pinned_original_sdk_receipt':seal(PRIOR),'whole_original_sdk_inputs':prior['whole_original_sdk_inputs'],'lossless_original_sdk_transport':seal(transport),'compiler_binary':prior['compiler_binary'],'sources':[seal(source_path),seal(alias_path)],'cases_input':seal(case_path),'environment':updates,'commands':[],'source_or_registered_context_proof':False,'native_Linux_release_acceptance':False,'closed_profile_source_admission_claimed':False}
def save():(run/'result.json').write_text(json.dumps(record,indent=2)+'\n')
def execute(name,cmd,input_data=None):
    save();start=time.monotonic();p=subprocess.run(cmd,input=input_data,env=env,capture_output=True,check=False)
    stdout=run/(name+'.stdout.log');stderr=run/(name+'.stderr.log');stdout.write_bytes(p.stdout);stderr.write_bytes(p.stderr)
    record['commands'].append({'name':name,'command':cmd,'exit_code':p.returncode,'seconds':time.monotonic()-start,'stdout':seal(stdout),'stderr':seal(stderr)});save();assert p.returncode==0 and not p.stderr
    return p
save()
try:
    version=execute('version',[prior['compiler_binary']['path'],'version']);assert version.stdout.decode().strip()=='go version go1.25.0 darwin/arm64'
    binary=Path('/private/tmp/mpk-go-slice-descriptor-native-oracle');execute('build',[prior['compiler_binary']['path'],'build','-o',str(binary),str(source_path)]);record['oracle_binary']=seal(binary)
    output=json.loads(execute('bounds',[str(binary)],case_path.read_bytes()).stdout);assert len(output)==len(cases)
    for row,case in zip(output,cases):
        assert row['id']==case['id']and row['nil']==case['nil']and row['length']==case['length']and row['capacity']==case['capacity']
        i,lo,hi,m=[case[n]for n in ['index','low','high','max']];n,c=case['length'],case['capacity']
        expected=[i<n,i<n and i<1<<63,lo<=hi<=c,lo<=hi<=c and lo<1<<63 and hi<1<<63,lo<=hi<=m<=c,lo<=hi<=m<=c and all(x<1<<63 for x in [lo,hi,m])]
        assert [row[k]for k in ['index_unsigned','index_signed','two_unsigned','two_signed','three_unsigned','three_signed']]==expected
    alias_binary=Path('/private/tmp/mpk-go-slice-descriptor-alias-oracle');execute('alias-build',[prior['compiler_binary']['path'],'build','-o',str(alias_binary),str(alias_path)]);record['alias_binary']=seal(alias_binary)
    observed=json.loads(execute('aliases',[str(alias_binary)]).stdout)
    assert observed=={'AfterChild':[1,9,3,4],'AfterParent':[5,6,7,8],'AfterTail':[5,6,7,17],'CopyCap':4,'EmptyLen':0,'EmptyCap':4,'NilSlice':True,'NonNil':False,'NilReslice':True,'NonNilReslice':False}
    verify_sdk();assert seal(binary)==record['oracle_binary']and seal(alias_binary)==record['alias_binary']
    record.update(status='passed',whole_input_cases=len(cases),actual_bounds_results=len(cases)*6,nil_and_non_nil_descriptors=len(descriptors),shared_view_observations=10,finished_at_utc=datetime.datetime.now(datetime.timezone.utc).isoformat());save()
    print(json.dumps({k:record[k]for k in ['status','whole_input_cases','actual_bounds_results','nil_and_non_nil_descriptors','shared_view_observations']}))
except BaseException:
    record['status']='failed';save();raise
