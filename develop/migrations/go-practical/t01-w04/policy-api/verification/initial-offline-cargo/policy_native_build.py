"""Offline public policy/API observer in a pinned local Linux test container."""
import os
import subprocess
import tomllib
from pathlib import Path
from frontend_native_build import linked_inputs,FLAGS
from run_proof_pilot import ROOT,HERE,sha
from check_scope_report import require

IMAGE='sha256:d617f1b5c2a18778ba357ba0528e973bd24992b0dd18000dfbca059f4a15ebda'
TARGET=Path('/private/tmp/mpk-go-practical-policy-linux-target')
DOCKER='/usr/local/bin/docker'


def command(args,**kw):
    p=subprocess.run(list(map(str,args)),cwd=ROOT,capture_output=True,timeout=600,**kw)
    require(p.returncode==0,'local public policy observer command: '+p.stderr.decode())
    return p


def prefix():
    endpoint=command([DOCKER,'context','inspect','--format','{{.Endpoints.docker.Host}}']).stdout.decode().strip()
    require(endpoint.startswith('unix:///'),'local Unix Docker endpoint only')
    image=command([DOCKER,'--host',endpoint,'image','inspect','--format','{{.Id}} {{.Architecture}} {{.Os}}',IMAGE]).stdout.decode().strip()
    require(image==IMAGE+' amd64 linux','exact cached Linux image/architecture')
    TARGET.mkdir(exist_ok=True);registry=Path.home()/'.cargo/registry';require(registry.is_dir(),'existing offline Cargo registry')
    p=[DOCKER,'--host',endpoint,'run','--pull=never','--rm','--platform','linux/amd64','--network','none','--read-only',
       '--cap-drop','ALL','--security-opt','no-new-privileges','--user',str(os.getuid())+':'+str(os.getgid()),
       '--tmpfs','/tmp:rw,exec,nosuid,nodev,size=128m,mode=1777','--tmpfs','/cargo:rw,exec,nosuid,nodev,size=64m,mode=1777',
       '--mount','type=bind,src='+str(ROOT)+',dst=/workspace,readonly',
       '--mount','type=bind,src='+str(registry)+',dst=/cargo/registry,readonly',
       '--mount','type=bind,src='+str(TARGET)+',dst=/target','--workdir','/workspace','--env','CARGO_HOME=/cargo','--env','CARGO_TARGET_DIR=/target']
    for k,v in FLAGS.items():p+=['--env',k+'='+v]
    return p+[IMAGE],{'image':image,'endpoint':endpoint,'platform':'local Linux amd64 emulation on arm64 Docker engine; not native Linux release evidence','network':'none','source_mount':'read-only','target':str(TARGET),'build_environment':FLAGS,'sources':linked_inputs()}


def build():
    folder=HERE/'policy-native';lock=folder/'Cargo.lock';require(lock.is_file(),'bootstrap own lock from retained root dependency versions first')
    allowed={(x['name'],x['version'],x.get('checksum')) for x in tomllib.loads((ROOT/'Cargo.lock').read_text())['package']}
    for x in tomllib.loads(lock.read_text())['package']:
        if 'source' in x:require((x['name'],x['version'],x.get('checksum')) in allowed,'unchanged root dependency version/checksum')
    p,audit=prefix();result=command(p+['cargo','build','--offline','--locked','--manifest-path','/workspace/develop/probes/go-practical/policy-native/Cargo.toml'])
    require(b'warning:' not in result.stderr,'new observer compiles without warnings')
    binary=TARGET/'debug/go-practical-policy-native';require(binary.read_bytes().startswith(b'\x7fELF'),'actual Linux observer executable')
    audit['binary_sha256']=sha(binary.read_bytes());audit['rustc']=command(p+['rustc','--version']).stdout.decode().strip();audit['cargo']=command(p+['cargo','--version']).stdout.decode().strip()
    return p+['/target/debug/go-practical-policy-native'],audit
