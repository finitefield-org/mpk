import copy,hashlib,importlib.util,json,os,pathlib,platform,re,subprocess,time
b=pathlib.Path(__file__).parent;r=pathlib.Path('/private/tmp/mpk-w09-bool-cases-registration-order');out=b/'registration-order-capacity-recursor-refresh';out.mkdir(exist_ok=False)
h=lambda p:hashlib.sha256(p.read_bytes()).hexdigest();canonical=lambda x:(json.dumps(x,sort_keys=True,separators=(',',':'),ensure_ascii=True)+'\n').encode();read=lambda p:json.loads(p.read_bytes())
old_duals=read(b/'registration-order-fix/dual-checkers-3/status.json');binaries=old_duals['binaries'];source=read(b/'registration-order-fix/source-manifest.json')
for group in ['source_hashes','fixture_hashes','document_hashes']:
 for p,sha in source[group].items():assert h(r/p)==sha,p
for z in binaries.values():assert h(pathlib.Path(z['path']))==z['sha256']
selection='The registration-order fix changes five source pins owned by both original W09 capacity/recursor reproducibility probes. Re-export exactly12 capacity and15 recursor certificates from their unchanged original owners; require both complete probe objects and rawcertificatehashes unchanged, then rerun48 capacity and60 recursor checker invocations with the already validated fixed binaries. Capture actual reports/timings andfresh source inventories. Keep all live source/input trees andcurrent I probe records untouched until their terminal promotions; whole gate stays finalW10.'
state={'status':'running','stages':[],'selection_reason':selection,'binaries':binaries,'application_proof_ids_pending':987,'full_t01_gate':'deferred_to_final_W10','full_t06_gate':'deferred_to_W12','supervisor_pid':os.getpid()}
def save():(out/'status.json').write_bytes(canonical(state))
def verify():
 for group in ['source_hashes','fixture_hashes','document_hashes']:
  for p,sha in source[group].items():assert h(r/p)==sha,p
 for z in binaries.values():assert h(pathlib.Path(z['path']))==z['sha256']
