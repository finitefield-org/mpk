"""Retain exact inputs and outputs for the installed Go compiler boundary."""
from pathlib import Path
import datetime
import hashlib
import json
import os
import shutil
import subprocess
import sys
import tarfile
import time

ROOT = Path(__file__).resolve().parent
REPO = ROOT.parents[5]
SDK = Path('/private/tmp/mpk-installed-compiler-205-original-go-output/toolchain')
HOST_GO = Path('/private/tmp/mpk-go125-20261025-download/go')
RUST = ['crates/mpk-cli/src/lib.rs', 'crates/mpk-cli/src/go_practical_installed_compiler.rs', 'crates/mpk-cli/tests/go_practical_installed_compiler.rs']
GO = ['go-tools/go2vir/practical_compiler_capture_test.go', 'go-tools/go2vir/practical_compiler_inputs_test.go']


def seal(path):
    path = Path(path)
    digest = hashlib.sha256()
    size = 0
    with path.open('rb') as stream:
        for block in iter(lambda: stream.read(1024 * 1024), b''):
            digest.update(block)
            size += len(block)
    return {'path': str(path.resolve()), 'bytes': size, 'sha256': digest.hexdigest(), 'mode': path.stat().st_mode & 0o777}


def write(path, value):
    path.write_text(json.dumps(value, indent=2) + '\n')


def sdk_files():
    candidate = json.loads((REPO / 'release/bundles/candidates/go.json').read_bytes())['toolchain_bundles'][0]
    rows = []
    for entry in candidate['inventory']['files']:
        actual = seal(SDK / entry['path'])
        assert (actual['bytes'], actual['sha256'], bool(actual['mode'] & 0o111)) == (entry['size_bytes'], entry['sha256'], entry['executable'])
        rows.append(actual)
    assert {str(p.relative_to(SDK)) for p in SDK.rglob('*') if p.is_file()} == {e['path'] for e in candidate['inventory']['files']}
    return rows


