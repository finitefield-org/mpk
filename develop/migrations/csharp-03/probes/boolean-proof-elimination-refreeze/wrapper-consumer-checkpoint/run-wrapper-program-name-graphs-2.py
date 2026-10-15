import hashlib,json,os,pathlib,subprocess,time
b=pathlib.Path(__file__).parent;r=pathlib.Path('/private/tmp/mpk-w09-refreeze-metadata-integration');source=json.loads((b/'wrapper-program-comparison-3.json').read_bytes());out=b/'wrapper-program-name-graphs-2';out.mkdir(exist_ok=False);h=lambda p:hashlib.sha256(p.read_bytes()).hexdigest();mapping=b/'context-name-rebindings-6.json';binary=b/'compare-certificate-context-names-2';cases=[]
for row in source['programs']:
 occ=[x for x in row['occurrences'] if not x['pointer'].endswith('/parsers')]
 changed=[x for x in occ if not x['certificate_bytes_preserved']]
 if not changed:continue
 oldfiles={p for x in changed for p in x['original_certificate_files']};assert oldfiles,row['label'];old=r/sorted(oldfiles)[0];new=b/'wrapper-program-regeneration/generated'/(row['label']+'.hex');assert len({h(r/p) for p in oldfiles})==1
 cases.append(dict(label=row['label'],schema=row['schema'],original_path=str(old),generated_path=str(new),original_raw_sha256=h(old),generated_raw_sha256=h(new)))
assert len(cases)==115
paths={mapping,binary,b/'compare-certificate-context-names-2.rs',b/'wrapper-program-comparison-3.json'}|{pathlib.Path(x[k]) for x in cases for k in ['original_path','generated_path']};manifest={str(p):h(p) for p in sorted(paths)};(out/'manifest.json').write_text(json.dumps(manifest,sort_keys=True,indent=2)+'\n');state={'status':'running','stages':[],'selection_reason':'Compare complete term/type/declaration/name/level/export graphs for all 115 actually changed JSON wrapper certificates after the independently derived context, attachment and field-name mappings. The 36 stale aggregate records are first linked to their already reviewed count-helper refresh, using exact old/new hashes and metadata paths. This run compares against the current pinned predecessor bytes and does not count stale count refreshes as byte retention','original_application_proof_ids_pending':987,'full_t01_gate':'deferred_to_final_W10','full_t06_gate':'deferred_to_W12','supervisor_pid':os.getpid()}
def save():(out/'status.json').write_text(json.dumps(state,sort_keys=True,indent=2)+'\n')
previous=json.loads((b/'wrapper-program-name-graphs/status.json').read_bytes())
assert previous['status']=='failed'
retained={x['label']:x for x in previous['stages'] if x['exit_code']==0}
old_map=json.loads((b/'context-name-rebindings-5.json').read_bytes());new_map=json.loads(mapping.read_bytes());added={k:v for k,v in new_map.items() if k not in old_map};assert all(new_map[k]==v for k,v in old_map.items())
save()
try:
 for case in cases:
  label=case['label'];state['stage']=label
  if label in retained:
   row=retained[label];assert all(row[k]==case[k] for k in case)
   original_raw=bytes.fromhex(pathlib.Path(case['original_path']).read_text())
   assert not any(k.encode() in original_raw for k in added),label
   import shutil
   for suffix in ['.json','.stderr.txt']:shutil.copyfile(b/'wrapper-program-name-graphs'/(label+suffix),out/(label+suffix))
   state['stages'].append(dict(row,verification='reused_pass_from_unchanged_inputs_and_no_added_mapping_key_in_original_certificate_names'))
   save();continue
  start=time.monotonic()
  with (out/(label+'.json')).open('wb') as stdout,(out/(label+'.stderr.txt')).open('wb') as stderr:
   p=subprocess.Popen([str(binary),str(mapping),case['original_path'],case['generated_path']],stdout=stdout,stderr=stderr);state['process_pid']=p.pid;save();code=p.wait()
  report=json.loads((out/(label+'.json')).read_bytes());row={**case,'exit_code':code,'elapsed_seconds':time.monotonic()-start,'report_sha256':h(out/(label+'.json')),'stderr_sha256':h(out/(label+'.stderr.txt')),'graph_sha256':report['new_graph_sha256']};state['stages'].append(row);save();assert code==0 and report['status']=='passed_exact_term_declaration_graph_after_actual_context_name_rebinding',label
  print(label+': graph preserved',flush=True)
 assert all(h(pathlib.Path(p))==sha for p,sha in manifest.items());state['status']='passed_all_115_complete_definition_graphs_after_actual_context_name_rebinding'
except BaseException as e:state['status']='failed';state['error']=repr(e);raise
finally:save()
