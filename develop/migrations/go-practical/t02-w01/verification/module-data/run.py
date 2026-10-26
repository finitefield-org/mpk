"""Complete prelaunch source preservation and affected local handoff checks."""
from pathlib import Path
import datetime,hashlib,json,os,shutil,subprocess,sys,tarfile,time

ROOT=Path(__file__).resolve().parent
REPO=Path.cwd()
SDK=Path('/private/tmp/mpk-go125-20261025-download/go')
CHANGED=['crates/mpk-vc/src/go_practical_type_keys.rs','crates/mpk-vc/src/go_practical_type_keys/module_observations.rs','crates/mpk-vc/src/go_practical_type_keys/snapshot_inputs.rs','crates/mpk-vc/src/go_practical_type_keys/module_observations/data.rs','crates/mpk-vc/tests/go_practical_module_data.rs',*sorted(str(p.relative_to(REPO)) for p in (REPO/'crates/mpk-vc/src/go_practical_type_keys/module_observations/data').rglob('*.rs'))]
GO_CHANGED=['go-tools/go2vir/practical_module_data_oracle_test.go','go-tools/go2vir/practical_module_data_compat_test.go']

def seal(path):
    path=Path(path);raw=path.read_bytes()
    return {'path':str(path.resolve()),'bytes':len(raw),'sha256':hashlib.sha256(raw).hexdigest(),'mode':path.stat().st_mode&0o777}