def main():
    epoch, mode = sys.argv[1:]
    assert epoch.replace('-', '').isalnum()
    assert REPO == Path.cwd()
    out = ROOT / epoch
    out.mkdir(exist_ok=False)
    files = {Path(__file__).resolve()}
    files.update(REPO / p for p in RUST + GO)
    files.add(REPO / 'crates/mpk-cli/Cargo.toml')
    if mode not in ['rust-format-write', 'rust-format', 'go-format-write', 'go-format']:
        sys.dont_write_bytecode = True
        sys.path.insert(0, str(REPO / 'develop/probes/go-practical'))
        from frontend_native_build import linked_inputs
        files.update(REPO / r['path'] for r in linked_inputs())
        for folder in ['go-tools/go2vir', 'go-tools/mpk-checker-ref']:
            files.update(p for p in (REPO / folder).rglob('*') if p.is_file())
        files.update(REPO / p for p in ['develop/probes/go-practical/frontend_native_build.py', 'release/bundles/candidates/go.json'])
    sources = [seal(p) for p in sorted(files)]
    with tarfile.open(out / 'inputs.tar.gz', 'x:gz', dereference=True) as archive:
        for p in sorted(files):
            archive.add(p, arcname=str(p.relative_to(REPO)), recursive=False)
    installed_sdk = sdk_files() if mode == 'native-owner' else []
    write(out / 'inputs.json', {'sources': sources, 'source_archive': seal(out / 'inputs.tar.gz'), 'whole_registered_SDK': installed_sdk})
    command = {
        'rust-format-write': ['rustfmt', '--edition=2021', '--config', 'skip_children=true', *RUST],
        'rust-format': ['rustfmt', '--edition=2021', '--check', '--config', 'skip_children=true', *RUST],
        'go-format-write': [str(HOST_GO / 'bin/gofmt'), '-w', *GO],
        'go-format': [str(HOST_GO / 'bin/gofmt'), '-l', *GO],
        'host-check': ['cargo', 'check', '--offline', '--locked', '-p', 'mpk-cli', '--lib'],
        'host-test': ['cargo', 'test', '--offline', '--locked', '-p', 'mpk-cli', '--test', 'go_practical_installed_compiler'],
        'host-clippy': ['cargo', 'clippy', '--offline', '--locked', '-p', 'mpk-cli', '--lib', '--test', 'go_practical_installed_compiler', '--', '-D', 'warnings'],
    }.get(mode)
    changes = {'CARGO_NET_OFFLINE': 'true', 'CARGO_INCREMENTAL': '0', 'CARGO_PROFILE_DEV_DEBUG': '0'}
    if mode == 'native-owner':
        evidence = out / 'native-originals'
        evidence.mkdir()
        cache = Path('/private/tmp/mpk-installed-compiler-205-go-cache')
        cache.mkdir(exist_ok=True)
        image = 'docker.io/library/golang@sha256:ebd54034f076819b3054f155db53660ded951612bb4dfd277f933e62059e5d5a'
        command = ['/usr/local/bin/docker', 'run', '--pull=never', '--rm', '--platform', 'linux/amd64', '--network', 'none', '--read-only', '--cap-drop', 'ALL', '--security-opt', 'no-new-privileges', '--user', f'{os.getuid()}:{os.getgid()}', '--tmpfs', '/tmp:rw,exec,nosuid,nodev,size=2g,mode=1777', '--mount', f'type=bind,src={REPO},dst=/workspace,readonly', '--mount', f'type=bind,src={SDK},dst=/registered-sdk,readonly', '--mount', f'type=bind,src={evidence},dst=/evidence', '--mount', f'type=bind,src={cache},dst=/cache', '--workdir', '/workspace/go-tools/go2vir']
        go_env = {'GOROOT': '/registered-sdk/go', 'GOTOOLCHAIN': 'local', 'GOENV': 'off', 'GOWORK': 'off', 'GOFLAGS': '-mod=vendor', 'GOPROXY': 'off', 'GOSUMDB': 'off', 'CGO_ENABLED': '0', 'GOOS': 'linux', 'GOARCH': 'amd64', 'GOAMD64': 'v1', 'GOMAXPROCS': '1', 'GOCACHE': '/cache/build', 'GOMODCACHE': '/cache/mod', 'MPK_GO_PRACTICAL_SYNTAX_EVIDENCE': '/evidence'}
        for k, v in go_env.items():
            command.extend(['--env', f'{k}={v}'])
        command.extend([image, '/registered-sdk/go/bin/go', 'test', '-json', '-count=1', '-timeout=40m', '-run', '^TestPracticalInstalledCompilerInputsKeepWholeRegisteredSDKAndPackages$'])
    assert command is not None
    report = {'status': 'running', 'started_at_utc': datetime.datetime.now(datetime.timezone.utc).isoformat(), 'command': command, 'environment': changes, 'inputs': seal(out / 'inputs.json'), 'selection_reason': 'Compile and lint the new private installed-SDK owner and its real CLI consumers. The new native root retains the existing complete HTTP application, all dependencies, unused bodies, original init and Root selection, with no contract overlay; it creates original inputs with the actual registered Linux SDK identity. No narrowed fixtures, removed-language tests, practical profile activation, source/program proof or native Linux release acceptance. Formats cover changed files only.', 'full_T_gate': 'Deferred to final T02-W04: ./scripts/check-fast.sh', 'native_Linux_release_acceptance': False}
    write(out / 'result.json', report)
    start = time.monotonic()
    with (out / 'stdout.log').open('xb') as stdout, (out / 'stderr.log').open('xb') as stderr:
        code = subprocess.run(command, cwd=REPO, env={**os.environ, **changes}, stdout=stdout, stderr=stderr, check=False).returncode
    unchanged = sources == [seal(p) for p in sorted(files)]
    sdk_unchanged = installed_sdk == sdk_files() if installed_sdk else True
    formatter = mode.endswith('-format-write')
    passed = code == 0 and (unchanged or formatter) and sdk_unchanged
    if mode == 'go-format':
        passed = passed and not (out / 'stdout.log').read_bytes()
    report.update(status='passed' if passed else 'failed', exit_code=code, seconds=time.monotonic() - start, consumed_inputs_unchanged=unchanged, expected_formatter_mutation=formatter, whole_registered_SDK_unchanged=sdk_unchanged, stdout=seal(out / 'stdout.log'), stderr=seal(out / 'stderr.log'))
    write(out / 'result.json', report)
    print(json.dumps({k: report[k] for k in ['status', 'exit_code', 'seconds', 'consumed_inputs_unchanged', 'whole_registered_SDK_unchanged']}))
    raise SystemExit(not passed)


if __name__ == '__main__':
    main()
