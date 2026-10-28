"""Record actual case-folded profile resolution against the entire source tree."""
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
    epoch,mode=sys.argv[1:];assert mode in ('folded','sensitive')
    assert epoch.replace('-','').isalnum() and Path.cwd()==REPO
    out=ROOT/epoch
    out.mkdir(exist_ok=False)
    temp=Path('/private/tmp')/('mpk-analysis-pgo-casefold-'+epoch)
    assert not temp.exists()
    temp.mkdir(mode=0o700)
    (temp/'tmp').mkdir(); (temp/'cache').mkdir()
    paths={Path(__file__).resolve(),ROOT/'mode-casefold-test.go',ROOT.parent/'installed-compiler/original-go-source.tar',ROOT.parent/'installed-compiler/original-go-source.json'}
    paths.update(p for p in (REPO/'go-tools/go2vir').rglob('*') if p.is_file())
    before=[seal(p) for p in sorted(paths)];sdk=sdk_files()
    with tarfile.open(out/'inputs.tar.gz','x:gz',dereference=True) as archive:
        for p in sorted(paths): archive.add(p,arcname=str(p.relative_to(REPO)),recursive=False)
    overlay={'Replace':{'/workspace/go-tools/go2vir/practical_pgo_casefold_experiment_test.go':'/case/mode-casefold-test.go'}}
    write(out/'overlay.json',overlay)
    env={'GOROOT':'/sdk/go','GOTOOLCHAIN':'local','GOENV':'off','GOWORK':'off','GOFLAGS':'-mod=vendor','GOPROXY':'off','GOSUMDB':'off','CGO_ENABLED':'0','GOOS':'linux','GOARCH':'amd64','GOAMD64':'v1','GOMAXPROCS':'1','GOCACHE':'/cache','TMPDIR':'/case-tmp/tmp' if mode=='folded' else '/tmp','HOME':'/case-tmp/tmp','MPK_PGO_CASEFOLD_OUTPUT':'/evidence','MPK_CASE_FOLD_EXPECTED':'1' if mode=='folded' else '0'}
    command=['/usr/local/bin/docker','run','--pull=never','--rm','--platform','linux/amd64','--network','none','--read-only','--cap-drop','ALL','--security-opt','no-new-privileges','--user',f'{os.getuid()}:{os.getgid()}','--tmpfs','/tmp:rw,exec,nosuid,nodev,size=2g,mode=1777','--mount',f'type=bind,src={REPO},dst=/workspace,readonly','--mount',f'type=bind,src={SDK},dst=/sdk,readonly','--mount','type=bind,src=/private/tmp/mpk-go-registered-analysis-vet-cache,dst=/cache','--mount',f'type=bind,src={ROOT},dst=/case,readonly','--mount',f'type=bind,src={temp},dst=/case-tmp','--mount',f'type=bind,src={out},dst=/evidence','--workdir','/workspace/go-tools/go2vir']
    for key,value in env.items():command.extend(['--env',f'{key}={value}'])
    command.extend([IMAGE,'/sdk/go/bin/go','test','-overlay=/evidence/overlay.json','-json','-count=1','-timeout=20m','-run','^TestPracticalAnalysisCaseSensitiveAndFoldedPGO$'])
    write(out/'inputs.json',{'selection_reason':'Use the complete original registered frontend and actual SDK profile on the original case-insensitive host filesystem, and compare Go default PGO selection with the current exact-name capture. No original source is replaced or omitted.','inputs':before,'archive':seal(out/'inputs.tar.gz'),'registered_sdk':sdk,'command':command,'environment':env,'overlay':seal(out/'overlay.json')})
    report={'status':'running','native_Linux_release_acceptance':False,'full_T_gate':'Deferred to final T02-W04: ./scripts/check-fast.sh'};write(out/'result.json',report);start=time.monotonic()
    with (out/'stdout.log').open('xb') as stdout,(out/'stderr.log').open('xb') as stderr:
        code=subprocess.run(command,cwd=REPO,stdout=stdout,stderr=stderr,check=False).returncode
    unchanged=before==[seal(p) for p in sorted(paths)] and sdk==sdk_files()
    observation=json.loads((out/'native.json').read_bytes()) if (out/'native.json').exists() else {}
    report.update(status='passed' if code==0 and unchanged else 'failed',exit_code=code,seconds=time.monotonic()-start,original_inputs_unchanged=unchanged,observation={'go_lookup_resolves':observation.get('go_lookup_resolves'),'source_objects':len(observation.get('original_sources',[])),'build_objects':len(observation.get('build_objects',[]))},stdout=seal(out/'stdout.log'),stderr=seal(out/'stderr.log'))
    write(out/'result.json',report)
    print(json.dumps({k:report[k] for k in ['status','exit_code','seconds','original_inputs_unchanged','observation']}),flush=True)
    raise SystemExit(report['status']!='passed')

if __name__=='__main__':main()
