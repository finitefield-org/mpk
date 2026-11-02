"""Check only the changed Go package and oracle with the registered SDK."""
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
    out = ROOT / 'go-checks'
    out.mkdir(exist_ok=False)
    cache = Path('/private/tmp/mpk-go-analysis-vendor-vet-cache')
    cache.mkdir(exist_ok=False)
    files = {Path(__file__).resolve(), REPO / 'release/bundles/candidates/go.json'}
    files.update(p for p in (REPO / 'go-tools/go2vir').rglob('*') if p.is_file())
    before = [seal(p) for p in sorted(files)]
    sdk = sdk_files()
    archive = out / 'inputs.tar.gz'
    with tarfile.open(archive, 'x:gz', dereference=True) as target:
        for path in sorted(files):
            target.add(path, arcname=str(path.relative_to(REPO)), recursive=False)
    env = {'GOROOT': '/sdk/go', 'GOTOOLCHAIN': 'local', 'GOENV': 'off', 'GOWORK': 'off', 'GOFLAGS': '-mod=vendor', 'GOPROXY': 'off', 'GOSUMDB': 'off', 'CGO_ENABLED': '0', 'GOOS': 'linux', 'GOARCH': 'amd64', 'GOAMD64': 'v1', 'GOMAXPROCS': '1', 'GOCACHE': '/cache'}
    base = ['/usr/local/bin/docker', 'run', '--pull=never', '--rm', '--platform', 'linux/amd64', '--network', 'none', '--read-only', '--cap-drop', 'ALL', '--security-opt', 'no-new-privileges', '--user', f'{os.getuid()}:{os.getgid()}', '--tmpfs', '/tmp:rw,exec,nosuid,nodev,size=1g,mode=1777', '--mount', f'type=bind,src={REPO},dst=/workspace,readonly', '--mount', f'type=bind,src={SDK},dst=/sdk,readonly', '--mount', f'type=bind,src={cache},dst=/cache', '--workdir', '/workspace/go-tools/go2vir']
    for key, value in env.items():
        base.extend(['--env', f'{key}={value}'])
    commands = [('format', base + [IMAGE, '/sdk/go/bin/gofmt', '-d', 'practical_analysis_vendor_differential_test.go']), ('vet', base + [IMAGE, '/sdk/go/bin/go', 'vet', '.'])]
    write(out / 'inputs.json', {'inputs': before, 'source_archive': seal(archive), 'registered_sdk': sdk, 'commands': commands, 'environment': env, 'selection_reason': 'Check format of the changed Go vendor-manifest oracle and vet its complete affected Go package using the exact registered SDK. This is local emulation, not native Linux release acceptance or a whole T gate.'})
    report = {'status': 'running', 'native_Linux_release_acceptance': False, 'full_T_gate': 'Deferred to final T02-W04: ./scripts/check-fast.sh'}
    write(out / 'result.json', report)
    start, results = time.monotonic(), []
    for name, command in commands:
        child = subprocess.run(command, capture_output=True, check=False)
        stdout, stderr = out / f'{name}.stdout', out / f'{name}.stderr'
        stdout.write_bytes(child.stdout)
        stderr.write_bytes(child.stderr)
        results.append({'name': name, 'exit_code': child.returncode, 'stdout': seal(stdout), 'stderr': seal(stderr)})
        write(out / 'commands.json', results)
    unchanged = before == [seal(p) for p in sorted(files)] and sdk == sdk_files()
    passed = unchanged and all(row['exit_code'] == 0 and row['stdout']['bytes'] == 0 and row['stderr']['bytes'] == 0 for row in results)
    report.update(status='passed' if passed else 'failed', seconds=time.monotonic()-start, consumed_inputs_unchanged=unchanged, results=results)
    write(out / 'result.json', report)
    print(json.dumps({'status': report['status'], 'consumed_inputs_unchanged': unchanged, 'exit_codes': [row['exit_code'] for row in results]}))
    raise SystemExit(not passed)


if __name__ == '__main__':
    main()
