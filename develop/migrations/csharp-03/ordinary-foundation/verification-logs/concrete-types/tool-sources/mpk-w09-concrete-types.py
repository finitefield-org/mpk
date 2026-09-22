from pathlib import Path
import hashlib,json,os,re,shutil,subprocess,sys,time
repo=Path('/Users/kazuyoshitoshiya/mpk');root=repo/'develop/migrations/csharp-03/ordinary-foundation';e=root/'verification-logs/concrete-types';e.mkdir(parents=True,exist_ok=True)
mode=sys.argv[1];path=e/(mode+'.json');assert not path.exists(), 'Do not overwrite earlier verification';profile=dict(CARGO_TARGET_DIR='/tmp/mpk-w09-optimized-target',CARGO_PROFILE_TEST_OPT_LEVEL='2',CARGO_PROFILE_TEST_DEBUG='0',CARGO_PROFILE_TEST_DEBUG_ASSERTIONS='true',CARGO_PROFILE_TEST_OVERFLOW_CHECKS='true',CARGO_INCREMENTAL='0');env=os.environ.copy();env.update(profile);env['GOCACHE']='/tmp/mpk-w09-unit4-go-cache'
def sha(p):return hashlib.sha256(Path(p).read_bytes()).hexdigest()
record=dict(status='running',supervisor_pid=os.getpid(),profile=profile,runs=[],full_gate='deferred_to_T06_W12',selection_reason={'build':'Compile exact closed type predicate links and original W06 concrete_type_equivalence conditions plus their targeted test owner.','definitions':'All 45 original contexts and 87 original concrete type sequents: exact complete instance/recipe preservation, strict import/context/field/hash/limit mutations, complete independent public-domain closure, and both unchanged checkers with hash-corruption rejection.','semantics':'Original type predicates over all retained concrete instances: independently preserved recursive domains, admitted and invalid representations, and conditions becoming false when only the concrete predicate is changed. No limits or precomputed truths.','preservation':'Rebuild all 53 source-invariant certificate and metadata pins under current exports; the source-invariant scoped verification stays valid for identical bytes.','quality':'Clippy for changed library/integration tests, five artifact inventory tests, and format checks. No full T gate.'}[mode])
def save():path.write_text(json.dumps(record,indent=2)+'\n')
def run(label,cmd,expected=(),cwd=repo):
 log=e/(label+'.log');row=dict(label=label,command=cmd,status='running',log=log.name);record['runs'].append(row);save();start=time.monotonic()
 with log.open('wb') as f:
  p=subprocess.Popen(cmd,cwd=cwd,env=env,stdin=subprocess.DEVNULL,stdout=f,stderr=subprocess.STDOUT);row['pid']=p.pid;save();code=p.wait()
 text=log.read_text();passed=code==0 and all(x in text for x in expected);row.update(status='passed' if passed else 'failed',exit_code=code,elapsed_seconds=round(time.monotonic()-start,3),log_sha256=sha(log));save()
 if not passed:raise RuntimeError(label+' failed')
 return log
save()
try:
 if mode=='build':
  files=[p for d in ['crates/mpk-vc/src','crates/mpk-vc/tests'] for p in (repo/d).rglob('*.rs')];record['source_sha256']={str(p.relative_to(repo)):sha(p) for p in files};save()
  log=run('build',['cargo','test','-p','mpk-vc','--test','csharp_practical_vc','--no-run']);paths=re.findall(r'Executable .* \(([^\n]+)\)',log.read_text());assert len(paths)==1,paths;record['binaries']={}
  for value in paths:
   binary=Path(value);kind='integration' if binary.name.startswith('csharp_practical_vc-') else 'unit';digest=sha(binary);snapshot=Path('/tmp/mpk-w09-concrete-types-'+kind+'-'+digest[:16]);shutil.copy2(binary,snapshot);record['binaries'][kind]=dict(path=str(snapshot),sha256=digest)
  record['sources_unchanged_during_build']=all(sha(repo/p)==h for p,h in record['source_sha256'].items());assert record['sources_unchanged_during_build']
 else:
  build=json.loads((e/'build.json').read_text());assert build['status']=='passed';record['build']='build.json';bins=build['binaries'];assert all(sha(d['path'])==d['sha256']for d in bins.values());record['binaries']=bins
  binary=bins['integration']['path']
  def test(label,name,binary=binary):return run(label,[binary,name,'--nocapture','--test-threads=1'],['test result: ok. 1 passed; 0 failed;'])
  if mode=='definitions':
   env['MPK_W09_CONCRETE_TYPES_OUT']=str(root/'concrete-types');test('definitions','csharp_03_t06_w09_concrete_types_original_source_certificates');manifest=json.loads((root/'concrete-types/certificates.json').read_text());record['certificate_corpus']={r['id']:r['metadata']['certificate_sha256']for r in manifest['sources']};save();name='TestCheckerAgreementWithRustCLIConcreteTypes';run('checkers',['go','test','-tags=checkeragreement','-count=1','-timeout=0','-run','^'+name+'$','-v','.'],['--- PASS: '+name+'/'+r['id']+'.hex ' for r in manifest['sources']],repo/'go-tools/mpk-checker-ref')
  elif mode=='semantics':
   env.pop('MPK_CORE_MAX_STEPS',None);env.pop('MPK_CORE_MAX_SECONDS',None);record['semantic_limits']='none';save()
   test('predicates','csharp_03_t06_w09_concrete_types_original_predicates')
  elif mode=='preservation':
   env.pop('MPK_W09_SOURCE_INVARIANTS_OUT',None);record['source_invariant_output_enabled']=False;save()
   test('source-invariant-preservation','csharp_03_t06_w09_source_invariants_original_source_certificates')
  elif mode=='quality':
   run('clippy',['cargo','clippy','-p','mpk-vc','--lib','--test','csharp_practical_vc','--','-D','warnings']);run('inventory',['cargo','test','-p','mpk-vc','--test','csharp_practical_inventory'],['test result: ok. 5 passed; 0 failed;']);run('format',['cargo','fmt','--all','--','--check'])
 record['status']='passed'
except Exception as ex:record['status']='failed';record['failure']=str(ex)
save();print(json.dumps({k:v for k,v in record.items()if k!='source_sha256'}),flush=True);sys.exit(0 if record['status']=='passed'else 1)
