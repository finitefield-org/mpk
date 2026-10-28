"""Observe real Go builds with the exact original compiler PGO profile."""
from pathlib import Path
import hashlib
import json
import os
import subprocess
import sys
import time

ROOT=Path(__file__).resolve().parent;REPO=ROOT.parents[5];VERIFY=ROOT.parent;INSTALLED=VERIFY/'installed-compiler'
SDK=Path('/private/tmp/mpk-installed-compiler-205-original-go-output/toolchain');REGISTERED=Path('/private/tmp/mpk-installed-compiler-205-original-go-output/frontend/bin/go2vir')
IMAGE='docker.io/library/golang@sha256:ebd54034f076819b3054f155db53660ded951612bb4dfd277f933e62059e5d5a'
sys.dont_write_bytecode=True;sys.path.insert(0,str(INSTALLED));from run_scoped import seal,sdk_files,write


def main():
    epoch=sys.argv[1];assert epoch.replace('-','').isalnum() and Path.cwd()==REPO
    out=ROOT/epoch;out.mkdir(exist_ok=False);temp=Path('/private/tmp/mpk-go-analysis-pgo-build-'+epoch);temp.mkdir(mode=0o700,exist_ok=False)
    inputs={Path(__file__).resolve(),VERIFY/'registered-analysis/restore_registered.py',INSTALLED/'original-go-source.tar',INSTALLED/'original-go-source.json',ROOT/'native-before-fix/native.json',ROOT/'native-before-fix/original-profile.raw',ROOT/'native-after-fix/native.json',ROOT/'native-after-fix/original-profile.raw',REPO/'release/bundles/candidates/go.json'}
    before=[seal(p) for p in sorted(inputs)];sdk=sdk_files();registered=seal(REGISTERED)
    profile=(SDK/'go/src/cmd/compile/default.pgo').read_bytes();packet=json.loads((ROOT/'native-after-fix/native.json').read_bytes());assert profile==(ROOT/'native-after-fix/original-profile.raw').read_bytes()==(ROOT/'native-before-fix/original-profile.raw').read_bytes() and hashlib.sha256(profile).hexdigest()==packet['profile_sha256']
    assert registered['sha256']=='ccc4a984d41b5da7b3a6838cbfaeee3eb23f675a2e4d7fb5d4f9fcd7e597d538'
    commands=[];builds=[];report={'status':'running','native_Linux_release_acceptance':False,'full_T_gate':'Deferred to final T02-W04: ./scripts/check-fast.sh'};write(out/'result.json',report);start=time.monotonic()
    for mode in ['auto','off']:
        source=temp/f'full-source-{mode}';output=temp/f'build-{mode}';output.mkdir(mode=0o700)
        restore=['python3','-B',str(VERIFY/'registered-analysis/restore_registered.py'),str(source)];r=subprocess.run(restore,cwd=REPO,capture_output=True,check=False)
        (out/f'restore-{mode}.stdout').write_bytes(r.stdout);(out/f'restore-{mode}.stderr').write_bytes(r.stderr);commands.append({'name':'restore-'+mode,'command':restore,'exit_code':r.returncode,'stdout':seal(out/f'restore-{mode}.stdout'),'stderr':seal(out/f'restore-{mode}.stderr')});write(out/'commands.json',commands)
        if r.returncode!=0:break
        target=source/'go-tools/go2vir/default.pgo';assert not target.exists();target.write_bytes(profile);target.chmod(0o644)
        command=['/usr/local/bin/docker','run','--pull=never','--rm','--platform','linux/amd64','--network','none','--read-only','--cap-drop','ALL','--security-opt','no-new-privileges','--user',f'{os.getuid()}:{os.getgid()}','--tmpfs','/tmp:rw,exec,nosuid,nodev,size=2g,mode=1777','--mount',f'type=bind,src={source},dst=/source,readonly','--mount',f'type=bind,src={SDK},dst=/sdk,readonly','--mount',f'type=bind,src={output},dst=/out','--workdir','/source/go-tools/go2vir',IMAGE,'/usr/bin/env','-i','PATH=/sdk/go/bin:/usr/bin:/bin','HOME=/tmp','GOCACHE=/tmp/cache','GOROOT=/sdk/go','CGO_ENABLED=0','GOOS=linux','GOARCH=amd64','GOAMD64=v1','GOENV=off','GOFLAGS=','GOWORK=off','GOPROXY=off','GOSUMDB=off','GOTOOLCHAIN=local','TZ=UTC','LANG=C','LC_ALL=C','SOURCE_DATE_EPOCH=0','/sdk/go/bin/go','build','-mod=vendor','-trimpath','-buildvcs=false','-ldflags=-buildid=']
        if mode=='off':command.append('-pgo=off')
        command.extend(['-o','/out/go2vir','.']);begin=time.monotonic()
        with (out/f'build-{mode}.stdout').open('xb') as stdout,(out/f'build-{mode}.stderr').open('xb') as stderr:code=subprocess.run(command,cwd=REPO,stdout=stdout,stderr=stderr,check=False).returncode
        commands.append({'name':'build-'+mode,'command':command,'exit_code':code,'seconds':time.monotonic()-begin,'stdout':seal(out/f'build-{mode}.stdout'),'stderr':seal(out/f'build-{mode}.stderr')});write(out/'commands.json',commands)
        if code!=0 or not (output/'go2vir').is_file():break
        raw=(output/'go2vir').read_bytes();encoded=out/f'{mode}.elf.hex';encoded.write_bytes(raw.hex().encode());assert bytes.fromhex(encoded.read_text())==raw
        version=['/usr/local/bin/docker','run','--pull=never','--rm','--platform','linux/amd64','--network','none','--read-only','--cap-drop','ALL','--security-opt','no-new-privileges','--user',f'{os.getuid()}:{os.getgid()}','--mount',f'type=bind,src={SDK},dst=/sdk,readonly','--mount',f'type=bind,src={output},dst=/out,readonly',IMAGE,'/sdk/go/bin/go','version','-m','/out/go2vir']
        observed=subprocess.run(version,cwd=REPO,capture_output=True,check=False);(out/f'version-{mode}.stdout').write_bytes(observed.stdout);(out/f'version-{mode}.stderr').write_bytes(observed.stderr);commands.append({'name':'version-'+mode,'command':version,'exit_code':observed.returncode,'stdout':seal(out/f'version-{mode}.stdout'),'stderr':seal(out/f'version-{mode}.stderr')});write(out/'commands.json',commands)
        if observed.returncode!=0:break
        lines=[line.strip() for line in observed.stdout.decode().splitlines() if line.strip().startswith('build\t-pgo=')]
        builds.append({'mode':mode,'binary':seal(output/'go2vir'),'encoded':seal(encoded),'build_info_pgo_lines':lines,'byte_equal_registered':raw==REGISTERED.read_bytes(),'profile_exact_after_build':target.read_bytes()==profile})
    unchanged=before==[seal(p) for p in sorted(inputs)] and sdk==sdk_files() and registered==seal(REGISTERED)
    auto=next((x for x in builds if x['mode']=='auto'),{});off=next((x for x in builds if x['mode']=='off'),{})
    passed=unchanged and len(builds)==2 and auto.get('build_info_pgo_lines')==['build\t-pgo=default.pgo'] and not off.get('build_info_pgo_lines') and not auto.get('byte_equal_registered') and off.get('byte_equal_registered') and all(x['profile_exact_after_build'] for x in builds)
    write(out/'inputs.json',{'selection_reason':'Build the complete original registered source twice with the exact registered SDK compiler profile added, observing actual automatic PGO and explicit -pgo=off results with independent empty caches. Preserve the complete source archive, SDK and original profile bytes.','inputs':before,'registered_sdk':sdk,'registered_frontend':registered,'original_source_files_per_build':269,'original_frontend_files_per_build':242,'added_profile_bytes':len(profile),'added_profile_sha256':hashlib.sha256(profile).hexdigest()})
    report.update(status='passed' if passed else 'failed',seconds=time.monotonic()-start,original_inputs_unchanged=unchanged,builds=builds,commands=seal(out/'commands.json'))
    write(out/'result.json',report);print(json.dumps({'status':report['status'],'seconds':report['seconds'],'original_inputs_unchanged':unchanged,'build_modes':[x['mode'] for x in builds]}),flush=True);raise SystemExit(not passed)


if __name__=='__main__':main()
