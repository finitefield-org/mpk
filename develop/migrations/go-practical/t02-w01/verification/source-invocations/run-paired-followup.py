"""Check each remaining complete stream once its actual Rust report is ready.

No timeout, narrowed source/U, or producer restart. Previously launched full
streams are excluded by their immutable prelaunch receipts, then reused by the
final complete paired matrix after every original execution is terminal.
"""
from pathlib import Path
import datetime,hashlib,json,subprocess,time
ROOT=Path(__file__).resolve().parent
OUT=ROOT/'paired-followup';OUT.mkdir(exist_ok=False)
def seal(p):
 b=Path(p).read_bytes();return {'path':str(p),'bytes':len(b),'sha256':hashlib.sha256(b).hexdigest()}
def read_json(p):
 try:return json.loads(Path(p).read_bytes())
 except (FileNotFoundError,json.JSONDecodeError):return None
prior_path=ROOT.parent/'core-interfaces/paired/result.json';prior=json.loads(prior_path.read_bytes());binary=prior['go_binary'];assert seal(binary['path'])==binary
for row in prior['go_production_sources']:assert seal(row['path'])==row
producer=ROOT/'rust-entries';source_receipt=producer/'inputs.json'
excluded={};exclusion_receipts=[]
for group in ['paired-pure','go-wide-probe']:
 p=ROOT/group/'prelaunch-inputs.json';rows=json.loads(p.read_bytes());exclusion_receipts.append(p)
 for row in rows:
  original=Path(row['path'])
  if original.suffix!='.mpcert':continue
  assert seal(original)=={k:row[k] for k in ['path','bytes','sha256']}
  if original.stem in excluded:assert excluded[original.stem]['certificate']=={k:row[k] for k in ['path','bytes','sha256']}
  else:excluded[original.stem]={'certificate':{k:row[k] for k in ['path','bytes','sha256']},'already_launched_prelaunch_receipts':[]}
  excluded[original.stem]['already_launched_prelaunch_receipts'].append(seal(p))
driver=OUT/'driver.py';driver.write_bytes(Path(__file__).read_bytes())
record={'status':'running','started_at_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'actual_driver_prelaunch':seal(driver),'go_binary':binary,'go_production_sources':prior['go_production_sources'],'excluded_already_launched_complete_streams':excluded,'cases':[],'complete_final_matrix':False,'scope':'Each remaining whole original source-specific invocation stream is checked by the unchanged Go core after its actual Rust report becomes available. Full source/U and all results remain; already launched streams are selected only by exact immutable prelaunch byte identities. This continuation batch alone is not the complete final matrix and grants no source execution, callee, effect/frame or context/program proof authority.'}
receipt=OUT/'prelaunch-inputs.json';records=[];captured={}
def save():
 receipt.write_text(json.dumps(records,indent=2)+'\n');record['complete_prelaunch_inputs']=seal(receipt)
 (OUT/'result.json').write_text(json.dumps(record,indent=2)+'\n')
def capture(p):
 p=Path(p)
 if str(p) in captured:assert seal(p)==captured[str(p)];return
 raw=p.read_bytes();row={**seal(p),'mode':p.stat().st_mode&0o777}
 target=OUT/'inputs'/f'{len(records):03d}.original';target.parent.mkdir(exist_ok=True);target.write_bytes(raw);target.chmod(row['mode'])
 row['complete_prelaunch_snapshot']=seal(target);records.append(row);captured[str(p)]={k:row[k] for k in ['path','bytes','sha256']}
for p in [Path(__file__),driver,prior_path,Path(binary['path']),source_receipt,*exclusion_receipts,*[Path(r['path']) for r in prior['go_production_sources']]]:capture(p)
save()
try:
 done=set(excluded)
 while True:
  parent=read_json(producer/'result.json')
  if parent is None:time.sleep(2);continue
  assert parent['source_receipt']==seal(source_receipt)
  pending=[]
  for p in sorted((producer/'certificates').glob('*.mpcert')):
   if p.stem in done:continue
   rust=read_json(p.with_suffix('.rust.json'))
   if rust is not None:pending.append((p,rust))
  if not pending:
   if parent['status']!='running':
    assert parent['status']=='passed' and parent['whole_source_inputs_unchanged']
    terminal=producer/'result.json';capture(terminal)
    assert done=={p.stem for p in (producer/'certificates').glob('*.mpcert')}
    break
   time.sleep(2);continue
  for path,rust in pending:
   report_path=path.with_suffix('.rust.json')
   inputs=[path,report_path]
   if path.with_suffix('.sources.json').exists():inputs.append(path.with_suffix('.sources.json'))
   for p in inputs:capture(p)
   command=[binary['path'],'verify',str(path)]
   save();case_receipt=OUT/'case-prelaunch'/f'{path.stem}.json';case_receipt.parent.mkdir(exist_ok=True);case_receipt.write_bytes(receipt.read_bytes())
   record['active_complete_stream']={'certificate':seal(path),'rust_report':seal(report_path),'command':command,'prelaunch_inputs':seal(case_receipt)}
   save();start=time.monotonic()
   result=subprocess.run(command,stdout=subprocess.PIPE,stderr=subprocess.PIPE,check=False)
   stdout=OUT/f'{path.stem}.stdout.log';stderr=OUT/f'{path.stem}.stderr.log';stdout.write_bytes(result.stdout);stderr.write_bytes(result.stderr)
   go=json.loads(result.stdout)
   case={'name':path.stem,'certificate':seal(path),'rust_report':seal(report_path),'command':command,'exit_code':result.returncode,'seconds':time.monotonic()-start,'stdout':seal(stdout),'stderr':seal(stderr),'rust_observation':rust,'go_observation':go,'complete_case_prelaunch_inputs':seal(case_receipt),'reused_exact_terminal_execution':None}
   record['cases'].append(case);save()
   assert not result.stderr
   if rust['verdict']=='accepted':
    assert result.returncode==0 and go['verdict']=='accepted' and rust['axiom_count']==0 and not rust['axiom_report']['entries']
    assert rust['hashes']==go['hashes'] and rust['declaration_count']==go['declaration_count']
   else:
    assert result.returncode!=0 and rust['verdict']==go['verdict']=='rejected' and rust['error_code']=='KERNEL_CORE_CHECK' and go['error_kind']=='core_check'
    assert json.loads(rust['error_detail'])['code']=='CORE_TYPE_MISMATCH'
   for p in inputs:assert seal(p)==captured[str(p)]
   done.add(path.stem);record.pop('active_complete_stream',None);save()
   print(json.dumps({'name':path.stem,'verdict':go['verdict'],'seconds':case['seconds']}),flush=True)
 for row in records:
  assert seal(row['path'])=={k:row[k] for k in ['path','bytes','sha256']} and Path(row['path']).stat().st_mode&0o777==row['mode']
  assert seal(row['complete_prelaunch_snapshot']['path'])==row['complete_prelaunch_snapshot']
 record.update(status='passed',whole_inputs_unchanged=True,same_byte_streams=len(record['cases']),zero_axiom_acceptances=sum(c['rust_observation']['verdict']=='accepted' for c in record['cases']),genuine_type_rejections=sum(c['rust_observation']['verdict']=='rejected' for c in record['cases']),finished_at_utc=datetime.datetime.now(datetime.timezone.utc).isoformat());save()
except BaseException:
 record['status']='failed';save();raise
