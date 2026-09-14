import os, subprocess, pathlib, json, time, sys
binary='/Users/kazuyoshitoshiya/mpk/target/release/deps/csharp_practical_vc-0951b3e219e92435'
root=pathlib.Path(os.environ.get('MPK_W09_MEMORY_OUTPUT','/tmp/mpk-w09-decimal-memory'));root.mkdir(exist_ok=True)
profiles={
 'source':('source','add',None,None,None),
 'generate-add':('generate','add',None,None,None),
 'convert-zero':('evaluate','add','decimal.conversion.int32_to_decimal',0,'success_relation'),
 'add-small':('evaluate','add','decimal.add',0,'success_relation'),
 'convert-fraction':('evaluate','add','decimal.conversion.decimal_to_int32',0,'success_relation'),
 'divide-half':('evaluate','divide','decimal.divide',0,'success_relation'),
 'round-tie':('evaluate','round','decimal.round.ToEven.2',0,'success_relation'),
}
for label in sys.argv[1:]:
 mode,context,operation,case,role=profiles[label]
 env=os.environ.copy()
 for key in list(env):
  if key.startswith('MPK_W09_DECIMAL_'): del env[key]
 env.update(MPK_W09_DECIMAL_PROBE_CONTEXT=context,MPK_W09_DECIMAL_PROBE_MODE=mode)
 if operation:env.update(MPK_W09_DECIMAL_PROBE_OPERATION=operation,MPK_W09_DECIMAL_PROBE_CASE=str(case),MPK_W09_DECIMAL_PROBE_ROLE=role,MPK_W09_DECIMAL_PROBE_WRONG='false')
 cmd=['/usr/bin/time','-l',binary,'csharp_03_t06_w09_decimal_data_memory_probe','--nocapture','--test-threads=1']
 print('START',label,flush=True)
 start=time.monotonic()
 with (root/(label+'.log')).open('w') as log:
  completed=subprocess.run(cmd,env=env,stdout=log,stderr=subprocess.STDOUT)
 result={'label':label,'mode':mode,'context':context,'operation':operation,'case':case,'role':role,'exit_code':completed.returncode,'wall_seconds':time.monotonic()-start,'command':cmd,'environment':{k:v for k,v in env.items() if k.startswith('MPK_W09_DECIMAL_PROBE_') or k=='MPK_W09_CORE_CACHE_POLICY'}}
 (root/(label+'.json')).write_text(json.dumps(result,indent=2)+'\n')
 print('END',label,json.dumps(result),flush=True)
 if completed.returncode:sys.exit(completed.returncode)
