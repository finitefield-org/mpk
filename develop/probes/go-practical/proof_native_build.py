"""Build the new public retained VC/skeleton observer with unchanged dependencies."""
import os
import subprocess
import tomllib
from pathlib import Path
from run_proof_pilot import ROOT,HERE,sha
from check_scope_report import require
from frontend_native_build import private_codec_paths

TARGET=Path('/private/tmp/mpk-go-practical-proof-native-target')
FLAGS={'CARGO_INCREMENTAL':'0','CARGO_PROFILE_DEV_DEBUG':'0','CARGO_PROFILE_DEV_CODEGEN_UNITS':'1'}


def build():
    folder=HERE/'proof-native';lock=folder/'Cargo.lock'
    if not lock.exists():lock.write_bytes((ROOT/'Cargo.lock').read_bytes())
    args=['cargo','build','--offline','--manifest-path',str(folder/'Cargo.toml')];env={**os.environ,**FLAGS,'CARGO_TARGET_DIR':str(TARGET)}
    p=subprocess.run(args,cwd=ROOT,env=env,capture_output=True,timeout=600)
    require(p.returncode==0 and b'warning:' not in p.stderr,'public VC observer build: '+p.stderr.decode())
    original={(p['name'],p['version'],p.get('checksum')) for p in tomllib.loads((ROOT/'Cargo.lock').read_text())['package']}
    for p in tomllib.loads(lock.read_text())['package']:
        if 'source' in p:require((p['name'],p['version'],p.get('checksum')) in original,'retained dependency versions/checksums')
    p=subprocess.run(args+['--locked'],cwd=ROOT,env=env,capture_output=True,timeout=600)
    require(p.returncode==0 and b'warning:' not in p.stderr,'locked public VC observer build: '+p.stderr.decode())
    paths=[p for n in ('mpk-core','mpk-cert','mpk-theory','mpk-vc') for p in (ROOT/'crates'/n/'src').rglob('*') if p.is_file()]
    paths=list(set(paths) | private_codec_paths() | {ROOT/'Cargo.toml',ROOT/'Cargo.lock'} | {ROOT/'crates'/n/'Cargo.toml' for n in ('mpk-core','mpk-cert','mpk-theory','mpk-vc')})
    return TARGET/'debug/go-practical-proof-native',{'build_environment':FLAGS,'sources':[{'path':p.relative_to(ROOT).as_posix(),'sha256':sha(p.read_bytes())} for p in sorted(paths)]}
