import os,subprocess,signal,json,time,hashlib
from pathlib import Path
out=Path('develop/migrations/csharp-03/ordinary-foundation/verification-logs/domain-dependent-refresh')
cases=[('public-domains','PUBLIC_DOMAINS','public_domains_construction_sources'),('public-defaults','PUBLIC_DEFAULTS','public_defaults_source_conditions'),('structural-public','STRUCTURAL_PUBLIC','structural_public_composes_existing_definitions'),('structural-boundary','STRUCTURAL_BOUNDARY','structural_boundary_original_sources')]
(out/'generation-plan.json').write_text(json.dumps(cases,indent=2)+'\n')
for family,var,test in cases:
 env={k:v for k,v in os.environ.items() if not k.startswith('MPK_W09_') and not k.startswith('MPK_CORE_')}
 env['MPK_W09_'+var+'_OUT']='/tmp/mpk-domain-dependent-'+family;env['MPK_CORE_MAX_SECONDS']='45'
 command=['target/release/deps/csharp_practical_vc-0951b3e219e92435','csharp_03_t06_w09_'+test,'--nocapture','--test-threads=1']
 print('START',family,flush=True);start=time.monotonic();log=out/(family+'-generation.log')
 with log.open('w') as stream:
  proc=subprocess.Popen(command,env=env,stdout=stream,stderr=subprocess.STDOUT,start_new_session=True)
  try:code=proc.wait(timeout=55)
  except subprocess.TimeoutExpired:os.killpg(proc.pid,signal.SIGKILL);proc.wait();code=124
 record={'family':family,'command':command,'environment':{'MPK_W09_'+var+'_OUT':env['MPK_W09_'+var+'_OUT'],'MPK_CORE_MAX_SECONDS':'45','MPK_CORE_MAX_STEPS':None},'exit_code':code,'wall_seconds':time.monotonic()-start,'process_limit_seconds':55,'log_sha256':hashlib.sha256(log.read_bytes()).hexdigest()}
 (out/(family+'-generation.json')).write_text(json.dumps(record,indent=2)+'\n')
 print('END',family,code,round(record['wall_seconds'],2),flush=True)
 if code:print(log.read_text()[-4000:],flush=True);raise SystemExit(code)
