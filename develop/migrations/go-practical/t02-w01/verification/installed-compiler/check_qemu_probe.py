"""Vet and format-check only the standalone syscall diagnostic."""
from pathlib import Path
import json
import os
import subprocess
import sys

ROOT = Path(__file__).resolve().parent
SDK = Path('/private/tmp/mpk-installed-compiler-205-original-go-output/toolchain')
IMAGE = 'docker.io/library/golang@sha256:ebd54034f076819b3054f155db53660ded951612bb4dfd277f933e62059e5d5a'
sys.dont_write_bytecode = True
sys.path.insert(0, str(ROOT))
from run_scoped import seal, write, sdk_files


def main():
    out = ROOT / 'qemu-probe-check'
    out.mkdir(exist_ok=False)
    source = ROOT / 'qemu_openat2.go'
    inputs = [seal(path) for path in [Path(__file__).resolve(), ROOT / 'run_scoped.py', source]]
    sdk = sdk_files()
    env = {'GOROOT': '/sdk/go', 'GOTOOLCHAIN': 'local', 'GOENV': 'off', 'GOWORK': 'off', 'GO111MODULE': 'off', 'GOPROXY': 'off', 'GOSUMDB': 'off', 'CGO_ENABLED': '0', 'GOOS': 'linux', 'GOARCH': 'amd64', 'GOCACHE': '/tmp/cache', 'GOMAXPROCS': '1'}
    base = ['/usr/local/bin/docker', 'run', '--pull=never', '--rm', '--platform', 'linux/amd64', '--network', 'none', '--read-only', '--cap-drop', 'ALL', '--security-opt', 'no-new-privileges', '--user', f'{os.getuid()}:{os.getgid()}', '--tmpfs', '/tmp:rw,exec,nosuid,nodev,size=512m,mode=1777', '--mount', f'type=bind,src={SDK},dst=/sdk,readonly', '--mount', f'type=bind,src={source},dst=/probe.go,readonly']
    for key, value in env.items():
        base.extend(['--env', f'{key}={value}'])
    commands = [('format', base + [IMAGE, '/sdk/go/bin/gofmt', '-d', '/probe.go']), ('vet', base + [IMAGE, '/sdk/go/bin/go', 'vet', '/probe.go'])]
    write(out / 'inputs.json', {'inputs': inputs, 'registered_sdk': sdk, 'environment': env, 'commands': commands, 'selection_reason': 'Format and vet only the standalone Linux syscall diagnostic added for this investigation. This is local emulation, not native Linux release acceptance or the cumulative T02 gate.'})
    results = []
    for name, command in commands:
        child = subprocess.run(command, capture_output=True, check=False)
        stdout, stderr = out / f'{name}.stdout', out / f'{name}.stderr'
        stdout.write_bytes(child.stdout)
        stderr.write_bytes(child.stderr)
        results.append({'name': name, 'exit_code': child.returncode, 'stdout': seal(stdout), 'stderr': seal(stderr)})
    unchanged = inputs == [seal(path) for path in [Path(__file__).resolve(), ROOT / 'run_scoped.py', source]] and sdk == sdk_files()
    passed = unchanged and all(row['exit_code'] == 0 and row['stdout']['bytes'] == 0 and row['stderr']['bytes'] == 0 for row in results)
    report = {'status': 'passed' if passed else 'failed', 'inputs_unchanged': unchanged, 'results': results, 'native_Linux_release_acceptance': False, 'full_T_gate': 'Deferred to final T02-W04: ./scripts/check-fast.sh'}
    write(out / 'result.json', report)
    print(json.dumps({'status': report['status'], 'inputs_unchanged': unchanged, 'exit_codes': [row['exit_code'] for row in results]}))
    raise SystemExit(not passed)


if __name__ == '__main__':
    main()
