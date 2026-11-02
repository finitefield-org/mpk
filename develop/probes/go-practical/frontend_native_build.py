"""Build a new observer against unchanged public CLI/VC/core consumers."""
import os
import re
import subprocess
import tomllib
from pathlib import Path
from run_proof_pilot import ROOT,HERE,sha
from check_scope_report import require

TARGET=Path('/private/tmp/mpk-go-practical-frontend-native-target')
FLAGS={'CARGO_INCREMENTAL':'0','CARGO_PROFILE_DEV_DEBUG':'0','CARGO_PROFILE_DEV_CODEGEN_UNITS':'1'}
CRATES=('mpk-api','mpk-cli','mpk-cert','mpk-core','mpk-kernel','mpk-theory','mpk-vc')


def private_codec_paths():
    codec=ROOT/'third_party/ruzstd-go125'
    require(codec.is_dir() and (codec/'Cargo.toml').is_file() and (codec/'src/mpk_go125_blocks.rs').is_file(),'complete private Go codec source package')
    return {p for p in codec.rglob('*') if p.is_file()}


def linked_inputs():
    paths={ROOT/'Cargo.toml',ROOT/'Cargo.lock',ROOT/'release/bundles/bundle-registry.json',ROOT/'release/bundles/semantic-profile-registry.json'}
    paths.update(private_codec_paths())
    for name in CRATES:
        crate=ROOT/'crates'/name;paths.add(crate/'Cargo.toml')
        if (crate/'build.rs').exists():paths.add(crate/'build.rs')
        for p in (crate/'src').rglob('*.rs'):
            paths.add(p);text=p.read_text()
            for s in re.findall(r'include_(?:str|bytes)!\(\s*"([^"\n]+)"',text):
                target=(p.parent/s).resolve()
                if target.is_file():paths.add(target)
            for s in re.findall(r'include_(?:str|bytes)!\(\s*concat!\(\s*env!\("CARGO_MANIFEST_DIR"\),\s*"([^"\n]+)"',text):
                target=Path(str(crate)+s).resolve()
                if target.is_file():paths.add(target)
    return [{'path':p.relative_to(ROOT).as_posix(),'sha256':sha(p.read_bytes())} for p in sorted(paths)]


def build():
    folder=HERE/'frontend-native';lock=folder/'Cargo.lock'
    if not lock.exists():lock.write_bytes((ROOT/'Cargo.lock').read_bytes())
    args=['cargo','build','--offline','--manifest-path',str(folder/'Cargo.toml')];env={**os.environ,**FLAGS,'CARGO_TARGET_DIR':str(TARGET)}
    p=subprocess.run(args,cwd=ROOT,env=env,capture_output=True,timeout=600)
    require(p.returncode==0 and b'warning:' not in p.stderr,'public frontend observer build: '+p.stderr.decode())
    allowed={(x['name'],x['version'],x.get('checksum')) for x in tomllib.loads((ROOT/'Cargo.lock').read_text())['package']}
    for x in tomllib.loads(lock.read_text())['package']:
        if 'source' in x:require((x['name'],x['version'],x.get('checksum')) in allowed,'unchanged dependency version/checksum')
    p=subprocess.run(args+['--locked'],cwd=ROOT,env=env,capture_output=True,timeout=600)
    require(p.returncode==0 and b'warning:' not in p.stderr,'locked public frontend observer build: '+p.stderr.decode())
    return TARGET/'debug/go-practical-frontend-native',{'build_environment':FLAGS,'sources':linked_inputs()}