paths={'capacity':'develop/migrations/csharp-03/probes/checker-capacity.json','recursor':'develop/migrations/csharp-03/probes/recursor-feasibility.json'}
prior={k:read(r/p) for k,p in paths.items()};(out/'before').mkdir()
for k,p in paths.items(): (out/'before'/pathlib.Path(p).name).write_bytes((r/p).read_bytes())
source_paths=set(source['source_hashes']);source_paths.update(z['path'] for k in prior.values() for z in k['source_inventory']);source_paths.update(paths.values());source_paths.update(['develop/probes/csharp-03/run-recursor-probe.py','develop/probes/csharp-03/run-checker-capacity-probe.py'])
manifest={p:h(r/p) for p in sorted(source_paths)};(out/'source-manifest.json').write_bytes(canonical({'hashes':manifest,'selection_reason':selection}));save()
try:
 env=os.environ.copy()
 for key in list(env):
  if key.startswith(('MPK_W09_','MPK_T06_')) or key=='MPK_W14_REQUEST_OUTPUT':env.pop(key)
 env.update(CARGO_TARGET_DIR='/private/tmp/mpk-w09-bool-cases-registration-order-target',CARGO_INCREMENTAL='0',PYTHONDONTWRITEBYTECODE='1')
 capacity_dir=out/'capacity-certificates';recursor_file=out/'recursor-probes.json'
 exports=[('capacity','recursor_feasibility::csharp_03_t01_w09_checker_capacity_bytes_are_reproducible','MPK_W09_CAPACITY_EXPORT',str(capacity_dir)),('recursor','recursor_feasibility::csharp_03_t01_w09_recursor_probe_bytes_are_reproducible','MPK_W09_RECURSOR_EXPORT',str(recursor_file))]
 for label,selector,key,value in exports:
  command=['/Users/kazuyoshitoshiya/.cargo/bin/cargo','test','-p','mpk-vc','--test','csharp_practical_spec',selector,'--','--exact','--nocapture','--test-threads=1'];stage=label+'-original-export';state['stage']=stage;save();started=time.monotonic();log=out/(stage+'.log.txt')
  with log.open('wb') as stream:
   proc=subprocess.Popen(command,cwd=r,env={**env,key:value},stdout=stream,stderr=subprocess.STDOUT);state['process_pid']=proc.pid;save();code=proc.wait()
  counts=re.findall(r'test result: ok\. (\d+) passed; 0 failed;',log.read_text());row={'stage':stage,'selector':selector,'command':command,'exit_code':code,'test_counts':counts,'elapsed_seconds':time.monotonic()-started,'log_sha256':h(log)};state['stages'].append(row);save();assert code==0 and counts==['1'],row;verify()
 probes={'capacity':read(capacity_dir/'manifest.json'),'recursor':read(recursor_file)}
 for label,x in probes.items():assert x==prior[label]['probe'],label+' completeprobechanged'
 assert len(probes['capacity']['cases'])==12 and len(probes['recursor']['cases'])==15
 recursor_dir=out/'recursor-certificates';recursor_dir.mkdir()
 for case in probes['recursor']['cases']:
  p=recursor_dir/(case['id']+'.mpcert');p.write_bytes(bytes.fromhex(case['certificate_hex']));assert h(p)==case['raw_sha256']
 assert all(b'.cases' not in p.read_bytes() for d in [capacity_dir,recursor_dir] for p in d.glob('*.mpcert'))
 results={}
 for label in ['capacity','recursor']:
  runs=[];folder=capacity_dir if label=='capacity' else recursor_dir
  for repetition in [1,2]:
   observations=[]
   for case in probes[label]['cases']:
    p=folder/(case['id']+'.mpcert');assert h(p)==case['raw_sha256'];expected='accepted' if label=='capacity' else case['expected'];obs={'id':case['id']}
    for backend,key in [('rust','rust'),('go','reference')]:
     stage=f'{label}-{repetition}-{case["id"]}-{backend}';state['stage']=stage;started=time.monotonic();command=[binaries[backend]['path'],'verify',str(p)];stdout=out/(stage+'.stdout.txt');stderr=out/(stage+'.stderr.txt')
     with stdout.open('wb') as so,stderr.open('wb') as se:
      proc=subprocess.Popen(command,cwd=r,stdout=so,stderr=se);state['process_pid']=proc.pid;save();code=proc.wait()
     elapsed=time.monotonic()-started;a=stdout.read_text();z=stderr.read_text()
     if backend=='go':
      report=json.loads(a)
      if code==0 and report['verdict']=='accepted':verdict='accepted'
      elif code==1 and report['verdict']=='rejected' and report.get('error_kind')=='core_check' and re.fullmatch(r'term [0-9]+ inferred (pi|const) but expected const',report.get('error_detail','')):verdict='type_mismatch'
      else:raise AssertionError((stage,code,report))
     elif code==0 and a.startswith('ok module=') and ' axioms=0' in a:verdict='accepted'
     elif code==1 and 'type_mismatch' in z:verdict='type_mismatch'
     else:raise AssertionError((stage,code,a,z))
     row={'stage':stage,'exit_code':code,'verdict':verdict,'input_sha256':h(p),'stdout_sha256':h(stdout),'stderr_sha256':h(stderr),'elapsed_seconds':elapsed};state['stages'].append(row);save();assert verdict==expected,(stage,verdict,expected)
     ob={'result':verdict,'exit_code':code,'stdout':a,'stderr':z}
     if label=='capacity':
      ob.update(stdout_sha256=h(stdout),stderr_sha256=h(stderr),elapsed_ms=max(1,int(elapsed*1000+0.999)));assert 0<ob['elapsed_ms']<60000,(stage,ob['elapsed_ms'])
     obs[key]=ob
    observations.append(obs)
   runs.append({'repetition':repetition,'observations':observations})
  new=copy.deepcopy(prior[label]);new['runs']=runs;inventory=[]
  for row in prior[label]['source_inventory']:inventory.append({'path':row['path'],'raw_sha256':h(r/row['path'])})
  delta=[z['path'] for z,o in zip(inventory,prior[label]['source_inventory']) if z!=o];assert set(delta)=={'crates/mpk-core/src/bool_cases.rs','crates/mpk-core/src/inductive_gen.rs','crates/mpk-kernel/src/bool_cases_tests.rs','go-tools/mpk-checker-ref/bool_cases.go','go-tools/mpk-checker-ref/core_check.go'},(label,delta)
  new['source_inventory']=inventory;new['source_inventory_sha256']=hashlib.sha256(canonical(inventory)).hexdigest();new['host']={'system':platform.system(),'machine':platform.machine()}
  assert new['probe']==prior[label]['probe'];results[label]=new;print(label+' allactualcheckerobservations passed',flush=True)
 results['recursor']['capacity_measurement']['raw_sha256']=hashlib.sha256(canonical(results['capacity'])).hexdigest()
 for label in ['capacity','recursor']:
  before=copy.deepcopy(prior[label]);after=copy.deepcopy(results[label]);allowed={'runs','source_inventory','source_inventory_sha256','host'}
  if label=='recursor':allowed.add('capacity_measurement')
  assert {k:v for k,v in before.items() if k not in allowed}=={k:v for k,v in after.items() if k not in allowed},label
  if label=='recursor':assert results[label]['runs']==prior[label]['runs'],'fullrecursorobservationschanged'
  else:
   for n,o in zip(results[label]['runs'],prior[label]['runs']):
    aa=copy.deepcopy(n);zz=copy.deepcopy(o)
    for run in [aa,zz]:
     for obs in run['observations']:
      for key in ['rust','reference']:obs[key].pop('elapsed_ms')
    assert aa==zz,'completecapacityobservationschanged_excepttimings'
  (out/pathlib.Path(paths[label]).name).write_bytes(canonical(results[label]))
 verify()
 for p,sha in manifest.items():assert h(r/p)==sha,p
 state.update(status='passed_exact_original_capacity_and_recursor_probe_bytes_and108_fixed_checker_observations',checker_stages=108,original_export_owner_tests=2,capacity_invocations=48,recursor_invocations=60,capacity_probes=12,recursor_probes=15,complete_probe_objects_unchanged=True,complete_capacity_observations_unchanged_except_actual_timings=True,complete_recursor_observations_unchanged=True,current_I_probe_records_unchanged=True,source_inventory_changed_exactly_five_reviewed_paths=True,record_hashes={label:h(out/pathlib.Path(paths[label]).name) for label in paths})
except BaseException as error:state.update(status='failed',error=repr(error));raise
finally:save()
