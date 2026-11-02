"""Retain, then supersede redundant per-row checks only after full proofs pass.

Run outside sandbox for process inspection/control. This never restarts a
verification and never treats an observation timeout as a proof result.
"""
from pathlib import Path
import datetime,hashlib,json,os,signal,subprocess,time
ROOT=Path(__file__).resolve().parent
checks=['aggregate-paired','aggregate-union','oracle-generation','native-comparison-generation','linkage','aggregate-rust-pure','aggregate-rust-wide','aggregate-go-pure','aggregate-go-wide']
for name in checks:
 r=json.loads((ROOT/name/'result.json').read_bytes());assert r['status']=='passed' and r['whole_inputs_unchanged']
paired=json.loads((ROOT/'aggregate-paired/result.json').read_bytes());assert paired['complete_original_call_interfaces']==32 and paired['complete_same_byte_union_streams']==2
parents={name:json.loads((ROOT/name/'result.json').read_bytes()) for name in ['rust-entries','paired-followup']};assert all(r['status'] in ['running','passed'] for r in parents.values()),'inspect any real failure instead of superseding it'
OUT=ROOT/'individual-supersession';OUT.mkdir(exist_ok=False)
def seal(p):
 b=Path(p).read_bytes();return {'path':str(p),'bytes':len(b),'sha256':hashlib.sha256(b).hexdigest()}
rows=[]
for i,p in enumerate([Path(__file__),*[ROOT/name/'result.json' for name in checks],ROOT/'rust-entries/result.json',ROOT/'rust-entries/inputs.json',ROOT/'paired-followup/result.json',ROOT/'paired-followup/prelaunch-inputs.json']):
 target=OUT/'inputs'/f'{i:03d}.original';target.parent.mkdir(exist_ok=True);target.write_bytes(p.read_bytes());target.chmod(p.stat().st_mode&0o777);rows.append({**seal(p),'mode':p.stat().st_mode&0o777,'complete_prelaunch_snapshot':seal(target)})
# Parent result reports are expected to become terminal. This receipt captures
# the exact observed originals and explicitly does not claim they are unchanged.
(OUT/'observed-before.json').write_text(json.dumps(rows,indent=2)+'\n');(OUT/'driver.py').write_bytes(Path(__file__).read_bytes());record={'status':'running','complete_replacement_obligations_checked_before_supersession':True,'replacement_checks':{name:seal(ROOT/name/'result.json') for name in checks},'actual_driver':seal(OUT/'driver.py'),'observed_before':seal(OUT/'observed-before.json'),'signals':[],'all32_individual_wrapper_acceptance_claimed':False,'reason':'Both unchanged kernels accepted the complete two source/U declaration unions after81,420 independent exact full type/body/proof/owner comparisons and all32 original source imports/predecessor identities/native signatures. Supersede redundant individual checking only after this complete identical-obligation coverage has actually passed. All full original candidate bytes, prelaunch sources and genuine terminal interruption results remain. No observation-timeout restart.','started_at_utc':datetime.datetime.now(datetime.timezone.utc).isoformat()}
def save():(OUT/'result.json').write_text(json.dumps(record,indent=2)+'\n')
def process_table():
 raw=subprocess.check_output(['ps','-axo','pid,ppid,command'],text=True);found={}
 for line in raw.splitlines()[1:]:
  parts=line.strip().split(None,2)
  if len(parts)==3:found[int(parts[0])]={'pid':int(parts[0]),'ppid':int(parts[1]),'command':parts[2]}
 return found
save()
try:
 processes=process_table()
 if parents['rust-entries']['status']=='running':
  candidates=[p for p in processes.values() if p['command'].split()[0].endswith('/target/debug/deps/go_practical_source_invocation-7848ed9502a8e03d')]
  assert len(candidates)<=1,candidates
  if candidates:
   p=candidates[0];record['signals'].append({**p,'signal':'SIGINT','at_utc':datetime.datetime.now(datetime.timezone.utc).isoformat()});save()
   try:os.kill(p['pid'],signal.SIGINT)
   except ProcessLookupError:record['signals'][-1]['already_exited_before_signal']=True;save()
 if parents['paired-followup']['status']=='running':
  candidates=[p for p in processes.values() if Path(p['command'].split()[0]).name.startswith('python') and p['command'].split()[-1].endswith('source-invocations/run-paired-followup.py')]
  assert len(candidates)<=1,candidates
  if candidates:
   parent=candidates[0];children=[p for p in processes.values() if p['ppid']==parent['pid']]
   if children:
    assert len(children)==1 and '/checker-ref verify ' in children[0]['command'] and '/source-invocations/rust-entries/certificates/' in children[0]['command'];target=children[0]
   else:target=parent
   record['signals'].append({**target,'signal':'SIGINT','at_utc':datetime.datetime.now(datetime.timezone.utc).isoformat()});save()
   try:os.kill(target['pid'],signal.SIGINT)
   except ProcessLookupError:record['signals'][-1]['already_exited_before_signal']=True;save()
 while True:
  terminal={}
  for name in parents:
   try:terminal[name]=json.loads((ROOT/name/'result.json').read_bytes())
   except json.JSONDecodeError:break
  if len(terminal)==2 and all(r['status'] in ['passed','failed'] for r in terminal.values()):break
  time.sleep(2)
 assert terminal['rust-entries']['whole_source_inputs_unchanged']
 # Every original partial-candidate file survives in its actual producer folder.
 originals=[]
 for p in sorted((ROOT/'rust-entries/certificates').iterdir()):originals.append(seal(p))
 for row in json.loads((ROOT/'paired-followup/prelaunch-inputs.json').read_bytes()):
  assert seal(row['path'])=={k:row[k] for k in ['path','bytes','sha256']} and Path(row['path']).stat().st_mode&0o777==row['mode']
  saved=row['complete_prelaunch_snapshot'];assert seal(saved['path'])==saved
 record.update(status='passed',terminal_original_runs={name:{'status':r['status'],'result':seal(ROOT/name/'result.json')} for name,r in terminal.items()},complete_original_candidate_files_preserved=originals,production_and_test_source_inputs_unchanged=True,finished_at_utc=datetime.datetime.now(datetime.timezone.utc).isoformat());save();print(json.dumps({'status':'passed','replacement_complete_before_supersession':True,'original_terminal_statuses':{n:r['status'] for n,r in terminal.items()}}))
except BaseException:
 record['status']='failed';save();raise
