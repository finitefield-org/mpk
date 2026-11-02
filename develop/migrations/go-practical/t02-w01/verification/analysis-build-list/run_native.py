"""Compare the complete Go input list and reproduce two offline frontend builds."""
from pathlib import Path
import json
import os
import subprocess
import sys
import tarfile
import time

ROOT=Path(__file__).resolve().parent
REPO=ROOT.parents[5]
SDK=Path('/private/tmp/mpk-installed-compiler-205-original-go-output/toolchain')
IMAGE='docker.io/library/golang@sha256:ebd54034f076819b3054f155db53660ded951612bb4dfd277f933e62059e5d5a'
sys.dont_write_bytecode=True
sys.path.insert(0,str(ROOT.parent/'installed-compiler'))
from run_scoped import seal,sdk_files,write


def main():
    out=ROOT/'native';out.mkdir(exist_ok=False);(out/'build').mkdir()
    files={Path(__file__).resolve(),ROOT/'prelaunch/result.json',REPO/'release/bundles/candidates/go.json'}
    files.update(p for p in (REPO/'go-tools/go2vir').rglob('*') if p.is_file())
    before=[seal(p) for p in sorted(files)];sdk=sdk_files();archive=out/'inputs.tar.gz'
    with tarfile.open(archive,'x:gz',dereference=True) as target:
        for path in sorted(files):target.add(path,arcname=str(path.relative_to(REPO)),recursive=False)
    env={'GOROOT':'/sdk/go','GOTOOLCHAIN':'local','GOENV':'off','GOWORK':'off','GOFLAGS':'-mod=vendor','GOPROXY':'off','GOSUMDB':'off','CGO_ENABLED':'0','GOOS':'linux','GOARCH':'amd64','GOAMD64':'v1','GOMAXPROCS':'1','GOCACHE':'/tmp/cache','MPK_GO_ANALYSIS_BUILD_LIST_OUTPUT':'/evidence','MPK_GO_PRACTICAL_ANALYSIS_EVIDENCE':'/evidence/build','MPK_GO_PRACTICAL_ANALYSIS_BINARY_OUTPUT':'/evidence/build'}
    command=['/usr/local/bin/docker','run','--pull=never','--rm','--platform','linux/amd64','--network','none','--read-only','--cap-drop','ALL','--security-opt','no-new-privileges','--user',f'{os.getuid()}:{os.getgid()}','--tmpfs','/tmp:rw,exec,nosuid,nodev,size=2g,mode=1777','--mount',f'type=bind,src={REPO},dst=/workspace,readonly','--mount',f'type=bind,src={SDK},dst=/sdk,readonly','--mount',f'type=bind,src={out},dst=/evidence','--workdir','/workspace/go-tools/go2vir']
    for key,value in env.items():command.extend(['--env',f'{key}={value}'])
    command.extend([IMAGE,'/sdk/go/bin/go','test','-json','-count=1','-timeout=20m','-run','^(TestPracticalAnalysisBuildListDifferential|TestPracticalAnalysisCapturedInputsRebuildTheSameActualFrontend)$'])
    write(out/'inputs.json',{'inputs':before,'source_archive':seal(archive),'registered_sdk':sdk,'command':command,'environment':env,'selection_reason':'Retain every original analysis build input and registered SDK byte, compare Go-derived input list and two independent offline builds from complete owned sources. No installed practical context or application proof is inferred.'})
    report={'status':'running','native_Linux_release_acceptance':False,'full_T_gate':'Deferred to final T02-W04: ./scripts/check-fast.sh'};write(out/'result.json',report)
    start=time.monotonic()
    with (out/'stdout.log').open('xb') as stdout,(out/'stderr.log').open('xb') as stderr:
        code=subprocess.run(command,stdout=stdout,stderr=stderr,check=False).returncode
    unchanged=before==[seal(p) for p in sorted(files)] and sdk==sdk_files()
    observations=json.loads((out/'native.json').read_bytes()) if (out/'native.json').exists() else {}
    built=json.loads((out/'build/reproduced-builds.json').read_bytes()) if (out/'build/reproduced-builds.json').exists() else {}
    captured=json.loads((out/'build/captured-inputs.json').read_bytes()) if (out/'build/captured-inputs.json').exists() else {}
    binaries=[out/'build/frontend-0.elf',out/'build/frontend-1.elf']
    binary_records=[seal(p) for p in binaries if p.exists()]
    list_match=bool(observations) and captured.get('build_inputs')==observations.get('build_objects')
    equal=len(binary_records)==2 and binary_records[0]['sha256']==binary_records[1]['sha256'] and binary_records[0]['bytes']==binary_records[1]['bytes']
    passed=code==0 and unchanged and observations.get('original_sources')==213 and len(observations.get('build_objects',[]))==201 and list_match and equal and built.get('byte_exact_equal') is True and len(built.get('builds',[]))==2
    report.update(status='passed' if passed else 'failed',exit_code=code,seconds=time.monotonic()-start,consumed_inputs_unchanged=unchanged,original_sources=observations.get('original_sources'),build_inputs=len(observations.get('build_objects',[])),native_build_inputs_equal=list_match,whole_binary_bytes_equal=equal,stdout=seal(out/'stdout.log'),stderr=seal(out/'stderr.log'),binary_outputs=binary_records)
    if observations:report['native_observations']=seal(out/'native.json')
    if built:report['reproduced_builds']=seal(out/'build/reproduced-builds.json')
    if captured:report['captured_inputs']=seal(out/'build/captured-inputs.json')
    write(out/'result.json',report);print(json.dumps({k:report[k] for k in ['status','exit_code','seconds','consumed_inputs_unchanged','build_inputs','native_build_inputs_equal','whole_binary_bytes_equal']}));raise SystemExit(not passed)


if __name__=='__main__':main()
