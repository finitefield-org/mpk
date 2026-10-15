import datetime,hashlib,json,os,pathlib,subprocess,sys
commit=sys.argv[1];assert len(commit)==40 and all(c in '0123456789abcdef'for c in commit)
parent=pathlib.Path('/root/mpk-w09-pattern-primitives-0aed36d4');control=pathlib.Path('/root/mpk-w09-context-application-'+commit[:8]+'-control');checkout=pathlib.Path('/root/mpk-w09-context-application-'+commit[:8]);reports=checkout.with_name(checkout.name+'-linux');target=checkout.with_name(checkout.name+'-target')
def git(*args,cwd=parent):return subprocess.check_output(['/usr/bin/git',*args],cwd=cwd,text=True).strip()
manifest=control/'source-manifest.json';runner=control/'run-targeted.py';pins=json.loads((control/'control-hashes.json').read_text())
for n,h in pins.items():assert hashlib.sha256((control/n).read_bytes()).hexdigest()==h,n
git('fetch','https://github.com/finitefield-org/mpk.git','codex/w09-packed-pattern-proofs');assert git('rev-parse','FETCH_HEAD')==commit
assert not checkout.exists()and not reports.exists()and not target.exists();git('worktree','add','--detach',str(checkout),commit)
assert git('rev-parse','HEAD',cwd=checkout)==commit and git('status','--porcelain',cwd=checkout)==''
m=json.loads(manifest.read_text());files={}
for group in ('source_hashes','fixture_hashes'):
 for n,h in m[group].items():
  data=(checkout/n).read_bytes();assert hashlib.sha256(data).hexdigest()==h and subprocess.check_output(['/usr/bin/git','show',f'{commit}:{n}'],cwd=checkout)==data,n;files[n]=h
assert len(files)==1065
reports.mkdir();command=['python3',str(runner),'--repo',str(checkout),'--reports',str(reports),'--manifest',str(manifest),'--target',str(target),'--cargo','/usr/bin/cargo','--expected-certificates',str(control/'expected-certificates.json')]
with(reports/'supervisor.log.txt').open('wb')as output:p=subprocess.Popen(command,cwd=checkout,stdout=output,stderr=subprocess.STDOUT,stdin=subprocess.DEVNULL,start_new_session=True)
launch=dict(status='launched_incomplete_checkpoint_tests',commit=commit,supervisor_pid=p.pid,started_at=datetime.datetime.now(datetime.timezone.utc).isoformat(),repo=str(checkout),reports=str(reports),target=str(target),control=str(control),command=command,git_blob_files_verified=len(files),control_hashes=pins,clean_checkout=True,public_git_source=True,local_acceptance_receipt_pending=True,exact_export_comparison_pending=True,application_scope_pending=True,full_t_gate='deferred to T06-W12')
(reports/'launch.json').write_text(json.dumps(launch,indent=2,sort_keys=True)+'\n');print(json.dumps(launch))
