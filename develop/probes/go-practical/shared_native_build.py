"""Private source mirror with an appended observation bridge; no production edits."""
import os
import shutil
import subprocess
import tomllib
from pathlib import Path
from run_proof_pilot import ROOT,HERE,sha
from check_scope_report import require

WORK=Path('/private/tmp/mpk-go-practical-shared-native-workspace')
TARGET=Path('/private/tmp/mpk-go-practical-shared-native-target')


def build():
    WORK.mkdir(exist_ok=True);marker=WORK/'.mpk-go-shared-probe'
    if marker.exists():require(marker.read_text()=='owned private Go shared-artifact probe\n','owned private build workspace')
    else:
        require(not list(WORK.iterdir()),'new empty private build workspace');marker.write_text('owned private Go shared-artifact probe\n')
    cargo=(ROOT/'Cargo.toml').read_text();start=cargo.index('members = [');end=cargo.index(']',start)+1
    cargo=cargo[:start]+'members = ["crates/mpk-core","crates/mpk-cert","crates/mpk-kernel","crates/mpk-theory","crates/mpk-vc","probe"]'+cargo[end:]
    (WORK/'Cargo.toml').write_text(cargo);(WORK/'Cargo.lock').write_bytes((ROOT/'Cargo.lock').read_bytes())
    rows=[]
    for name in ('mpk-core','mpk-cert','mpk-kernel','mpk-theory','mpk-vc'):
        origin=ROOT/'crates'/name;dest=WORK/'crates'/name;dest.mkdir(parents=True,exist_ok=True)
        (dest/'Cargo.toml').write_bytes((origin/'Cargo.toml').read_bytes())
        if (dest/'src').exists():shutil.rmtree(dest/'src')
        shutil.copytree(origin/'src',dest/'src')
        for p in (origin/'src').rglob('*'):
            if p.is_file():require((dest/'src'/p.relative_to(origin/'src')).read_bytes()==p.read_bytes(),'unchanged mirrored source');rows.append({'path':p.relative_to(ROOT).as_posix(),'sha256':sha(p.read_bytes())})
    for name in ('develop','release','proofs','README.md'):
        dest=WORK/name
        if not dest.exists():dest.symlink_to(ROOT/name,target_is_directory=(ROOT/name).is_dir())
    bridge=(HERE/'shared-native/bridge.rs').read_bytes();source=WORK/'crates/mpk-vc/src/successor_vc.rs';prefix=source.read_bytes()
    source.write_bytes(prefix+b'\n'+bridge)
    require(source.read_bytes()==(ROOT/'crates/mpk-vc/src/successor_vc.rs').read_bytes()+b'\n'+bridge,'only appended observation facade')
    probe=WORK/'probe';(probe/'src').mkdir(parents=True,exist_ok=True)
    (probe/'Cargo.toml').write_text('[package]\nname="go-practical-shared-native"\nversion="0.1.0"\nedition="2021"\n[dependencies]\nmpk-vc={path="../crates/mpk-vc"}\nserde_json="=1.0.150"\nsha2="=0.10.9"\n')
    (probe/'src/main.rs').write_bytes((HERE/'shared-native/main.rs').read_bytes())
    flags={'CARGO_INCREMENTAL':'0','CARGO_PROFILE_DEV_DEBUG':'0','CARGO_PROFILE_DEV_CODEGEN_UNITS':'1'}
    env={**os.environ,'CARGO_TARGET_DIR':str(TARGET),**flags}
    # Resolve only the new local observer against the copied root lock; never
    # generate a fresh latest-compatible dependency set.
    lock=subprocess.run(['cargo','build','--offline','-p','go-practical-shared-native'],cwd=WORK,env=env,capture_output=True,timeout=600)
    require(lock.returncode==0 and b'warning:' not in lock.stderr,'private locked-dependency preparation: '+lock.stderr.decode())
    original={(p['name'],p['version'],p.get('checksum')) for p in tomllib.loads((ROOT/'Cargo.lock').read_text())['package']}
    for p in tomllib.loads((WORK/'Cargo.lock').read_text())['package']:
        if 'source' in p:require((p['name'],p['version'],p.get('checksum')) in original,'unchanged original dependency versions/checksums')
    result=subprocess.run(['cargo','build','--offline','--locked','-p','go-practical-shared-native'],cwd=WORK,env=env,capture_output=True,timeout=600)
    require(result.returncode==0 and b'warning:' not in result.stderr,'unchanged retained functions plus private bridge build: '+result.stderr.decode())
    return TARGET/'debug/go-practical-shared-native',{'sources':sorted(rows,key=lambda r:r['path']),'bridge_sha256':sha(bridge),'modified_mirror_file':'crates/mpk-vc/src/successor_vc.rs','original_prefix_sha256':sha(prefix),'build_environment':flags}
