from pathlib import Path
import hashlib,json,os,re,shutil,subprocess,sys,time
repo=Path('/Users/kazuyoshitoshiya/mpk');root=repo/'develop/migrations/csharp-03/ordinary-foundation';e=root/'verification-logs/concrete-operations/review-isolated-input-guard';mode=sys.argv[1];path=e/(mode+'.json')
# Independent quality checks may be started while long semantics run. The
# pipeline later adopts their exact terminal receipt without repeating commands.
if mode=='quality' and path.exists():
 while True:
  try: prior=json.loads(path.read_text())
  except json.JSONDecodeError: time.sleep(0.5);continue
  if prior['status']!='running':break
  time.sleep(1)
 for row in prior['runs']:
  assert hashlib.sha256((e/row['log']).read_bytes()).hexdigest()==row['log_sha256']
 print(json.dumps(dict(adopted_existing_quality=path.name,status=prior['status'])),flush=True)
 sys.exit(0 if prior['status']=='passed'else 1)
assert not path.exists(),'Preserve earlier evidence'
profile=dict(CARGO_TARGET_DIR='/tmp/mpk-w09-optimized-target',CARGO_PROFILE_TEST_OPT_LEVEL='2',CARGO_PROFILE_TEST_DEBUG='0',CARGO_PROFILE_TEST_DEBUG_ASSERTIONS='true',CARGO_PROFILE_TEST_OVERFLOW_CHECKS='true',CARGO_INCREMENTAL='0');env=os.environ.copy();env.update(profile);env['GOCACHE']='/tmp/mpk-w09-unit4-go-cache'
def sha(p):return hashlib.sha256(Path(p).read_bytes()).hexdigest()
selection={'build':'Compile the changed multi-operand W06 sequent lowerer, structural extension, operation adapters and targeted test owner.','definitions':'All 45 original contexts, all 467 original operation recipes including uninvoked operations, exact source signatures/check tables, same transitive base definition closures, operand order and original goal/assumption ASTs, strict import/context/field/hash/limit rejection, and both unchanged checkers with report equality and hash-corruption rejection.','routing':'Exhaust every failure truth assignment for every resolved failing operation, including selected first failure, success, and one-sided single-bit mutations masked only by an earlier failure. Constant test copies isolate the adapter; exact underlying algorithm closures are separately compared.','real':'Real closed, multi-operand, guarded and NaN-containing operations; changed concrete normal values must falsify normal agreement and original applicable conditions. The original source-domain guard must remain effective. No evaluator limits.','preservation':'The structural extension refactor must preserve all previous standalone, boundary and public profile bytes. The unary concrete-type consumer must preserve all 45 prior certificates. No unrelated full gate.','quality':'Clippy on changed library and integration test owner, five source-consumer inventory tests, Rust formatting and Go formatting.'}
r=dict(status='running',supervisor_pid=os.getpid(),profile=profile,selection_reason=('Isolate a one-bit NaN payload change while preserving the key and all 32 observed float bits; reject actual malformed Option padding while retaining the original full guarded condition.'if mode=='isolated'else selection[mode]),runs=[],full_gate='deferred_to_T06_W12')
def save():path.write_text(json.dumps(r,indent=2)+'\n')
def run(label,cmd,expected=(),cwd=repo):
 log=e/(label+'.log');assert not log.exists();row=dict(label=label,command=cmd,status='running',log=log.name);r['runs'].append(row);save();start=time.monotonic()
 with log.open('wb')as f:
  p=subprocess.Popen(cmd,cwd=cwd,env=env,stdin=subprocess.DEVNULL,stdout=f,stderr=subprocess.STDOUT);row['pid']=p.pid;save();code=p.wait()
 text=log.read_text();ok=code==0 and all(s in text for s in expected);row.update(status='passed'if ok else'failed',exit_code=code,elapsed_seconds=round(time.monotonic()-start,3),log_sha256=sha(log));save()
 if not ok:raise RuntimeError(label+' failed')
 return log
save()
try:
 if mode=='build':
  files=[p for d in ['crates/mpk-vc/src','crates/mpk-vc/tests']for p in (repo/d).rglob('*.rs')];r['source_sha256']={str(p.relative_to(repo)):sha(p)for p in files};save()
  log=run('build',['cargo','test','-p','mpk-vc','--test','csharp_practical_vc','--no-run']);paths=re.findall(r'Executable .* \(([^\n]+)\)',log.read_text());assert len(paths)==1,paths;r['binaries']={}
  for value in paths:
   binary=Path(value);digest=sha(binary);snapshot=Path('/tmp/mpk-w09-concrete-operations-reviewed-integration-'+digest[:16]);shutil.copy2(binary,snapshot);r['binaries']['integration']=dict(path=str(snapshot),sha256=digest)
  r['sources_unchanged_during_build']=all(sha(repo/p)==h for p,h in r['source_sha256'].items());assert r['sources_unchanged_during_build']
 else:
  build=json.loads((e/'build.json').read_text());assert build['status']=='passed';r['build']='build.json';r['binaries']=build['binaries'];assert all(sha(x['path'])==x['sha256']for x in r['binaries'].values());binary=r['binaries']['integration']['path']
  def test(label,name):return run(label,[binary,name,'--nocapture','--test-threads=1'],['test result: ok. 1 passed; 0 failed;'])
  if mode=='definitions':
   env.pop('MPK_W09_CONCRETE_OPERATIONS_OUT',None);r['corpus_output_enabled']=False;save();test('definitions','csharp_03_t06_w09_concrete_operations_original_source_certificates');manifest=json.loads((root/'concrete-operations/certificates.json').read_text());r['unchanged_certificate_corpus']={x['id']:x['metadata']['certificate_sha256']for x in manifest['sources']};save()
  elif mode=='isolated':
   env.pop('MPK_CORE_MAX_STEPS',None);env.pop('MPK_CORE_MAX_SECONDS',None);r['semantic_limits']='none';save();test('isolated','csharp_03_t06_w09_concrete_operations_isolated_payload_and_invalid_input')
  elif mode in ['routing','real']:
   env.pop('MPK_CORE_MAX_STEPS',None);env.pop('MPK_CORE_MAX_SECONDS',None);r['semantic_limits']='none';save();test(mode,'csharp_03_t06_w09_concrete_operations_'+('ordered_outcomes'if mode=='routing'else'real_values_and_guards'))
  elif mode=='preservation':
   for key in list(env):
    if key.startswith('MPK_W09_')and key.endswith('_OUT'):env.pop(key)
   for label,name in [('structural-foundations','structural_foundation_original_sources'),('structural-boundary','structural_boundary_original_sources'),('structural-public','structural_public_composes_existing_definitions'),('concrete-types','concrete_types_original_source_certificates')]:test('preserve-'+label,'csharp_03_t06_w09_'+name)
  elif mode=='quality':
   run('clippy',['cargo','clippy','-p','mpk-vc','--lib','--test','csharp_practical_vc','--','-D','warnings']);run('inventory',['cargo','test','-p','mpk-vc','--test','csharp_practical_inventory'],['test result: ok. 5 passed; 0 failed;']);run('format',['cargo','fmt','--all','--','--check']);log=run('go-format',['gofmt','-l','go-tools/mpk-checker-ref/checker_agreement_test.go']);assert not log.read_text().strip()
 r['status']='passed'
except Exception as ex:r['status']='failed';r['failure']=str(ex)
save();print(json.dumps({k:v for k,v in r.items()if k!='source_sha256'}),flush=True);sys.exit(0 if r['status']=='passed'else 1)
