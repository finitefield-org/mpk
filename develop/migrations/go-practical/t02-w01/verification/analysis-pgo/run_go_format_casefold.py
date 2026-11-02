"""Check Go formatting with the exact registered toolchain and retain inputs."""
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
sys.path.insert(0, str(ROOT.parent/'installed-compiler'))
from run_scoped import seal, sdk_files, write


def main():
    epoch = sys.argv[1]
    assert epoch.replace('-', '').isalnum() and Path.cwd() == REPO
    out = ROOT/epoch
    out.mkdir(exist_ok=False)
    paths = [Path(__file__).resolve(), REPO/'go-tools/go2vir/practical_analysis.go', REPO/'go-tools/go2vir/practical_analysis_pgo_test.go', ROOT.parent/'analysis-pgo-casefold/mode-casefold-test.go']
    before = [seal(p) for p in paths]
    sdk = sdk_files()
    with tarfile.open(out/'inputs.tar.gz', 'x:gz') as archive:
        for p in paths:
            archive.add(p, arcname=str(p.relative_to(REPO)), recursive=False)
    command = ['/usr/local/bin/docker', 'run', '--pull=never', '--rm', '--platform', 'linux/amd64', '--network', 'none', '--read-only', '--cap-drop', 'ALL', '--security-opt', 'no-new-privileges', '--user', f'{os.getuid()}:{os.getgid()}', '--mount', f'type=bind,src={REPO},dst=/workspace,readonly', '--mount', f'type=bind,src={SDK},dst=/sdk,readonly', '--workdir', '/workspace', IMAGE, '/sdk/go/bin/gofmt', '-l', 'go-tools/go2vir/practical_analysis.go', 'go-tools/go2vir/practical_analysis_pgo_test.go', 'develop/migrations/go-practical/t02-w01/verification/analysis-pgo-casefold/mode-casefold-test.go']
    write(out/'inputs.json', {'selection_reason': 'Check only the changed Go producer and its new direct tests with the registered gofmt; preserve original input bytes and complete SDK inventory.', 'inputs': before, 'archive': seal(out/'inputs.tar.gz'), 'registered_sdk': sdk, 'command': command})
    report = {'status': 'running', 'native_Linux_release_acceptance': False, 'full_T_gate': 'Deferred to final T02-W04: ./scripts/check-fast.sh'}
    write(out/'result.json', report)
    start = time.monotonic()
    with (out/'stdout.log').open('xb') as stdout, (out/'stderr.log').open('xb') as stderr:
        code = subprocess.run(command, cwd=REPO, stdout=stdout, stderr=stderr, check=False).returncode
    unchanged = before == [seal(p) for p in paths] and sdk == sdk_files()
    passed = code == 0 and unchanged and (out/'stdout.log').read_bytes() == b''
    report.update(status='passed' if passed else 'failed', exit_code=code, seconds=time.monotonic()-start, original_inputs_unchanged=unchanged, stdout=seal(out/'stdout.log'), stderr=seal(out/'stderr.log'))
    write(out/'result.json', report)
    print(json.dumps({k: report[k] for k in ['status', 'exit_code', 'seconds', 'original_inputs_unchanged']}), flush=True)
    raise SystemExit(not passed)


if __name__ == '__main__':
    main()
