"""Retain full prelaunch input bytes/modes and actual affected local results.
Run outside the sandbox: every Git read uses the authorized system Git.
"""
from pathlib import Path
import datetime,hashlib,json,os,shutil,subprocess,sys,time
ROOT=Path(__file__).resolve().parent;REPO=Path.cwd();BASE='75cc4973b1411337707b3c096215648309e827c8'
epoch,mode=sys.argv[1:];assert epoch.replace('-','').isalnum()
def seal(p):
 b=Path(p).read_bytes();return {'path':str(p),'bytes':len(b),'sha256':hashlib.sha256(b).hexdigest()}
def git(*args):return subprocess.check_output(['/usr/bin/git',*args])
assert git('rev-parse','HEAD').decode().strip()==BASE
out=ROOT/epoch;out.mkdir(exist_ok=False);tracked={}
for row in git('ls-files','--stage','-z').split(b'\0'):
 if row:
  meta,name=row.split(b'\t',1);permissions,oid,stage=meta.decode().split();assert stage=='0';tracked[name.decode()]=oid
sources=[p for base in ['crates','proofs/std','go-tools/go2vir','go-tools/mpk-checker-ref'] for p in Path(base).rglob('*') if p.is_file()]
sources.extend([Path('Cargo.toml'),Path('Cargo.lock'),Path(__file__).relative_to(REPO)])
sources.extend([ROOT.parent/'source-execution/corpus.json',ROOT.parent/'source-execution/program.go',ROOT.parent/'source-string-operations/corpus.json',ROOT/'additions.go.txt'])
sources.extend(p for p in ROOT.glob('*.json') if p.name=='corpus.json')
sources.extend(p for p in Path('.cargo').rglob('*') if p.is_file())
sources.extend(p for p in [Path('rust-toolchain'),Path('rust-toolchain.toml')] if p.exists())
records=[]
for p in sorted(set(sources)):
 b=p.read_bytes();r={**seal(p),'mode':p.stat().st_mode&0o777};oid=tracked.get(str(p))
 if oid and hashlib.sha1(b'blob '+str(len(b)).encode()+b'\0'+b).hexdigest()==oid:r.update(original_commit=BASE,original_git_blob=oid)
 else:
  target=out/'inputs'/p;target.parent.mkdir(parents=True,exist_ok=True);target.write_bytes(b);target.chmod(r['mode']);r['complete_changed_input']=str(target.relative_to(REPO))
 records.append(r)
driver=out/'driver.py';driver.write_bytes(Path(__file__).read_bytes());receipt=out/'inputs.json';receipt.write_text(json.dumps({'basis_commit':BASE,'sources':records,'actual_driver_prelaunch':seal(driver)},indent=2)+'\n')
changed=['crates/mpk-vc/src/go_practical_type_keys.rs','crates/mpk-vc/src/go_practical_type_keys/source_scalar_typing.rs','crates/mpk-vc/src/go_practical_type_keys/source_scalar_typing/call_sites.rs','crates/mpk-vc/src/go_practical_ordinary.rs','crates/mpk-vc/src/go_practical_ordinary/source_invocation.rs']
if Path('crates/mpk-vc/tests/go_practical_source_invocation.rs').exists():changed.append('crates/mpk-vc/tests/go_practical_source_invocation.rs')
sdk=Path('/private/tmp/mpk-go-practical-original-packages-sdk-20261021/go')
commands={'format-write':['rustfmt','--edition=2021','--config','skip_children=true',*changed],'format':['rustfmt','--edition=2021','--check','--config','skip_children=true',*changed],
'test-build':['cargo','test','--offline','--locked','-p','mpk-vc','--test','go_practical_source_invocation','--no-run'],
'compile':['cargo','check','--offline','--locked','-p','mpk-vc','--lib'],
'rust':['cargo','test','--offline','--locked','-p','mpk-vc','--test','go_practical_source_invocation','--','--nocapture','--test-threads=1'],

'clippy':['cargo','clippy','--offline','--locked','-p','mpk-vc','--lib','--test','go_practical_source_invocation','--','-D','warnings'],
'native':[str(sdk/'bin/go'),'test','-json','-count=1','-timeout=40m','-run','^TestPracticalSourceInvocationOriginalSignatures$'],
'go-format-write':[str(sdk/'bin/gofmt'),'-w','go-tools/go2vir/practical_source_invocation_test.go'],
'go-format':[str(sdk/'bin/gofmt'),'-l','go-tools/go2vir/practical_source_invocation_test.go']}
environment={'CARGO_NET_OFFLINE':'true','CARGO_PROFILE_TEST_OPT_LEVEL':'1','CARGO_PROFILE_TEST_DEBUG_ASSERTIONS':'true','CARGO_PROFILE_TEST_OVERFLOW_CHECKS':'true','CARGO_INCREMENTAL':'0','RUST_TEST_THREADS':'1'}
if mode=='rust':environment['MPK_GO_SOURCE_INVOCATION_OUTPUT']=str(out/'certificates')
if mode=='native' or mode.startswith('go-'):
 environment.update({'GOROOT':str(sdk),'GOTOOLCHAIN':'local','GOENV':'off','GOWORK':'off','GOFLAGS':'-mod=vendor','GOPROXY':'off','GOSUMDB':'off','CGO_ENABLED':'0','GOMAXPROCS':'1','GOCACHE':'/private/tmp/mpk-go-practical-t02-go-cache','GOMODCACHE':'/private/tmp/mpk-go-practical-t02-go-mod'})
