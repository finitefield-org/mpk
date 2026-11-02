"""Complete prelaunch source preservation and affected local handoff checks."""
from pathlib import Path
import datetime,hashlib,json,os,shutil,subprocess,sys,tarfile,time

ROOT=Path(__file__).resolve().parent
REPO=Path.cwd()
SDK=Path('/private/tmp/mpk-go125-20261025-download/go')
CHANGED=['crates/mpk-vc/src/go_practical_type_keys/source_build.rs','crates/mpk-vc/src/go_practical_type_keys/source_package.rs','crates/mpk-vc/src/go_practical_type_keys/source_package/header.rs','crates/mpk-vc/src/go_practical_type_keys/source_package/cgo.rs','crates/mpk-vc/tests/go_practical_cgo_directives.rs','crates/mpk-vc/tests/go_practical_snapshot_inputs.rs']
GO_CHANGED=['go-tools/go2vir/practical_cgo_directives_test.go']

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
    files.update(p for p in (ROOT.parent/'selection-inputs/method-init-go-validity').rglob('*') if p.is_file())
    if mode in ['native','rust','build','clippy']:
        for name in ['namespace/snapshot-inputs.json','namespace/namespace-lookups.json','namespace/raw-native-name-unit.json','pure/complete-capture-inputs.json','wide/complete-capture-inputs.json']:
            files.add(ROOT.parent/'snapshot-inputs/native/native-originals'/name)
        files.add(ROOT.parent/'snapshot-inputs/native-final/native-originals/owned/capture-inputs.json')
    preserved=set((ROOT.parent/'snapshot-inputs/snapshot-storage').glob('*'))
    files.update(preserved)
    for name in ['pure/complete-capture-inputs.json','wide/complete-capture-inputs.json']:
        files.add(ROOT.parent/'snapshot-inputs/native/native-originals'/name)
    files.add(ROOT.parent/'snapshot-inputs/native-final/native-originals/owned/capture-inputs.json')
    if mode in ['native','rust','build','clippy']:
        for packet in [ROOT.parent/'snapshot-inputs/native/native-originals/pure/complete-capture-inputs.json',ROOT.parent/'snapshot-inputs/native/native-originals/wide/complete-capture-inputs.json',ROOT.parent/'snapshot-inputs/native-final/native-originals/owned/capture-inputs.json']:
            for row in json.loads(packet.read_bytes())['snapshot_sources']:
                preserved.add(packet.parent/'original-source-objects'/row['raw_file'])
        files.update(preserved)
    if mode in ['rust','build','clippy']:
        files.update(p for p in (ROOT/'native/native-originals').rglob('*') if p.is_file())
    for old in ['build-constraints/native/native-originals','package-files/native/native-originals','package-files/native-final/native-originals','auxiliary-inputs/native/native-originals']:
        for p in (ROOT.parent/old).rglob('*'):
            if p.is_file():
                files.add(p)
                if 'original-source-objects' in p.parts:preserved.add(p)
    native=out/'native-originals'
    sources=[seal(p) for p in sorted(files)]
    with tarfile.open(out/'inputs.tar.gz','x:gz',dereference=True) as archive:
        for p in sorted(files-preserved):archive.add(p,arcname=str(p.relative_to(REPO)),recursive=False)
    receipt=out/'inputs.json';receipt.write_text(json.dumps({'basis_commit':'f4641e836','sources':sources,'complete_archive':seal(out/'inputs.tar.gz'),'preserved_original_paths':[str(p) for p in sorted(preserved)],'preserved_scope':'The unchanged prior snapshot-storage retains every original raw byte/path/mode. Every consumed original is separately sealed before and after this worker, without repacking or narrowing it.'},indent=2)+'\n')
    rust=['cargo','test','--offline','--locked','-p','mpk-vc']
    commands={
        'format-write':['rustfmt','--edition=2021','--config','skip_children=true',*CHANGED],
        'format':['rustfmt','--edition=2021','--check','--config','skip_children=true',*CHANGED],
        'go-format-write':[str(SDK/'bin/gofmt'),'-w',*GO_CHANGED],
        'go-format':[str(SDK/'bin/gofmt'),'-l',*GO_CHANGED],
        'rust':[*rust,'--test','go_practical_source_build','--test','go_practical_snapshot_inputs','--test','go_practical_package_files','--test','go_practical_cgo_directives','--test','go_practical_auxiliary_inputs','--test','go_practical_source_inputs','--','--nocapture','--test-threads=1'],
        'build':[*rust,'--test','go_practical_source_build','--test','go_practical_snapshot_inputs','--test','go_practical_package_files','--test','go_practical_cgo_directives','--test','go_practical_auxiliary_inputs','--test','go_practical_source_inputs','--no-run'],
        'contracts':[*rust,'--test','go_practical_contract_modules','--','--nocapture','--test-threads=1'],
        'clippy':['cargo','clippy','--offline','--locked','-p','mpk-vc','--lib','--test','go_practical_source_build','--test','go_practical_snapshot_inputs','--test','go_practical_package_files','--test','go_practical_cgo_directives','--test','go_practical_auxiliary_inputs','--test','go_practical_source_inputs','--','-D','warnings'],
        'native':[str(SDK/'bin/go'),'test','-json','-count=1','-timeout=40m','-run','^TestPracticalOriginalCgo(DirectiveHeaders|WholePackageUniverses|OwnedCapture)$'],
        'native-method':[str(SDK/'bin/go'),'test','-json','-count=1','-timeout=40m','-run','^TestPractical(CaptureInputsKeepOriginalOwnedUniverse|Export.*|StandardLibraryOriginalSourcesTypecheckWithNoFilesystemFallback)$'],
        'native-languages':[str(SDK/'bin/go'),'test','-json','-count=1','-timeout=40m','-run','^TestPracticalSourceLanguagesMatchPinnedOriginalParserAndTypes$'],
        'go-vet':[str(SDK/'bin/go'),'vet','.'],
        'legacy':['/usr/local/go/bin/go','test','-run','^$','.'],
    }
    env=os.environ.copy()
    changes={'CARGO_NET_OFFLINE':'true','CARGO_PROFILE_TEST_OPT_LEVEL':'1','CARGO_PROFILE_TEST_DEBUG_ASSERTIONS':'true','CARGO_PROFILE_TEST_OVERFLOW_CHECKS':'true','CARGO_INCREMENTAL':'0','RUST_TEST_THREADS':'1'}
    go=mode in ['native','native-method','native-languages','go-vet','go-format','go-format-write','legacy']
    if go:
        changes.update({'GOROOT':str(SDK) if mode!='legacy' else '/usr/local/go','GOTOOLCHAIN':'local','GOENV':'off','GOWORK':'off','GOFLAGS':'-mod=vendor','GOPROXY':'off','GOSUMDB':'off','CGO_ENABLED':'0','GOMAXPROCS':'1','GOCACHE':'/private/tmp/mpk-go-cgo-directives-187-cache','GOMODCACHE':'/private/tmp/mpk-go-cgo-directives-187-mod'})
    if mode in ['native','native-method','native-languages']:
        native.mkdir(parents=True,exist_ok=False);changes['MPK_GO_PRACTICAL_SYNTAX_EVIDENCE']=str(native)
    if mode=='rust':
        changes.update({'MPK_GO_SNAPSHOT_INPUTS_OUTPUT':str(out/'observations/snapshots'),'MPK_GO_SOURCE_BUILD_OUTPUT':str(out/'observations/build'),'MPK_GO_SOURCE_LANGUAGES_OUTPUT':str(out/'observations/languages'),'MPK_GO_SOURCE_INPUTS_OUTPUT':str(out/'observations/sources'),'MPK_GO_PACKAGE_FILES_OUTPUT':str(out/'observations/packages'),'MPK_GO_AUXILIARY_OUTPUT':str(out/'observations/auxiliary'),'MPK_GO_CGO_DIRECTIVES_OUTPUT':str(out/'observations/cgo')})
    env.update(changes)
    for k in ['RUST_MIN_STACK','MPK_CORE_MAX_STEPS','MPK_CORE_MAX_SECONDS']:env.pop(k,None)
    command=commands[mode]
    report={'status':'running','command':command,'environment':changes,'inputs':seal(receipt),'driver':seal(__file__),
            'tools':{n:seal(shutil.which(n)) for n in ['cargo','rustc','rustfmt']},'started_at_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),
            'selection_reason':'Independently validate the original import-C documentation even when fixed cgo-disabled selection excludes its file. Compare exact native original comment grouping/text, conditions, quoted arguments, SRCDIR/safe-name rules and every original package universe. Exercise the actual original owned capture and coherent changed excluded directives with identical source selection/contracts. Run only21 affected Rust roots, three new Go roots, strict Clippy, Go vet and changed-file formats. Preserve all43-root/zero-or70-error/type domains and32 proof bytes. Unchanged full lexer/parser/language/kernel/foundation corpora are retained without repetition; no cgo build, new source language, retired-language tests or intermediate whole gate.',
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
