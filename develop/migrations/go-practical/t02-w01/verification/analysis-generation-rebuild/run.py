"""Compare the later complete source epoch using the original release flags."""
from pathlib import Path
import hashlib
import json
import os
import subprocess
import sys
import time

ROOT=Path(__file__).resolve().parent;REPO=ROOT.parents[5];VERIFY=ROOT.parent
INSTALLED=VERIFY/'installed-compiler';SDK=Path('/private/tmp/mpk-installed-compiler-205-original-go-output/toolchain');REGISTERED=Path('/private/tmp/mpk-installed-compiler-205-original-go-output/frontend/bin/go2vir')
IMAGE='docker.io/library/golang@sha256:ebd54034f076819b3054f155db53660ded951612bb4dfd277f933e62059e5d5a'
sys.dont_write_bytecode=True;sys.path.insert(0,str(INSTALLED));from run_scoped import seal,sdk_files,write


def main():
    epoch=sys.argv[1];assert epoch.replace('-','').isalnum() and Path.cwd()==REPO
    out=ROOT/epoch;out.mkdir(exist_ok=False);temp=Path('/private/tmp/mpk-analysis-generation-rebuild-'+epoch);temp.mkdir(mode=0o700,exist_ok=False)
    inputs={Path(__file__).resolve(),ROOT/'restore_current.py',INSTALLED/'run_scoped.py',VERIFY/'analysis-build-list/native/inputs.tar.gz',VERIFY/'analysis-build-list/native/inputs.json',VERIFY/'analysis-build-list/native/native.json',VERIFY/'analysis-namespace/native/native.json',VERIFY/'registered-analysis/native-fixed/native.json',VERIFY/'registered-analysis/rebuild-original/result.json',REPO/'release/bundles/candidates/go.json'}
    before=[seal(p) for p in sorted(inputs)];sdk=sdk_files();registered=seal(REGISTERED)
    assert registered['bytes']==9004378 and registered['sha256']=='ccc4a984d41b5da7b3a6838cbfaeee3eb23f675a2e4d7fb5d4f9fcd7e597d538'
    report={'status':'running','native_Linux_release_acceptance':False,'full_T_gate':'Deferred to final T02-W04: ./scripts/check-fast.sh'};write(out/'result.json',report)
    commands=[];built=[];start=time.monotonic()
    for i in range(2):
        source=temp/f'full-source-{i}';output=temp/f'build-{i}';output.mkdir(mode=0o700)
        restore=['python3','-B',str(ROOT/'restore_current.py'),str(source)];result=subprocess.run(restore,cwd=REPO,capture_output=True,check=False)
        (out/f'restore-{i}.stdout').write_bytes(result.stdout);(out/f'restore-{i}.stderr').write_bytes(result.stderr);commands.append({'command':restore,'exit_code':result.returncode,'stdout':seal(out/f'restore-{i}.stdout'),'stderr':seal(out/f'restore-{i}.stderr')});write(out/'commands.json',commands)
        if result.returncode!=0:break
        command=['/usr/local/bin/docker','run','--pull=never','--rm','--platform','linux/amd64','--network','none','--read-only','--cap-drop','ALL','--security-opt','no-new-privileges','--user',f'{os.getuid()}:{os.getgid()}','--tmpfs','/tmp:rw,exec,nosuid,nodev,size=2g,mode=1777','--mount',f'type=bind,src={source},dst=/source,readonly','--mount',f'type=bind,src={SDK},dst=/sdk,readonly','--mount',f'type=bind,src={output},dst=/out','--workdir','/source/go-tools/go2vir',IMAGE,'/usr/bin/env','-i','PATH=/sdk/go/bin:/usr/bin:/bin','HOME=/tmp','GOCACHE=/tmp/cache','GOROOT=/sdk/go','CGO_ENABLED=0','GOOS=linux','GOARCH=amd64','GOAMD64=v1','GOENV=off','GOFLAGS=','GOWORK=off','GOPROXY=off','GOSUMDB=off','GOTOOLCHAIN=local','TZ=UTC','LANG=C','LC_ALL=C','SOURCE_DATE_EPOCH=0','/sdk/go/bin/go','build','-mod=vendor','-trimpath','-buildvcs=false','-ldflags=-buildid=','-o','/out/go2vir','.']
        begin=time.monotonic()
        with (out/f'build-{i}.stdout').open('xb') as stdout,(out/f'build-{i}.stderr').open('xb') as stderr:
            code=subprocess.run(command,cwd=REPO,stdout=stdout,stderr=stderr,check=False).returncode
        commands.append({'command':command,'exit_code':code,'seconds':time.monotonic()-begin,'stdout':seal(out/f'build-{i}.stdout'),'stderr':seal(out/f'build-{i}.stderr')});write(out/'commands.json',commands)
        if code!=0 or not (output/'go2vir').is_file():break
        binary=(output/'go2vir').read_bytes();encoded=out/f'frontend-{i}.elf.hex';encoded.write_bytes(binary.hex().encode('ascii'));assert bytes.fromhex(encoded.read_text())==binary
        built.append({'binary':seal(output/'go2vir'),'encoded':seal(encoded),'byte_equal_registered':binary==REGISTERED.read_bytes()})
    unchanged=before==[seal(p) for p in sorted(inputs)] and sdk==sdk_files() and registered==seal(REGISTERED)
    identical=len(built)==2 and built[0]['binary']['bytes']==built[1]['binary']['bytes'] and built[0]['binary']['sha256']==built[1]['binary']['sha256'] and bytes.fromhex((out/'frontend-0.elf.hex').read_text())==bytes.fromhex((out/'frontend-1.elf.hex').read_text())
    different=identical and all(not item['byte_equal_registered'] for item in built)
    write(out/'inputs.json',{'selection_reason':'Rebuild every recorded later frontend source file twice offline with the same registered SDK and release flags used for the original epoch; preserve complete input archives and binary streams. This compares captured epochs, not the evolving checkout or native release acceptance.','inputs':before,'registered_sdk':sdk,'registered_frontend':registered,'source_archive_files_per_build':331,'frontend_files_per_build':328,'build_recipe_matches_registered_epoch':True})
    report.update(status='passed' if unchanged and identical and different else 'failed',seconds=time.monotonic()-start,original_inputs_unchanged=unchanged,two_complete_binaries_equal=identical,complete_binaries_differ_from_registered=different,builds=built,commands=seal(out/'commands.json'))
    write(out/'result.json',report);print(json.dumps({k:report[k] for k in ['status','seconds','original_inputs_unchanged','two_complete_binaries_equal','complete_binaries_differ_from_registered']}),flush=True);raise SystemExit(report['status']!='passed')


if __name__=='__main__':main()
