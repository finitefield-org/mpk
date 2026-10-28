"""Reproduce and verify the independent Rust default-profile boundary."""
from pathlib import Path
import json
import os
import subprocess
import sys
import tarfile
import time

ROOT=Path(__file__).resolve().parent;REPO=ROOT.parents[5]
sys.dont_write_bytecode=True;sys.path.insert(0,str(ROOT.parent/'installed-compiler'));from run_scoped import seal,write


def main():
    epoch,mode=sys.argv[1:];assert epoch.replace('-','').isalnum() and Path.cwd()==REPO
    out=ROOT/epoch;out.mkdir(exist_ok=False)
    directories=[ROOT/'native-before-fix']+([ROOT/'native-after-fix'] if mode in ('after','clippy') else [])
    expected=['failed']+(['passed'] if len(directories)==2 else [])
    for p,status in zip(directories,expected):assert json.loads((p/'result.json').read_bytes())['status']==status
    paths={Path(__file__).resolve(),REPO/'Cargo.toml',REPO/'Cargo.lock',REPO/'crates/mpk-vc/tests/go_practical_analysis_pgo.rs',REPO/'go-tools/go2vir/practical_analysis_pgo_test.go'}
    for directory in directories:
        paths.update([directory/'native.json',directory/'original-profile.raw']);paths.update(p for p in (directory/'original-bytes').rglob('*') if p.is_file())
    for crate in ['mpk-vc','mpk-cert','mpk-core','mpk-kernel','mpk-theory']:
        base=REPO/'crates'/crate;paths.add(base/'Cargo.toml');paths.update(p for p in (base/'src').rglob('*') if p.is_file());paths.update(p for p in base.glob('build.rs') if p.is_file())
    probe=REPO/'develop/probes/go-practical';sys.path.insert(0,str(probe));from frontend_native_build import linked_inputs
    paths.update(REPO/r['path'] for r in linked_inputs());paths.update(probe/name for name in ['frontend_native_build.py','run_proof_pilot.py','check_scope_report.py'])
    before=[seal(p) for p in sorted(paths)]
    with tarfile.open(out/'inputs.tar.gz','x:gz',dereference=True) as archive:
        for path in sorted(paths):archive.add(path,arcname=str(path.relative_to(REPO)),recursive=False)
    commands={'before':['cargo','test','--offline','--locked','-p','mpk-vc','--test','go_practical_analysis_pgo','observed_but_uncaptured_default_profile_rejects'], 'after':['cargo','test','--offline','--locked','-p','mpk-vc','--test','go_practical_analysis_pgo'], 'clippy':['cargo','clippy','--offline','--locked','-p','mpk-vc','--lib','--test','go_practical_analysis_pgo','--','-D','warnings']}
    command=commands[mode];env={'CARGO_TARGET_DIR':str(REPO/'target'),'CARGO_NET_OFFLINE':'true','CARGO_INCREMENTAL':'0','CARGO_PROFILE_TEST_DEBUG':'0','CARGO_PROFILE_TEST_OPT_LEVEL':'1','CARGO_PROFILE_DEV_DEBUG':'0'}
    write(out/'inputs.json',{'selection_reason':'Use the complete original Go frontend and actual compiler profile to reproduce or check Rust rejection of an observed but uncaptured profile and exact acceptance after complete capture.','inputs':before,'archive':seal(out/'inputs.tar.gz'),'command':command,'environment':env})
    report={'status':'running','native_Linux_release_acceptance':False,'full_T_gate':'Deferred to final T02-W04: ./scripts/check-fast.sh'};write(out/'result.json',report);start=time.monotonic()
    with (out/'stdout.log').open('xb') as stdout,(out/'stderr.log').open('xb') as stderr:
        code=subprocess.run(command,cwd=REPO,env={**os.environ,**env},stdout=stdout,stderr=stderr,check=False).returncode
    unchanged=before==[seal(p) for p in sorted(paths)]
    report.update(status='passed' if code==0 and unchanged else 'failed',exit_code=code,seconds=time.monotonic()-start,original_inputs_unchanged=unchanged,stdout=seal(out/'stdout.log'),stderr=seal(out/'stderr.log'))
    write(out/'result.json',report);print(json.dumps({k:report[k] for k in ['status','exit_code','seconds','original_inputs_unchanged']}),flush=True);raise SystemExit(report['status']!='passed')


if __name__=='__main__':main()
