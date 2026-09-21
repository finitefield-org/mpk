import os,subprocess,signal,json,time,hashlib
from pathlib import Path
out=Path('develop/migrations/csharp-03/ordinary-foundation/verification-logs/domain-dependent-refresh')
cases=[('current-total-cell-boundary','target/release/deps/csharp_practical_vc-0951b3e219e92435','csharp_03_t06_w09_domains_source_total_cell_boundary'),('current-full-capacity-aggregate','target/release/deps/mpk_vc-4f3748aad12c0f7e','aggregate_fold_full_capacity_sums_exact_logical_bound'),('current-decimal-domain-order','target/release/deps/csharp_practical_vc-0951b3e219e92435','csharp_03_t06_w09_domains_source_decimal_collection_order')]
for label,binary,test in cases:
 env={k:v for k,v in os.environ.items() if not k.startswith('MPK_W09_') and not k.startswith('MPK_CORE_')};env['MPK_CORE_MAX_SECONDS']='45'
 command=[binary,test,'--nocapture','--test-threads=1'];print('START',label,flush=True);start=time.monotonic();log=out/(label+'.log')
 with log.open('w') as stream:
  proc=subprocess.Popen(command,env=env,stdout=stream,stderr=subprocess.STDOUT,start_new_session=True)
  try:code=proc.wait(timeout=55)
  except subprocess.TimeoutExpired:os.killpg(proc.pid,signal.SIGKILL);proc.wait();code=124
 text=log.read_text();budget=code==124 or (code!=0 and 'MPK_CORE_MAX_SECONDS exhausted' in text)
 if code==0:assert '1 passed; 0 failed' in text
 record={'label':label,'command':command,'exit_code':code,'status':'passed' if code==0 else 'user_long_check_pending' if budget else 'failed','wall_seconds':time.monotonic()-start,'evaluator_seconds':45,'process_limit_seconds':55,'log_sha256':hashlib.sha256(log.read_bytes()).hexdigest()}
 (out/(label+'.json')).write_text(json.dumps(record,indent=2)+'\n');print('END',label,record['status'],round(record['wall_seconds'],2),flush=True);print(text[-1800:],flush=True)
 if code and not budget:raise SystemExit(code)
