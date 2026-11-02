from pathlib import Path
import datetime,hashlib,json,os,re,subprocess,sys,time
repo=Path.cwd();root=Path(__file__).resolve().parent.relative_to(repo.resolve());mode=sys.argv[1];attempt=sys.argv[2] if len(sys.argv)>2 else 'initial'
assert attempt in ['initial','final','reviewed']
directory=root/attempt/mode;directory.mkdir(parents=True,exist_ok=False)
def seal(p):
 p=Path(p);raw=p.read_bytes();return {'path':str(p),'sha256':hashlib.sha256(raw).hexdigest(),'bytes':len(raw)}
rust=['crates/mpk-core/src/'+p for p in ['lib.rs','infer.rs','env.rs','reduce_inductive.rs','nat_induct.rs','bool_rec_type.rs']]+['crates/mpk-kernel/src/decl_driver.rs']
go=['go-tools/mpk-checker-ref/'+p for p in ['core_check.go','defeq.go','nat_induct.go','nat_induct_test.go','bool_rec_type.go','bool_rec_type_test.go']]
test_sources=['go-tools/mpk-checker-ref/'+p for p in ['core_check_test.go','core_term_intern_test.go','defeq_test.go','bool_cases_test.go','nat_induct_test.go','bool_rec_type_test.go','proof_check_test.go']]
names=sorted({n for p in test_sources for n in re.findall(r'^func (Test[A-Za-z0-9_]+)\(t \*testing.T\)',Path(p).read_text(),re.M)})
assert names and all('CSharp' not in n for n in names)
env=os.environ.copy();cwd=repo;overrides={}
if mode in ['rust-core','rust-lint']:
 overrides={'CARGO_NET_OFFLINE':'true','CARGO_PROFILE_TEST_OPT_LEVEL':'1','CARGO_PROFILE_TEST_DEBUG_ASSERTIONS':'true','CARGO_PROFILE_TEST_OVERFLOW_CHECKS':'true','CARGO_INCREMENTAL':'0','RUST_TEST_THREADS':'1'}
 for k in ['RUST_MIN_STACK','MPK_CORE_MAX_STEPS','MPK_CORE_MAX_SECONDS']:env.pop(k,None)
 command=['cargo','test','--offline','--locked','-p','mpk-core','--lib','--','--nocapture','--test-threads=1']
 if mode=='rust-lint':command=['cargo','clippy','--offline','--locked','-p','mpk-core','-p','mpk-kernel','--lib','--tests','--','-D','warnings']
elif mode in ['go-core','go-legacy','go-vet','go-build']:
 goroot=Path('/usr/local/go') if mode=='go-legacy' else Path('/private/tmp/mpk-go-practical-toolchains/go1.25.0-darwin-arm64/go')
 overrides={'GOROOT':str(goroot),'GOENV':'off','GOWORK':'off','GOTOOLCHAIN':'local','GOPROXY':'off','GOSUMDB':'off','GOTELEMETRY':'off','GOFLAGS':'','GOEXPERIMENT':'','CGO_ENABLED':'0','GOCACHE':'/private/tmp/mpk-go-practical-t02-legacy-build-cache' if mode=='go-legacy' else '/private/tmp/mpk-go-practical-t02-build-cache'}
 selected=[n for n in names if n.startswith(('TestNatInduct','TestBoolRecType'))] if mode=='go-legacy' else names
 command=[str(goroot/'bin/go'),'test','-mod=readonly','-run','^('+'|'.join(selected)+')$','-count=1','-json','.'];cwd=repo/'go-tools/mpk-checker-ref'
 if mode=='go-vet':command=[str(goroot/'bin/go'),'vet','-mod=readonly','.']
 if mode=='go-build':
  binary=Path('/private/tmp/mpk-go-practical-t02-core-interfaces/checker-ref');binary.parent.mkdir(exist_ok=True)
  command=[str(goroot/'bin/go'),'build','-mod=readonly','-trimpath','-buildvcs=false','-o',str(binary),'./cmd/mpk-checker-ref']
