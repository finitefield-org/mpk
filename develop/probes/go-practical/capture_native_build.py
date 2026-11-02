"""Build the new observer against unchanged public production source validators."""
import os
import subprocess
import tomllib
from pathlib import Path
from run_proof_pilot import ROOT,HERE,sha
from check_scope_report import require
from frontend_native_build import private_codec_paths

TARGET=Path('/private/tmp/mpk-go-practical-capture-native-target')
FLAGS={'CARGO_INCREMENTAL':'0','CARGO_PROFILE_DEV_DEBUG':'0','CARGO_PROFILE_DEV_CODEGEN_UNITS':'1'}


def build():
    folder=HERE/'capture-native';lock=folder/'Cargo.lock'
    if not lock.exists():lock.write_bytes((ROOT/'Cargo.lock').read_bytes())
    env={**os.environ,**FLAGS,'CARGO_TARGET_DIR':str(TARGET)}
    args=['cargo','build','--offline','--manifest-path',str(folder/'Cargo.toml')]
    prepared=subprocess.run(args,cwd=ROOT,env=env,capture_output=True,timeout=600)
    require(prepared.returncode==0 and b'warning:' not in prepared.stderr,'new public source validator observer build: '+prepared.stderr.decode())
    original={(p['name'],p['version'],p.get('checksum')) for p in tomllib.loads((ROOT/'Cargo.lock').read_text())['package']}
    for p in tomllib.loads(lock.read_text())['package']:
        if 'source' in p:require((p['name'],p['version'],p.get('checksum')) in original,'original dependency version/checksum')
    locked=subprocess.run(args+['--locked'],cwd=ROOT,env=env,capture_output=True,timeout=600)
    require(locked.returncode==0 and b'warning:' not in locked.stderr,'locked public observer build: '+locked.stderr.decode())
    sources=[p for name in ('mpk-core','mpk-cert','mpk-theory','mpk-vc') for p in (ROOT/'crates'/name/'src').rglob('*') if p.is_file()]
    sources=list(set(sources) | private_codec_paths() | {ROOT/'Cargo.toml',ROOT/'Cargo.lock'} | {ROOT/'crates'/n/'Cargo.toml' for n in ('mpk-core','mpk-cert','mpk-theory','mpk-vc')})
    return TARGET/'debug/go-practical-capture-native',{'build_environment':FLAGS,'sources':[{'path':p.relative_to(ROOT).as_posix(),'sha256':sha(p.read_bytes())} for p in sorted(sources)]}
