"""Offline public policy/API observer in a pinned local Linux test container."""
import os
import subprocess
import tomllib
from pathlib import Path
from frontend_native_build import linked_inputs as retained_linked_inputs,FLAGS
from run_proof_pilot import ROOT,HERE,sha
from check_scope_report import require

IMAGE='sha256:d617f1b5c2a18778ba357ba0528e973bd24992b0dd18000dfbca059f4a15ebda'
TARGET=Path('/private/tmp/mpk-go-practical-policy-linux-target')
CARGO=Path('/private/tmp/mpk-go-practical-policy-linux-cargo')
DOCKER='/usr/local/bin/docker'


def linked_inputs():
    rows={x['path']:x['sha256'] for x in retained_linked_inputs()};crate=ROOT/'crates/mpk-linux-sandbox'
    for p in [crate/'Cargo.toml']+list((crate/'src').rglob('*.rs')):rows[p.relative_to(ROOT).as_posix()]=sha(p.read_bytes())
    return [{'path':p,'sha256':h} for p,h in sorted(rows.items())]


def command(args,**kw):
    p=subprocess.run(list(map(str,args)),cwd=ROOT,capture_output=True,timeout=600,**kw)
    require(p.returncode==0,'local public policy observer command: '+p.stderr.decode())
    return p


def prefix():
    endpoint=command([DOCKER,'context','inspect','--format','{{.Endpoints.docker.Host}}']).stdout.decode().strip()
    require(endpoint.startswith('unix:///'),'local Unix Docker endpoint only')
    image=command([DOCKER,'--host',endpoint,'image','inspect','--format','{{.Id}} {{.Architecture}} {{.Os}}',IMAGE]).stdout.decode().strip()
    require(image==IMAGE+' amd64 linux','exact cached Linux image/architecture')
    TARGET.mkdir(exist_ok=True);registry=Path.home()/'.cargo/registry';require(registry.is_dir(),'existing offline Cargo registry');archives=[];index_names=set()
    for row in tomllib.loads((HERE/'policy-native/Cargo.lock').read_text())['package']:
        if 'source' not in row:continue
        require(row['source']=='registry+https://github.com/rust-lang/crates.io-index','retained crates.io source only')
        candidates=sorted(p for p in (registry/'cache').glob('*/'+row['name']+'-'+row['version']+'.crate') if sha(p.read_bytes())==row['checksum'])
        require(candidates,'exact cached dependency archive: '+row['name']);p=candidates[0];index_names.add(p.parent.name)
        dest=CARGO/'registry/cache'/p.parent.name/p.name;dest.parent.mkdir(parents=True,exist_ok=True)
        if not dest.exists() or sha(dest.read_bytes())!=row['checksum']:dest.write_bytes(p.read_bytes())
        archives.append({'name':row['name'],'version':row['version'],'sha256':row['checksum']})
    require(len(index_names)==1,'one retained offline source registry');(CARGO/'registry/src').mkdir(parents=True,exist_ok=True)
    p=[DOCKER,'--host',endpoint,'run','-i','--pull=never','--rm','--platform','linux/amd64','--network','none','--read-only',
       '--cap-drop','ALL','--security-opt','no-new-privileges','--user',str(os.getuid())+':'+str(os.getgid()),
       '--tmpfs','/tmp:rw,exec,nosuid,nodev,size=128m,mode=1777',
       '--mount','type=bind,src='+str(ROOT)+',dst=/workspace,readonly',
       '--mount','type=bind,src='+str(CARGO)+',dst=/cargo',
       '--mount','type=bind,src='+str(registry/'index')+',dst=/cargo/registry/index,readonly',
       '--mount','type=bind,src='+str(TARGET)+',dst=/target','--workdir','/workspace','--env','CARGO_HOME=/cargo','--env','CARGO_TARGET_DIR=/target']
    for k,v in FLAGS.items():p+=['--env',k+'='+v]
    return p+[IMAGE],{'image':image,'endpoint':endpoint,'platform':'local Linux amd64 emulation on arm64 Docker engine; not native Linux release evidence','network':'none','source_mount':'read-only','target':str(TARGET),'build_environment':FLAGS,'sources':linked_inputs(),'dependency_archives':archives}


def build():
    folder=HERE/'policy-native';lock=folder/'Cargo.lock';require(lock.is_file(),'bootstrap own lock from retained root dependency versions first')
    def source_pins():return linked_inputs()+[{'path':p.relative_to(ROOT).as_posix(),'sha256':sha(p.read_bytes())} for p in sorted(folder.rglob('*')) if p.is_file()]
    before=source_pins()
    allowed={(x['name'],x['version'],x.get('checksum')) for x in tomllib.loads((ROOT/'Cargo.lock').read_text())['package']}
    for x in tomllib.loads(lock.read_text())['package']:
        if 'source' in x:require((x['name'],x['version'],x.get('checksum')) in allowed,'unchanged root dependency version/checksum')
    p,audit=prefix();result=command(p+['cargo','build','--offline','--locked','--manifest-path','/workspace/develop/probes/go-practical/policy-native/Cargo.toml'])
    require(b'warning:' not in result.stderr,'new observer compiles without warnings')
    require(before==source_pins(),'all observed build sources remain fixed throughout the build')
    binary=TARGET/'debug/go-practical-policy-native';require(binary.read_bytes().startswith(b'\x7fELF'),'actual Linux observer executable')
    audit['binary_sha256']=sha(binary.read_bytes());audit['build_source_inputs']=before;audit['rustc']=command(p+['rustc','--version']).stdout.decode().strip();audit['cargo']=command(p+['cargo','--version']).stdout.decode().strip()
    return p+['/target/debug/go-practical-policy-native'],audit
