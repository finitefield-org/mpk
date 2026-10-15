import hashlib,json,os,pathlib,subprocess,time
b=pathlib.Path(__file__).parent;r=pathlib.Path('/private/tmp/mpk-w09-refreeze-metadata-integration');out=b/'control-execution-dual-checkers';out.mkdir(exist_ok=False);original=json.loads((b/'ordinary-checker-plan-1-final.json').read_bytes());rows=json.loads((b/'control-execution-changed-certificates.json').read_bytes())['programs'];h=lambda p:hashlib.sha256(p.read_bytes()).hexdigest();cases=[]
for row in rows:
 if any(not o['pointer'].endswith('/parsers') and not o['certificate_bytes_preserved'] for o in row['occurrences']):
  p=b/'ordinary-regeneration-10/goldens/control-predicates/with-execution'/(row['label']+'.hex');assert h(p)==row['new_certificate_raw_sha256'];cases.append(dict(label=row['label'],path=str(p),raw_sha256=h(p)))
assert len(cases)==12;plan={'cases':cases,'binaries':original['binaries'],'source_hashes':original['source_hashes'],'selection_reason':'Run both standalone source-free checkers on the 12 actually changed complete control-predicate definition byte sets. Require accepted verdicts, matching certificate/export hashes and zero axioms; exact name-only graph comparison is a separate check. Ordinary helper acceptance does not discharge any of the 987 application proof IDs.'};(out/'plan.json').write_text(json.dumps(plan,sort_keys=True,indent=2)+'\n');state={'status':'running','stages':[],'selection_reason':plan['selection_reason'],'binaries':plan['binaries'],'original_application_proof_ids_pending':987,'full_t01_gate':'deferred_to_final_W10','full_t06_gate':'deferred_to_W12','supervisor_pid':os.getpid()}
def save():(out/'status.json').write_text(json.dumps(state,sort_keys=True,indent=2)+'\n')
def verify():
 for p,sha in plan['source_hashes'].items():assert h(r/p)==sha,p
 for name,row in plan['binaries'].items():assert h(pathlib.Path(row['path']))==row['sha256'],name
save()
try:
 verify()
 for case in cases:
  name=case['label'];source=pathlib.Path(case['path']);assert h(source)==case['raw_sha256'];data=bytes.fromhex(source.read_text());input_path=out/(name+'.mpcert');input_path.write_bytes(data);matches=[]
  for kind,binary in plan['binaries'].items():
   label=name+'-'+kind;state['stage']=label;start=time.monotonic()
   with (out/(label+'.json')).open('wb') as stdout,(out/(label+'.stderr.txt')).open('wb') as stderr:
    p=subprocess.Popen([binary['path'],'check' if kind=='rust' else 'verify',str(input_path)],stdout=stdout,stderr=stderr);state['process_pid']=p.pid;save();code=p.wait()
   result=json.loads((out/(label+'.json')).read_bytes());row={'stage':label,'case':name,'backend':kind,'exit_code':code,'verdict':result['verdict'],'elapsed_seconds':time.monotonic()-start,'input_sha256':h(input_path),'report_sha256':h(out/(label+'.json')),'stderr_sha256':h(out/(label+'.stderr.txt'))};state['stages'].append(row);save();assert code==0 and result['verdict']=='accepted',row
   assert result['hashes']['certificate']==hashlib.sha256(b'MPK-MODULE-CERT-0.1\0'+data).hexdigest()
   if kind=='rust':assert all(x==0 for x in result['axiom_report']['summary'].values())
   assert result.get('axiom_count',0)==0;matches.append({k:result[k] for k in ['module','hashes']});verify()
  assert matches[0]==matches[1],name;print(name+': both accepted with matching hashes and zero axioms',flush=True)
 state['status']='passed_all_12_changed_definition_certificates_both_checkers_zero_axioms'
except BaseException as e:state['status']='failed';state['error']=repr(e);raise
finally:save()
