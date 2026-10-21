"""Affected lexical checks, with full source preimages sealed before launch."""
from pathlib import Path
import datetime,fcntl,hashlib,json,os,subprocess,sys,time
ROOT=Path.cwd();OUT=Path(__file__).resolve().parent;BASE='13edcaf13d267febe501e5b7b66135d20915fbc8';epoch,mode=sys.argv[1:];assert epoch.replace('-','').isalnum()
def git(*args):return subprocess.check_output(['/usr/bin/git',*args])
def seal(p):
 raw=Path(p).read_bytes();return {'path':str(p),'bytes':len(raw),'sha256':hashlib.sha256(raw).hexdigest()}
assert git('rev-parse','HEAD').decode().strip()==BASE
archive=OUT/epoch;archive.mkdir(exist_ok=True)
with (Path('/private/tmp')/('mpk-go-lex-'+epoch+'-preimage.lock')).open('a+b')as lock:
 fcntl.flock(lock,fcntl.LOCK_EX);original={}
 for row in git('ls-files','--stage','-z').split(b'\0'):
  if row:
   meta,name=row.split(b'\t',1);permissions,oid,stage=meta.decode().split();assert stage=='0';original[name.decode()]=oid
 sources=sorted(p for root in [Path('go-tools/go2vir'),Path('crates'),Path('proofs/std')]for p in root.rglob('*')if p.is_file());sources+=[Path('Cargo.toml'),Path('Cargo.lock')]
 for name in ['smoke.json','byte-bound-native.json']:
  if(OUT/name).exists():sources.append(OUT/name)
 inputs=[]
 for path in sources:
  raw=path.read_bytes();row=seal(path);oid=original.get(str(path))
  if oid and hashlib.sha1(b'blob '+str(len(raw)).encode()+b'\0'+raw).hexdigest()==oid:row.update(original_commit=BASE,original_git_blob=oid)
  else:
   target=archive/'inputs'/path;target.parent.mkdir(parents=True,exist_ok=True)
   if target.exists():assert target.read_bytes()==raw
   else:target.write_bytes(raw)
   row['complete_changed_input']=str(target.relative_to(ROOT))
  inputs.append(row)
 driver=archive/'run-checks.py'
 if driver.exists():assert driver.read_bytes()==Path(__file__).read_bytes()
 else:driver.write_bytes(Path(__file__).read_bytes())
 receipt={'basis_commit':BASE,'sources':inputs,'actual_driver_prelaunch':seal(driver)};input_path=archive/'inputs.json'
 if input_path.exists():assert json.loads(input_path.read_bytes())==receipt
 else:input_path.write_text(json.dumps(receipt,indent=2)+'\n')
goroot=Path('/private/tmp/mpk-go-practical-original-packages-sdk-20261021/go');go=str(goroot/'bin/go')
rust_changed=['crates/mpk-vc/src/go_practical_type_keys.rs','crates/mpk-vc/src/go_practical_type_keys/validate.rs','crates/mpk-vc/src/go_practical_type_keys/validate/unicode.rs','crates/mpk-vc/src/go_practical_type_keys/lex.rs','crates/mpk-vc/tests/go_practical_lex.rs']
commands={
 'go-format-write':[str(goroot/'bin/gofmt'),'-w','practical_lex_reference_test.go'],
 'rust-format-write':['rustfmt','--edition=2021',*rust_changed],
 'go-byte-bound':[go,'test','-json','-count=1','-timeout=20m','-run','^TestPracticalLexByteBound$'],
 'go-oracle':[go,'test','-json','-count=1','-timeout=20m','-run','^TestPracticalLexOriginalGoReference$'],
 'vet':[go,'vet','./...'],
 'go-format':[str(goroot/'bin/gofmt'),'-l','practical_lex_reference_test.go'],
 'rust-paired':['cargo','test','--offline','-p','mpk-vc','--test','go_practical_lex','exact_native_go_token_streams_and_rejections','--','--nocapture'],
 'rust-smoke':['cargo','test','--offline','-p','mpk-vc','--test','go_practical_lex','--','--nocapture'],
 'clippy':['cargo','clippy','--offline','-p','mpk-vc','--lib','--test','go_practical_lex','--','-D','warnings'],
 'rust-format':['rustfmt','--edition=2021','--check',*rust_changed],
};assert mode in commands
is_go=mode in {'go-format-write','go-oracle','go-byte-bound','vet','go-format'}
environment={'GOROOT':str(goroot),'GOTOOLCHAIN':'local','GOENV':'off','GOWORK':'off','GOFLAGS':'-mod=vendor','GOPROXY':'off','GOSUMDB':'off','CGO_ENABLED':'0','GOMAXPROCS':'1','GOCACHE':'/private/tmp/mpk-go-practical-t02-go-cache','GOMODCACHE':'/private/tmp/mpk-go-practical-t02-go-mod'}if is_go else{}
external_inputs=[]
if mode=='go-oracle':
 corpus=json.loads((OUT/'corpus-inputs.json').read_bytes());external_inputs=[corpus['manifest']]+[r['actual']for r in corpus['complete_inputs']]
 for row in external_inputs:assert seal(row['path'])==row
 output=Path('/private/tmp')/('mpk-go-lex-'+epoch+'-native');output.mkdir(exist_ok=False);environment.update(MPK_GO_LEX_INPUT=corpus['input_directory'],MPK_GO_LEX_OUTPUT=str(output))
