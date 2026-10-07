from pathlib import Path
import datetime,hashlib,json,subprocess
control=Path(__file__).parent
repo=Path('/root/mpk-w09-bool-cases-fd99c03c');reports=repo.with_name(repo.name+'-linux-2');target=repo.with_name(repo.name+'-target');commit='fd99c03c8e3535bcc31f0b9d1441d4b74de80ebf'
h=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
def git(*args):return subprocess.check_output(['/usr/bin/git',*args],cwd=repo,text=True).strip()
pins=json.loads((control/'control-hashes.json').read_bytes());assert all(h(control/name)==sha for name,sha in pins.items())
assert git('rev-parse','HEAD')==commit and not git('status','--porcelain')
prior=json.loads((repo.with_name(repo.name+'-linux')/'status.json').read_bytes());assert prior['status']=='failed' and len(prior['stages'])==7 and prior['stages'][-1]['exit_code']==101
assert not reports.exists() and target.exists()
git('sparse-checkout','add','proofs','release','develop/migrations/csharp-03/foundation')
assert not git('status','--porcelain')
manifest=json.loads((control/'source-manifest.json').read_bytes());source={}
for group in ['source_hashes','fixture_hashes','document_hashes']:
 for name,sha in manifest[group].items():
  data=(repo/name).read_bytes();assert h(repo/name)==sha,name;assert subprocess.check_output(['/usr/bin/git','show',commit+':'+name],cwd=repo)==data,name;source[name]=sha
extra={}
for folder in ['proofs','release','develop/migrations/csharp-03/foundation']:
 for p in (repo/folder).rglob('*'):
  if not p.is_file():continue
  name=str(p.relative_to(repo));data=p.read_bytes();assert subprocess.check_output(['/usr/bin/git','show',commit+':'+name],cwd=repo)==data,name;extra[name]=h(p)
reports.mkdir();audit=dict(status='passed_exact_public_source_and_additional_embedded_inputs_before_resume',source_commit=commit,selected_source_fixture_document_pins=source,additional_embedded_inputs=extra,source_clean=True,first_failed_compile_exit_code=101,prefix_stages_reused=6)
(reports/'resume-source-audit.json').write_text(json.dumps(audit,indent=2,sort_keys=True)+'\n')
command=['python3',str(control/'run-linux-2.py'),'--repo',str(repo),'--reports',str(reports),'--target',str(target),'--control',str(control),'--commit',commit]
with (reports/'supervisor.log.txt').open('wb') as log:p=subprocess.Popen(command,cwd=repo,stdin=subprocess.DEVNULL,stdout=log,stderr=subprocess.STDOUT,start_new_session=True)
launch=dict(commit=commit,supervisor_pid=p.pid,command=command,source=str(repo),reports=str(reports),target=str(target),control=str(control),git_blob_files_verified=len(source)+len(extra),source_clean=True,public_git_source=True,control_hashes=pins,started_at=datetime.datetime.now(datetime.timezone.utc).isoformat())
(reports/'launch.json').write_text(json.dumps(launch,sort_keys=True,indent=2)+'\n');print(json.dumps(launch))