elif mode=='rust-format':command=['rustfmt','--check','--edition','2021','--config','skip_children=true',*rust]
elif mode=='go-format':command=['/private/tmp/mpk-go-practical-toolchains/go1.25.0-darwin-arm64/go/bin/gofmt','-l',*go]
else:raise ValueError(mode)
if attempt=='reviewed' and mode=='rust-core':
 command=['cargo','test','--offline','--locked','-p','mpk-core','--lib','bool_rec_type::tests::rejects_noncanonical_constructor_metadata_without_needing_cases','--','--nocapture','--test-threads=1']
if attempt=='reviewed' and mode in ['go-core','go-legacy']:
 selected=['TestBoolRecTypeRejectsNoncanonicalConstructorMetadataWithoutCases','TestNatInductRejectsHigherUniverseFamily']
 command=[str(goroot/'bin/go'),'test','-mod=readonly','-run','^('+'|'.join(selected)+')$','-count=1','-json','.']
env.update(overrides)
record={'status':'running','command':command,'cwd':str(cwd.relative_to(repo)) or '.','environment':overrides,'sources':[seal(p) for p in sorted(set(rust+go+test_sources))],'selection_reason':'Stage exactly the two frozen additive Sort0 interfaces as W01 foundation prerequisites, preserving all frozen source members and v0 bytes. All mpk-core tests are affected by global inference/environment/normalization integration. Go selects only core declarations/term interning/defeq/Bool cases/new interfaces/structural proof tests, excluding all CSharp agreement tests and unchanged decoder/hash/axiom-report targets. Legacy compiles/exercises only the new primitive matrices. Current Go vet/format and Rust changed-file format; fresh ordinary certificates and paired current checkers are subsequent scoped checks. No CSharp tests or intermediate full gate.','started_at_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'CSharp_tests_executed':False,'T02_W01_complete':False,'full_T02_gate':'deferred to W04: check-fast.sh --skip-csharp'}
if mode in ['go-core','go-legacy']:
 record['selected_test_names']=selected;record['selected_test_source_files']=test_sources
if mode.startswith('go-') and mode!='go-format':
 record['compiler']=seal(goroot/'bin/go');record['compiler_version']=subprocess.check_output([str(goroot/'bin/go'),'version'],env=env,text=True).strip()
if attempt=='reviewed':record['selection_reason']='Review adds explicit standalone constructor order/count/generated-flag rejection without Bool.cases, and independent Go higher-Nat rejection. Production interface/normalization code is unchanged. Run only the new Rust and two Go guard roots on the current/legacy compilers, current lint/format; retain the passing142 Rust,33 Go/37 subtest,9 legacy and4 ordinary integration baselines. No CSharp tests or intermediate full gate.'
result=directory/'result.json';result.write_text(json.dumps(record,indent=2)+'\n');started=time.monotonic()
with (directory/'stdout.log').open('wb') as stdout,(directory/'stderr.log').open('wb') as stderr:
 code=subprocess.run(command,cwd=cwd,env=env,stdout=stdout,stderr=stderr).returncode
record.update(status='passed' if code==0 else 'failed',exit_code=code,wall_seconds=time.monotonic()-started,stdout=seal(directory/'stdout.log'),stderr=seal(directory/'stderr.log'),finished_at_utc=datetime.datetime.now(datetime.timezone.utc).isoformat())
if mode=='go-format' and (directory/'stdout.log').stat().st_size:record['status']='failed'
if mode in ['go-core','go-legacy'] and code==0:
 events=[json.loads(line) for line in (directory/'stdout.log').read_text().splitlines()];passed=[e['Test'] for e in events if e['Action']=='pass' and 'Test' in e]
 record['root_tests_passed']=[n for n in passed if '/' not in n];record['subtests_passed']=[n for n in passed if '/' in n];assert sorted(record['root_tests_passed'])==selected
if mode=='rust-core' and code==0:
 match=re.search(r'test result: ok\. (\d+) passed; 0 failed;', (directory/'stdout.log').read_text());assert match;record['tests_passed']=int(match[1])
if mode=='go-build' and code==0:record['checker_binary']=seal(binary)
result.write_text(json.dumps(record,indent=2)+'\n');print(json.dumps({k:record[k] for k in ['status','exit_code','wall_seconds']}),flush=True)
raise SystemExit(0 if record['status']=='passed' else 1)
