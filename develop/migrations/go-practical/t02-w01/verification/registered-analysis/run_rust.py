"""Check the new independent Rust owner against the full Go observation."""
from pathlib import Path
import json
import os
import subprocess
import sys
import tarfile
import time

ROOT=Path(__file__).resolve().parent
REPO=ROOT.parents[5]
sys.dont_write_bytecode=True;sys.path.insert(0,str(ROOT.parent/'installed-compiler'))
from run_scoped import seal,write


def main():
    epoch,mode,native_name=sys.argv[1:];assert all(x.replace('-','').isalnum() for x in (epoch,native_name)) and Path.cwd()==REPO
    out=ROOT/epoch;out.mkdir(exist_ok=False);native=ROOT/native_name
    assert json.loads((native/'result.json').read_bytes())['status']=='passed'
    paths={Path(__file__).resolve(),REPO/'Cargo.toml',REPO/'Cargo.lock',REPO/'crates/mpk-vc/tests/go_practical_registered_analysis.rs',REPO/'go-tools/go2vir/practical_registered_analysis_test.go',native/'native.json'}
    paths.update(p for p in (native/'original-bytes').rglob('*') if p.is_file())
    for crate in ['mpk-vc','mpk-cert','mpk-core','mpk-kernel','mpk-theory']:
        base=REPO/'crates'/crate;paths.add(base/'Cargo.toml');paths.update(p for p in (base/'src').rglob('*') if p.is_file());paths.update(p for p in base.glob('build.rs') if p.is_file())
    probe=REPO/'develop/probes/go-practical';sys.path.insert(0,str(probe));from frontend_native_build import linked_inputs
    paths.update(REPO/r['path'] for r in linked_inputs());paths.update(probe/name for name in ['frontend_native_build.py','run_proof_pilot.py','check_scope_report.py'])
    before=[seal(p) for p in sorted(paths)]
    with tarfile.open(out/'inputs.tar.gz','x:gz',dereference=True) as archive:
        for path in sorted(paths):archive.add(path,arcname=str(path.relative_to(REPO)),recursive=False)
    command={'test':['cargo','test','--offline','--locked','-p','mpk-vc','--test','go_practical_registered_analysis'], 'clippy':['cargo','clippy','--offline','--locked','-p','mpk-vc','--test','go_practical_registered_analysis','--','-D','warnings'], 'format':['rustfmt','--edition','2021','--check','--config','skip_children=true',str(REPO/'crates/mpk-vc/tests/go_practical_registered_analysis.rs')]}[mode]
    env={'MPK_GO_REGISTERED_ANALYSIS_INPUT':str(native),'CARGO_TARGET_DIR':str(REPO/'target'),'CARGO_NET_OFFLINE':'true','CARGO_INCREMENTAL':'0','CARGO_PROFILE_TEST_DEBUG':'0','CARGO_PROFILE_TEST_OPT_LEVEL':'1','CARGO_PROFILE_DEV_DEBUG':'0'}
    write(out/'inputs.json',{'selection_reason':'Compare all original registered frontend build objects with the independently imported Go snapshot and its unmodified bytes; run only the new Rust consumer and relevant lint or format.','inputs':before,'archive':seal(out/'inputs.tar.gz'),'command':command,'environment':env})
    report={'status':'running','native_Linux_release_acceptance':False,'full_T_gate':'Deferred to final T02-W04: ./scripts/check-fast.sh'};write(out/'result.json',report);start=time.monotonic()
    with (out/'stdout.log').open('xb') as stdout,(out/'stderr.log').open('xb') as stderr:
        code=subprocess.run(command,cwd=REPO,env={**os.environ,**env},stdout=stdout,stderr=stderr,check=False).returncode
    unchanged=before==[seal(p) for p in sorted(paths)]
    report.update(status='passed' if code==0 and unchanged else 'failed',exit_code=code,seconds=time.monotonic()-start,original_inputs_unchanged=unchanged,stdout=seal(out/'stdout.log'),stderr=seal(out/'stderr.log'))
    write(out/'result.json',report);print(json.dumps({k:report[k] for k in ['status','exit_code','seconds','original_inputs_unchanged']}),flush=True);raise SystemExit(report['status']!='passed')


if __name__=='__main__':main()
