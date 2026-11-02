"""Zero length does not bypass Go element-type comparability."""
from pathlib import Path
import datetime,hashlib,json,os,subprocess,time
OUT=Path(__file__).resolve().parent
prior_path=OUT/'native-go/result.json';prior=json.loads(prior_path.read_bytes());assert prior['status']=='passed'
def seal(p):
 raw=Path(p).read_bytes();return {'path':str(p),'sha256':hashlib.sha256(raw).hexdigest(),'bytes':len(raw)}
assert seal(prior['compiler_binary']['path'])==prior['compiler_binary'] and seal(prior['toolchain_archive']['path'])==prior['toolchain_archive']
inputs=json.loads(Path(prior['whole_original_sdk_inputs']['path']).read_bytes());assert all(seal(r['path'])==r for r in inputs)
run=OUT/'native-comparability';run.mkdir(exist_ok=False);(run/'observe-go-comparability.py').write_bytes(Path(__file__).read_bytes())
rows={
 'zero-slice-elements':'package bad\nfunc bad(a,b [0][]byte)bool{return a==b}\n',
 'zero-map-elements':'package bad\nfunc bad(a,b [0]map[int]int)bool{return a==b}\n',
 'zero-function-elements':'package bad\nfunc bad(a,b [0]func())bool{return a==b}\n',
 'struct-containing-slice':'package bad\ntype Item struct { Values []int }\nfunc bad(a,b [1]Item)bool{return a==b}\n',
}
for name,raw in rows.items():(run/(name+'.go')).write_text(raw)
record={'status':'running','started_at_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'actual_driver_prelaunch':seal(run/'observe-go-comparability.py'),'pinned_sdk_receipt':seal(prior_path),'compiler_binary':prior['compiler_binary'],'whole_original_sdk_inputs':prior['whole_original_sdk_inputs'],'toolchain_archive':prior['toolchain_archive'],'sources':[seal(run/(n+'.go'))for n in rows],'environment':prior['environment'],'cases':[],'ordinary_source_or_registered_profile_proof':False}
def save():(run/'result.json').write_text(json.dumps(record,indent=2)+'\n')
env=os.environ.copy();env.update(record['environment']);save()
try:
 for name in rows:
  command=[record['compiler_binary']['path'],'tool','compile','-o',str(Path('/private/tmp')/('mpk-array-compare-'+name+'.o')),str(run/(name+'.go'))];start=time.monotonic()
  p=subprocess.run(command,env=env,capture_output=True,check=False);out=run/(name+'.stdout.log');err=run/(name+'.stderr.log');out.write_bytes(p.stdout);err.write_bytes(p.stderr)
  record['cases'].append({'name':name,'command':command,'exit_code':p.returncode,'seconds':time.monotonic()-start,'source':seal(run/(name+'.go')),'stdout':seal(out),'stderr':seal(err)});save()
  assert p.returncode!=0 and b'cannot be compared'in p.stdout
 assert all(seal(r['path'])==r for r in inputs)
 record.update(status='passed',typed_original_source_rejections=4,finished_at_utc=datetime.datetime.now(datetime.timezone.utc).isoformat());save();print(json.dumps({'status':'passed','typed_original_source_rejections':4}))
except BaseException:
 record['status']='failed';save();raise
