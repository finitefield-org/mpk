"""Observe the complete Go build-info controls with the registered Linux SDK."""
from pathlib import Path
import json
import os
import subprocess
import sys
import tarfile
import time

ROOT = Path(__file__).resolve().parent
REPO = ROOT.parents[5]
SDK = Path('/private/tmp/mpk-installed-compiler-205-original-go-output/toolchain')
IMAGE = 'docker.io/library/golang@sha256:ebd54034f076819b3054f155db53660ded951612bb4dfd277f933e62059e5d5a'
sys.dont_write_bytecode = True
sys.path.insert(0, str(ROOT.parent / 'installed-compiler'))
from run_scoped import seal, sdk_files, write


def main():
    out = ROOT / 'native'
    out.mkdir(exist_ok=False)
    files = {Path(__file__).resolve(), ROOT / 'prelaunch/result.json', REPO / 'release/bundles/candidates/go.json'}
    files.update(p for p in (REPO / 'go-tools/go2vir').rglob('*') if p.is_file())
    before = [seal(p) for p in sorted(files)]
    sdk = sdk_files()
    archive = out / 'inputs.tar.gz'
    with tarfile.open(archive, 'x:gz', dereference=True) as target:
        for path in sorted(files):
            target.add(path, arcname=str(path.relative_to(REPO)), recursive=False)
    env = {'GOROOT': '/sdk/go', 'GOTOOLCHAIN': 'local', 'GOENV': 'off', 'GOWORK': 'off', 'GOFLAGS': '-mod=vendor', 'GOPROXY': 'off', 'GOSUMDB': 'off', 'CGO_ENABLED': '0', 'GOOS': 'linux', 'GOARCH': 'amd64', 'GOAMD64': 'v1', 'GOMAXPROCS': '1', 'GOCACHE': '/tmp/cache', 'MPK_GO_ANALYSIS_BUDGET_OUTPUT': '/evidence'}
    command = ['/usr/local/bin/docker', 'run', '--pull=never', '--rm', '--platform', 'linux/amd64', '--network', 'none', '--read-only', '--cap-drop', 'ALL', '--security-opt', 'no-new-privileges', '--user', f'{os.getuid()}:{os.getgid()}', '--tmpfs', '/tmp:rw,exec,nosuid,nodev,size=1g,mode=1777', '--mount', f'type=bind,src={REPO},dst=/workspace,readonly', '--mount', f'type=bind,src={SDK},dst=/sdk,readonly', '--mount', f'type=bind,src={out},dst=/evidence', '--workdir', '/workspace/go-tools/go2vir']
    for key, value in env.items():
        command.extend(['--env', f'{key}={value}'])
    command.extend([IMAGE, '/sdk/go/bin/go', 'test', '-json', '-count=1', '-timeout=15m', '-run', '^TestPracticalAnalysisBudgetDifferential$'])
    write(out / 'inputs.json', {'inputs': before, 'source_archive': seal(archive), 'registered_sdk': sdk, 'command': command, 'environment': env, 'selection_reason': 'Retain the full original analysis owner and compare the exact and one-over shared directory/archive namespace budgets; original source objects remain in the preceding native capture. No application, SDK, root or proof scope is narrowed.'})
    report = {'status': 'running', 'native_Linux_release_acceptance': False, 'full_T_gate': 'Deferred to final T02-W04: ./scripts/check-fast.sh'}
    write(out / 'result.json', report)
    start = time.monotonic()
    with (out / 'stdout.log').open('xb') as stdout, (out / 'stderr.log').open('xb') as stderr:
        code = subprocess.run(command, stdout=stdout, stderr=stderr, check=False).returncode
    unchanged = before == [seal(p) for p in sorted(files)] and sdk == sdk_files()
    observations = json.loads((out / 'native.json').read_bytes()) if (out / 'native.json').exists() else {}
    passed = code == 0 and unchanged and observations.get('original_sources') == 213 and observations.get('vendor_files') == 133 and len(observations.get('cases', [])) == 3
    report.update(status='passed' if passed else 'failed', exit_code=code, seconds=time.monotonic()-start, consumed_inputs_unchanged=unchanged, observations=len(observations.get('cases', [])), stdout=seal(out / 'stdout.log'), stderr=seal(out / 'stderr.log'))
    if observations:
        report['native_observations'] = seal(out / 'native.json')
        report['snapshot_controls'] = [seal(p) for p in sorted(out.glob('*.snapshot.json'))]
    write(out / 'result.json', report)
    print(json.dumps({key: report[key] for key in ['status', 'exit_code', 'seconds', 'consumed_inputs_unchanged', 'observations']}))
    raise SystemExit(not passed)


if __name__ == '__main__':
    main()
