"""Check only the new Go and Rust test files with their intended formatters."""
from pathlib import Path
import json
import os
import subprocess
import sys

ROOT=Path(__file__).resolve().parent;REPO=ROOT.parents[5]
SDK=Path('/private/tmp/mpk-installed-compiler-205-original-go-output/toolchain')
IMAGE='docker.io/library/golang@sha256:ebd54034f076819b3054f155db53660ded951612bb4dfd277f933e62059e5d5a'
sys.dont_write_bytecode=True;sys.path.insert(0,str(ROOT.parent/'installed-compiler'))
from run_scoped import seal,write
files=[REPO/name for name in ['go-tools/go2vir/practical_registered_analysis_test.go','crates/mpk-vc/tests/go_practical_registered_analysis.rs','crates/mpk-cli/tests/go_practical_registered_roots.rs']]
before=[seal(p) for p in files]
commands=[['rustfmt','--edition','2021','--check','--config','skip_children=true',*map(str,files[1:])],['/usr/local/bin/docker','run','--pull=never','--rm','--platform','linux/amd64','--network','none','--read-only','--cap-drop','ALL','--security-opt','no-new-privileges','--user',f'{os.getuid()}:{os.getgid()}','--mount',f'type=bind,src={REPO},dst=/workspace,readonly','--mount',f'type=bind,src={SDK},dst=/sdk,readonly',IMAGE,'/sdk/go/bin/gofmt','-l','/workspace/go-tools/go2vir/practical_registered_analysis_test.go']]
rows=[]
for command in commands:
    result=subprocess.run(command,cwd=REPO,capture_output=True,check=False)
    rows.append({'command':command,'exit_code':result.returncode,'stdout':result.stdout.decode(),'stderr':result.stderr.decode()})
unchanged=before==[seal(p) for p in files]
passed=unchanged and all(r['exit_code']==0 and not r['stdout'] for r in rows)
out=ROOT/'formats.json';assert not out.exists();write(out,{'status':'passed' if passed else 'failed','selection_reason':'Check only the changed Go and Rust tests using rustfmt and the registered Go 1.25 formatter, without formatting unrelated files.','inputs':before,'inputs_unchanged':unchanged,'results':rows,'full_T_gate':'Deferred to final T02-W04: ./scripts/check-fast.sh'});print(json.dumps({'status':'passed' if passed else 'failed','files':len(files)}));raise SystemExit(not passed)
