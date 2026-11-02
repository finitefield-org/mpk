"""Outside sandbox: observe actual product drivers and disposable containers."""
from pathlib import Path
import datetime,hashlib,json,re,subprocess
root=Path(__file__).resolve().parent.relative_to(Path.cwd().resolve())
rows=[];containers=[]
for path in sorted(root.glob('**/result.json')):
 if 'audit-' in str(path):continue
 raw=path.read_bytes();r=json.loads(raw)
 assert r['status'] in {'passed','failed'} and r.get('finished_at_utc')
 rows.append({'path':str(path),'sha256':hashlib.sha256(raw).hexdigest(),'bytes':len(raw),'status':r['status']})
 for check in r.get('checks',[]):
  cmd=check['command'];name=cmd[cmd.index('--name')+1]
  observed=subprocess.run(['/usr/local/bin/docker','inspect','--format','{{.State.Running}} {{.State.ExitCode}}',name],capture_output=True,text=True)
  assert (observed.returncode==0 and observed.stdout.startswith('false ')) or (observed.returncode==1 and 'no such object' in observed.stderr)
  containers.append({'name':name,'recorded_exit_code':check['exit_code'],'inspect_exit_code':observed.returncode,'stdout':observed.stdout,'stderr':observed.stderr})
ps=subprocess.check_output(['/bin/ps','-axo','pid=,command='],text=True)
pattern=re.compile(r'python[^\n]*native-host/(?:run-(?:selected|linux)|capture-inputs)\.py(?:\s|$)')
live=[line for line in ps.splitlines() if pattern.search(line)];assert not live,live
(root/'terminal-handles.json').write_text(json.dumps({'status':'passed','observed_at_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'terminal_results':rows,'containers':containers,'live_matching_drivers':live,'all_actual_validation_handles_terminal':True,'observation_timeout_treated_as_terminal':False,'live_handle_restarted':False},indent=2)+'\n')
print(json.dumps({'status':'passed','terminal_results':len(rows),'terminal_containers':len(containers)}))
