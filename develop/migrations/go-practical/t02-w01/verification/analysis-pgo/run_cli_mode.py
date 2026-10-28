"""Run directly affected Linux root/analysis integration checks."""
from pathlib import Path
import json
import os
import subprocess
import sys
import tarfile
import time

ROOT=Path(__file__).resolve().parent;REPO=ROOT.parents[5];VERIFY=ROOT.parent
TEMP=Path('/private/tmp/mpk-go-source-root-matching');CARGO_REGISTRY=Path('/Users/kazuyoshitoshiya/.cargo/registry');IMAGE='sha256:7cf1e580ef5539f03b58560753e8ab84c8c360960d99dff714004aa98f203977'
sys.dont_write_bytecode=True;sys.path.insert(0,str(VERIFY/'installed-compiler'));from run_scoped import seal,write


def main():
    epoch,mode=sys.argv[1:];assert epoch.replace('-','').isalnum() and Path.cwd()==REPO
    out=ROOT/epoch;out.mkdir(exist_ok=False)
    for name,status in [('native-before-fix','failed'),('native-mode-aware','passed')]:assert json.loads((ROOT/name/'result.json').read_bytes())['status']==status
    paths={Path(__file__).resolve(),VERIFY/'registered-analysis/restore_registered.py',VERIFY/'installed-compiler/original-go-source.tar',VERIFY/'installed-compiler/original-go-source.json',VERIFY/'registered-analysis/native-fixed/native.json',VERIFY/'analysis-namespace/native/native.json',VERIFY/'analysis-build-list/native/native.json',REPO/'Cargo.toml',REPO/'Cargo.lock',REPO/'crates/mpk-cli/tests/go_practical_analysis_root_binding.rs',REPO/'crates/mpk-cli/tests/go_practical_analysis_root_pgo.rs'}
    for directory in [VERIFY/'registered-analysis/native-fixed/original-bytes',VERIFY/'analysis-namespace/native/original-bytes',ROOT/'native-before-fix/original-bytes',ROOT/'native-mode-aware/original-bytes']:paths.update(p for p in directory.rglob('*') if p.is_file())
    for directory in [ROOT/'native-before-fix',ROOT/'native-mode-aware']:paths.update([directory/'native.json',directory/'original-profile.raw'])
    for crate in ['mpk-cli','mpk-api','mpk-cert','mpk-core','mpk-kernel','mpk-linux-sandbox','mpk-theory','mpk-vc']:
        base=REPO/'crates'/crate;paths.add(base/'Cargo.toml');paths.update(p for p in (base/'src').rglob('*') if p.is_file());paths.update(p for p in base.glob('build.rs') if p.is_file())
    paths.update(p for p in (REPO/'go-tools/go2vir').rglob('*') if p.is_file());paths.update(p for p in (REPO/'release/bundles').rglob('*') if p.is_file())
    probe=REPO/'develop/probes/go-practical';sys.path.insert(0,str(probe));from frontend_native_build import linked_inputs
    paths.update(REPO/r['path'] for r in linked_inputs());paths.update(probe/name for name in ['frontend_native_build.py','run_proof_pilot.py','check_scope_report.py'])
    before=[seal(p) for p in sorted(paths)]
    with tarfile.open(out/'inputs.tar.gz','x:gz',dereference=True) as archive:
        for path in sorted(paths):archive.add(path,arcname=str(path.relative_to(REPO)),recursive=False)
    if mode in ('pgo','baseline'):
        TEMP.mkdir(exist_ok=True);cargo=TEMP/'cargo';cargo.mkdir(exist_ok=True);(cargo/'registry').mkdir(exist_ok=True);target=TEMP/'target';target.mkdir(exist_ok=True)
        env={'CARGO_HOME':'/cargo','CARGO_TARGET_DIR':'/target','CARGO_NET_OFFLINE':'true','CARGO_INCREMENTAL':'0','CARGO_PROFILE_TEST_DEBUG':'0','CARGO_PROFILE_TEST_OPT_LEVEL':'1','RUST_TEST_THREADS':'1','HOME':'/tmp'}
        command=['/usr/local/bin/docker','run','--pull=never','--rm','--platform','linux/arm64','--network','none','--read-only','--cap-drop','ALL','--security-opt','no-new-privileges','--user',f'{os.getuid()}:{os.getgid()}','--tmpfs','/tmp:rw,exec,nosuid,nodev,size=1g,mode=1777','--mount',f'type=bind,src={REPO},dst=/workspace,readonly','--mount',f'type=bind,src={cargo},dst=/cargo','--mount',f'type=bind,src={CARGO_REGISTRY},dst=/cargo/registry,readonly','--mount',f'type=bind,src={target},dst=/target','--workdir','/workspace']
        for key,value in env.items():command.extend(['--env',f'{key}={value}'])
        selected={'pgo':('go_practical_analysis_root_pgo','root_matched_analysis_retains_the_real_default_profile'),'baseline':('go_practical_analysis_root_binding','independent_analysis_owner_binds_only_to_the_same_complete_roots')}[mode]
        command.extend([IMAGE,'cargo','test','--offline','--locked','-p','mpk-cli','--test',selected[0],selected[1],'--','--nocapture'])
    elif mode=='clippy':
        env={'CARGO_TARGET_DIR':'/private/tmp/mpk-go-source-root-clippy-target','CARGO_NET_OFFLINE':'true','CARGO_INCREMENTAL':'0','CARGO_PROFILE_DEV_DEBUG':'0'}
        command=['cargo','clippy','--offline','--locked','-p','mpk-cli','--lib','--test','go_practical_analysis_root_pgo','--test','go_practical_analysis_root_binding','--target','x86_64-unknown-linux-gnu','--','-D','warnings']
    else:
        env={};command=['rustfmt','--edition','2021','--check','--config','skip_children=true',str(REPO/'crates/mpk-vc/src/go_practical_type_keys/analysis_vendor/source.rs'),str(REPO/'crates/mpk-vc/src/go_practical_type_keys/analysis_vendor.rs'),str(REPO/'crates/mpk-vc/src/go_practical_type_keys.rs'),str(REPO/'crates/mpk-vc/tests/go_practical_analysis_pgo.rs'),str(REPO/'crates/mpk-cli/tests/go_practical_analysis_root_pgo.rs')]
    write(out/'inputs.json',{'selection_reason':'Exercise the directly affected analysis/root consumer with the real captured Go profile and the complete earlier/current analysis universes; retain the missing-profile negative and baseline source/namespace controls.','inputs':before,'archive':seal(out/'inputs.tar.gz'),'command':command,'environment':env})
    report={'status':'running','native_Linux_release_acceptance':False,'full_T_gate':'Deferred to final T02-W04: ./scripts/check-fast.sh'};write(out/'result.json',report);start=time.monotonic()
    with (out/'stdout.log').open('xb') as stdout,(out/'stderr.log').open('xb') as stderr:
        code=subprocess.run(command,cwd=REPO,env={**os.environ,**env} if mode not in ('pgo','baseline') else os.environ,stdout=stdout,stderr=stderr,check=False).returncode
    unchanged=before==[seal(p) for p in sorted(paths)]
    report.update(status='passed' if code==0 and unchanged else 'failed',exit_code=code,seconds=time.monotonic()-start,original_inputs_unchanged=unchanged,stdout=seal(out/'stdout.log'),stderr=seal(out/'stderr.log'))
    write(out/'result.json',report);print(json.dumps({k:report[k] for k in ['status','exit_code','seconds','original_inputs_unchanged']}),flush=True);raise SystemExit(report['status']!='passed')


if __name__=='__main__':main()
