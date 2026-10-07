import hashlib,json,os,pathlib,subprocess,time
b=pathlib.Path(__file__).parent;r=pathlib.Path('/private/tmp/mpk-w09-bool-cases-registration-order');out=b/'dual-checkers';out.mkdir(exist_ok=False);h=lambda p:hashlib.sha256(p.read_bytes()).hexdigest();manifest=json.loads((b/'source-manifest.json').read_bytes())
assert json.loads((b/'local-4/status.json').read_bytes())['status']=='passed_selected_tests_lint_format_build'
assert json.loads((b/'go-local-4/status.json').read_bytes())['status']=='passed_selected_go_tests_build'
bins={'rust':pathlib.Path('/private/tmp/mpk-w09-bool-cases-registration-order-target/debug/mpk'),'go':b/'go-local-4/go-checker'}
good={'right-identity','constructor-false','constructor-true','open-motive','conjunction-left','conjunction-right','legacy-prior-constructor-value-levels','legacy-prior-theorem-proof-levels','unused-constructor-levels','prior-other-family-levels'}
cases=[dict(label=p.stem,path=str(p),good=p.stem in good,fixture_hex=True) for p in sorted((r/'fixtures/core-bool-cases').glob('*.hex'))]
predecessors=json.loads((r/'develop/migrations/csharp-03/probes/boolean-proof-elimination-logs/new-rule-linux/control/expected.json').read_bytes())
for name in ['predecessor-std-bool','predecessor-zero-axiom','predecessor-one-theorem']:
 p=r/'develop/migrations/csharp-03/probes/boolean-proof-elimination-logs/producer/base.mpcert' if name=='predecessor-std-bool' else r/'fixtures/cert-basic'/(name.removeprefix('predecessor-')+'.hex')
 cases.append(dict(label=name,path=str(p),good=True,fixture_hex=name!='predecessor-std-bool',previous_expected=predecessors[name]))
assert len(cases)==34
state={'status':'running','stages':[],'supervisor_pid':os.getpid(),'selection_reason':manifest['selection_reason'],'binaries':{k:{'path':str(p),'sha256':h(p)} for k,p in bins.items()},'original_application_proof_ids_pending':987,'full_t01_gate':'deferred_to_final_W10'}
def save():(out/'status.json').write_text(json.dumps(state,sort_keys=True,indent=2)+'\n')
def verify():
 for group in ['source_hashes','fixture_hashes','document_hashes']:
  for path,sha in manifest[group].items():assert h(r/path)==sha,path
 for k,p in bins.items():assert h(p)==state['binaries'][k]['sha256'],k
save()
try:
 verify()
 for row in cases:
  label=row['label'];p=pathlib.Path(row['path']);data=bytes.fromhex(p.read_text()) if row['fixture_hex'] else p.read_bytes();inp=out/(label+'.mpcert');inp.write_bytes(data);pairs=[]
  for kind,exe in bins.items():
   name=label+'-'+kind;state['stage']=name;cmd=[str(exe),'check' if kind=='rust' else 'verify',str(inp)];start=time.monotonic()
   with (out/(name+'.json')).open('wb') as stdout,(out/(name+'.stderr.txt')).open('wb') as stderr:
    process=subprocess.Popen(cmd,stdout=stdout,stderr=stderr);state['process_pid']=process.pid;save();code=process.wait()
   rep=json.loads((out/(name+'.json')).read_bytes());state['stages'].append(dict(stage=name,command=cmd,exit_code=code,verdict=rep['verdict'],input_sha256=h(inp),report_sha256=h(out/(name+'.json')),stderr_sha256=h(out/(name+'.stderr.txt')),elapsed_seconds=time.monotonic()-start));save();assert code==(0 if row['good'] else 1) and rep['verdict']==('accepted' if row['good'] else 'rejected'),(name,rep)
   assert rep['hashes']['certificate']==hashlib.sha256(b'MPK-MODULE-CERT-0.1\0'+data).hexdigest()
   if row['good']:
    if 'previous_expected' in row:
     prior=row['previous_expected']['reports'][kind]
     for key in ['module','hashes','declaration_count','axiom_report','axiom_count']:
      if key in prior:assert rep[key]==prior[key],(name,key)
    else:
     assert rep.get('axiom_count',0)==0
     if kind=='rust':assert all(v==0 for v in rep['axiom_report']['summary'].values())
    pairs.append({key:rep[key] for key in ['module','hashes','declaration_count']})
   else:assert rep['error_code' if kind=='rust' else 'error_kind']==('KERNEL_CORE_CHECK' if kind=='rust' else 'core_check'),(name,rep)
  if row['good']:assert pairs[0]==pairs[1],label
  print(label+': '+('accepted both' if row['good'] else 'rejected both'),flush=True)
 verify();state['status']='passed_31_shared_fixtures_three_predecessors_both_checkers';state['cases']=34;state['checker_stages']=68
except BaseException as error:state['status']='failed';state['error']=repr(error);raise
finally:save()
