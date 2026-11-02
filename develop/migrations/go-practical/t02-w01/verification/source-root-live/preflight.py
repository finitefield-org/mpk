"""Seal complete inputs and check the two original capture observers locally."""
from pathlib import Path
import concurrent.futures
import json
import os
import subprocess
import sys
import tarfile
import time

ROOT=Path(__file__).resolve().parent
REPO=ROOT.parents[5]
SDK=Path('/private/tmp/mpk-installed-compiler-205-original-go-output/toolchain')
CARGO=Path('/private/tmp/mpk-go-source-root-matching/cargo')
TARGET=Path('/private/tmp/mpk-go-source-root-matching/target')
REGISTRY=Path('/Users/kazuyoshitoshiya/.cargo/registry')
GOCACHE=Path('/private/tmp/mpk-installed-compiler-205-go-cache')
RUST_IMAGE='sha256:7cf1e580ef5539f03b58560753e8ab84c8c360960d99dff714004aa98f203977'
GO_IMAGE='docker.io/library/golang@sha256:ebd54034f076819b3054f155db53660ded951612bb4dfd277f933e62059e5d5a'
sys.dont_write_bytecode=True
sys.path.insert(0,str(ROOT.parent/'installed-compiler'))
from run_scoped import seal,sdk_files,write