def main():
    epoch,mode=sys.argv[1:];assert epoch.replace('-','').isalnum()
    out=ROOT/epoch;out.mkdir(exist_ok=False)
    files={REPO/'Cargo.toml',REPO/'Cargo.lock',Path(__file__).resolve()}
    files.update(ROOT.glob('*.py'))
    files.add(REPO/'develop/probes/go-practical/module-cases.json')
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
    if mode in ['native','native-compat','rust','controls-remaining','build','clippy','glob','rust-groups','clippy-groups']:
        for name in ['namespace/snapshot-inputs.json','namespace/namespace-lookups.json','namespace/raw-native-name-unit.json','pure/complete-capture-inputs.json','wide/complete-capture-inputs.json']:
            files.add(ROOT.parent/'snapshot-inputs/native/native-originals'/name)
        files.add(ROOT.parent/'snapshot-inputs/native-final/native-originals/owned/capture-inputs.json')
    preserved=set((ROOT.parent/'snapshot-inputs/snapshot-storage').glob('*'))
    files.update(preserved)
    for name in ['pure/complete-capture-inputs.json','wide/complete-capture-inputs.json']:
        files.add(ROOT.parent/'snapshot-inputs/native/native-originals'/name)
    files.add(ROOT.parent/'snapshot-inputs/native-final/native-originals/owned/capture-inputs.json')
    if mode in ['native','native-compat','rust','controls-remaining','build','clippy','glob','rust-groups','clippy-groups']:
        for packet in [ROOT.parent/'snapshot-inputs/native/native-originals/pure/complete-capture-inputs.json',ROOT.parent/'snapshot-inputs/native/native-originals/wide/complete-capture-inputs.json',ROOT.parent/'snapshot-inputs/native-final/native-originals/owned/capture-inputs.json']:
            for row in json.loads(packet.read_bytes())['snapshot_sources']:
                preserved.add(packet.parent/'original-source-objects'/row['raw_file'])
        files.update(preserved)
    if mode in ['rust','controls-remaining','glob','rust-groups','fold']:
        for name in ['native/native-originals','native-compat/native-originals']:
            files.update(p for p in (ROOT/name).rglob('*') if p.is_file())
    for old in ['module-observations/native/native-originals','export-inputs/native/native-originals','build-constraints/native/native-originals','package-files/native/native-originals','package-files/native-final/native-originals','auxiliary-inputs/native/native-originals','cgo-directives/native/native-originals','embedded-inputs/native/native-originals/owned','embedded-inputs/native/native-originals/matrix','embedded-selection/native/native-originals','embedded-selection/native-groups/native-originals']:
        for p in (ROOT.parent/old).rglob('*'):
            if p.is_file():
                files.add(p)
                if 'original-source-objects' in p.parts:preserved.add(p)
    external_originals=[]
    dependencies=json.loads((ROOT/'dependency-sources/manifest.json').read_bytes())
    files.add(ROOT/'dependency-sources/manifest.json')
    for package in dependencies['packages']:
        files.add(Path(package['archive']['retained_path']))
        external_originals.append({'original':seal(package['archive']['path']),'retained':seal(package['archive']['retained_path'])})
        for row in package['sources']:
            external_originals.append({'original':seal(row['path']),'retained':seal(row['retained_path'])})
            files.add(Path(row['retained_path']))
    files.update(p for p in (ROOT/'primary').rglob('*') if p.is_file())
    native=out/'native-originals'
    sources=[seal(p) for p in sorted(files)]
    with tarfile.open(out/'inputs.tar.gz','x:gz',dereference=True) as archive:
        for p in sorted(files-preserved):archive.add(p,arcname=str(p.relative_to(REPO)),recursive=False)
    receipt=out/'inputs.json';receipt.write_text(json.dumps({'basis_commit':'e076521003e58377c1954c6944e62f9d55330a5f','sources':sources,'complete_archive':seal(out/'inputs.tar.gz'),'preserved_original_paths':[str(p) for p in sorted(preserved)],'complete_external_originals':external_originals,'preserved_scope':'The unchanged prior snapshot-storage retains every original raw byte/path/mode. Every consumed original is separately sealed before and after this worker, without repacking or narrowing it.'},indent=2)+'\n')
    rust=['cargo','test','--offline','--locked','-p','mpk-vc']
    targets=[]
    for name in ['go_practical_module_data','go_practical_export_inputs','go_practical_snapshot_inputs','go_practical_embedded_selection','go_practical_source_inputs']:
        targets.extend(['--test',name])
    commands={
        'controls-remaining':[*rust,'--test','go_practical_module_data','--test','go_practical_snapshot_inputs','--test','go_practical_source_inputs','--','--nocapture','--test-threads=1','--skip','actual_original_versioned_and_local_module_universes_verify_without_host_inputs','--skip','every_original_go_sum_line_matches_the_pinned_native_parser','--skip','every_whole_original_zip_matches_native_go_at_full_protocol_boundaries'],
        'clippy-control':['cargo','clippy','--offline','--locked','-p','mpk-vc','--test','go_practical_module_data','--','-D','warnings'],
        'format-write-control':['rustfmt','--edition=2021','--config','skip_children=true','crates/mpk-vc/tests/go_practical_module_data.rs'],
        'format-control':['rustfmt','--edition=2021','--check','--config','skip_children=true','crates/mpk-vc/tests/go_practical_module_data.rs'],
        'deps':['cargo','metadata','--offline','--format-version=1'],
        'fold':[*rust,'--lib','whole_pinned_go_unicode_fold_agrees_for_every_scalar','--','--nocapture','--test-threads=1'],
        'format-write':['rustfmt','--edition=2021','--config','skip_children=true',*CHANGED],
        'format':['rustfmt','--edition=2021','--check','--config','skip_children=true',*CHANGED],
        'go-format-write':[str(SDK/'bin/gofmt'),'-w',*GO_CHANGED],
        'go-format':[str(SDK/'bin/gofmt'),'-l',*GO_CHANGED],
        'rust':[*rust,*targets,'--','--nocapture','--test-threads=1'],
        'rust-groups':[*rust,'--test','go_practical_embedded_selection','original_non_test_resources_keep_internal_and_external_test_patterns_separate','--','--nocapture','--test-threads=1'],
        'clippy-groups':['cargo','clippy','--offline','--locked','-p','mpk-vc','--test','go_practical_embedded_selection','--','-D','warnings'],
        'native-capture':[str(SDK/'bin/go'),'test','-json','-count=1','-timeout=40m','-run','^TestPracticalCaptureInputsKeepOriginalOwnedUniverse$'],
        'native-groups':[str(SDK/'bin/go'),'test','-json','-count=1','-timeout=40m','-run','^TestPracticalEmbeddedPatternGroupsOriginalBuild$'],
        'glob':[*rust,'--lib','every_actual_original_pinned_path_match_case','--','--nocapture','--test-threads=1'],
        'build':[*rust,'--lib',*targets,'--no-run'],
        'contracts':[*rust,'--test','go_practical_contract_modules','--','--nocapture','--test-threads=1'],
        'clippy':['cargo','clippy','--offline','--locked','-p','mpk-vc','--lib',*targets,'--','-D','warnings'],
        'native-compat':[str(SDK/'bin/go'),'test','-json','-count=1','-timeout=40m','-run','^TestPracticalModuleDataZIPCompatibilityOracle$'],
        'native':[str(SDK/'bin/go'),'test','-json','-count=1','-timeout=40m','-run','^TestPractical(Archive|ModuleSums|ModuleDataOriginalOracle)'],
        'native-method':[str(SDK/'bin/go'),'test','-json','-count=1','-timeout=40m','-run','^TestPractical(CaptureInputsKeepOriginalOwnedUniverse|Export.*|StandardLibraryOriginalSourcesTypecheckWithNoFilesystemFallback)$'],
        'native-languages':[str(SDK/'bin/go'),'test','-json','-count=1','-timeout=40m','-run','^TestPracticalSourceLanguagesMatchPinnedOriginalParserAndTypes$'],
        'go-vet':[str(SDK/'bin/go'),'vet','.'],
        'legacy':['/usr/local/go/bin/go','test','-run','^$','.'],
    }
    env=os.environ.copy()
    changes={'CARGO_NET_OFFLINE':'true','CARGO_PROFILE_TEST_OPT_LEVEL':'1','CARGO_PROFILE_TEST_DEBUG_ASSERTIONS':'true','CARGO_PROFILE_TEST_OVERFLOW_CHECKS':'true','CARGO_INCREMENTAL':'0','RUST_TEST_THREADS':'1'}
    go=mode in ['native','native-compat','native-capture','native-groups','native-method','native-languages','go-vet','go-format','go-format-write','legacy']
    if go:
        changes.update({'GOROOT':str(SDK) if mode!='legacy' else '/usr/local/go','GOTOOLCHAIN':'local','GOENV':'off','GOWORK':'off','GOFLAGS':'-mod=vendor','GOPROXY':'off','GOSUMDB':'off','CGO_ENABLED':'0','GOMAXPROCS':'1','GOCACHE':'/private/tmp/mpk-go-export-inputs-190-cache','GOMODCACHE':'/private/tmp/mpk-go-module-observations-191-mod'})
    if mode in ['native','native-compat','native-capture','native-groups','native-method','native-languages']:
        native.mkdir(parents=True,exist_ok=False);changes['MPK_GO_PRACTICAL_SYNTAX_EVIDENCE']=str(native)
    if mode in ['rust','controls-remaining','glob','rust-groups','fold']:
        changes.update({'MPK_GO_MODULE_DATA_OUTPUT':str(out/'observations/data'),'MPK_GO_EXPORT_OUTPUT':str(out/'observations/exports'),'MPK_GO_EMBEDDED_OUTPUT':str(out/'observations/embedded'),'MPK_GO_SNAPSHOT_INPUTS_OUTPUT':str(out/'observations/snapshots'),'MPK_GO_SOURCE_BUILD_OUTPUT':str(out/'observations/build'),'MPK_GO_SOURCE_LANGUAGES_OUTPUT':str(out/'observations/languages'),'MPK_GO_SOURCE_INPUTS_OUTPUT':str(out/'observations/sources'),'MPK_GO_PACKAGE_FILES_OUTPUT':str(out/'observations/packages'),'MPK_GO_AUXILIARY_OUTPUT':str(out/'observations/auxiliary'),'MPK_GO_CGO_DIRECTIVES_OUTPUT':str(out/'observations/cgo')})
    env.update(changes)
    for k in ['RUST_MIN_STACK','MPK_CORE_MAX_STEPS','MPK_CORE_MAX_SECONDS']:env.pop(k,None)
    command=commands[mode]
    report={'status':'running','command':command,'environment':changes,'inputs':seal(receipt),'driver':seal(__file__),
            'tools':{n:seal(shutil.which(n)) for n in ['cargo','rustc','rustfmt']},'started_at_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),
            'selection_reason':'Independently verify every original Go sum entry, canonical module identity, full module ZIP namespace/content/CRC/Hash1, cached manifest/info/ziphash and captured expanded sources. Preserve complete original native module/source/export/proof universes. Compare pinned Go over complete original sum/archive cases, all Unicode scalars, full500MiB expansion and131072 namespace boundaries; run affected Rust consumers, strict Clippy and changed-file formats. Keep general MVS, complete context/root/compiler/source/program authority and native Linux release gates open.',
            'full_gate':'Deferred to final T02-W04: ./scripts/check-fast.sh'}
    if go:report['go_tool']=seal(command[0])
    path=out/'result.json';path.write_text(json.dumps(report,indent=2)+'\n');start=time.monotonic()
    with (out/'stdout.log').open('xb') as stdout,(out/'stderr.log').open('xb') as stderr:
        code=subprocess.run(command,cwd=REPO/'go-tools/go2vir' if mode in ['native','native-compat','native-capture','native-groups','native-method','native-languages','go-vet','legacy'] else REPO,env=env,stdout=stdout,stderr=stderr,check=False).returncode
    external_unchanged=all(row['original']==seal(row['original']['path']) for row in external_originals)
    unchanged=sources==[seal(p) for p in sorted(files)] and external_unchanged
    formatting=mode in ['format-write','format-write-control','go-format-write','deps']
    report.update(status='passed' if code==0 and (unchanged or formatting) else 'failed',exit_code=code,seconds=time.monotonic()-start,consumed_inputs_unchanged=unchanged,expected_formatter_mutation=formatting,external_originals_unchanged=external_unchanged,stdout=seal(out/'stdout.log'),stderr=seal(out/'stderr.log'),finished_at_utc=datetime.datetime.now(datetime.timezone.utc).isoformat())
    path.write_text(json.dumps(report,indent=2)+'\n')
    print(json.dumps({k:report[k] for k in ['status','exit_code','seconds','consumed_inputs_unchanged']}));raise SystemExit(report['status']!='passed')

if __name__=='__main__':main()
