import hashlib,json,os,pathlib,subprocess,time
b=pathlib.Path(__file__).parent;r=pathlib.Path('/private/tmp/mpk-w09-refreeze-metadata-integration');source=json.loads((b/'control-execution-changed-certificates.json').read_bytes());out=b/'control-execution-name-graphs';out.mkdir(exist_ok=False);h=lambda p:hashlib.sha256(p.read_bytes()).hexdigest();mapping=b/'context-name-rebindings-7.json';binary=b/'compare-certificate-context-names-2';cases=[]
for row in source['programs']:
 occ=[x for x in row['occurrences'] if not x['pointer'].endswith('/parsers')]
 changed=[x for x in occ if not x['certificate_bytes_preserved']]
 if not changed:continue
 oldfiles={p for x in changed for p in x['original_certificate_files']};assert oldfiles,row['label'];old=r/sorted(oldfiles)[0];new=b/'ordinary-regeneration-10/goldens/control-predicates/with-execution'/(row['label']+'.hex');assert len({h(r/p) for p in oldfiles})==1
 cases.append(dict(label=row['label'],schema=row['schema'],original_path=str(old),generated_path=str(new),original_raw_sha256=h(old),generated_raw_sha256=h(new)))
assert len(cases)==12
paths={mapping,binary,b/'compare-certificate-context-names-2.rs',b/'control-execution-changed-certificates.json'}|{pathlib.Path(x[k]) for x in cases for k in ['original_path','generated_path']};manifest={str(p):h(p) for p in sorted(paths)};(out/'manifest.json').write_text(json.dumps(manifest,sort_keys=True,indent=2)+'\n');state={'status':'running','stages':[],'selection_reason':'Compare complete typed term/declaration/export/name/level graphs for the twelve changed control-predicate certificates after independently recomputing all 103 old/new sequent names from their complete ordered source records. Source obligations, assumptions, goals and node/argument order are retained exactly; attachment identities alone are independently rebound. No original application proof ID is discharged','original_application_proof_ids_pending':987,'full_t01_gate':'deferred_to_final_W10','full_t06_gate':'deferred_to_W12','supervisor_pid':os.getpid()}
def save():(out/'status.json').write_text(json.dumps(state,sort_keys=True,indent=2)+'\n')
save()
try:
 for case in cases:
  label=case['label'];state['stage']=label;start=time.monotonic()
  with (out/(label+'.json')).open('wb') as stdout,(out/(label+'.stderr.txt')).open('wb') as stderr:
   p=subprocess.Popen([str(binary),str(mapping),case['original_path'],case['generated_path']],stdout=stdout,stderr=stderr);state['process_pid']=p.pid;save();code=p.wait()
  report=json.loads((out/(label+'.json')).read_bytes());row={**case,'exit_code':code,'elapsed_seconds':time.monotonic()-start,'report_sha256':h(out/(label+'.json')),'stderr_sha256':h(out/(label+'.stderr.txt')),'graph_sha256':report['new_graph_sha256']};state['stages'].append(row);save();assert code==0 and report['status']=='passed_exact_term_declaration_graph_after_actual_context_name_rebinding',label
  print(label+': graph preserved',flush=True)
 assert all(h(pathlib.Path(p))==sha for p,sha in manifest.items());state['status']='passed_all_12_complete_definition_graphs_after_actual_context_name_rebinding'
except BaseException as e:state['status']='failed';state['error']=repr(e);raise
finally:save()
