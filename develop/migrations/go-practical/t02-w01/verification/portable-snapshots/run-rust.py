"""Compile and run the real affected consumers in a relocated complete workspace."""
from pathlib import Path
import datetime,hashlib,json,os,shutil,subprocess,tarfile,time

REPO=Path.cwd()
ROOT=Path(__file__).resolve().parent
DATA=ROOT.parent/'snapshot-inputs'
WORK=Path('/private/tmp/mpk-portable-snapshot-183-workspace')
TARGET=Path('/private/tmp/mpk-portable-snapshot-183-cargo-target')

def seal(path):
    raw=path.read_bytes()
    return {'path':str(path),'bytes':len(raw),'sha256':hashlib.sha256(raw).hexdigest(),'mode':path.stat().st_mode&0o777}

def main():
    out=ROOT/'relocated-rust';out.mkdir(exist_ok=False);WORK.mkdir(exist_ok=False)
    files={REPO/'Cargo.toml',REPO/'Cargo.lock',Path(__file__).resolve()}
    for name in ['crates','proofs/std','go-tools/go2vir','go-tools/mpk-checker-ref','develop/specs/vectors','fixtures','release/bundles','.cargo']:
        files.update(p for p in (REPO/name).rglob('*') if p.is_file())
    for name in ['rust-toolchain','rust-toolchain.toml']:
        if (REPO/name).is_file():files.add(REPO/name)
    for name in ['source-inputs/corpus.json','source-invocations/corpus.json']:
        files.add(ROOT.parent/name)
    files.update((ROOT.parent/'source-invocations/generation-linked/certificates').glob('*.mpcert'))
    for name in ['namespace/snapshot-inputs.json','namespace/namespace-lookups.json','namespace/raw-native-name-unit.json','pure/complete-capture-inputs.json','wide/complete-capture-inputs.json']:
        files.add(DATA/'native/native-originals'/name)
    files.add(DATA/'native-final/native-originals/owned/capture-inputs.json')
    files.add(DATA/'preserve-originals.py')
    storage=set((DATA/'snapshot-storage').glob('*'))
    files.update(storage)
    before=[seal(p) for p in sorted(files)]
    # Existing reversible storage retains every source object; retain all other
    # consumed workspace files in this new epoch, without repacking old bytes.
    with tarfile.open(out/'workspace-inputs.tar.gz','x:gz',dereference=True) as packed:
        for path in sorted(files):
            name=path.relative_to(REPO)
            if path not in storage:packed.add(path,arcname=str(name),recursive=False)
            dest=WORK/name;dest.parent.mkdir(parents=True,exist_ok=True);shutil.copy2(path,dest)
    snapshot=WORK/DATA.relative_to(REPO)
    assert not list(snapshot.glob('native*/native-originals/**/original-source-objects/*'))
    env=os.environ.copy()
    changes={'CARGO_TARGET_DIR':str(TARGET),'CARGO_NET_OFFLINE':'true','CARGO_PROFILE_TEST_OPT_LEVEL':'1','CARGO_PROFILE_TEST_DEBUG_ASSERTIONS':'true','CARGO_PROFILE_TEST_OVERFLOW_CHECKS':'true','CARGO_INCREMENTAL':'0','RUST_TEST_THREADS':'1','MPK_GO_SNAPSHOT_INPUTS_OUTPUT':str(out/'observations/snapshots'),'MPK_GO_SOURCE_INPUTS_OUTPUT':str(out/'observations/sources')}
    env.update(changes)
    for name in ['RUST_MIN_STACK','MPK_CORE_MAX_STEPS','MPK_CORE_MAX_SECONDS']:env.pop(name,None)
    command=['cargo','test','--offline','--locked','-p','mpk-vc','--test','go_practical_snapshot_inputs','--test','go_practical_source_inputs','--','--nocapture','--test-threads=1']
    pre={'basis_commit':'2e1330a6dc4e155399f836002491fd1e2dc13200','command':command,'cwd':str(WORK),'environment':changes,'started_at_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'sources':before,'workspace_archive':seal(out/'workspace-inputs.tar.gz'),'tools':{name:seal(Path(shutil.which(name))) for name in ['cargo','rustc','python3']},'original_snapshot_copies_before_launch':0,'selection_reason':'Only the two direct Rust consumers of the changed automatic snapshot restorer: all five snapshot/capture roots and four complete original source/proof roots. Recompile the actual full workspace source at its new absolute path so CARGO_MANIFEST_DIR changes. Restore every original snapshot member from the complete unchanged original manifest/chunks; no narrowed root/type/case/proof domains. No changed Go/Rust production, kernel or unrelated foundation tests.','full_T_gate':'Deferred to final T02-W04: ./scripts/check-fast.sh'}
    (out/'prelaunch.json').write_text(json.dumps(pre,indent=2)+'\n')
    start=time.monotonic()
    with (out/'stdout.log').open('xb') as stdout,(out/'stderr.log').open('xb') as stderr:
        code=subprocess.run(command,cwd=WORK,env=env,stdout=stdout,stderr=stderr,check=False).returncode
    unchanged=before==[seal(p) for p in sorted(files)]
    copy_unchanged=all({k:seal(WORK/p.relative_to(REPO))[k] for k in ['bytes','sha256','mode']}=={k:r[k] for k in ['bytes','sha256','mode']} for p,r in zip(sorted(files),before))
    record=json.loads((snapshot/'snapshot-storage/manifest.json').read_bytes())
    restored=[]
    for row in record['originals']:
        path=snapshot/row['member'];actual=seal(path)
        assert not path.is_symlink() and all(actual[k]==row[k] for k in ['bytes','sha256','mode'])
        restored.append(path)
    observed={p for p in snapshot.glob('native*/native-originals/**/original-source-objects/*') if p.is_file()}
    assert observed==set(restored)
    report={'status':'passed' if code==0 and unchanged and copy_unchanged else 'failed','exit_code':code,'seconds':time.monotonic()-start,'prelaunch':seal(out/'prelaunch.json'),'stdout':seal(out/'stdout.log'),'stderr':seal(out/'stderr.log'),'original_and_relocated_consumed_inputs_unchanged':unchanged and copy_unchanged,'original_snapshot_members_restored':len(restored),'original_snapshot_bytes_restored':sum(r['bytes'] for r in record['originals']),'all_original_relative_paths_bytes_and_modes_exact':True,'historical_manifest_byte_identical':(snapshot/'snapshot-storage/manifest.json').read_bytes()==(DATA/'snapshot-storage/manifest.json').read_bytes(),'whole_T_gate_run':False}
    (out/'result.json').write_text(json.dumps(report,indent=2)+'\n')
    print(json.dumps({k:report[k] for k in ['status','exit_code','seconds','original_snapshot_members_restored']}))
    raise SystemExit(report['status']!='passed')

if __name__=='__main__':main()