if mode=='native':(out/'native-originals').mkdir();environment['MPK_GO_PRACTICAL_SYNTAX_EVIDENCE']=str(out/'native-originals')
env=os.environ.copy();env.update(environment)
for name in ['RUST_MIN_STACK','MPK_CORE_MAX_STEPS','MPK_CORE_MAX_SECONDS']:env.pop(name,None)
r={'status':'running','command':commands[mode],'environment':environment,'source_receipt':seal(receipt),'actual_driver_prelaunch':seal(driver),'formatter_mutation_expected':mode in ['format-write','go-format-write'],'tools':{n:seal(p) for n,p in ({'go':sdk/'bin/go','gofmt':sdk/'bin/gofmt'} if mode=='native' or mode.startswith('go-') else {n:shutil.which(n) for n in ['cargo','rustc','rustfmt']}).items()},'selection_reason':'Derive complete actual-source call argument groups, tuple projections, signatures and static/closed-error targets. Check complete source-specific ordinary result/post-state joint records, source identity ownership, every original type/U, zero/16 result and65-argument consumers and genuine altered-law controls. Source callee execution, preconditions/effects/frame and source/IR/program/context proofs remain separate. No pruned source/U, new cap, unrelated old matrix or intermediate broad gate.','full_T_gate':'Deferred to final GO-PRAC-T02-W04: ./scripts/check-fast.sh','started_at_utc':datetime.datetime.now(datetime.timezone.utc).isoformat()}
result=out/'result.json';result.write_text(json.dumps(r,indent=2)+'\n');start=time.monotonic()
with (out/'stdout.log').open('wb') as stdout,(out/'stderr.log').open('wb') as stderr:code=subprocess.run(commands[mode],cwd='go-tools/go2vir' if mode=='native' else '.',env=env,stdout=stdout,stderr=stderr,check=False).returncode
unchanged=all(seal(x['path'])=={k:x[k] for k in ['path','bytes','sha256']} and Path(x['path']).stat().st_mode&0o777==x['mode'] for x in records)
r.update(status='passed' if code==0 and (unchanged or r['formatter_mutation_expected']) else 'failed',exit_code=code,seconds=time.monotonic()-start,whole_source_inputs_unchanged=unchanged,stdout=seal(out/'stdout.log'),stderr=seal(out/'stderr.log'),finished_at_utc=datetime.datetime.now(datetime.timezone.utc).isoformat());result.write_text(json.dumps(r,indent=2)+'\n');print(json.dumps({k:r[k] for k in ['status','exit_code','seconds','whole_source_inputs_unchanged']}));raise SystemExit(r['status']!='passed')