def main():
    assert Path.cwd()==REPO
    epoch=sys.argv[1] if len(sys.argv)>1 else 'preflight'
    go_only=len(sys.argv)>2 and sys.argv[2]=='go-only'
    assert epoch.replace('-','').isalnum()
    out=ROOT/epoch;out.mkdir(exist_ok=False)
    GOCACHE.mkdir(exist_ok=True)
    paths={Path(__file__).resolve(),REPO/'crates/mpk-cli/examples/go_practical_live_roots.rs',REPO/'crates/mpk-cli/tests/go_practical_source_roots.rs',REPO/'release/bundles/candidates/go.json'}
    sys.path.insert(0,str(REPO/'develop/probes/go-practical'))
    from frontend_native_build import linked_inputs
    paths.update(REPO/r['path'] for r in linked_inputs())
    paths.update(p for p in (REPO/'go-tools/go2vir').rglob('*') if p.is_file())
    paths.update(p for p in (REPO/'go-tools/mpk-checker-ref').rglob('*') if p.is_file())
    before=[seal(p) for p in sorted(paths)];sdk=sdk_files()
    archive=out/'inputs.tar.gz'
    with tarfile.open(archive,'x:gz',dereference=True) as target:
        for p in sorted(paths):target.add(p,arcname=str(p.relative_to(REPO)),recursive=False)
    rust=['/usr/local/bin/docker','run','--pull=never','--rm','--platform','linux/arm64','--network','none','--read-only','--cap-drop','ALL','--security-opt','no-new-privileges','--user',f'{os.getuid()}:{os.getgid()}','--tmpfs','/tmp:rw,exec,nosuid,nodev,size=1g,mode=1777','--mount',f'type=bind,src={REPO},dst=/workspace,readonly','--mount',f'type=bind,src={CARGO},dst=/cargo','--mount',f'type=bind,src={REGISTRY},dst=/cargo/registry,readonly','--mount',f'type=bind,src={TARGET},dst=/target','--workdir','/workspace']
    renv={'CARGO_HOME':'/cargo','CARGO_TARGET_DIR':'/target','CARGO_NET_OFFLINE':'true','CARGO_INCREMENTAL':'0','CARGO_PROFILE_DEV_DEBUG':'0','HOME':'/tmp'}
    for k,v in renv.items():rust.extend(['--env',f'{k}={v}'])
    rust.extend([RUST_IMAGE,'cargo','check','--offline','--locked','-p','mpk-cli','--example','go_practical_live_roots'])
    go=['/usr/local/bin/docker','run','--pull=never','--rm','--platform','linux/amd64','--network','none','--read-only','--cap-drop','ALL','--security-opt','no-new-privileges','--user',f'{os.getuid()}:{os.getgid()}','--tmpfs','/tmp:rw,exec,nosuid,nodev,size=2g,mode=1777','--mount',f'type=bind,src={REPO},dst=/workspace,readonly','--mount',f'type=bind,src={SDK},dst=/sdk,readonly','--mount',f'type=bind,src={GOCACHE},dst=/cache','--workdir','/workspace/go-tools/go2vir']
    genv={'GOROOT':'/sdk/go','GOTOOLCHAIN':'local','GOENV':'off','GOWORK':'off','GOFLAGS':'-mod=vendor','GOPROXY':'off','GOSUMDB':'off','CGO_ENABLED':'0','GOOS':'linux','GOARCH':'amd64','GOAMD64':'v1','GOMAXPROCS':'1','GOCACHE':'/cache/build','GOMODCACHE':'/cache/mod'}
    for k,v in genv.items():go.extend(['--env',f'{k}={v}'])
    base=go+[GO_IMAGE,'/sdk/go/bin/go']
    commands={'rust-check':rust,'go-compile':base+['test','-count=1','-timeout=20m','-run','^TestPracticalCompilerInputsRequireOriginalImageOwner$'],'go-vet':base+['vet','./...'],'go-format':go+[GO_IMAGE,'/sdk/go/bin/gofmt','-l','practical_compiler_inputs_test.go','practical_live_root_witness_test.go'],'rust-format':['rustfmt','--edition=2021','--config','skip_children=true','--check','crates/mpk-cli/examples/go_practical_live_roots.rs']}
    write(out/'inputs.json',{'selection_reason':'Seal complete original Go frontend, linked Rust implementation and registered SDK; compile both sides and check the changed Go and Rust formats/vet before the live original-application comparison. No full T gate or native release acceptance.','sources':before,'registered_sdk':sdk,'archive':seal(archive),'commands':commands,'rust_environment':renv,'go_environment':genv})
    report={'status':'running','native_Linux_release_acceptance':False,'full_T_gate':'Deferred to final T02-W04: ./scripts/check-fast.sh'};write(out/'result.json',report)
    def run(name):
        started=time.monotonic()
        with (out/f'{name}.stdout').open('xb') as stdout,(out/f'{name}.stderr').open('xb') as stderr:
            code=subprocess.run(commands[name],cwd=REPO,stdout=stdout,stderr=stderr,check=False).returncode
        okay=code==0 and (name!='go-format' or not (out/f'{name}.stdout').read_bytes())
        row={'status':'passed' if okay else 'failed','exit_code':code,'seconds':time.monotonic()-started,'stdout':seal(out/f'{name}.stdout'),'stderr':seal(out/f'{name}.stderr')}
        print(json.dumps({'check':name,'status':row['status'],'seconds':round(row['seconds'],2)}),flush=True)
        return name,row
    if go_only:
        previous=json.loads((ROOT/'preflight/result.json').read_bytes())
        original=json.loads((ROOT/'preflight/inputs.json').read_bytes())['sources']
        assert previous['checks']['rust-check']['status']=='passed' and previous['checks']['rust-format']['status']=='passed' and previous['original_inputs_unchanged']
        assert all(row==seal(Path(row['path'])) for row in original if row['path']!=str(Path(__file__).resolve()))
        checks=dict([run('go-compile')])
    else:
        with concurrent.futures.ThreadPoolExecutor(max_workers=2) as pool:
            checks=dict(pool.map(run,['rust-check','go-compile']))
    for name in (['go-vet','go-format'] if go_only else ['go-vet','go-format','rust-format']):
        key,value=run(name);checks[key]=value
    unchanged=before==[seal(p) for p in sorted(paths)] and sdk==sdk_files()
    report.update(status='passed' if unchanged and all(r['status']=='passed' for r in checks.values()) else 'failed',checks=checks,original_inputs_unchanged=unchanged)
    write(out/'result.json',report);print(json.dumps({'status':report['status'],'original_inputs_unchanged':unchanged}),flush=True);raise SystemExit(report['status']!='passed')


if __name__=='__main__':main()
