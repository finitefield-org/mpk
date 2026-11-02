"""Complete prelaunch source preservation and affected local handoff checks."""
from pathlib import Path
import datetime,hashlib,json,os,shutil,subprocess,sys,tarfile,time

ROOT=Path(__file__).resolve().parent
REPO=Path.cwd()
SDK=Path('/private/tmp/mpk-go125-20261025-download/go')
CHANGED=['crates/mpk-vc/src/go_practical_type_keys/source_overlays/module.rs','crates/mpk-vc/src/go_practical_type_keys/source_overlays.rs','crates/mpk-vc/src/go_practical_type_keys/selection_inputs.rs','crates/mpk-vc/tests/go_practical_selection_inputs.rs','crates/mpk-vc/src/go_practical_type_keys.rs']
GO_CHANGED=['go-tools/go2vir/practical_contract_inputs.go','go-tools/go2vir/practical_selection_inputs.go','go-tools/go2vir/practical_selection_inputs_test.go','go-tools/go2vir/practical_source_invocation_test.go']

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
    for name in ['source-inputs/corpus.json','source-invocations/corpus.json','contract-modules/corpus-receivers.json','source-execution/program.go','source-invocations/additions.go.txt']:
        files.add(ROOT.parent/name)
    files.update((ROOT.parent/'source-invocations/generation-linked/certificates').glob('*.mpcert'))
    for name in ['owned/contract-inputs.json','empty/contract-inputs-empty.json']:
        files.add(ROOT.parent/'contract-inputs/native-final/native-originals'/name)
    for name in ['native-final/native-originals/probe/language-corpus.json','native/native-originals/owned/language-owned-inputs.json','native/native-originals/owned/language-owned-versions.json']:
        files.add(ROOT.parent/'source-languages'/name)
    if mode=='rust':
        for name in ['selection-owned/selection-inputs.json','selection-owned/selection-boundary-inputs.json','selection-owned/physical-root-observations.json','pure/physical-selection-inputs.json','wide/physical-selection-inputs.json']:
            files.add(ROOT/'native/native-originals'/name)
    files.update(p for p in (ROOT/'spec-reference').rglob('*') if p.is_file())
    files.update(p for p in (ROOT/'method-init-go-validity').rglob('*') if p.is_file())
    if mode=='rust':
        files.update([ROOT/'native-final/native-originals/method-init/selection-method-init.json',ROOT/'native-final/native-originals/method-init/method-init-observations.json'])
    native=out/'native-originals'
    sources=[seal(p) for p in sorted(files)]
    with tarfile.open(out/'inputs.tar.gz','x:gz',dereference=True) as archive:
        for p in sorted(files):archive.add(p,arcname=str(p.relative_to(REPO)),recursive=False)
    receipt=out/'inputs.json';receipt.write_text(json.dumps({'basis_commit':'69eb69236bfc25ca8227559eee6fe5f2c29cc16d','sources':sources,'complete_archive':seal(out/'inputs.tar.gz')},indent=2)+'\n')
    rust=['cargo','test','--offline','--locked','-p','mpk-vc']
    commands={
        'format-write':['rustfmt','--edition=2021','--config','skip_children=true',*CHANGED],
        'format':['rustfmt','--edition=2021','--check','--config','skip_children=true',*CHANGED],
        'go-format-write':[str(SDK/'bin/gofmt'),'-w',*GO_CHANGED],
        'go-format':[str(SDK/'bin/gofmt'),'-l',*GO_CHANGED],
        'rust':[*rust,'--test','go_practical_selection_inputs','--test','go_practical_contract_modules','--test','go_practical_source_languages','--test','go_practical_contract_inputs','--test','go_practical_source_inputs','--','--nocapture','--test-threads=1'],
        'build':[*rust,'--test','go_practical_selection_inputs','--test','go_practical_contract_modules','--test','go_practical_source_languages','--test','go_practical_contract_inputs','--test','go_practical_source_inputs','--no-run'],
        'contracts':[*rust,'--test','go_practical_contract_modules','--','--nocapture','--test-threads=1'],
        'clippy':['cargo','clippy','--offline','--locked','-p','mpk-vc','--lib','--test','go_practical_selection_inputs','--test','go_practical_contract_modules','--test','go_practical_source_languages','--test','go_practical_contract_inputs','--test','go_practical_source_inputs','--','-D','warnings'],
        'native':[str(SDK/'bin/go'),'test','-json','-count=1','-timeout=40m','-run','^TestPractical(SelectionInputs|ContractInputs|SourceInvocationOriginalSignatures)'],
        'native-method':[str(SDK/'bin/go'),'test','-json','-count=1','-timeout=40m','-run','^TestPracticalSelectionInputsMethodInitIsOrdinary$'],
        'native-languages':[str(SDK/'bin/go'),'test','-json','-count=1','-timeout=40m','-run','^TestPracticalSourceLanguagesMatchPinnedOriginalParserAndTypes$'],
        'go-vet':[str(SDK/'bin/go'),'vet','.'],
        'legacy':['/usr/local/go/bin/go','test','-run','^$','.'],
    }
    env=os.environ.copy()
    changes={'CARGO_NET_OFFLINE':'true','CARGO_PROFILE_TEST_OPT_LEVEL':'1','CARGO_PROFILE_TEST_DEBUG_ASSERTIONS':'true','CARGO_PROFILE_TEST_OVERFLOW_CHECKS':'true','CARGO_INCREMENTAL':'0','RUST_TEST_THREADS':'1'}
    go=mode in ['native','native-method','native-languages','go-vet','go-format','go-format-write','legacy']
    if go:
        changes.update({'GOROOT':str(SDK) if mode!='legacy' else '/usr/local/go','GOTOOLCHAIN':'local','GOENV':'off','GOWORK':'off','GOFLAGS':'-mod=vendor','GOPROXY':'off','GOSUMDB':'off','CGO_ENABLED':'0','GOMAXPROCS':'1','GOCACHE':'/private/tmp/mpk-go-selection-inputs-181-cache','GOMODCACHE':'/private/tmp/mpk-go-selection-inputs-181-mod'})
    if mode in ['native','native-method','native-languages']:
        native.mkdir(parents=True,exist_ok=False);changes['MPK_GO_PRACTICAL_SYNTAX_EVIDENCE']=str(native)
    if mode=='rust':changes['MPK_GO_SELECTION_INPUTS_OUTPUT']=str(out/'observations')
    env.update(changes)
    for k in ['RUST_MIN_STACK','MPK_CORE_MAX_STEPS','MPK_CORE_MAX_SECONDS']:env.pop(k,None)
    command=commands[mode]
    report={'status':'running','command':command,'environment':changes,'inputs':seal(receipt),'driver':seal(__file__),
            'tools':{n:seal(shutil.which(n)) for n in ['cargo','rustc','rustfmt']},'started_at_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),
            'selection_reason':'Bind complete original physical function roots and the exact owned source/sidecar handoff. Compare independent physical parsing with actual original Go declarations, same-name receiver methods and every original root of both complete zero/70-case source units. Exercise the exact8192 root boundary and8193 rejection without changing source/proof scopes or budgets. Recheck shared contract-module consumers, source-language guarding, contract-input ownership and full32 source-input call-proof byte replay. No unchanged kernel, full foundation or retired-language tests.',
            'full_gate':'Deferred to final T02-W04: ./scripts/check-fast.sh'}
    if go:report['go_tool']=seal(command[0])
    path=out/'result.json';path.write_text(json.dumps(report,indent=2)+'\n');start=time.monotonic()
    with (out/'stdout.log').open('xb') as stdout,(out/'stderr.log').open('xb') as stderr:
        code=subprocess.run(command,cwd=REPO/'go-tools/go2vir' if mode in ['native','native-method','native-languages','go-vet','legacy'] else REPO,env=env,stdout=stdout,stderr=stderr,check=False).returncode
    unchanged=sources==[seal(p) for p in sorted(files)]
    formatting=mode in ['format-write','go-format-write']
    report.update(status='passed' if code==0 and (unchanged or formatting) else 'failed',exit_code=code,seconds=time.monotonic()-start,consumed_inputs_unchanged=unchanged,expected_formatter_mutation=formatting,stdout=seal(out/'stdout.log'),stderr=seal(out/'stderr.log'),finished_at_utc=datetime.datetime.now(datetime.timezone.utc).isoformat())
    path.write_text(json.dumps(report,indent=2)+'\n')
    print(json.dumps({k:report[k] for k in ['status','exit_code','seconds','consumed_inputs_unchanged']}));raise SystemExit(report['status']!='passed')

if __name__=='__main__':main()
