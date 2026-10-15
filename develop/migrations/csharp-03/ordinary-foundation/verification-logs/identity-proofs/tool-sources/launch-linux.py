import hashlib,json,os,pathlib,subprocess,sys
from datetime import datetime,timezone
commit=sys.argv[1];assert len(commit)==40 and all(c in '0123456789abcdef' for c in commit)
parent=pathlib.Path('/root/mpk-w09-pattern-primitives-0aed36d4');checkout=pathlib.Path('/root/mpk-w09-identity-proofs-'+commit[:8]);reports=checkout.with_name(checkout.name+'-linux');target=checkout.with_name(checkout.name+'-target')
def git(*args,cwd=parent):return subprocess.check_output(['/usr/bin/git',*args],cwd=cwd,text=True).strip()
git('fetch','https://github.com/finitefield-org/mpk.git','codex/w09-packed-pattern-proofs');assert git('rev-parse','FETCH_HEAD')==commit
assert not checkout.exists() and not reports.exists() and not target.exists();git('worktree','add','--detach',str(checkout),commit)
assert git('rev-parse','HEAD',cwd=checkout)==commit and git('status','--porcelain',cwd=checkout)==''
base=checkout/'develop/migrations/csharp-03/ordinary-foundation/verification-logs/identity-proofs';manifest=base/'source-manifest.json';m=json.loads(manifest.read_bytes());files={}
for group in ('source_hashes','fixture_hashes'):
 for n,h in m[group].items():
  data=(checkout/n).read_bytes();assert hashlib.sha256(data).hexdigest()==h and subprocess.check_output(['/usr/bin/git','show',f'{commit}:{n}'],cwd=checkout)==data,n;files[n]=h
reports.mkdir();runner=base/'tool-sources/run-targeted.py';command=['python3',str(runner),'--repo',str(checkout),'--reports',str(reports),'--manifest',str(manifest),'--expected-certificates',str(base/'expected-certificates.json'),'--target',str(target),'--cargo','/usr/bin/cargo']
with (reports/'supervisor.log.txt').open('wb') as output:p=subprocess.Popen(command,cwd=checkout,stdout=output,stderr=subprocess.STDOUT,stdin=subprocess.DEVNULL,start_new_session=True)
launch=dict(status='launched',commit=commit,supervisor_pid=p.pid,started_at=datetime.now(timezone.utc).isoformat(),repo=str(checkout),reports=str(reports),target=str(target),command=command,git_blob_files_verified=len(files),manifest_sha256=hashlib.sha256(manifest.read_bytes()).hexdigest(),runner_sha256=hashlib.sha256(runner.read_bytes()).hexdigest(),clean_checkout=True,public_git_source=True)
(reports/'launch.json').write_text(json.dumps(launch,indent=2,sort_keys=True)+'\n');print(json.dumps(launch))
