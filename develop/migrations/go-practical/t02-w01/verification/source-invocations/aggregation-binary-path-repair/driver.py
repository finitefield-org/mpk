from pathlib import Path
import hashlib,json,os,subprocess,shutil,datetime
ROOT=Path('/Users/kazuyoshitoshiya/ff/mpk/develop/migrations/go-practical/t02-w01/verification/source-invocations')
def seal(p):
 b=Path(p).read_bytes();return {'path':str(p),'bytes':len(b),'sha256':hashlib.sha256(b).hexdigest(),'mode':Path(p).stat().st_mode&0o777}
old=json.loads((ROOT/'aggregation/result.json').read_bytes());new=json.loads((ROOT/'aggregation-interned/result.json').read_bytes());path=Path(new['generated_binary']['path']);assert {k:seal(path)[k] for k in ['path','bytes','sha256']}==new['generated_binary']
source=ROOT/'aggregate-all.rs';current_source=source.read_bytes();current_mode=source.stat().st_mode&0o777
rows=json.loads((ROOT/'aggregation/prelaunch-inputs.json').read_bytes());prior=next(r for r in rows if Path(r['path'])==source);original_source=Path(prior['complete_prelaunch_snapshot']['path']).read_bytes();assert hashlib.sha256(original_source).hexdigest()==prior['sha256']
retained_new=Path('/private/tmp/mpk-go-source-invocations-aggregate-interned-174-retained');retained_old=Path('/private/tmp/mpk-go-source-invocations-aggregate-initial-174-retained');assert not retained_new.exists() and not retained_old.exists()
shutil.copy2(path,retained_new);before=seal(path);output=ROOT/'aggregation-binary-path-repair';output.mkdir(exist_ok=False);record={'status':'running','reason':'The auxiliary aggregation compiler reused the same temporary output path after a real term-table-cap failure. Recover the exact original failed tool bytes using its complete original source/command and unchanged compiler/libraries; retain both tools under distinct paths. No kernel process is restarted and no production/test source is edited.','original_failed_binary_identity':old['generated_binary'],'current_binary_before':before,'current_source_before':seal(source),'retained_new':seal(retained_new),'original_compile_command':old['compile_command'],'started_at_utc':datetime.datetime.now(datetime.timezone.utc).isoformat()}
(output/'driver.py').write_bytes(Path(__file__).read_bytes());record['actual_repair_driver']=seal(output/'driver.py')
def save():(output/'result.json').write_text(json.dumps(record,indent=2)+'\n')
save();env=os.environ.copy();env['CARGO_MANIFEST_DIR']='/Users/kazuyoshitoshiya/ff/mpk/crates/mpk-vc'
try:
 source.write_bytes(original_source);source.chmod(prior['mode'])
 with (output/'stdout.log').open('wb') as out,(output/'stderr.log').open('wb') as err:code=subprocess.run(old['compile_command'],env=env,stdout=out,stderr=err,check=False).returncode
 record['recompile_exit_code']=code;record['stdout']=seal(output/'stdout.log');record['stderr']=seal(output/'stderr.log');assert code==0
 recovered=seal(path);assert {k:recovered[k] for k in ['path','bytes','sha256']}==old['generated_binary'],('not the exact original tool bytes',recovered,old['generated_binary'])
 shutil.copy2(path,retained_old);record['exact_recovered_original_retained']=seal(retained_old)
 record['status']='recovered'
except BaseException:
 record['status']='failed';raise
finally:
 source.write_bytes(current_source);source.chmod(current_mode);shutil.copy2(retained_new,path)
 record['current_source_after']=seal(source);record['current_binary_after']=seal(path);assert record['current_source_after']==record['current_source_before'] and record['current_binary_after']==before
 record['finished_at_utc']=datetime.datetime.now(datetime.timezone.utc).isoformat()
 if record['status']=='recovered':record['status']='passed'
 save()
print(json.dumps({'status':record['status'],'exact_original_tool_recovered':True,'current_source_and_tool_restored':True}))
