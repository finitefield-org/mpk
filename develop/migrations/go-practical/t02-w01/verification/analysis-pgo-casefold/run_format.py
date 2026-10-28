"""Check formatting of each changed Rust producer and direct consumer."""
from pathlib import Path
import json
import subprocess
import sys
import time

ROOT=Path(__file__).resolve().parent;REPO=ROOT.parents[5]
sys.dont_write_bytecode=True;sys.path.insert(0,str(ROOT.parent/'installed-compiler'));from run_scoped import seal,write


def main():
    epoch=sys.argv[1];assert epoch.replace('-','').isalnum() and Path.cwd()==REPO
    out=ROOT/epoch;out.mkdir(exist_ok=False)
    paths=[REPO/name for name in ['crates/mpk-vc/src/go_practical_type_keys.rs','crates/mpk-vc/src/go_practical_type_keys/analysis_vendor.rs','crates/mpk-vc/src/go_practical_type_keys/analysis_vendor/source.rs','crates/mpk-vc/tests/go_practical_analysis_pgo.rs','crates/mpk-vc/tests/go_practical_analysis_casefold.rs','crates/mpk-cli/src/go_practical_source_roots.rs','crates/mpk-cli/src/go_practical_analysis_root_binding.rs','crates/mpk-cli/tests/go_practical_analysis_root_pgo.rs','crates/mpk-cli/tests/go_practical_analysis_casefold.rs']]
    before=[seal(path) for path in paths]
    command=['rustfmt','--edition','2021','--check','--config','skip_children=true']+[str(p) for p in paths]
    write(out/'inputs.json',{'selection_reason':'Format-check each changed source and directly affected PGO/root test; no workspace-wide gate is run during W01.','inputs':before,'command':command})
    report={'status':'running','native_Linux_release_acceptance':False,'full_T_gate':'Deferred to final T02-W04: ./scripts/check-fast.sh'};write(out/'result.json',report);start=time.monotonic()
    with (out/'stdout.log').open('xb') as stdout,(out/'stderr.log').open('xb') as stderr:code=subprocess.run(command,cwd=REPO,stdout=stdout,stderr=stderr,check=False).returncode
    unchanged=before==[seal(path) for path in paths]
    report.update(status='passed' if code==0 and unchanged else 'failed',exit_code=code,seconds=time.monotonic()-start,original_inputs_unchanged=unchanged,stdout=seal(out/'stdout.log'),stderr=seal(out/'stderr.log'))
    write(out/'result.json',report);print(json.dumps({k:report[k] for k in ['status','exit_code','seconds','original_inputs_unchanged']}),flush=True);raise SystemExit(report['status']!='passed')

if __name__=='__main__':main()