if mode=='go-byte-bound':
 output=Path('/private/tmp')/('mpk-go-lex-'+epoch+'-byte-bound-native');output.mkdir(exist_ok=False);environment['MPK_GO_LEX_OUTPUT']=str(output)
if mode in {'rust-paired','rust-smoke'}:
 output=Path('/private/tmp')/('mpk-go-lex-'+epoch+'-'+mode+'-outputs');output.mkdir(exist_ok=False);environment['MPK_GO_LEX_RUST_OUTPUT']=str(output)
 if mode=='rust-paired':
  corpus=Path('/private/tmp/mpk-go-lex-formatted-native');environment['MPK_GO_LEX_CORPUS']=str(corpus);external_inputs=[seal(p)for p in sorted(corpus.rglob('*'))if p.is_file()]
env=os.environ.copy();env.update(environment);directory=archive/mode;directory.mkdir(exist_ok=False);result=directory/'result.json'
record={'status':'running','command':commands[mode],'cwd':'go-tools/go2vir'if is_go else'.','environment':environment,'sources':inputs,'source_receipt':seal(input_path),'actual_driver_prelaunch':seal(driver),'external_inputs':external_inputs,'selection_reason':'New independent Rust lexer consumes immutable Go source buffers and pinned Unicode tables. Compare exact kind/physical offset/literal bytes for all accepted native Go1.25 streams; both sides reject invalid UTF8/NUL/BOM/numbers/escapes/comments/directives without claiming matching recovery diagnostics. Use all original captured Go files, byte-derived token bounds beyond the independent AST budget, every constant string expression in the pinned scanner tests, all operator pairs and crafted lexical boundaries. Default committed edge corpus and source ownership controls also execute; strict lint/changed-file format and Go test vet cover consumers. Go production frontend189-source closure, registry and ordinary proofs/evaluators are unchanged; retain passed prior runtime/proof/rebuild/public-boundary checks. No whole/equivalent intermediate T gate. No source parsing, typing, nominal/store/program authority inferred.','formatter_mutation_expected':mode.endswith('format-write'),'full_T_gate':'deferred to GO-PRAC-T02-W04: ./scripts/check-fast.sh','started_at_utc':datetime.datetime.now(datetime.timezone.utc).isoformat()};result.write_text(json.dumps(record,indent=2)+'\n');start=time.monotonic()
with(directory/'stdout.log').open('wb')as stdout,(directory/'stderr.log').open('wb')as stderr:code=subprocess.run(commands[mode],cwd=record['cwd'],env=env,stdout=stdout,stderr=stderr,check=False).returncode
record.update(status='passed'if code==0 else'failed',exit_code=code,seconds=time.monotonic()-start,finished_at_utc=datetime.datetime.now(datetime.timezone.utc).isoformat(),stdout=seal(directory/'stdout.log'),stderr=seal(directory/'stderr.log'),whole_source_inputs_unchanged=all(seal(r['path'])=={k:r[k]for k in ['path','bytes','sha256']}for r in inputs),whole_external_inputs_unchanged=all(seal(r['path'])==r for r in external_inputs))
if is_go and mode in {'go-oracle','go-byte-bound'}:
 events=[json.loads(line)for line in(directory/'stdout.log').read_text().splitlines()];record['passed_tests']=[e['Test']for e in events if e['Action']=='pass'and'Test'in e];record['failed_tests']=[e['Test']for e in events if e['Action']=='fail'and'Test'in e];record['skipped_tests']=[e['Test']for e in events if e['Action']=='skip'and'Test'in e]
 if not record['passed_tests']or record['failed_tests']or record['skipped_tests']:record['status']='failed'
if not record['formatter_mutation_expected']and not record['whole_source_inputs_unchanged']or not record['whole_external_inputs_unchanged']:record['status']='failed'
if mode=='go-format'and(directory/'stdout.log').stat().st_size:record['status']='failed'
result.write_text(json.dumps(record,indent=2)+'\n');print(json.dumps({k:record[k]for k in ['status','exit_code','seconds','whole_source_inputs_unchanged','whole_external_inputs_unchanged']}));raise SystemExit(record['status']!='passed')
