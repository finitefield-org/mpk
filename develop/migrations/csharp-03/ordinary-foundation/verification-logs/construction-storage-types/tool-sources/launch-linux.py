import json,os,subprocess,sys
from datetime import datetime,timezone
from hashlib import sha256
from pathlib import Path
commit=sys.argv[1]
assert len(commit)==40 and all(c in '0123456789abcdef' for c in commit)
parent=Path('/root/mpk-w09-pattern-primitives-0aed36d4')
checkout=Path('/root/mpk-w09-construction-storage-types-'+commit[:8])
reports=checkout.with_name(checkout.name+'-linux')
target=checkout.with_name(checkout.name+'-target')
branch='codex/w09-packed-pattern-proofs'
public='https://github.com/finitefield-org/mpk.git'
def git(*args,cwd=parent):
    return subprocess.check_output(['/usr/bin/git',*args],cwd=cwd,text=True).strip()
git('fetch',public,branch)
assert git('rev-parse','FETCH_HEAD')==commit
assert not checkout.exists() and not reports.exists() and not target.exists()
git('worktree','add','--detach',str(checkout),commit)
assert git('rev-parse','HEAD',cwd=checkout)==commit
assert git('status','--porcelain',cwd=checkout)==''
base=checkout/'develop/migrations/csharp-03/ordinary-foundation/verification-logs/construction-storage-types'
manifest_path=base/'source-manifest.json'
manifest=json.loads(manifest_path.read_bytes())
all_hashes={}
for group in ('source_hashes','fixture_hashes'):
    for name,digest in manifest[group].items():
        data=(checkout/name).read_bytes()
        assert sha256(data).hexdigest()==digest,name
        assert subprocess.check_output(['/usr/bin/git','show',f'{commit}:{name}'],cwd=checkout)==data,name
        assert name not in all_hashes or all_hashes[name]==digest
        all_hashes[name]=digest
runner=base/'tool-sources/run-targeted.py'
reports.mkdir()
command=['python3',str(runner),'--repo',str(checkout),'--reports',str(reports),'--manifest',str(manifest_path),'--expected-certificates',str(base/'expected-certificates.json'),'--target',str(target),'--cargo','/usr/bin/cargo']
with (reports/'supervisor.log').open('wb') as output:
    process=subprocess.Popen(command,cwd=checkout,stdin=subprocess.DEVNULL,stdout=output,stderr=subprocess.STDOUT,start_new_session=True)
launch={'status':'launched','started_at':datetime.now(timezone.utc).isoformat(),'commit':commit,'repo':str(checkout),'reports':str(reports),'target':str(target),'command':command,'supervisor_pid':process.pid,'git_blob_files_verified':len(all_hashes),'manifest_sha256':sha256(manifest_path.read_bytes()).hexdigest(),'runner_sha256':sha256(runner.read_bytes()).hexdigest(),'clean_checkout':True,'public_git_source':True}
(reports/'launch.json').write_text(json.dumps(launch,indent=2,sort_keys=True)+'\n')
print(json.dumps(launch,sort_keys=True),flush=True)
