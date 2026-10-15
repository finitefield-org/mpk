import hashlib,json,os,pathlib,subprocess,time
b=pathlib.Path(__file__).parent;r=pathlib.Path('/private/tmp/mpk-w09-refreeze-metadata-integration');source=json.loads((b/'control-pattern-consumer-changed-certificates.json').read_bytes());out=b/'control-pattern-consumer-name-graphs';out.mkdir(exist_ok=False);h=lambda p:hashlib.sha256(p.read_bytes()).hexdigest();mapping=b/'context-name-rebindings-11.json';binary=b/'compare-certificate-context-names-2';cases=[{k:row[k] for k in ['label','original_path','generated_path','original_raw_sha256','generated_raw_sha256']} for row in source['cases'] if not row['certificate_bytes_preserved'] and row['same_complete_graph_pair_as_parent'] is None]
assert len(cases)==21
paths={mapping,binary,b/'compare-certificate-context-names-2.rs',b/'control-pattern-consumer-changed-certificates.json'}|{pathlib.Path(x[k]) for x in cases for k in ['original_path','generated_path']};manifest={str(p):h(p) for p in sorted(paths)};(out/'manifest.json').write_text(json.dumps(manifest,sort_keys=True,indent=2)+'\n');state={'status':'running','stages':[],'selection_reason':'Compare the twenty-one complete current pattern-consumer graphs whose original/fresh byte pairs differ from the execution owner. Thirty-three other exact original/fresh certificate pairs reuse the completed execution graph checks. Independently verify all 1473 independently computed source/scope/packed proposition and premise names using the actual producer tuples; no term, declaration, type, argument order or application obligation is omitted.','original_application_proof_ids_pending':987,'full_t01_gate':'deferred_to_final_W10','full_t06_gate':'deferred_to_W12','supervisor_pid':os.getpid()}
def save():(out/'status.json').write_text(json.dumps(state,sort_keys=True,indent=2)+'\n')
save()
try:
 for case in cases:
  label=case['label'];state['stage']=label;start=time.monotonic()
  with (out/(label+'.json')).open('wb') as stdout,(out/(label+'.stderr.txt')).open('wb') as stderr:
   p=subprocess.Popen([str(binary),str(mapping),case['original_path'],case['generated_path']],stdout=stdout,stderr=stderr);state['process_pid']=p.pid;save();code=p.wait()
  report=json.loads((out/(label+'.json')).read_bytes());row={**case,'exit_code':code,'elapsed_seconds':time.monotonic()-start,'report_sha256':h(out/(label+'.json')),'stderr_sha256':h(out/(label+'.stderr.txt')),'graph_sha256':report['new_graph_sha256']};state['stages'].append(row);save();assert code==0 and report['status']=='passed_exact_term_declaration_graph_after_actual_context_name_rebinding',label
  print(label+': graph preserved',flush=True)
 assert all(h(pathlib.Path(p))==sha for p,sha in manifest.items());state['status']='passed_all_21_distinct_consumer_definition_graphs_after_actual_context_name_rebinding'
except BaseException as e:state['status']='failed';state['error']=repr(e);raise
finally:save()
