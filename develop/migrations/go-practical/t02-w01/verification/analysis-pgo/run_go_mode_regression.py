"""Run affected existing Go analysis consumers and full original rebuilds."""
from pathlib import Path
import json
import os
import subprocess
import sys
import tarfile
import time

ROOT=Path(__file__).resolve().parent;REPO=ROOT.parents[5];SDK=Path('/private/tmp/mpk-installed-compiler-205-original-go-output/toolchain')
IMAGE='docker.io/library/golang@sha256:ebd54034f076819b3054f155db53660ded951612bb4dfd277f933e62059e5d5a'
sys.dont_write_bytecode=True;sys.path.insert(0,str(ROOT.parent/'installed-compiler'));from run_scoped import seal,sdk_files,write


def main():
    epoch=sys.argv[1];assert epoch.replace('-','').isalnum() and Path.cwd()==REPO
    out=ROOT/epoch;out.mkdir(exist_ok=False);(out/'build').mkdir()
    files={Path(__file__).resolve(),REPO/'release/bundles/candidates/go.json',ROOT.parent/'installed-compiler/original-go-source.tar',ROOT.parent/'installed-compiler/original-go-source.json'};files.update(p for p in (REPO/'go-tools/go2vir').rglob('*') if p.is_file());files.update([ROOT/'actual-build-original/auto.elf.hex',ROOT/'actual-build-original/off.elf.hex'])
    before=[seal(p) for p in sorted(files)];sdk=sdk_files()
    with tarfile.open(out/'inputs.tar.gz','x:gz',dereference=True) as archive:
        for p in sorted(files):archive.add(p,arcname=str(p.relative_to(REPO)),recursive=False)
    env={'GOROOT':'/sdk/go','GOTOOLCHAIN':'local','GOENV':'off','GOWORK':'off','GOFLAGS':'-mod=vendor','GOPROXY':'off','GOSUMDB':'off','CGO_ENABLED':'0','GOOS':'linux','GOARCH':'amd64','GOAMD64':'v1','GOMAXPROCS':'1','GOCACHE':'/tmp/cache','HOME':'/tmp','MPK_GO_ANALYSIS_BUILD_LIST_OUTPUT':'/evidence','MPK_GO_PRACTICAL_ANALYSIS_EVIDENCE':'/evidence/build','MPK_GO_PRACTICAL_ANALYSIS_BINARY_OUTPUT':'/evidence/build'}
    tests=['TestPracticalAnalysisSourceDifferential','TestPracticalAnalysisBuildListDifferential','TestPracticalAnalysisCapturedInputsRebuildTheSameActualFrontend','TestPracticalRegisteredAnalysisOriginalCapture','TestPracticalAnalysisActualBuildInfoPGO']
    command=['/usr/local/bin/docker','run','--pull=never','--rm','--platform','linux/amd64','--network','none','--read-only','--cap-drop','ALL','--security-opt','no-new-privileges','--user',f'{os.getuid()}:{os.getgid()}','--tmpfs','/tmp:rw,exec,nosuid,nodev,size=2g,mode=1777','--mount',f'type=bind,src={REPO},dst=/workspace,readonly','--mount',f'type=bind,src={SDK},dst=/sdk,readonly','--mount',f'type=bind,src={out},dst=/evidence','--workdir','/workspace/go-tools/go2vir']
    for key,value in env.items():command.extend(['--env',f'{key}={value}'])
    command.extend([IMAGE,'/sdk/go/bin/go','test','-json','-count=1','-timeout=20m','-run','^('+'|'.join(tests)+')$'])
    write(out/'inputs.json',{'selection_reason':'Exercise the changed Go analysis capture and all direct build-list consumers, including two complete original frontend builds, while retaining all current files, original registered source archive and SDK.','inputs':before,'source_archive':seal(out/'inputs.tar.gz'),'registered_sdk':sdk,'command':command,'environment':env,'selected_tests':tests})
    report={'status':'running','native_Linux_release_acceptance':False,'full_T_gate':'Deferred to final T02-W04: ./scripts/check-fast.sh'};write(out/'result.json',report);start=time.monotonic()
    with (out/'stdout.log').open('xb') as stdout,(out/'stderr.log').open('xb') as stderr:
        code=subprocess.run(command,cwd=REPO,stdout=stdout,stderr=stderr,check=False).returncode
    unchanged=before==[seal(p) for p in sorted(files)] and sdk==sdk_files()
    events=[json.loads(line) for line in (out/'stdout.log').read_bytes().splitlines() if line]
    passed_tests={event.get('Test') for event in events if event.get('Action')=='pass'}
    captured=json.loads((out/'native.json').read_bytes()) if (out/'native.json').exists() else {}
    build=json.loads((out/'build/reproduced-builds.json').read_bytes()) if (out/'build/reproduced-builds.json').exists() else {}
    binary=[]
    for i in range(2):
        path=out/f'build/frontend-{i}.elf'
        if path.exists():
            raw=path.read_bytes();encoded=Path(str(path)+'.hex');encoded.write_bytes(raw.hex().encode('ascii'));assert bytes.fromhex(encoded.read_text())==raw
            binary.append({'original':seal(path),'encoded':seal(encoded),'exact_recovery':True});path.unlink()
    equal=len(binary)==2 and binary[0]['original']['bytes']==binary[1]['original']['bytes'] and binary[0]['original']['sha256']==binary[1]['original']['sha256']
    passed=code==0 and unchanged and set(tests).issubset(passed_tests) and captured.get('original_sources')==213 and len(captured.get('build_objects',[]))==201 and build.get('byte_exact_equal') is True and equal
    report.update(status='passed' if passed else 'failed',exit_code=code,seconds=time.monotonic()-start,original_inputs_unchanged=unchanged,selected_tests_passed=sorted(set(tests)&passed_tests),original_sources=captured.get('original_sources'),build_objects=len(captured.get('build_objects',[])),two_complete_binaries_equal=equal,binaries=binary,stdout=seal(out/'stdout.log'),stderr=seal(out/'stderr.log'))
    if captured:report['native_observation']=seal(out/'native.json')
    if build:report['build_observation']=seal(out/'build/reproduced-builds.json')
    write(out/'result.json',report);print(json.dumps({k:report[k] for k in ['status','exit_code','seconds','original_inputs_unchanged','selected_tests_passed','original_sources','build_objects','two_complete_binaries_equal']}),flush=True);raise SystemExit(not passed)


if __name__=='__main__':main()
