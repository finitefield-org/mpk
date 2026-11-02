"""Complete prelaunch source preservation and affected local handoff checks."""
from pathlib import Path
import datetime,hashlib,json,os,shutil,subprocess,sys,tarfile,time

ROOT=Path(__file__).resolve().parent
REPO=Path.cwd()
SDK=Path('/private/tmp/mpk-go125-20261025-download/go')
CHANGED=['crates/mpk-vc/src/go_practical_type_keys/contract_inputs.rs','crates/mpk-vc/tests/go_practical_contract_inputs.rs','crates/mpk-vc/src/go_practical_type_keys.rs']
GO_CHANGED=['go-tools/go2vir/practical_source_inputs.go','go-tools/go2vir/practical_contract_inputs.go','go-tools/go2vir/practical_storage_test.go','go-tools/go2vir/practical_contract_inputs_test.go']

def seal(path):
    path=Path(path);raw=path.read_bytes()
    return {'path':str(path.resolve()),'bytes':len(raw),'sha256':hashlib.sha256(raw).hexdigest(),'mode':path.stat().st_mode&0o777}

def main():
    epoch,mode=sys.argv[1:];assert epoch.replace('-','').isalnum()
    out=ROOT/epoch;out.mkdir(exist_ok=False)
    files={REPO/'Cargo.toml',REPO/'Cargo.lock',Path(__file__).resolve()}
    files.update(ROOT.glob('*.py'))
    for name in ['crates','proofs/std','go-tools/go2vir','go-tools/mpk-checker-ref','develop/specs/vectors','fixtures','release/bundles','.cargo']:
        files.update(p for p in (REPO/name).rglob('*') if p.is_file())
    for name in ['rust-toolchain','rust-toolchain.toml']:
        if (REPO/name).is_file():files.add(REPO/name)
    for name in ['source-inputs/corpus.json','source-invocations/corpus.json','contract-modules/corpus-receivers.json']:
        files.add(ROOT.parent/name)
    files.update((ROOT.parent/'source-invocations/generation-linked/certificates').glob('*.mpcert'))
    if mode=='rust':
        files.update([ROOT/'native-final/native-originals/owned/contract-inputs.json',ROOT/'native-final/native-originals/empty/contract-inputs-empty.json'])
    files.update(p for p in (ROOT/'toolchain').rglob('*') if p.is_file())
    native=out/'native-originals'
    sources=[seal(p) for p in sorted(files)]
    with tarfile.open(out/'inputs.tar.gz','x:gz',dereference=True) as archive:
        for p in sorted(files):archive.add(p,arcname=str(p.relative_to(REPO)),recursive=False)
    receipt=out/'inputs.json';receipt.write_text(json.dumps({'basis_commit':'8ac557f1eb3d13ebf6ef7b41b6679075857b1a0a','sources':sources,'complete_archive':seal(out/'inputs.tar.gz')},indent=2)+'\n')
    rust=['cargo','test','--offline','--locked','-p','mpk-vc']
    commands={
        'format-write':['rustfmt','--edition=2021','--config','skip_children=true',*CHANGED],
        'format':['rustfmt','--edition=2021','--check','--config','skip_children=true',*CHANGED],
        'go-format-write':[str(SDK/'bin/gofmt'),'-w',*GO_CHANGED],
        'go-format':[str(SDK/'bin/gofmt'),'-l',*GO_CHANGED],
        'rust':[*rust,'--test','go_practical_contract_inputs','--test','go_practical_source_inputs','--','--nocapture','--test-threads=1'],
        'build':[*rust,'--test','go_practical_contract_inputs','--test','go_practical_source_inputs','--no-run'],
        'contracts':[*rust,'--test','go_practical_contract_modules','--','--nocapture','--test-threads=1'],
        'clippy':['cargo','clippy','--offline','--locked','-p','mpk-vc','--lib','--test','go_practical_contract_inputs','--test','go_practical_source_inputs','--','-D','warnings'],
        'native':[str(SDK/'bin/go'),'test','-json','-count=1','-timeout=40m','-run','^(TestPracticalContractInputs.*|TestPracticalSourceInputsRequireOriginalOwner|TestPracticalSourceInputsKeepOriginalReplacementPackage)$'],
        'go-vet':[str(SDK/'bin/go'),'vet','.'],
        'legacy':['/usr/local/go/bin/go','test','-run','^$','.'],
    }
    env=os.environ.copy()
    changes={'CARGO_NET_OFFLINE':'true','CARGO_PROFILE_TEST_OPT_LEVEL':'1','CARGO_PROFILE_TEST_DEBUG_ASSERTIONS':'true','CARGO_PROFILE_TEST_OVERFLOW_CHECKS':'true','CARGO_INCREMENTAL':'0','RUST_TEST_THREADS':'1'}
    go=mode in ['native','go-vet','go-format','go-format-write','legacy']
    if go:
        changes.update({'GOROOT':str(SDK) if mode!='legacy' else '/usr/local/go','GOTOOLCHAIN':'local','GOENV':'off','GOWORK':'off','GOFLAGS':'-mod=vendor','GOPROXY':'off','GOSUMDB':'off','CGO_ENABLED':'0','GOMAXPROCS':'1','GOCACHE':'/private/tmp/mpk-go-contract-inputs-179-cache','GOMODCACHE':'/private/tmp/mpk-go-contract-inputs-179-mod'})
    if mode=='native':
        native.mkdir(parents=True,exist_ok=False);changes['MPK_GO_PRACTICAL_SYNTAX_EVIDENCE']=str(native)
    if mode=='rust':changes['MPK_GO_CONTRACT_INPUTS_OUTPUT']=str(out/'observations')
    env.update(changes)
    for k in ['RUST_MIN_STACK','MPK_CORE_MAX_STEPS','MPK_CORE_MAX_SECONDS']:env.pop(k,None)
    command=commands[mode]
    report={'status':'running','command':command,'environment':changes,'inputs':seal(receipt),'driver':seal(__file__),
            'tools':{n:seal(shutil.which(n)) for n in ['cargo','rustc','rustfmt']},'started_at_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),
            'selection_reason':'New same-owner complete source/contract handoff, strict independent import and direct contract derivation. Native whole original packages retain unselected bodies and sidecars after original paths/image retirement. Source-only component bytes are unchanged. Recheck its source-input and contract-module consumers; do not repeat unchanged kernels, mathematical proof matrices or unrelated frontends.',
            'full_gate':'Deferred to final T02-W04: ./scripts/check-fast.sh'}
    if go:report['go_tool']=seal(command[0])
    path=out/'result.json';path.write_text(json.dumps(report,indent=2)+'\n');start=time.monotonic()
    with (out/'stdout.log').open('xb') as stdout,(out/'stderr.log').open('xb') as stderr:
        code=subprocess.run(command,cwd=REPO/'go-tools/go2vir' if mode in ['native','go-vet','legacy'] else REPO,env=env,stdout=stdout,stderr=stderr,check=False).returncode
    unchanged=sources==[seal(p) for p in sorted(files)]
    formatting=mode in ['format-write','go-format-write']
    report.update(status='passed' if code==0 and (unchanged or formatting) else 'failed',exit_code=code,seconds=time.monotonic()-start,consumed_inputs_unchanged=unchanged,expected_formatter_mutation=formatting,stdout=seal(out/'stdout.log'),stderr=seal(out/'stderr.log'),finished_at_utc=datetime.datetime.now(datetime.timezone.utc).isoformat())
    path.write_text(json.dumps(report,indent=2)+'\n')
    print(json.dumps({k:report[k] for k in ['status','exit_code','seconds','consumed_inputs_unchanged']}));raise SystemExit(report['status']!='passed')

if __name__=='__main__':main()
