import hashlib,json,os,pathlib,subprocess,time
b=pathlib.Path(__file__).parent;r=pathlib.Path('/private/tmp/mpk-w09-refreeze-metadata-integration');compiled=pathlib.Path('/private/tmp/mpk-w09-context-application-integration');out=b/'control-pattern-consumer-regeneration';out.mkdir(exist_ok=False);h=lambda p:hashlib.sha256(p.read_bytes()).hexdigest();binary=b/'regenerate-control-pattern-consumers';inputs=['develop/migrations/csharp-03/control-vc/loop-requests.json','develop/migrations/csharp-03/control-emission/loop-responses.json','develop/migrations/csharp-03/control-vc/measure-requests.json','develop/migrations/csharp-03/control-vc/measure-responses.json','develop/migrations/csharp-03/control-emission/source-cases.json'];paths=[binary,b/'regenerate-control-pattern-consumers.rs',b/'control-pattern-consumer-compile-command.json',b/'run-control-pattern-consumer-regeneration.py',r/'crates/mpk-cli/tests/support/csharp_practical_data_context.rs']+[r/p for p in inputs];paths+=list(r.glob('crates/mpk-vc/src/**/*.rs'))+list(r.glob('crates/mpk-cert/src/**/*.rs'))+list((r/'develop/migrations/csharp-03/ordinary-foundation/control-predicates/with-pattern-captures').glob('*.json'))
for arg in json.loads((b/'control-pattern-consumer-compile-command.json').read_text()):
 if '.rlib' in arg:paths.append(pathlib.Path(arg.split('=',1)[1]))
production=[]
for p in sorted(r.glob('crates/mpk-vc/src/**/*.rs')):
 assert p.read_bytes()==(compiled/p.relative_to(r)).read_bytes();production.append(str(p.relative_to(r)))
manifest={'hashes':{str(p):h(p) for p in sorted(set(paths))},'compiled_library_production_sources_equal':production,'selection_reason':'Regenerate and strictly import only the three remaining current pattern consumer producers, from the same eighteen original source contexts used by the completed capture owner. Verify input/source/library/binary hashes before and after, canonical certificate structure and zero axioms. Subsequent complete metadata lineage, graph checks, both checkers and original semantic owner tests remain required. Historical 61-condition checkpoints are retained separately; no application proof ID is discharged.'};(out/'manifest.json').write_text(json.dumps(manifest,sort_keys=True,indent=2)+'\n');state={'status':'running','selection_reason':manifest['selection_reason'],'application_proof_ids_pending':987,'full_t01_gate':'deferred_to_final_W10','full_t06_gate':'deferred_to_W12'}
def save():(out/'status.json').write_text(json.dumps(state,sort_keys=True,indent=2)+'\n')
save();env=os.environ.copy()
for k in list(env):
 if k.startswith(('MPK_W09_','MPK_T06_')) or k=='MPK_W14_REQUEST_OUTPUT':env.pop(k)
start=time.monotonic();command=[str(binary),str(r),str(out/'generated')]
try:
 with (out/'stdout.txt').open('wb') as stdout,(out/'stderr.txt').open('wb') as stderr:
  p=subprocess.Popen(command,env=env,stdout=stdout,stderr=stderr);state['process_pid']=p.pid;save();code=p.wait()
 unchanged=all(h(pathlib.Path(p))==sha for p,sha in manifest['hashes'].items());state.update(exit_code=code,source_input_binary_bytes_unchanged=unchanged,elapsed_seconds=time.monotonic()-start,command=command);assert code==0 and unchanged
 generation=json.loads((out/'generated/generation.json').read_text());assert generation['status']=='passed_actual_generation_strict_import_and_structure' and len(generation['programs'])==54
 for row in generation['programs']:
  parent=json.loads((b/'ordinary-regeneration-12/goldens/control-predicates/with-pattern-captures'/(row['id']+'.json')).read_text());assert row['source_ir_sha256']==parent['source_ir_sha256'],row
 state['generated_files']={str(p.relative_to(out/'generated')):h(p) for p in sorted((out/'generated').rglob('*')) if p.is_file()};state['status']='passed_actual_54_pattern_programs_strict_import_and_same_18_source_contexts'
except BaseException as e:state['status']='failed';state['error']=repr(e);raise
finally:save()
print(json.dumps({k:v for k,v in state.items() if k!='generated_files'}))
