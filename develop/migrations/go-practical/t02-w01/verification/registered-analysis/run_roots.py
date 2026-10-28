"""Match the historical registered capture against independently held roots."""
from pathlib import Path
import json
import os
import subprocess
import sys
import tarfile
import time

ROOT=Path(__file__).resolve().parent
REPO=ROOT.parents[5]
TEMP=Path('/private/tmp/mpk-go-source-root-matching')
CARGO_REGISTRY=Path('/Users/kazuyoshitoshiya/.cargo/registry')
IMAGE='sha256:7cf1e580ef5539f03b58560753e8ab84c8c360960d99dff714004aa98f203977'
sys.dont_write_bytecode=True;sys.path.insert(0,str(ROOT.parent/'installed-compiler'))
from run_scoped import seal,write


def main():
    epoch,native_name=sys.argv[1:];assert all(x.replace('-','').isalnum() for x in (epoch,native_name)) and Path.cwd()==REPO
    out=ROOT/epoch;out.mkdir(exist_ok=False);native=ROOT/native_name
    assert json.loads((native/'result.json').read_bytes())['status']=='passed'
    paths={Path(__file__).resolve(),ROOT/'restore_registered.py',ROOT.parent/'installed-compiler/original-go-source.tar',ROOT.parent/'installed-compiler/original-go-source.json',native/'native.json',REPO/'Cargo.toml',REPO/'Cargo.lock',REPO/'crates/mpk-cli/tests/go_practical_registered_roots.rs'}
    paths.update(p for p in (native/'original-bytes').rglob('*') if p.is_file())
    for crate in ['mpk-cli','mpk-api','mpk-cert','mpk-core','mpk-kernel','mpk-linux-sandbox','mpk-theory','mpk-vc']:
        base=REPO/'crates'/crate;paths.add(base/'Cargo.toml');paths.update(p for p in (base/'src').rglob('*') if p.is_file());paths.update(p for p in base.glob('build.rs') if p.is_file())
    paths.update(p for p in (REPO/'go-tools/go2vir').rglob('*') if p.is_file())
    paths.update(p for p in (REPO/'release/bundles').rglob('*') if p.is_file())
    probe=REPO/'develop/probes/go-practical';sys.path.insert(0,str(probe));from frontend_native_build import linked_inputs
    paths.update(REPO/r['path'] for r in linked_inputs());paths.update(probe/name for name in ['frontend_native_build.py','run_proof_pilot.py','check_scope_report.py'])
    before=[seal(p) for p in sorted(paths)]
    with tarfile.open(out/'inputs.tar.gz','x:gz',dereference=True) as archive:
        for path in sorted(paths):archive.add(path,arcname=str(path.relative_to(REPO)),recursive=False)
    TEMP.mkdir(exist_ok=True);cargo=TEMP/'cargo';cargo.mkdir(exist_ok=True);(cargo/'registry').mkdir(exist_ok=True);target=TEMP/'target';target.mkdir(exist_ok=True)
    env={'CARGO_HOME':'/cargo','CARGO_TARGET_DIR':'/target','CARGO_NET_OFFLINE':'true','CARGO_INCREMENTAL':'0','CARGO_PROFILE_TEST_DEBUG':'0','CARGO_PROFILE_TEST_OPT_LEVEL':'1','RUST_TEST_THREADS':'1','HOME':'/tmp','MPK_GO_REGISTERED_ANALYSIS_INPUT':'/workspace/'+str(native.relative_to(REPO))}
    command=['/usr/local/bin/docker','run','--pull=never','--rm','--platform','linux/arm64','--network','none','--read-only','--cap-drop','ALL','--security-opt','no-new-privileges','--user',f'{os.getuid()}:{os.getgid()}','--tmpfs','/tmp:rw,exec,nosuid,nodev,size=1g,mode=1777','--mount',f'type=bind,src={REPO},dst=/workspace,readonly','--mount',f'type=bind,src={cargo},dst=/cargo','--mount',f'type=bind,src={CARGO_REGISTRY},dst=/cargo/registry,readonly','--mount',f'type=bind,src={target},dst=/target','--workdir','/workspace']
    for key,value in env.items():command.extend(['--env',f'{key}={value}'])
    command.extend([IMAGE,'cargo','test','--offline','--locked','-p','mpk-cli','--test','go_practical_registered_roots','registered_frontend_capture_matches_all_restored_original_root_bytes','--','--nocapture'])
    write(out/'inputs.json',{'selection_reason':'Open restored registered-epoch frontend and original archive roots independently; compare every native snapshot object and directory entry, then exercise source and unread-entry controls. Preserve all original source files and the complete earlier capture.','inputs':before,'archive':seal(out/'inputs.tar.gz'),'command':command,'environment':env})
    report={'status':'running','native_Linux_release_acceptance':False,'full_T_gate':'Deferred to final T02-W04: ./scripts/check-fast.sh'};write(out/'result.json',report);start=time.monotonic()
    with (out/'stdout.log').open('xb') as stdout,(out/'stderr.log').open('xb') as stderr:
        code=subprocess.run(command,cwd=REPO,stdout=stdout,stderr=stderr,check=False).returncode
    unchanged=before==[seal(p) for p in sorted(paths)]
    report.update(status='passed' if code==0 and unchanged else 'failed',exit_code=code,seconds=time.monotonic()-start,original_inputs_unchanged=unchanged,stdout=seal(out/'stdout.log'),stderr=seal(out/'stderr.log'))
    write(out/'result.json',report);print(json.dumps({k:report[k] for k in ['status','exit_code','seconds','original_inputs_unchanged']}),flush=True);raise SystemExit(report['status']!='passed')


if __name__=='__main__':main()
