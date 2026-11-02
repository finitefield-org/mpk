"""Run affected Rust source and PGO-resolution tests on complete inputs."""
from pathlib import Path
import json
import os
import subprocess
import sys
import tarfile
import time

ROOT=Path(__file__).resolve().parent;REPO=ROOT.parents[5];PREV=ROOT.parent/'analysis-pgo'
sys.dont_write_bytecode=True;sys.path.insert(0,str(ROOT.parent/'installed-compiler'));from run_scoped import seal,write


def main():
    epoch,mode=sys.argv[1:];assert epoch.replace('-','').isalnum() and mode in ('test','clippy') and Path.cwd()==REPO
    out=ROOT/epoch;out.mkdir(exist_ok=False)
    dirs=[PREV/'native-before-fix',PREV/'native-mode-aware',ROOT/'folded-final',ROOT/'sensitive-final']
    for path,status in zip(dirs,['failed','passed','passed','passed']):assert json.loads((path/'result.json').read_bytes())['status']==status
    tests=['go_practical_analysis_pgo','go_practical_analysis_casefold','go_practical_analysis_source','go_practical_analysis_build_list','go_practical_registered_analysis','go_practical_analysis_build_info']
    paths={Path(__file__).resolve(),REPO/'Cargo.toml',REPO/'Cargo.lock'}
    paths.update(REPO/'crates/mpk-vc/tests'/f'{name}.rs' for name in tests)
    for path in dirs:
        paths.update([path/'native.json',path/'original-profile.raw']);paths.update(p for p in (path/'original-bytes').rglob('*') if p.is_file())
    for directory in [PREV/'native-mode-aware/controls',ROOT.parent/'analysis-namespace/native',ROOT.parent/'analysis-source/native',ROOT.parent/'analysis-build-list/native',ROOT.parent/'registered-analysis/native-fixed']:
        paths.update(p for p in directory.rglob('*') if p.is_file() and ('original-bytes' in p.parts or 'changed-bytes' in p.parts or p.name=='native.json'))
    paths.update([ROOT.parent/'analysis-build-info/native/native.json',ROOT.parent/'analysis-build-info/registered-go2vir.elf',PREV/'actual-build-original/auto.elf.hex',PREV/'actual-build-original/off.elf.hex'])
    for crate in ['mpk-vc','mpk-cert','mpk-core','mpk-kernel','mpk-theory']:
        base=REPO/'crates'/crate;paths.add(base/'Cargo.toml');paths.update(p for p in (base/'src').rglob('*') if p.is_file());paths.update(p for p in base.glob('build.rs') if p.is_file())
    probe=REPO/'develop/probes/go-practical';sys.path.insert(0,str(probe));from frontend_native_build import linked_inputs
    paths.update(REPO/r['path'] for r in linked_inputs());paths.update(probe/name for name in ['frontend_native_build.py','run_proof_pilot.py','check_scope_report.py'])
    before=[seal(p) for p in sorted(paths)]
    with tarfile.open(out/'inputs.tar.gz','x:gz',dereference=True) as archive:
        for path in sorted(paths):archive.add(path,arcname=str(path.relative_to(REPO)),recursive=False)
    command=['cargo',mode,'--offline','--locked','-p','mpk-vc']
    if mode=='clippy':command+=['--lib'];tests=tests[:2]
    for name in tests:command+=['--test',name]
    if mode=='clippy':command+=['--','-D','warnings']
    env={'CARGO_TARGET_DIR':str(REPO/'target'),'CARGO_NET_OFFLINE':'true','CARGO_INCREMENTAL':'0','CARGO_PROFILE_TEST_DEBUG':'0','CARGO_PROFILE_TEST_OPT_LEVEL':'1','CARGO_PROFILE_DEV_DEBUG':'0'}
    write(out/'inputs.json',{'selection_reason':'Validate mode-aware PGO resolution and all directly affected source, build-list, historical binary and original-generation consumers, preserving complete inputs.','inputs':before,'archive':seal(out/'inputs.tar.gz'),'command':command,'environment':env})
    report={'status':'running','native_Linux_release_acceptance':False,'full_T_gate':'Deferred to final T02-W04: ./scripts/check-fast.sh'};write(out/'result.json',report);start=time.monotonic()
    with (out/'stdout.log').open('xb') as stdout,(out/'stderr.log').open('xb') as stderr:code=subprocess.run(command,cwd=REPO,env={**os.environ,**env},stdout=stdout,stderr=stderr,check=False).returncode
    unchanged=before==[seal(p) for p in sorted(paths)]
    report.update(status='passed' if code==0 and unchanged else 'failed',exit_code=code,seconds=time.monotonic()-start,original_inputs_unchanged=unchanged,stdout=seal(out/'stdout.log'),stderr=seal(out/'stderr.log'))
    write(out/'result.json',report);print(json.dumps({k:report[k] for k in ['status','exit_code','seconds','original_inputs_unchanged']}),flush=True);raise SystemExit(report['status']!='passed')

if __name__=='__main__':main()
