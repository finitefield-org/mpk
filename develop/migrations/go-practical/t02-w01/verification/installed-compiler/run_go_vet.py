"""Run only Go frontend vet with the original registered Linux SDK."""
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
sys.path.insert(0, str(ROOT))
from run_scoped import seal, write, sdk_files


def main():
    out = ROOT / 'go-vet-final'
    out.mkdir(exist_ok=False)
    sources = {Path(__file__).resolve(), ROOT / 'run_scoped.py', REPO / 'release/bundles/candidates/go.json'}
    sources.update(p for p in (REPO / 'go-tools/go2vir').rglob('*') if p.is_file())
    before = [seal(p) for p in sorted(sources)]
    sdk = sdk_files()
    with tarfile.open(out / 'inputs.tar.gz', 'x:gz', dereference=True) as archive:
        for p in sorted(sources):
            archive.add(p, arcname=str(p.relative_to(REPO)), recursive=False)
    write(out / 'inputs.json', {'sources': before, 'archive': seal(out / 'inputs.tar.gz'), 'complete_registered_SDK': sdk, 'selection_reason': 'Vet the changed Go frontend test package and all its actual package sources with the complete fixed Go1.25 Linux SDK; run locally without network. This is not a native Linux release gate or a whole T02 gate.'})
    cache = Path('/private/tmp/mpk-installed-compiler-205-vet-cache')
    cache.mkdir(exist_ok=False)
    command = ['/usr/local/bin/docker', 'run', '--pull=never', '--rm', '--platform', 'linux/amd64', '--network', 'none', '--read-only', '--cap-drop', 'ALL', '--security-opt', 'no-new-privileges', '--user', f'{os.getuid()}:{os.getgid()}', '--tmpfs', '/tmp:rw,exec,nosuid,nodev,size=2g,mode=1777', '--mount', f'type=bind,src={REPO},dst=/workspace,readonly', '--mount', f'type=bind,src={SDK},dst=/sdk,readonly', '--mount', f'type=bind,src={cache},dst=/cache', '--workdir', '/workspace/go-tools/go2vir']
    environment = {'GOROOT': '/sdk/go', 'GOTOOLCHAIN': 'local', 'GOENV': 'off', 'GOWORK': 'off', 'GOFLAGS': '-mod=vendor', 'GOPROXY': 'off', 'GOSUMDB': 'off', 'CGO_ENABLED': '0', 'GOOS': 'linux', 'GOARCH': 'amd64', 'GOAMD64': 'v1', 'GOMAXPROCS': '1', 'GOCACHE': '/cache/build', 'GOMODCACHE': '/cache/mod'}
    for key, value in environment.items():
        command.extend(['--env', f'{key}={value}'])
    command.extend([IMAGE, '/sdk/go/bin/go', 'vet', '.'])
    report = {'status': 'running', 'command': command, 'environment': environment, 'inputs': seal(out / 'inputs.json'), 'native_Linux_release_acceptance': False, 'full_T_gate': 'Deferred to final T02-W04: ./scripts/check-fast.sh'}
    write(out / 'result.json', report)
    start = time.monotonic()
    with (out / 'stdout.log').open('xb') as stdout, (out / 'stderr.log').open('xb') as stderr:
        code = subprocess.run(command, cwd=REPO, stdout=stdout, stderr=stderr, check=False).returncode
    sources_unchanged = before == [seal(p) for p in sorted(sources)]
    sdk_unchanged = sdk == sdk_files()
    passed = code == 0 and sources_unchanged and sdk_unchanged
    report.update(status='passed' if passed else 'failed', exit_code=code, seconds=time.monotonic()-start, sources_unchanged=sources_unchanged, sdk_unchanged=sdk_unchanged, stdout=seal(out/'stdout.log'), stderr=seal(out/'stderr.log'))
    write(out / 'result.json', report)
    print(json.dumps({k:report[k] for k in ['status','exit_code','seconds','sources_unchanged','sdk_unchanged']}))
    raise SystemExit(not passed)


if __name__ == '__main__':
    main()
