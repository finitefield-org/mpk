"""Seal affected production inputs and check Linux compilation locally."""
from pathlib import Path
import json
import os
import subprocess
import sys
import tarfile
import time

ROOT=Path(__file__).resolve().parent
REPO=ROOT.parents[5]
TARGET=Path('/private/tmp/mpk-go-source-root-clippy-target')
sys.dont_write_bytecode=True
sys.path.insert(0,str(ROOT.parent/'installed-compiler'))
from run_scoped import seal,write


def main():
    epoch=sys.argv[1];assert epoch.replace('-','').isalnum() and Path.cwd()==REPO
    out=ROOT/epoch;out.mkdir(exist_ok=False)
    paths={Path(__file__).resolve(),REPO/'crates/mpk-cli/src/go_practical_capture_binding.rs',REPO/'crates/mpk-cli/src/go_practical_source_roots.rs',REPO/'crates/mpk-cli/src/go_practical_installed_compiler.rs',REPO/'crates/mpk-vc/src/go_practical_type_keys/snapshot_inputs.rs',ROOT.parent/'source-root-live/live-original/independent-inspection.json'}
    sys.path.insert(0,str(REPO/'develop/probes/go-practical'))
    from frontend_native_build import linked_inputs
    paths.update(REPO/r['path'] for r in linked_inputs())
    before=[seal(p) for p in sorted(paths)]
    with tarfile.open(out/'inputs.tar.gz','x:gz',dereference=True) as archive:
        for path in sorted(paths):archive.add(path,arcname=str(path.relative_to(REPO)),recursive=False)
    command=['cargo','check','--offline','--locked','-p','mpk-cli','--lib','--target','x86_64-unknown-linux-gnu']
    env={'CARGO_TARGET_DIR':str(TARGET),'CARGO_NET_OFFLINE':'true','CARGO_INCREMENTAL':'0','CARGO_PROFILE_DEV_DEBUG':'0'}
    write(out/'inputs.json',{'selection_reason':'Check the new private root/compiler binding and its complete linked Rust sources against the Linux x86_64 target before running the original multi-module owner tests.','sources':before,'archive':seal(out/'inputs.tar.gz'),'command':command,'environment':env})
    report={'status':'running','full_T_gate':'Deferred to final T02-W04: ./scripts/check-fast.sh','native_Linux_release_acceptance':False};write(out/'result.json',report)
    start=time.monotonic()
    with (out/'stdout.log').open('xb') as stdout,(out/'stderr.log').open('xb') as stderr:
        code=subprocess.run(command,cwd=REPO,env={**os.environ,**env},stdout=stdout,stderr=stderr,check=False).returncode
    unchanged=before==[seal(p) for p in sorted(paths)]
    report.update(status='passed' if code==0 and unchanged else 'failed',exit_code=code,seconds=time.monotonic()-start,original_inputs_unchanged=unchanged,stdout=seal(out/'stdout.log'),stderr=seal(out/'stderr.log'));write(out/'result.json',report)
    print(json.dumps({k:report[k] for k in ['status','exit_code','seconds','original_inputs_unchanged']}),flush=True);raise SystemExit(report['status']!='passed')


if __name__=='__main__':main()
