import hashlib,json,os,pathlib,subprocess,time
b=pathlib.Path(__file__).parent;r=pathlib.Path('/private/tmp/mpk-w09-refreeze-metadata-integration');compiled=pathlib.Path('/private/tmp/mpk-w09-context-application-integration');out=b/'wrapper-program-regeneration';out.mkdir(exist_ok=False);plan=json.loads((b/'wrapper-program-plan.json').read_bytes());h=lambda p:hashlib.sha256(p.read_bytes()).hexdigest();binary=b/'regenerate-wrapper-programs';paths=[binary,b/'wrapper-program-plan.json',b/'regenerate-wrapper-programs.rs',b/'build-wrapper-program-driver.py',b/'wrapper-program-compile-command.json',r/'crates/mpk-cli/tests/support/csharp_practical_data_context.rs'];paths.extend(r.glob('crates/mpk-vc/src/**/*.rs'));paths.extend(r.glob('crates/mpk-cert/src/**/*.rs'))
for target in plan['targets']:paths.extend([r/target['source']['request_path'],r/target['source']['response_path']]+[r/x['file'] for x in target['occurrences']])
for arg in json.loads((b/'wrapper-program-compile-command.json').read_bytes()):
 if '.rlib' in arg:paths.append(pathlib.Path(arg.split('=',1)[1]))
production=[]
for p in sorted(r.glob('crates/mpk-vc/src/**/*.rs')):
 q=compiled/p.relative_to(r);assert p.read_bytes()==q.read_bytes();production.append(str(p.relative_to(r)))
manifest={'hashes':{str(p):h(p) for p in sorted(set(paths))},'compiled_library_production_sources_equal_to_current_checkout':production,'source_commit':plan['source_commit'],'selection_reason':plan['selection_reason']};(out/'manifest.json').write_text(json.dumps(manifest,sort_keys=True,indent=2)+'\n')
state={'status':'running','selection_reason':plan['selection_reason'],'full_t01_gate':plan['full_t01_gate'],'full_t06_gate':plan['full_t06_gate']};(out/'status.json').write_text(json.dumps(state,indent=2)+'\n');env=os.environ.copy()
for k in list(env):
 if k.startswith(('MPK_W09_','MPK_T06_')) or k=='MPK_W14_REQUEST_OUTPUT':env.pop(k)
command=[str(binary),str(b/'wrapper-program-plan.json'),str(r),str(out/'generated')];start=time.monotonic()
with (out/'stdout.txt').open('wb') as stdout,(out/'stderr.txt').open('wb') as stderr:
 p=subprocess.Popen(command,stdout=stdout,stderr=stderr,env=env);state['process_pid']=p.pid;(out/'status.json').write_text(json.dumps(state,indent=2)+'\n');code=p.wait()
unchanged=all(h(pathlib.Path(p))==sha for p,sha in manifest['hashes'].items());state.update(status='passed' if code==0 and unchanged else 'failed',exit_code=code,source_input_binary_bytes_unchanged=unchanged,elapsed_seconds=time.monotonic()-start,command=command);(out/'status.json').write_text(json.dumps(state,sort_keys=True,indent=2)+'\n');print(json.dumps(state));raise SystemExit(0 if state['status']=='passed' else 1)
