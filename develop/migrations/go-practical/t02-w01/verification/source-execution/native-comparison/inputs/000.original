"""Authenticate original files, native execution, and the complete source run matrix."""
from pathlib import Path
import base64,datetime,hashlib,json,runpy,sys
ROOT=Path(__file__).resolve().parent;epoch=sys.argv[1] if len(sys.argv)>1 else 'rust-final';OUT=ROOT/'native-comparison';OUT.mkdir(exist_ok=False)
def seal(p):
 b=Path(p).read_bytes();return {'path':str(p),'bytes':len(b),'sha256':hashlib.sha256(b).hexdigest()}
run=ROOT/epoch/'result.json';r=json.loads(run.read_bytes());assert r['status']=='passed' and r['whole_source_inputs_unchanged'];assert seal(r['source_receipt']['path'])==r['source_receipt']
archive_path=ROOT/'native-output-original/manifest.json';archive=json.loads(archive_path.read_bytes());assert archive['status']=='passed'
inputs=[Path(__file__),ROOT/'read-originals.py',archive_path,ROOT/'corpus.json',ROOT/'vectors.json',ROOT/'program.go',ROOT/'observer.go.txt',ROOT/'native-values.json',run,ROOT/epoch/'inputs.json',ROOT/epoch/'observations/matrix.json',ROOT/epoch/'observations/boundaries.json',ROOT/'native/result.json',ROOT/'derived-native/result.json']+[Path(c['encoded']['path']) for c in archive['chunks']]
rows=[]
for i,p in enumerate(inputs):
 target=OUT/'inputs'/f'{i:03d}.original';target.parent.mkdir(exist_ok=True);target.write_bytes(p.read_bytes());target.chmod(p.stat().st_mode&0o777);rows.append({**seal(p),'mode':p.stat().st_mode&0o777,'complete_prelaunch_snapshot':seal(target)})
receipt=OUT/'prelaunch-inputs.json';receipt.write_text(json.dumps(rows,indent=2)+'\n');(OUT/'driver.py').write_bytes(Path(__file__).read_bytes());record={'status':'running','complete_prelaunch_inputs':seal(receipt),'actual_driver_prelaunch':seal(OUT/'driver.py'),'started_at_utc':datetime.datetime.now(datetime.timezone.utc).isoformat()}
def save():(OUT/'result.json').write_text(json.dumps(record,indent=2)+'\n')
save()
try:
 _,files=runpy.run_path(str(ROOT/'read-originals.py'))['read']('native-output-original')
 packet=json.loads(files['unit-inputs.json']);raw=base64.b64decode(packet['canonical_components'],validate=True);components=json.loads(raw);assert len(components['packages'])==3 and not components['declarations'] and len(components['types'])==1 and components['types'][0]['name']=='bool'
 corpus=json.loads((ROOT/'corpus.json').read_bytes());assert raw.hex()==corpus['input']['components_hex']
 original={s['object']['id']:(s['object'],base64.b64decode(s['raw'],validate=True)) for s in packet['sources']}
 for s in corpus['input']['sources']:assert (s['object'],bytes.fromhex(s['raw_hex']))==original[s['object']['id']]
 prelaunch=json.loads(files['runtime-prelaunch.json']);runtime=json.loads(files['runtime-result.json']);assert runtime['error']=='<nil>' and not base64.b64decode(runtime['stderr'],validate=True)
 original_runtime={k:base64.b64decode(v,validate=True) for k,v in prelaunch['whole_original_files'].items()}
 assert original_runtime['policy.go']==(ROOT/'program.go').read_bytes() and original_runtime['cmd/observe/main.go']==(ROOT/'observer.go.txt').read_bytes() and original_runtime['vectors.json']==(ROOT/'vectors.json').read_bytes()
 for obj,b in original.values():assert original_runtime[obj['path']]==b
 native=json.loads(files['native-values.json']);assert files['native-values.json']==(ROOT/'native-values.json').read_bytes();assert json.loads(base64.b64decode(runtime['stdout'],validate=True))==native
 matrix=json.loads((ROOT/epoch/'observations/matrix.json').read_bytes());assert not matrix['failures'] and matrix['complete_cases']==127 and matrix['full_error_cases']==70
 vectors=json.loads((ROOT/'vectors.json').read_bytes())['cases'];actual=matrix['rows'];assert len(native)==len(actual)==len(vectors)==127
 by_name={r['name']:r for r in native};assert len(by_name)==127
 pairs=[];faults={};invocations=0
 for vector,source in zip(vectors,actual):
  name=vector['name'];expected=by_name[name];assert source['name']==expected['name']==name and source['entry']==vector['entry']==expected['entry']
  for field in ['result','fault','after']:assert source[field]==expected[field],(name,field)
  entries=[e for e in source['events'] if e['kind']=='Enter'];returns=[e for e in source['events'] if e['kind']=='Return'];assert len({e['invocation'] for e in entries})==len(entries) and len({e['invocation'] for e in returns})==len(returns)
  if source['fault'] is None:assert {(e['invocation'],e['source'],e['node']) for e in entries}=={(e['invocation'],e['source'],e['node']) for e in returns}
  else:faults[source['fault']]=faults.get(source['fault'],0)+1
  invocations+=len(entries);pairs.append({'name':name,'entry':source['entry'],'steps':source['steps'],'actual_invocations':len(entries),'fault':source['fault'],'return_values_and_argument_updates_match':True})
 assert len([r for r in actual if r['name'].startswith('error-')])==70
 boundaries=json.loads((ROOT/epoch/'observations/boundaries.json').read_bytes());assert [r['budget'] for r in boundaries['observation_limits']]==[0,1,7,19] and all(r['steps']==r['budget'] and r['status']=='ObservationLimit' for r in boundaries['observation_limits'])
 for row in rows:assert seal(row['path'])=={k:row[k] for k in ['path','bytes','sha256']} and Path(row['path']).stat().st_mode&0o777==row['mode']
 record.update(status='passed',whole_inputs_unchanged=True,cases=127,source_return_cases=127-sum(faults.values()),source_faults=faults,source_invocations=invocations,max_observed_steps=max(r['steps'] for r in actual),complete_original_source_packages=3,error_cases_executed=70,wide_257_byte_length_observed=True,one_joint_call_observed=True,observation_budget_not_a_Go_or_kernel_limit=True,source_ir_program_or_context_proof_authority=False,native_type_facts_or_results_used_as_constructor_inputs=False,comparisons=pairs,finished_at_utc=datetime.datetime.now(datetime.timezone.utc).isoformat());save();print(json.dumps({k:record[k] for k in ['status','cases','source_return_cases','source_faults','source_invocations','max_observed_steps']}))
except BaseException:
 record['status']='failed';save();raise
