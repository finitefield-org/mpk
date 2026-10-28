"""Capture the complete original registered Go analysis source epoch."""
from pathlib import Path
import json
import os
import subprocess
import sys
import tarfile
import time

ROOT=Path(__file__).resolve().parent
REPO=ROOT.parents[5]
INSTALLED=ROOT.parent/'installed-compiler'
SDK=Path('/private/tmp/mpk-installed-compiler-205-original-go-output/toolchain')
IMAGE='docker.io/library/golang@sha256:ebd54034f076819b3054f155db53660ded951612bb4dfd277f933e62059e5d5a'
sys.dont_write_bytecode=True;sys.path.insert(0,str(INSTALLED))
from run_scoped import seal,sdk_files,write


def main():
    epoch=sys.argv[1];assert epoch.replace('-','').isalnum() and Path.cwd()==REPO
    out=ROOT/epoch;out.mkdir(exist_ok=False)
    files={Path(__file__).resolve(),INSTALLED/'run_scoped.py',INSTALLED/'original-go-source.tar',INSTALLED/'original-go-source.json',INSTALLED/'reproduced-go-current/result.json',REPO/'release/bundles/candidates/go.json'}
    files.update(p for p in (REPO/'go-tools/go2vir').rglob('*') if p.is_file())
    before=[seal(p) for p in sorted(files)];sdk=sdk_files();archive=out/'inputs.tar.gz'
    with tarfile.open(archive,'x:gz',dereference=True) as target:
        for p in sorted(files):target.add(p,arcname=str(p.relative_to(REPO)),recursive=False)
    env={'GOROOT':'/sdk/go','GOTOOLCHAIN':'local','GOENV':'off','GOWORK':'off','GOFLAGS':'-mod=vendor','GOPROXY':'off','GOSUMDB':'off','CGO_ENABLED':'0','GOOS':'linux','GOARCH':'amd64','GOAMD64':'v1','GOMAXPROCS':'1','GOCACHE':'/tmp/cache','HOME':'/tmp','MPK_GO_REGISTERED_ANALYSIS_OUTPUT':'/evidence'}
    command=['/usr/local/bin/docker','run','--pull=never','--rm','--platform','linux/amd64','--network','none','--read-only','--cap-drop','ALL','--security-opt','no-new-privileges','--user',f'{os.getuid()}:{os.getgid()}','--tmpfs','/tmp:rw,exec,nosuid,nodev,size=2g,mode=1777','--mount',f'type=bind,src={REPO},dst=/workspace,readonly','--mount',f'type=bind,src={SDK},dst=/sdk,readonly','--mount',f'type=bind,src={out},dst=/evidence','--workdir','/workspace/go-tools/go2vir']
    for key,value in env.items():command.extend(['--env',f'{key}={value}'])
    command.extend([IMAGE,'/sdk/go/bin/go','test','-json','-count=1','-timeout=20m','-run','^TestPracticalRegisteredAnalysisOriginalCapture$'])
    write(out/'inputs.json',{'inputs':before,'source_archive':seal(archive),'registered_sdk':sdk,'command':command,'environment':env,'selection_reason':'Restore every retained registered-source file and mode, capture its complete frontend/vendor/module inputs and immutable namespace with the pinned original SDK. Record the original Go test and complete byte inputs; this alone is not launcher or source/binary proof.'})
    report={'status':'running','native_Linux_release_acceptance':False,'full_T_gate':'Deferred to final T02-W04: ./scripts/check-fast.sh'};write(out/'result.json',report)
    start=time.monotonic()
    with (out/'stdout.log').open('xb') as stdout,(out/'stderr.log').open('xb') as stderr:
        code=subprocess.run(command,stdout=stdout,stderr=stderr,check=False).returncode
    unchanged=before==[seal(p) for p in sorted(files)] and sdk==sdk_files()
    native=json.loads((out/'native.json').read_bytes()) if (out/'native.json').exists() else {}
    passed=code==0 and unchanged and len(native.get('original_sources',[]))==196 and len(native.get('build_objects',[]))==184
    report.update(status='passed' if passed else 'failed',exit_code=code,seconds=time.monotonic()-start,original_inputs_unchanged=unchanged,source_objects=len(native.get('original_sources',[])),build_objects=len(native.get('build_objects',[])),stdout=seal(out/'stdout.log'),stderr=seal(out/'stderr.log'))
    if native:report['native_observation']=seal(out/'native.json')
    write(out/'result.json',report);print(json.dumps({k:report[k] for k in ['status','exit_code','seconds','original_inputs_unchanged','source_objects','build_objects']}),flush=True);raise SystemExit(not passed)


if __name__=='__main__':main()
