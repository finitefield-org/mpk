"""Check the complete isolated source copy while original large inputs stay fixed."""
from pathlib import Path
import datetime,hashlib,json,os,shutil,subprocess,sys,time
REPO=Path.cwd();ROOT=Path(__file__).resolve().parent
BASE='7ead1ea45e602338b52a0d9ddf6d30db92782432'
copy=Path(json.loads((ROOT/'isolated-copy.json').read_bytes())['copy'])
epoch,mode=sys.argv[1:];out=ROOT/epoch;out.mkdir(exist_ok=False)
def git(*args):return subprocess.check_output(['/usr/bin/git',*args])
def seal(p):
 p=Path(p);raw=p.read_bytes();return {'path':str(p),'bytes':len(raw),'sha256':hashlib.sha256(raw).hexdigest()}
assert git('rev-parse','HEAD').decode().strip()==BASE
tracked={}
for row in git('ls-files','--stage','-z').split(b'\0'):
 if row:
  meta,name=row.split(b'\t',1);permissions,oid,stage=meta.decode().split();assert stage=='0';tracked[name.decode()]=(oid,int(permissions,8)&0o777)
records=[]
actuals=[p for p in copy.rglob('*') if p.is_file() and 'target' not in p.relative_to(copy).parts]
if mode=='controls':
 accepted=ROOT/'small-fixed/certificates';assert json.loads((ROOT/'small-fixed/result.json').read_bytes())['status']=='passed'
 for p in accepted.iterdir():
  target=copy/p.relative_to(REPO);target.parent.mkdir(parents=True,exist_ok=True);target.write_bytes(p.read_bytes());target.chmod(p.stat().st_mode&0o777)
 actuals += [p for p in (copy/accepted.relative_to(REPO)).iterdir() if p.is_file()]
for p in sorted(set(actuals)):
 logical=str(p.relative_to(copy));raw=p.read_bytes();oid,expected_mode=tracked.get(logical,(None,None));mode_bits=p.stat().st_mode&0o777
 if oid and hashlib.sha1(b'blob '+str(len(raw)).encode()+b'\0'+raw).hexdigest()==oid and mode_bits==expected_mode:
  row={'path':logical,'actual_isolated_path':str(p),'bytes':len(raw),'sha256':hashlib.sha256(raw).hexdigest(),'mode':mode_bits,'original_commit':BASE,'original_git_blob':oid}
 else:
  saved=out/'inputs'/logical;saved.parent.mkdir(parents=True,exist_ok=True);saved.write_bytes(raw);saved.chmod(mode_bits)
  row={'path':logical,'actual_isolated_path':str(p),'bytes':len(raw),'sha256':hashlib.sha256(raw).hexdigest(),'mode':mode_bits,'complete_changed_input':str(saved.relative_to(REPO))}
 records.append(row)
# Retain the actual runner as well as the logical original-source buffers.
saved=out/'inputs'/Path(__file__).relative_to(REPO);saved.parent.mkdir(parents=True,exist_ok=True);saved.write_bytes(Path(__file__).read_bytes())
records.append({**seal(Path(__file__).relative_to(REPO)),'actual_isolated_path':str(Path(__file__)),'mode':Path(__file__).stat().st_mode&0o777,'complete_changed_input':str(saved.relative_to(REPO))})
driver=out/'driver.py';driver.write_bytes(Path(__file__).read_bytes())
receipt=out/'inputs.json';receipt.write_text(json.dumps({'basis_commit':BASE,'isolated_copy':str(copy),'sources':records,'actual_driver_prelaunch':seal(driver)},indent=2)+'\n')
changed=json.loads((ROOT/'isolated-copy.json').read_bytes())['overlaid_sources']
commands={'format-write':['rustfmt','--edition=2021','--config','skip_children=true',*changed],'small':['cargo','test','--offline','--locked','-p','mpk-vc','--test','go_practical_source_value_relations','closed_source_relations_empty','--','--nocapture','--test-threads=1'],'controls':['cargo','test','--offline','--locked','-p','mpk-vc','--test','go_practical_source_value_relations_controls','complete_relation_zero_copy_and_source_identity','--','--nocapture','--test-threads=1']}
commands['controls'][commands['controls'].index('complete_relation_zero_copy_and_source_identity')]='complete_relation_zero_copy_box_and_source_identity_controls'
environment={'CARGO_TARGET_DIR':str(copy/'target'),'CARGO_NET_OFFLINE':'true','CARGO_PROFILE_TEST_OPT_LEVEL':'0','CARGO_PROFILE_TEST_DEBUG_ASSERTIONS':'true','CARGO_PROFILE_TEST_OVERFLOW_CHECKS':'true','CARGO_BUILD_JOBS':'1','CARGO_INCREMENTAL':'0','RUST_TEST_THREADS':'1','MPK_GO_SOURCE_VALUE_RELATIONS_OUTPUT':str(out/'certificates')}
env=os.environ.copy();env.update(environment)
for key in ['RUST_MIN_STACK','MPK_CORE_MAX_STEPS','MPK_CORE_MAX_SECONDS']:env.pop(key,None)
record={'status':'running','command':commands[mode],'working_directory':str(copy),'environment':environment,'source_receipt':seal(receipt),'actual_driver_prelaunch':seal(driver),'tools':{n:seal(shutil.which(n)) for n in ['cargo','rustc','rustfmt']},'selection_reason':'A root single-field product containing a vector also needs the raw function-carrier identity adapter. Exercise the complete nil-only plus small source rows, then only the previously setup-failed first control root. The already passed heterogeneous and large negative roots remain original observations; source snapshots/executables of the running full large matrix stay untouched. This immutable isolated source differs only in that predicate and the corrected positive fixture path. Default unoptimized test builds, debug/overflow checks and native core caps remain; no runtime/resource acceptance claim. Final worktree bytes must exactly match these inputs after the original source consumers terminate.','full_T_gate':'Deferred to final GO-PRAC-T02-W04: ./scripts/check-fast.sh','formatter_mutation_expected':mode=='format-write','started_at_utc':datetime.datetime.now(datetime.timezone.utc).isoformat()}
result=out/'result.json';result.write_text(json.dumps(record,indent=2)+'\n');start=time.monotonic()
with (out/'stdout.log').open('wb') as stdout,(out/'stderr.log').open('wb') as stderr:code=subprocess.run(commands[mode],cwd=copy,env=env,stdout=stdout,stderr=stderr,check=False).returncode
unchanged=all(seal(r['actual_isolated_path'])['sha256']==r['sha256'] and Path(r['actual_isolated_path']).stat().st_mode&0o777==r['mode'] for r in records)
record.update(status='passed' if code==0 and (unchanged or mode=='format-write') else 'failed',exit_code=code,seconds=time.monotonic()-start,whole_source_inputs_unchanged=unchanged,stdout=seal(out/'stdout.log'),stderr=seal(out/'stderr.log'),certificates=[seal(p) for p in sorted((out/'certificates').glob('*.mpcert'))],finished_at_utc=datetime.datetime.now(datetime.timezone.utc).isoformat())
result.write_text(json.dumps(record,indent=2)+'\n');print(json.dumps({k:record[k] for k in ['status','exit_code','seconds','whole_source_inputs_unchanged']}));raise SystemExit(record['status']!='passed')
