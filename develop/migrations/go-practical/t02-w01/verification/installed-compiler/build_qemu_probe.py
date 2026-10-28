"""Build the standalone syscall probe with the exact registered Go SDK."""
from pathlib import Path
import json
import os
import subprocess
import sys

ROOT=Path(__file__).resolve().parent
REPO=ROOT.parents[5]
SDK=Path('/private/tmp/mpk-installed-compiler-205-original-go-output/toolchain')
OUT=Path('/private/tmp/mpk-go-installed-205-fs/qemu-probe')
IMAGE='docker.io/library/golang@sha256:ebd54034f076819b3054f155db53660ded951612bb4dfd277f933e62059e5d5a'
sys.dont_write_bytecode=True
sys.path.insert(0,str(ROOT))
from run_scoped import seal,write,sdk_files


def main():
    evidence=ROOT/'qemu-probe-build'
    evidence.mkdir(exist_ok=False)
    OUT.mkdir(exist_ok=False)
    sources=[Path(__file__).resolve(),ROOT/'qemu_openat2.go',ROOT/'run_scoped.py',REPO/'release/bundles/candidates/go.json']
    before=[seal(p) for p in sources]
    sdk=sdk_files()
    write(evidence/'inputs.json',{'sources':before,'registered_sdk':sdk,'scope':'Build a standalone x86-64 Go syscall probe with the exact original Go SDK. No production code, proof limit or installed profile is changed.'})
    command=['/usr/local/bin/docker','run','--pull=never','--rm','--platform','linux/amd64','--network','none','--read-only','--cap-drop','ALL','--security-opt','no-new-privileges','--user',f'{os.getuid()}:{os.getgid()}','--tmpfs','/tmp:rw,exec,nosuid,nodev,size=512m,mode=1777','--mount',f'type=bind,src={ROOT/"qemu_openat2.go"},dst=/probe.go,readonly','--mount',f'type=bind,src={SDK},dst=/sdk,readonly','--mount',f'type=bind,src={OUT},dst=/out']
    for k,v in {'GOROOT':'/sdk/go','GOTOOLCHAIN':'local','GOENV':'off','GOWORK':'off','GOPROXY':'off','GOSUMDB':'off','CGO_ENABLED':'0','GOOS':'linux','GOARCH':'amd64','GOAMD64':'v1','GOCACHE':'/tmp/cache','GOMAXPROCS':'1'}.items():command.extend(['--env',f'{k}={v}'])
    command.extend([IMAGE,'/sdk/go/bin/go','build','-trimpath','-ldflags=-buildid=','-o','/out/probe','/probe.go'])
    report={'status':'running','command':command,'inputs':seal(evidence/'inputs.json'),'native_Linux_release_acceptance':False}
    write(evidence/'result.json',report)
    result=subprocess.run(command,capture_output=True,check=False)
    (evidence/'stdout.log').write_bytes(result.stdout);(evidence/'stderr.log').write_bytes(result.stderr)
    unchanged=before==[seal(p) for p in sources] and sdk==sdk_files()
    report.update(status='passed' if result.returncode==0 and unchanged else 'failed',exit_code=result.returncode,inputs_unchanged=unchanged,stdout=seal(evidence/'stdout.log'),stderr=seal(evidence/'stderr.log'))
    if (OUT/'probe').is_file():report['executable']=seal(OUT/'probe')
    write(evidence/'result.json',report)
    print(json.dumps({k:report.get(k) for k in ['status','exit_code','inputs_unchanged','executable']}))
    raise SystemExit(report['status']!='passed')


if __name__=='__main__':
    main()
