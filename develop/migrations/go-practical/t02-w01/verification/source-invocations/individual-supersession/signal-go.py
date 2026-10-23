"""Finish owned Go supersession after observing macOS executable capitalization."""
from pathlib import Path
import datetime,hashlib,json,os,signal,subprocess
ROOT=Path(__file__).resolve().parent.parent;OUT=Path(__file__).resolve().parent
proof=ROOT/'aggregate-paired/result.json';r=json.loads(proof.read_bytes());assert r['status']=='passed' and r['complete_original_call_interfaces']==32 and r['complete_same_byte_union_streams']==2
status=json.loads((ROOT/'paired-followup/result.json').read_bytes());assert status['status']=='running'
def seal(p):
 b=Path(p).read_bytes();return {'path':str(p),'bytes':len(b),'sha256':hashlib.sha256(b).hexdigest()}
processes=[]
for line in subprocess.check_output(['ps','-axo','pid,ppid,command'],text=True).splitlines()[1:]:
 parts=line.strip().split(None,2)
 if len(parts)==3:processes.append({'pid':int(parts[0]),'ppid':int(parts[1]),'command':parts[2]})
parents=[p for p in processes if Path(p['command'].split()[0]).name.casefold().startswith('python') and p['command'].split()[-1].endswith('source-invocations/run-paired-followup.py')];assert len(parents)==1
parent=parents[0];children=[p for p in processes if p['ppid']==parent['pid']];assert len(children)==1
target=children[0];assert target['command'].startswith('/private/tmp/mpk-go-practical-t02-core-interfaces/checker-ref verify '+str(ROOT/'rust-entries/certificates')+'/')
record={'status':'about_to_signal','complete_replacement_obligations_checked_before_supersession':True,'replacement_proof':seal(proof),'actual_driver':seal(Path(__file__)),'original_parent':parent,'owned_child':target,'signal':'SIGINT','reason':'macOS reports executable basename Python, so the lowercase-only owner filter in the original guarded helper missed the waiting Go owner. Case-insensitive owner inspection identifies the exact original parent and its sole full-certificate checker child. The complete replacement proofs already passed; no verification restart or production/test source edit.','at_utc':datetime.datetime.now(datetime.timezone.utc).isoformat()}
path=OUT/'supplemental-go-signal.json';path.write_text(json.dumps(record,indent=2)+'\n');os.kill(target['pid'],signal.SIGINT);record['status']='signal_sent';path.write_text(json.dumps(record,indent=2)+'\n');print(json.dumps({'status':'owned_Go_child_signalled','pid':target['pid'],'replacement_proofs_already_passed':True}))
