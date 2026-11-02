"""Reproduce the old standalone dependency, then test exact repair from zero raws."""
from pathlib import Path
import hashlib,json,os,shutil,subprocess,tarfile,time

REPO=Path.cwd();ROOT=Path(__file__).resolve().parent;DATA=ROOT.parent/'snapshot-inputs'
WORK=Path('/private/tmp/mpk-build-constraints-184-standalone')
TARGET=Path('/private/tmp/mpk-build-constraints-184-cargo-target')
TEST='crates/mpk-vc/tests/go_practical_source_build.rs'
def seal(path):
    raw=path.read_bytes();return {'path':str(path),'bytes':len(raw),'sha256':hashlib.sha256(raw).hexdigest(),'mode':path.stat().st_mode&0o777}
def original_files():return set(WORK.glob(str(DATA.relative_to(REPO))+'/native*/native-originals/**/original-source-objects/*'))
def main():
    out=ROOT/'standalone';out.mkdir(exist_ok=False);WORK.mkdir(exist_ok=False)
    files={REPO/'Cargo.toml',REPO/'Cargo.lock',Path(__file__).resolve(),ROOT/'old-standalone-consumer.rs'}
    for name in ['crates','proofs/std','go-tools/go2vir','go-tools/mpk-checker-ref','develop/specs/vectors','fixtures','release/bundles','.cargo']:
        files.update(p for p in (REPO/name).rglob('*') if p.is_file())
    for name in ['rust-toolchain','rust-toolchain.toml']:
        if (REPO/name).is_file():files.add(REPO/name)
    for name in ['namespace/snapshot-inputs.json','namespace/namespace-lookups.json','namespace/raw-native-name-unit.json','pure/complete-capture-inputs.json','wide/complete-capture-inputs.json']:
        files.add(DATA/'native/native-originals'/name)
    files.add(DATA/'native-final/native-originals/owned/capture-inputs.json');files.add(DATA/'preserve-originals.py')
    files.update(p for p in (ROOT/'native/native-originals').rglob('*') if p.is_file())
    storage=set((DATA/'snapshot-storage').glob('*'));files.update(storage)
    source=[seal(p) for p in sorted(files)]
    with tarfile.open(out/'workspace-inputs.tar.gz','x:gz',dereference=True) as packed:
        for path in sorted(files):
            name=path.relative_to(REPO)
            if path not in storage:packed.add(path,arcname=str(name),recursive=False)
            target=WORK/name;target.parent.mkdir(parents=True,exist_ok=True);shutil.copy2(path,target)
    assert not original_files()
    common={'basis_commit':'7d2fa25c3a4ed6c4f142ea9c0bcd7195bbd14541','sources':source,'archive':seal(out/'workspace-inputs.tar.gz'),'exact_prior_storage_inputs':[seal(p) for p in sorted(storage)],'cwd':str(WORK),'driver':seal(Path(__file__).resolve()),'selection_reason':'The test helper alone changed after the complete15-root production validation. Recompile the actual standalone test at a new absolute CARGO_MANIFEST_DIR, with the complete unchanged snapshot manifest/chunks and all three full original capture packets, but no restored source objects. Reproduce the exact previous reader failure, then replace only that reader and run both affected roots with complete automatic restoration. No production change, source/domain reduction, kernel or retired-language test.','full_T_gate':'Deferred to final T02-W04: ./scripts/check-fast.sh'}
    (out/'inputs.json').write_text(json.dumps(common,indent=2)+'\n')
    command=['cargo','test','--offline','--locked','-p','mpk-vc','--test','go_practical_source_build','--','--nocapture','--test-threads=1']
    original_test=(REPO/TEST).read_bytes();phases=[]
    for phase,reader in [('old',ROOT/'old-standalone-consumer.rs'),('fixed',REPO/TEST)]:
        assert not original_files();(WORK/TEST).write_bytes(reader.read_bytes())
        epoch=out/phase;epoch.mkdir(exist_ok=False)
        before=[seal(WORK/p.relative_to(REPO)) for p in sorted(files)]
        env=os.environ.copy();changes={'CARGO_TARGET_DIR':str(TARGET),'CARGO_NET_OFFLINE':'true','CARGO_PROFILE_TEST_OPT_LEVEL':'1','CARGO_PROFILE_TEST_DEBUG_ASSERTIONS':'true','CARGO_PROFILE_TEST_OVERFLOW_CHECKS':'true','CARGO_INCREMENTAL':'0','RUST_TEST_THREADS':'1','MPK_GO_SOURCE_BUILD_OUTPUT':str(epoch/'observations')};env.update(changes)
        for name in ['RUST_MIN_STACK','MPK_CORE_MAX_STEPS','MPK_CORE_MAX_SECONDS']:env.pop(name,None)
        pre={'command':command,'cwd':str(WORK),'environment':changes,'sources':before,'reader':seal(reader),'original_source_objects_before_launch':0,'complete_original_inputs':seal(out/'inputs.json'),'tools':{name:seal(Path(shutil.which(name))) for name in ['cargo','rustc','python3']}}
        (epoch/'prelaunch.json').write_text(json.dumps(pre,indent=2)+'\n');start=time.monotonic()
        with (epoch/'stdout.log').open('xb') as stdout,(epoch/'stderr.log').open('xb') as stderr:
            code=subprocess.run(command,cwd=WORK,env=env,stdout=stdout,stderr=stderr,check=False).returncode
        unchanged=before==[seal(WORK/p.relative_to(REPO)) for p in sorted(files)]
        status='reproduced_original_missing_snapshot_failure' if phase=='old' and code!=0 and unchanged else 'passed' if phase=='fixed' and code==0 and unchanged else 'failed'
        result={'status':status,'exit_code':code,'seconds':time.monotonic()-start,'prelaunch':seal(epoch/'prelaunch.json'),'stdout':seal(epoch/'stdout.log'),'stderr':seal(epoch/'stderr.log'),'consumed_sources_unchanged':unchanged,'restored_source_objects_after_run':len(original_files()),'whole_T_gate_run':False}
        (epoch/'result.json').write_text(json.dumps(result,indent=2)+'\n');phases.append(result)
        print(json.dumps({'phase':phase,**{k:result[k] for k in ['status','exit_code','seconds','restored_source_objects_after_run']}}),flush=True)
        if phase=='old':
            assert status=='reproduced_original_missing_snapshot_failure' and not original_files()
            assert 'No such file or directory' in (epoch/'stderr.log').read_text()
            assert '1 passed; 1 failed;' in (epoch/'stdout.log').read_text()
        else:assert status=='passed' and '2 passed; 0 failed;' in (epoch/'stdout.log').read_text()
    assert source==[seal(p) for p in sorted(files)] and (WORK/TEST).read_bytes()==original_test
    manifest=json.loads((DATA/'snapshot-storage/manifest.json').read_bytes());snapshot=WORK/DATA.relative_to(REPO)
    restored=set()
    for row in manifest['originals']:
        path=snapshot/row['member'];actual=seal(path)
        assert not path.is_symlink() and all(actual[k]==row[k] for k in ['bytes','sha256','mode']);restored.add(path)
    assert restored==original_files() and len(restored)==77162
    result={'status':'passed','driver':seal(Path(__file__).resolve()),'inputs':seal(out/'inputs.json'),'original_reproduction':seal(out/'old/result.json'),'fixed_standalone':seal(out/'fixed/result.json'),'original_and_relocated_sources_unchanged_at_each_verification_epoch':True,'only_expected_between_epoch_reader_replacement':True,'all_original_files_restored':len(restored),'all_original_bytes_restored':sum(r['bytes'] for r in manifest['originals']),'every_original_relative_path_byte_and_mode_exact':True,'historical_manifest_unchanged':(snapshot/'snapshot-storage/manifest.json').read_bytes()==(DATA/'snapshot-storage/manifest.json').read_bytes(),'full_T_gate':'Deferred to final T02-W04: ./scripts/check-fast.sh'}
    (out/'result.json').write_text(json.dumps(result,indent=2)+'\n');print(json.dumps({'status':'passed','complete_original_files':len(restored)}))

if __name__=='__main__':main()
