from pathlib import Path
import hashlib,json,os,subprocess,time,sys
repo=Path('/Users/kazuyoshitoshiya/mpk');root=repo/'develop/migrations/csharp-03/ordinary-foundation';e=root/'verification-logs/decimal-fixed-formats-terminal';path=e/'verification.json'
def read(p):return json.loads(p.read_text())
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
assert not path.exists(),'Keep earlier evidence'
semantic=root/'verification-logs/binding-reconstruction/decimal-fixed-formats-resumed.json';runtime=read(semantic)
assert runtime['status']=='passed'and runtime['exit_code']==0
log=semantic.parent/runtime['log'];assert sha(log)==runtime['log_sha256'];text=log.read_text()
assert text.count('decimal fixed ')==355+1 and 'decimal fixed observations: 355'in text and 'test result: ok. 1 passed; 0 failed;'in text
buildpath=root/'verification-logs/concrete-operations/review-real-value-mutation/build.json';build=read(buildpath);assert build['status']=='passed';binary=Path(build['binaries']['integration']['path']);assert sha(binary)==build['binaries']['integration']['sha256']
env=os.environ.copy();env.update(build['profile']);env['GOCACHE']='/tmp/mpk-w09-unit4-go-cache'
for key in list(env):
 if key.startswith('MPK_W09_')and key.endswith('_OUT'):env.pop(key)
state=dict(status='running',supervisor_pid=os.getpid(),selection_reason='The full original 65-source, 145-configuration, 355-observation semantic run is terminal passed. Retain that run without repetition; replay all current source pins and recapture the six exact same-byte checker cases because the earlier checker log was stored only in a now-missing /tmp file.',retained_runtime=dict(receipt=str(semantic.relative_to(root)),receipt_sha256=sha(semantic),log_sha256=sha(log),elapsed_seconds=runtime['elapsed_seconds']),build=str(buildpath.relative_to(root)),build_sha256=sha(buildpath),current_binary_sha256=sha(binary),profile=build['profile'],runs=[],full_gate='deferred_to_T06_W12')
def save():path.write_text(json.dumps(state,indent=2)+'\n')
def run(label,command,markers,cwd=repo):
 p=e/(label+'.log');assert not p.exists();row=dict(label=label,command=command,status='running',log=p.name);state['runs'].append(row);save();start=time.monotonic()
 with p.open('wb')as f:
  child=subprocess.Popen(command,cwd=cwd,env=env,stdin=subprocess.DEVNULL,stdout=f,stderr=subprocess.STDOUT);row['pid']=child.pid;save();code=child.wait()
 text=p.read_text();ok=code==0 and all(x in text for x in markers);row.update(status='passed'if ok else'failed',exit_code=code,elapsed_seconds=round(time.monotonic()-start,3),log_sha256=sha(p));save();assert ok,label
 return text
save()
try:
 run('pins',[str(binary),'csharp_03_t06_w09_decimal_fixed_formats_pinned_sources','--nocapture','--test-threads=1'],['test result: ok. 1 passed; 0 failed;'])
 manifest_path=root/'decimal-fixed-formats/certificates.json';manifest=read(manifest_path);assert manifest['contexts_examined']==65 and len(manifest['sources'])==6
 name='TestCheckerAgreementWithRustCLIDecimalFixedFormats'
 text=run('checkers',['go','test','-tags=checkeragreement','-count=1','-timeout=0','-run','^'+name+'$','-v','.'],['--- PASS: '+name+'/'+x['id']+'.hex 'for x in manifest['sources']],repo/'go-tools/mpk-checker-ref')
 pins=[]
 for row in manifest['sources']:
  p=root/'decimal-fixed-formats'/(row['id']+'.hex');raw=bytes.fromhex(p.read_text());digest=hashlib.sha256(b'MPK-MODULE-CERT-0.1\0'+raw).hexdigest();m=row['metadata'];assert digest==m['certificate_sha256']
  assert 'Go accepted: certificate='+digest+' 'in text
  assert len(m['definitions'])==145 and {(d['rounding'],d['scale'])for d in m['definitions']}=={(mode,scale)for mode in ['ToEven','AwayFromZero','ToZero','ToNegativeInfinity','ToPositiveInfinity']for scale in range(29)}
  pins.append(dict(id=row['id'],certificate_sha256=digest,hex_sha256=sha(p)))
 for marker in ['Go accepted: certificate=','Rust accepted identical bytes and report:','Go rejected changed hash:','Rust rejected changed hash:',' axioms=0 ']:assert text.count(marker)==6,marker
 state.update(status='passed_scoped_component',source_contexts=65,configuration_count=145,configuration_occurrences=870,semantic_observations=355,manifest_sha256=sha(manifest_path),pins=pins,current_source_sha256={p:sha(repo/p)for p in build['source_sha256']if 'decimal' in p and ('ordinary_' in p or 'business' in p)},retained_test_sha256=sha(repo/'crates/mpk-vc/tests/support/csharp_practical_ordinary_decimal_fixed_format_tests.rs'))
except Exception as ex:state.update(status='failed',failure=str(ex))
save();print(json.dumps({k:v for k,v in state.items()if k not in ['pins','current_source_sha256']}),flush=True);sys.exit(0 if state['status']=='passed_scoped_component'else 1)
