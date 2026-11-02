from pathlib import Path
import hashlib,json,subprocess
ROOT=Path(__file__).resolve().parent
def run(*args):
 r=subprocess.run(['/usr/bin/git',*args],stdout=subprocess.PIPE,stderr=subprocess.PIPE,check=False)
 return {'command':['/usr/bin/git',*args],'exit_code':r.returncode,'stdout':r.stdout.decode(),'stderr':r.stderr.decode()}
first=run('diff','--cached','--check');exceptions=[]
for line in first['stdout'].splitlines():
 if not line:continue
 name,location,detail=line.split(':',2)
 assert detail.strip()=='new blank line at EOF.',line
 p=Path(name);assert p.name=='stdout.log' and p.read_bytes().endswith(b'\n\n')
 result=json.loads(p.with_name('result.json').read_bytes());assert result['status'] in ['passed','failed']
 raw=p.read_bytes();assert result['stdout']['sha256']==hashlib.sha256(raw).hexdigest() and result['stdout']['bytes']==len(raw)
 exceptions.append(name)
assert len(exceptions)==3,exceptions
others=run('diff','--cached','--check','--','.',*[':(exclude)'+p for p in exceptions]);assert others['exit_code']==0,others
outputs=run('-c','core.whitespace=-blank-at-eof','diff','--cached','--check','--',*exceptions);assert outputs['exit_code']==0,outputs
record={'status':'passed_with_exact_preserved_native_stdout_EOF_exceptions','initial_default_check':first,'all_other_staged_files_default_check':others,'native_stdout_only_EOF_check':outputs,'exact_original_stdout_paths':exceptions,'no_stored_Git_configuration_change':True,'source_or_proof_whitespace_exceptions':False}
(ROOT/'whitespace-audit.json').write_text(json.dumps(record,indent=2)+'\n');print(json.dumps({'status':record['status'],'exact_original_stdout_exceptions':len(exceptions)}))
