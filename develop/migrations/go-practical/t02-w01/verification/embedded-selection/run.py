"""Complete prelaunch source preservation and affected local handoff checks."""
from pathlib import Path
import datetime,hashlib,json,os,shutil,subprocess,sys,tarfile,time

ROOT=Path(__file__).resolve().parent
REPO=Path.cwd()
SDK=Path('/private/tmp/mpk-go125-20261025-download/go')
CHANGED=['crates/mpk-vc/src/go_practical_type_keys.rs','crates/mpk-vc/src/go_practical_type_keys/source_package.rs','crates/mpk-vc/src/go_practical_type_keys/source_package/embedded.rs','crates/mpk-vc/src/go_practical_type_keys/source_package/embedded/directives.rs','crates/mpk-vc/src/go_practical_type_keys/source_package/embedded/glob.rs','crates/mpk-vc/tests/go_practical_embedded_selection.rs']
GO_CHANGED=['go-tools/go2vir/practical_embedded_selection_test.go']

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
    if mode in ['native','rust','build','clippy','glob','rust-groups','clippy-groups']:
        for name in ['namespace/snapshot-inputs.json','namespace/namespace-lookups.json','namespace/raw-native-name-unit.json','pure/complete-capture-inputs.json','wide/complete-capture-inputs.json']:
            files.add(ROOT.parent/'snapshot-inputs/native/native-originals'/name)
        files.add(ROOT.parent/'snapshot-inputs/native-final/native-originals/owned/capture-inputs.json')
    preserved=set((ROOT.parent/'snapshot-inputs/snapshot-storage').glob('*'))
    files.update(preserved)
    for name in ['pure/complete-capture-inputs.json','wide/complete-capture-inputs.json']:
        files.add(ROOT.parent/'snapshot-inputs/native/native-originals'/name)
    files.add(ROOT.parent/'snapshot-inputs/native-final/native-originals/owned/capture-inputs.json')
    if mode in ['native','rust','build','clippy','glob','rust-groups','clippy-groups']:
        for packet in [ROOT.parent/'snapshot-inputs/native/native-originals/pure/complete-capture-inputs.json',ROOT.parent/'snapshot-inputs/native/native-originals/wide/complete-capture-inputs.json',ROOT.parent/'snapshot-inputs/native-final/native-originals/owned/capture-inputs.json']:
            for row in json.loads(packet.read_bytes())['snapshot_sources']:
                preserved.add(packet.parent/'original-source-objects'/row['raw_file'])
        files.update(preserved)
    if mode in ['rust','build','clippy','glob','rust-groups','clippy-groups']:
        for name in ['native/native-originals','native-groups/native-originals']:
            files.update(p for p in (ROOT/name).rglob('*') if p.is_file())
    for old in ['build-constraints/native/native-originals','package-files/native/native-originals','package-files/native-final/native-originals','auxiliary-inputs/native/native-originals','cgo-directives/native/native-originals','embedded-inputs/native/native-originals/owned','embedded-inputs/native/native-originals/matrix']:
        for p in (ROOT.parent/old).rglob('*'):
            if p.is_file():
                files.add(p)
                if 'original-source-objects' in p.parts:preserved.add(p)
    external_originals=[]
    if mode=='native':
        original=SDK/'src/path/match_test.go'
        saved=out/'pinned-sdk-match-test.original'
        saved.write_bytes(original.read_bytes());saved.chmod(original.stat().st_mode&0o777)
        files.add(saved);external_originals.append({'original':seal(original),'saved':seal(saved)})
    native=out/'native-originals'
    sources=[seal(p) for p in sorted(files)]
    with tarfile.open(out/'inputs.tar.gz','x:gz',dereference=True) as archive:
        for p in sorted(files-preserved):archive.add(p,arcname=str(p.relative_to(REPO)),recursive=False)
    receipt=out/'inputs.json';receipt.write_text(json.dumps({'basis_commit':'34940c6af','sources':sources,'complete_archive':seal(out/'inputs.tar.gz'),'preserved_original_paths':[str(p) for p in sorted(preserved)],'complete_external_originals':external_originals,'preserved_scope':'The unchanged prior snapshot-storage retains every original raw byte/path/mode. Every consumed original is separately sealed before and after this worker, without repacking or narrowing it.'},indent=2)+'\n')
    rust=['cargo','test','--offline','--locked','-p','mpk-vc']
    targets=[]
    for name in ['go_practical_embedded_selection','go_practical_snapshot_inputs','go_practical_package_files','go_practical_auxiliary_inputs','go_practical_cgo_directives','go_practical_source_inputs']:
        targets.extend(['--test',name])
    commands={
        'format-write':['rustfmt','--edition=2021','--config','skip_children=true',*CHANGED],
        'format':['rustfmt','--edition=2021','--check','--config','skip_children=true',*CHANGED],
        'go-format-write':[str(SDK/'bin/gofmt'),'-w',*GO_CHANGED],
        'go-format':[str(SDK/'bin/gofmt'),'-l',*GO_CHANGED],
        'rust':[*rust,*targets,'--','--nocapture','--test-threads=1'],
        'rust-groups':[*rust,'--test','go_practical_embedded_selection','original_non_test_resources_keep_internal_and_external_test_patterns_separate','--','--nocapture','--test-threads=1'],
        'clippy-groups':['cargo','clippy','--offline','--locked','-p','mpk-vc','--test','go_practical_embedded_selection','--','-D','warnings'],
        'native-groups':[str(SDK/'bin/go'),'test','-json','-count=1','-timeout=40m','-run','^TestPracticalEmbeddedPatternGroupsOriginalBuild$'],
        'glob':[*rust,'--lib','every_actual_original_pinned_path_match_case','--','--nocapture','--test-threads=1'],
        'build':[*rust,'--lib',*targets,'--no-run'],
        'contracts':[*rust,'--test','go_practical_contract_modules','--','--nocapture','--test-threads=1'],
        'clippy':['cargo','clippy','--offline','--locked','-p','mpk-vc','--lib',*targets,'--','-D','warnings'],
        'native':[str(SDK/'bin/go'),'test','-json','-count=1','-timeout=40m','-run','^TestPracticalEmbedded(SelectionOriginalCandidates|DirectivesOriginalBuild|PatternsWholeOriginalUniverses|GlobOriginalPinnedCases)$'],
        'native-method':[str(SDK/'bin/go'),'test','-json','-count=1','-timeout=40m','-run','^TestPractical(CaptureInputsKeepOriginalOwnedUniverse|Export.*|StandardLibraryOriginalSourcesTypecheckWithNoFilesystemFallback)$'],
        'native-languages':[str(SDK/'bin/go'),'test','-json','-count=1','-timeout=40m','-run','^TestPracticalSourceLanguagesMatchPinnedOriginalParserAndTypes$'],
        'go-vet':[str(SDK/'bin/go'),'vet','.'],
        'legacy':['/usr/local/go/bin/go','test','-run','^$','.'],
    }
    env=os.environ.copy()
    changes={'CARGO_NET_OFFLINE':'true','CARGO_PROFILE_TEST_OPT_LEVEL':'1','CARGO_PROFILE_TEST_DEBUG_ASSERTIONS':'true','CARGO_PROFILE_TEST_OVERFLOW_CHECKS':'true','CARGO_INCREMENTAL':'0','RUST_TEST_THREADS':'1'}
    go=mode in ['native','native-groups','native-method','native-languages','go-vet','go-format','go-format-write','legacy']
    if go:
        changes.update({'GOROOT':str(SDK) if mode!='legacy' else '/usr/local/go','GOTOOLCHAIN':'local','GOENV':'off','GOWORK':'off','GOFLAGS':'-mod=vendor','GOPROXY':'off','GOSUMDB':'off','CGO_ENABLED':'0','GOMAXPROCS':'1','GOCACHE':'/private/tmp/mpk-go-embedded-selection-189-cache','GOMODCACHE':'/private/tmp/mpk-go-embedded-selection-189-mod'})
    if mode in ['native','native-groups','native-method','native-languages']:
        native.mkdir(parents=True,exist_ok=False);changes['MPK_GO_PRACTICAL_SYNTAX_EVIDENCE']=str(native)
    if mode in ['rust','glob','rust-groups']:
        changes.update({'MPK_GO_EMBEDDED_OUTPUT':str(out/'observations/embedded'),'MPK_GO_SNAPSHOT_INPUTS_OUTPUT':str(out/'observations/snapshots'),'MPK_GO_SOURCE_BUILD_OUTPUT':str(out/'observations/build'),'MPK_GO_SOURCE_LANGUAGES_OUTPUT':str(out/'observations/languages'),'MPK_GO_SOURCE_INPUTS_OUTPUT':str(out/'observations/sources'),'MPK_GO_PACKAGE_FILES_OUTPUT':str(out/'observations/packages'),'MPK_GO_AUXILIARY_OUTPUT':str(out/'observations/auxiliary'),'MPK_GO_CGO_DIRECTIVES_OUTPUT':str(out/'observations/cgo')})
    env.update(changes)
    for k in ['RUST_MIN_STACK','MPK_CORE_MAX_STEPS','MPK_CORE_MAX_SECONDS']:env.pop(k,None)
    command=commands[mode]
    report={'status':'running','command':command,'environment':changes,'inputs':seal(receipt),'driver':seal(__file__),
            'tools':{n:seal(shutil.which(n)) for n in ['cargo','rustc','rustfmt']},'started_at_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),
            'selection_reason':'Independently derive original Go embed pattern bytes and non-test resource selection from the complete immutable snapshot, without caller-supplied patterns or filesystem fallback. Compare all54 unchanged original native cases including all27 rejections; preserve complete candidate namespaces/buffers before resource failure. Compare original pragma/comment/literal/raw-byte cases, the entire pinned path.Match corpus plus raw-byte vectors, and every2003 original package directory. Exercise the full5535-object native owner, exact resources and coherent omission/unknown-directory controls with the identical original Root contract. Compare nonempty internal/external test patterns and verify that invalid/missing test-resource patterns do not contaminate ordinary production resource selection. Run the new native observations and affected Rust capture/package/auxiliary/cgo/source consumers, strict Clippy, vet and changed-file formats; untouched lexer/parser/build/language/kernel/foundation matrices and original Go owner/export/invocation production retain prior full evidence without repeat. Keep both43-root/zero-or70-error units/type domains/all32 proof streams/frozen limits intact. Canonical source/compiler/root/context/program authority remains due.',
            'full_gate':'Deferred to final T02-W04: ./scripts/check-fast.sh'}
    if go:report['go_tool']=seal(command[0])
    path=out/'result.json';path.write_text(json.dumps(report,indent=2)+'\n');start=time.monotonic()
    with (out/'stdout.log').open('xb') as stdout,(out/'stderr.log').open('xb') as stderr:
        code=subprocess.run(command,cwd=REPO/'go-tools/go2vir' if mode in ['native','native-groups','native-method','native-languages','go-vet','legacy'] else REPO,env=env,stdout=stdout,stderr=stderr,check=False).returncode
    external_unchanged=all(row['original']==seal(row['original']['path']) for row in external_originals)
    unchanged=sources==[seal(p) for p in sorted(files)] and external_unchanged
    formatting=mode in ['format-write','go-format-write']
    report.update(status='passed' if code==0 and (unchanged or formatting) else 'failed',exit_code=code,seconds=time.monotonic()-start,consumed_inputs_unchanged=unchanged,expected_formatter_mutation=formatting,external_originals_unchanged=external_unchanged,stdout=seal(out/'stdout.log'),stderr=seal(out/'stderr.log'),finished_at_utc=datetime.datetime.now(datetime.timezone.utc).isoformat())
    path.write_text(json.dumps(report,indent=2)+'\n')
    print(json.dumps({k:report[k] for k in ['status','exit_code','seconds','consumed_inputs_unchanged']}));raise SystemExit(report['status']!='passed')

if __name__=='__main__':main()
