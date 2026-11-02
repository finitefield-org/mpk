"""Run the unchanged real kernel on a complete, independently compared union."""
from pathlib import Path
import datetime,hashlib,json,os,subprocess,sys,time
ROOT=Path(__file__).resolve().parent;engine,unit=sys.argv[1:];assert engine in ['rust','go'] and unit in ['pure','wide'];OUT=ROOT/f'aggregate-{engine}-{unit}';OUT.mkdir(exist_ok=False)
def seal(p):
 b=Path(p).read_bytes();return {'path':str(p),'bytes':len(b),'sha256':hashlib.sha256(b).hexdigest()}
union_path=ROOT/'aggregate-union/result.json';union=json.loads(union_path.read_bytes());assert union['status']=='passed' and union['whole_inputs_unchanged'];entry=next(r for r in union['units'] if r['unit']==unit);certificate=entry['certificate'];assert seal(certificate['path'])==certificate
if engine=='rust':
 build_path=ROOT/'aggregate-check-tools/result.json';build=json.loads(build_path.read_bytes());assert build['status']=='passed' and build['whole_inputs_unchanged'];binary=build['binary'];library=build['exact_original_kernel_library'];assert seal(library['path'])==library
 command=[binary['path'],certificate['path']];production=[]
else:
 build_path=ROOT.parent/'core-interfaces/paired/result.json';build=json.loads(build_path.read_bytes());assert build['status']=='passed';binary=build['go_binary'];production=build['go_production_sources'];command=[binary['path'],'verify',certificate['path']]
assert seal(binary['path'])==binary
for row in production:assert seal(row['path'])==row
inputs=[Path(__file__),Path(binary['path']),build_path,union_path,ROOT/'aggregate-union/prelaunch-inputs.json',Path(certificate['path']),Path(certificate['path']).with_suffix('.sources.json'),*[Path(r['path']) for r in production]];rows=[]
for i,p in enumerate(inputs):
 target=OUT/'inputs'/f'{i:03d}.original';target.parent.mkdir(exist_ok=True);target.write_bytes(p.read_bytes());target.chmod(p.stat().st_mode&0o777);rows.append({**seal(p),'mode':p.stat().st_mode&0o777,'complete_prelaunch_snapshot':seal(target)})
receipt=OUT/'prelaunch-inputs.json';receipt.write_text(json.dumps(rows,indent=2)+'\n');(OUT/'driver.py').write_bytes(Path(__file__).read_bytes());record={'status':'running','command':command,'engine':engine,'unit':unit,'actual_driver_prelaunch':seal(OUT/'driver.py'),'complete_prelaunch_inputs':seal(receipt),'binary':binary,'actual_unchanged_kernel_build_evidence':seal(build_path),'whole_original_named_union':entry,'certificate':certificate,'started_at_utc':datetime.datetime.now(datetime.timezone.utc).isoformat()}
def save():(OUT/'result.json').write_text(json.dumps(record,indent=2)+'\n')
save();start=time.monotonic();env=os.environ.copy()
for name in ['RUST_MIN_STACK','MPK_CORE_MAX_STEPS','MPK_CORE_MAX_SECONDS']:env.pop(name,None)
try:
 with (OUT/'stdout.log').open('wb') as out,(OUT/'stderr.log').open('wb') as err:code=subprocess.run(command,env=env,stdout=out,stderr=err,check=False).returncode
 record.update(exit_code=code,seconds=time.monotonic()-start,stdout=seal(OUT/'stdout.log'),stderr=seal(OUT/'stderr.log'));save();report=json.loads((OUT/'stdout.log').read_bytes());record['observation']=report
 assert code==0 and report['verdict']=='accepted' and not (OUT/'stderr.log').read_bytes()
 assert report['declaration_count']==entry['complete_original_named_declarations']
 if engine=='rust':assert report['axiom_count']==0 and not report['axiom_report']['entries']
 for row in rows:assert seal(row['path'])=={k:row[k] for k in ['path','bytes','sha256']} and Path(row['path']).stat().st_mode&0o777==row['mode']
 record.update(status='passed',whole_inputs_unchanged=True,finished_at_utc=datetime.datetime.now(datetime.timezone.utc).isoformat());save();print(json.dumps({'status':'passed','engine':engine,'unit':unit,'seconds':record['seconds']}))
except BaseException:
 record['status']='failed';save();raise
