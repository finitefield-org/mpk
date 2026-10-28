"""Reproduce registered Go frontend from the entire historical source epoch."""
from pathlib import Path
import hashlib
import json
import os
import subprocess
import sys
import time

ROOT=Path(__file__).resolve().parent
REPO=ROOT.parents[5]
INSTALLED=ROOT.parent/'installed-compiler'
SDK=Path('/private/tmp/mpk-installed-compiler-205-original-go-output/toolchain')
REGISTERED=Path('/private/tmp/mpk-installed-compiler-205-original-go-output/frontend/bin/go2vir')
IMAGE='docker.io/library/golang@sha256:ebd54034f076819b3054f155db53660ded951612bb4dfd277f933e62059e5d5a'
sys.dont_write_bytecode=True;sys.path.insert(0,str(INSTALLED))
from run_scoped import seal,sdk_files,write


def main():
    epoch=sys.argv[1];assert epoch.replace('-','').isalnum() and Path.cwd()==REPO
    out=ROOT/epoch;out.mkdir(exist_ok=False)
    temp=Path('/private/tmp/mpk-go-registered-rebuild-'+epoch);temp.mkdir(mode=0o700,exist_ok=False)
    files={Path(__file__).resolve(),ROOT/'restore_registered.py',INSTALLED/'run_scoped.py',INSTALLED/'original-go-source.tar',INSTALLED/'original-go-source.json',INSTALLED/'reproduced-go-current/result.json',REPO/'release/bundles/candidates/go.json',REPO/'develop/migrations/go-rust-only/verification/build-go/result.json'}
    before=[seal(p) for p in sorted(files)];sdk=sdk_files();registered=seal(REGISTERED)
    assert registered['sha256']=='ccc4a984d41b5da7b3a6838cbfaeee3eb23f675a2e4d7fb5d4f9fcd7e597d538'
    commands=[];builds=[]
    result={'status':'running','native_Linux_release_acceptance':False,'full_T_gate':'Deferred to final T02-W04: ./scripts/check-fast.sh'};write(out/'result.json',result)
    start=time.monotonic()
    for i in range(2):
        source=temp/f'full-source-{i}';output=temp/f'build-{i}';output.mkdir(mode=0o700)
        restore=['python3','-B',str(ROOT/'restore_registered.py'),str(source)]
        restored=subprocess.run(restore,cwd=REPO,capture_output=True,check=False)
        (out/f'restore-{i}.stdout').write_bytes(restored.stdout);(out/f'restore-{i}.stderr').write_bytes(restored.stderr)
        commands.append({'command':restore,'exit_code':restored.returncode,'stdout':seal(out/f'restore-{i}.stdout'),'stderr':seal(out/f'restore-{i}.stderr')});write(out/'commands.json',commands)
        if restored.returncode!=0:break
        image_command=['/usr/local/bin/docker','run','--pull=never','--rm','--platform','linux/amd64','--network','none','--read-only','--cap-drop','ALL','--security-opt','no-new-privileges','--user',f'{os.getuid()}:{os.getgid()}','--tmpfs','/tmp:rw,exec,nosuid,nodev,size=2g,mode=1777','--mount',f'type=bind,src={source},dst=/source,readonly','--mount',f'type=bind,src={SDK},dst=/sdk,readonly','--mount',f'type=bind,src={output},dst=/out','--workdir','/source/go-tools/go2vir',IMAGE,'/usr/bin/env','-i','PATH=/sdk/go/bin:/usr/bin:/bin','HOME=/tmp','GOCACHE=/tmp/cache','GOROOT=/sdk/go','CGO_ENABLED=0','GOOS=linux','GOARCH=amd64','GOAMD64=v1','GOENV=off','GOFLAGS=','GOWORK=off','GOPROXY=off','GOSUMDB=off','GOTOOLCHAIN=local','TZ=UTC','LANG=C','LC_ALL=C','SOURCE_DATE_EPOCH=0','/sdk/go/bin/go','build','-mod=vendor','-trimpath','-buildvcs=false','-ldflags=-buildid=','-o','/out/go2vir','.']
        begin=time.monotonic()
        with (out/f'build-{i}.stdout').open('xb') as stdout,(out/f'build-{i}.stderr').open('xb') as stderr:
            code=subprocess.run(image_command,cwd=REPO,stdout=stdout,stderr=stderr,check=False).returncode
        commands.append({'command':image_command,'exit_code':code,'seconds':time.monotonic()-begin,'stdout':seal(out/f'build-{i}.stdout'),'stderr':seal(out/f'build-{i}.stderr')});write(out/'commands.json',commands)
        if code!=0 or not (output/'go2vir').is_file():break
        binary=(output/'go2vir').read_bytes();encoded=out/f'frontend-{i}.elf.hex';encoded.write_bytes(binary.hex().encode('ascii'))
        assert bytes.fromhex(encoded.read_text())==binary
        builds.append({'binary':seal(output/'go2vir'),'encoded':seal(encoded),'byte_exact_registered':binary==REGISTERED.read_bytes()})
    unchanged=before==[seal(p) for p in sorted(files)] and sdk==sdk_files() and registered==seal(REGISTERED)
    equal=len(builds)==2 and all(item['byte_exact_registered'] for item in builds)
    write(out/'inputs.json',{'selection_reason':'Use every original source and path, the exact registered Go SDK and the original release flags in two independent offline empty-cache builds. Rehash all 5577 SDK files and compare complete output bytes to the registered frontend; preserve both outputs losslessly. This is local emulation, not launcher or native release acceptance.','inputs':before,'registered_sdk':sdk,'registered_frontend':registered,'registered_source_epoch':'ed67a4c388defe8babdbc55f29851552562121d9','source_files_per_build':269,'frontend_files_per_build':242})
    result.update(status='passed' if unchanged and equal else 'failed',seconds=time.monotonic()-start,consumed_inputs_unchanged=unchanged,complete_binary_bytes_equal_registered=equal,builds=builds,commands=seal(out/'commands.json'))
    write(out/'result.json',result);print(json.dumps({k:result[k] for k in ['status','seconds','consumed_inputs_unchanged','complete_binary_bytes_equal_registered']}),flush=True);raise SystemExit(result['status']!='passed')


if __name__=='__main__':main()
