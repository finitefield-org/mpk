from pathlib import Path
import hashlib,json,subprocess
b=Path(__file__).parent;p=b.parent/'pre-cases-universe-probe';r=Path('/private/tmp/mpk-w09-bool-cases-registration-order');out=b/'producer-replay';out.mkdir(exist_ok=False);h=lambda x:hashlib.sha256(x.read_bytes()).hexdigest()
lib=Path('/private/tmp/mpk-w09-bool-cases-target/debug/deps/libmpk_cert-73d78a7ad244656d.rlib');state=dict(status='running',producer_sources={x.name:h(x) for x in [p/'regression-main.rs',p/'builder.rs']},certificate_library_sha256=h(lib),stages=[])
try:
 command=['/Users/kazuyoshitoshiya/.cargo/bin/rustc','--edition=2021',str(p/'regression-main.rs'),'-L','dependency='+str(lib.parent),'--extern','mpk_cert='+str(lib),'-o',str(out/'producer')]
 with (out/'compile.log.txt').open('wb') as log:process=subprocess.run(command,stdout=log,stderr=subprocess.STDOUT)
 state['stages'].append(dict(stage='compile',command=command,exit_code=process.returncode,log_sha256=h(out/'compile.log.txt')));assert process.returncode==0
 command=[str(out/'producer'),str(r/'fixtures/core-bool-cases/right-identity.hex'),str(out/'generated')]
 with (out/'generate.log.txt').open('wb') as log:process=subprocess.run(command,stdout=log,stderr=subprocess.STDOUT)
 state['stages'].append(dict(stage='generate',command=command,exit_code=process.returncode,log_sha256=h(out/'generate.log.txt')));assert process.returncode==0
 rows=[]
 for x in sorted((out/'generated').glob('*.hex')):
  assert x.read_bytes()==(r/'fixtures/core-bool-cases'/x.name).read_bytes();assert x.read_bytes()==(p/'regression-inputs-3'/x.name).read_bytes();rows.append(dict(path=x.name,raw_sha256=h(x),exact_pinned_bytes=True))
 assert len(rows)==12;state.update(status='passed_all_12_actual_producer_outputs_exact_pinned_bytes',rows=rows,producer_binary_sha256=h(out/'producer'))
except BaseException as e:state['status']='failed';state['error']=repr(e);raise
finally:(out/'status.json').write_text(json.dumps(state,indent=2,sort_keys=True)+'\n')
print(state['status'])
