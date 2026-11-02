"""Check the changed export capture and its independent Linux observer."""
from pathlib import Path
import json
import os
import subprocess
import sys
import tarfile
import time

ROOT=Path(__file__).resolve().parent;REPO=ROOT.parents[5];VERIFY=ROOT.parent
SDK=Path('/private/tmp/mpk-installed-compiler-205-original-go-output/toolchain');CACHE=Path('/private/tmp/mpk-go-registered-analysis-vet-cache')
IMAGE='docker.io/library/golang@sha256:ebd54034f076819b3054f155db53660ded951612bb4dfd277f933e62059e5d5a'
sys.dont_write_bytecode=True;sys.path.insert(0,str(VERIFY/'installed-compiler'));from run_scoped import seal,sdk_files,write


def main():
    epoch=sys.argv[1];assert epoch.replace('-','').isalnum() and Path.cwd()==REPO
    out=ROOT/epoch;out.mkdir(exist_ok=False);CACHE.mkdir(exist_ok=True)
    paths={Path(__file__).resolve(),REPO/'Cargo.toml',REPO/'Cargo.lock',REPO/'crates/mpk-cli/examples/go_practical_live_export_root.rs',REPO/'release/bundles/candidates/go.json'}
    paths.update(p for p in (REPO/'go-tools/go2vir').rglob('*') if p.is_file())
    for crate in ['mpk-cli','mpk-api','mpk-cert','mpk-core','mpk-kernel','mpk-linux-sandbox','mpk-theory','mpk-vc']:
        base=REPO/'crates'/crate;paths.add(base/'Cargo.toml');paths.update(p for p in (base/'src').rglob('*') if p.is_file());paths.update(p for p in base.glob('build.rs') if p.is_file())
    probe=REPO/'develop/probes/go-practical';sys.path.insert(0,str(probe));from frontend_native_build import linked_inputs
    paths.update(REPO/r['path'] for r in linked_inputs());paths.update(probe/name for name in ['frontend_native_build.py','run_proof_pilot.py','check_scope_report.py'])
    before=[seal(p) for p in sorted(paths)];sdk=sdk_files()
    with tarfile.open(out/'inputs.tar.gz','x:gz',dereference=True) as archive:
        for path in sorted(paths):archive.add(path,arcname=str(path.relative_to(REPO)),recursive=False)
    common=['/usr/local/bin/docker','run','--pull=never','--rm','--platform','linux/amd64','--network','none','--read-only','--cap-drop','ALL','--security-opt','no-new-privileges','--user',f'{os.getuid()}:{os.getgid()}','--tmpfs','/tmp:rw,exec,nosuid,nodev,size=2g,mode=1777','--mount',f'type=bind,src={REPO},dst=/workspace,readonly','--mount',f'type=bind,src={SDK},dst=/sdk,readonly','--mount',f'type=bind,src={CACHE},dst=/cache','--workdir','/workspace/go-tools/go2vir']
    env={'GOROOT':'/sdk/go','GOTOOLCHAIN':'local','GOENV':'off','GOWORK':'off','GOFLAGS':'-mod=vendor','GOPROXY':'off','GOSUMDB':'off','CGO_ENABLED':'0','GOOS':'linux','GOARCH':'amd64','GOAMD64':'v1','GOMAXPROCS':'1','GOCACHE':'/cache','HOME':'/tmp'}
    go=common.copy()
    for key,value in env.items():go+=['--env',f'{key}={value}']
    go+=[IMAGE,'/sdk/go/bin/go','test','-json','-count=1','-timeout=20m','-run','^(TestPracticalCompilerInputsRequireOriginalImageOwner|TestPracticalCaptureOriginalModulesAndOwnedSnapshot)$']
    fmt=common+[IMAGE,'/sdk/go/bin/gofmt','-l','practical_exports.go','practical_compiler_inputs_test.go','practical_live_export_root_test.go']
    cargo=['cargo','clippy','--offline','--locked','-p','mpk-cli','--example','go_practical_live_export_root','--target','x86_64-unknown-linux-gnu','--','-D','warnings']
    rfmt=['rustfmt','--edition','2021','--check','--config','skip_children=true',str(REPO/'crates/mpk-cli/examples/go_practical_live_export_root.rs')]
    commands={'go-test-vet':go,'go-format':fmt,'rust-clippy':cargo,'rust-format':rfmt}
    renv={'CARGO_TARGET_DIR':'/private/tmp/mpk-go-source-root-clippy-target','CARGO_NET_OFFLINE':'true','CARGO_INCREMENTAL':'0','CARGO_PROFILE_DEV_DEBUG':'0'}
    write(out/'inputs.json',{'selection_reason':'Compile and vet the changed private export capture with original root and owner controls, and independently lint/format the Linux observer and changed files before exercising the full live application.','inputs':before,'archive':seal(out/'inputs.tar.gz'),'registered_sdk':sdk,'commands':commands,'go_environment':env,'rust_environment':renv})
    report={'status':'running','native_Linux_release_acceptance':False,'full_T_gate':'Deferred to final T02-W04: ./scripts/check-fast.sh'};write(out/'result.json',report);checks={};start=time.monotonic()
    for name,command in commands.items():
        with (out/(name+'.stdout')).open('xb') as stdout,(out/(name+'.stderr')).open('xb') as stderr:
            code=subprocess.run(command,cwd=REPO,env={**os.environ,**renv} if name=='rust-clippy' else os.environ,stdout=stdout,stderr=stderr,check=False).returncode
        ok=code==0 and (name!='go-format' or (out/(name+'.stdout')).read_bytes()==b'')
        checks[name]={'status':'passed' if ok else 'failed','exit_code':code,'stdout':seal(out/(name+'.stdout')),'stderr':seal(out/(name+'.stderr'))}
        print(json.dumps({'check':name,'status':checks[name]['status'],'elapsed_seconds':round(time.monotonic()-start)}),flush=True)
    unchanged=before==[seal(p) for p in sorted(paths)] and sdk==sdk_files();passed=unchanged and all(x['status']=='passed' for x in checks.values())
    report.update(status='passed' if passed else 'failed',seconds=time.monotonic()-start,original_inputs_unchanged=unchanged,checks=checks)
    write(out/'result.json',report);print(json.dumps({'status':report['status'],'original_inputs_unchanged':unchanged}),flush=True);raise SystemExit(not passed)

if __name__=='__main__':main()
