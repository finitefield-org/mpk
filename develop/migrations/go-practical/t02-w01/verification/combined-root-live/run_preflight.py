"""Check both-root ownership code before the original live capture."""
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
GOCACHE = Path('/private/tmp/mpk-go-registered-analysis-vet-cache')
CARGO = Path('/private/tmp/mpk-go-source-root-matching/cargo')
TARGET = Path('/private/tmp/mpk-go-source-root-matching/target')
REGISTRY = Path('/Users/kazuyoshitoshiya/.cargo/registry')
GO_IMAGE = 'docker.io/library/golang@sha256:ebd54034f076819b3054f155db53660ded951612bb4dfd277f933e62059e5d5a'
RUST_IMAGE = 'sha256:7cf1e580ef5539f03b58560753e8ab84c8c360960d99dff714004aa98f203977'
sys.dont_write_bytecode = True
sys.path.insert(0, str(ROOT.parent / 'installed-compiler'))
from run_scoped import seal, sdk_files, write


def main():
    epoch = sys.argv[1]
    assert epoch.replace('-', '').isalnum() and Path.cwd() == REPO
    out = ROOT / epoch
    out.mkdir(exist_ok=False)
    work = Path('/private/tmp') / ('mpk-combined-root-preflight-' + epoch)
    assert not work.exists()
    work.mkdir(mode=0o700)
    tmp = work / 'tmp'
    tmp.mkdir(mode=0o700)
    GOCACHE.mkdir(exist_ok=True)
    paths = {Path(__file__).resolve(), REPO / 'Cargo.toml', REPO / 'Cargo.lock', REPO / 'crates/mpk-cli/examples/go_practical_live_combined_roots.rs', REPO / 'crates/mpk-cli/tests/go_practical_live_export_projection.rs', REPO / 'release/bundles/candidates/go.json'}
    paths.update(p for p in (REPO / 'go-tools/go2vir').rglob('*') if p.is_file())
    paths.update((REPO / 'release/bundles').glob('*.json'))
    for crate in ['mpk-cli', 'mpk-api', 'mpk-cert', 'mpk-core', 'mpk-kernel', 'mpk-linux-sandbox', 'mpk-theory', 'mpk-vc']:
        base = REPO / 'crates' / crate
        paths.add(base / 'Cargo.toml')
        paths.update(p for p in (base / 'src').rglob('*') if p.is_file())
        paths.update(p for p in base.glob('build.rs') if p.is_file())
    probe = REPO / 'develop/probes/go-practical'
    sys.path.insert(0, str(probe))
    from frontend_native_build import linked_inputs
    paths.update(REPO / row['path'] for row in linked_inputs())
    paths.update(probe / name for name in ['frontend_native_build.py', 'run_proof_pilot.py', 'check_scope_report.py'])
    source = [seal(p) for p in sorted(paths)]
    with tarfile.open(out / 'inputs.tar.gz', 'x:gz', dereference=True) as archive:
        for path in sorted(paths):
            archive.add(path, arcname=str(path.relative_to(REPO)), recursive=False)
    old = ROOT.parent / 'export-root-live/live-original'
    storage = json.loads((old / 'evidence-storage.json').read_bytes())
    retained = [old / 'evidence-storage.json', old / 'original-evidence-manifest.json', old / 'coord/snapshot-ready.json', ROOT.parent / 'source-root-live/live-original/coord/snapshot-ready.json', ROOT.parent / 'export-root-live/restore_evidence.py']
    retained += [old / row['path'] for row in storage['chunks']]
    evidence = [seal(p) for p in retained]
    sdk = sdk_files()
    common = ['/usr/local/bin/docker', 'run', '--pull=never', '--rm', '--network', 'none', '--read-only', '--cap-drop', 'ALL', '--security-opt', 'no-new-privileges', '--user', f'{os.getuid()}:{os.getgid()}', '--mount', f'type=bind,src={REPO},dst=/workspace,readonly']
    genv = {'GOROOT': '/sdk/go', 'GOTOOLCHAIN': 'local', 'GOENV': 'off', 'GOWORK': 'off', 'GOFLAGS': '-mod=vendor', 'GOPROXY': 'off', 'GOSUMDB': 'off', 'CGO_ENABLED': '0', 'GOOS': 'linux', 'GOARCH': 'amd64', 'GOAMD64': 'v1', 'GOMAXPROCS': '1', 'GOCACHE': '/cache', 'HOME': '/tmp'}
    go = common + ['--platform', 'linux/amd64', '--tmpfs', '/tmp:rw,exec,nosuid,nodev,size=2g,mode=1777', '--mount', f'type=bind,src={SDK},dst=/sdk,readonly', '--mount', f'type=bind,src={GOCACHE},dst=/cache', '--workdir', '/workspace/go-tools/go2vir']
    for key, value in genv.items():
        go += ['--env', f'{key}={value}']
    renv = {'CARGO_HOME': '/cargo', 'CARGO_TARGET_DIR': '/target', 'CARGO_NET_OFFLINE': 'true', 'CARGO_INCREMENTAL': '0', 'CARGO_PROFILE_DEV_DEBUG': '0', 'HOME': '/tmp', 'MPK_GO_LIVE_EXPORT_BINDING_OUTPUT': '/out'}
    rust = common + ['--platform', 'linux/arm64', '--mount', f'type=bind,src={tmp},dst=/tmp', '--mount', f'type=bind,src={out},dst=/out', '--mount', f'type=bind,src={CARGO},dst=/cargo', '--mount', f'type=bind,src={REGISTRY},dst=/cargo/registry,readonly', '--mount', f'type=bind,src={TARGET},dst=/target', '--workdir', '/workspace']
    for key, value in renv.items():
        rust += ['--env', f'{key}={value}']
    commands = {
        'go-test-vet': go + [GO_IMAGE, '/sdk/go/bin/go', 'test', '-json', '-count=1', '-timeout=20m', '-run', '^(TestPracticalCompilerInputsRequireOriginalImageOwner|TestPracticalCaptureOriginalModulesAndOwnedSnapshot)$'],
        'go-format': go + [GO_IMAGE, '/sdk/go/bin/gofmt', '-l', 'practical_compiler_inputs_test.go', 'practical_live_combined_roots_test.go'],
        'rust-clippy': ['cargo', 'clippy', '--offline', '--locked', '-p', 'mpk-cli', '--test', 'go_practical_live_export_projection', '--example', 'go_practical_live_combined_roots', '--target', 'x86_64-unknown-linux-gnu', '--', '-D', 'warnings'],
        'rust-format': ['rustfmt', '--edition', '2021', '--check', '--config', 'skip_children=true', str(REPO / 'crates/mpk-cli/src/go_practical_capture_binding.rs'), str(REPO / 'crates/mpk-cli/examples/go_practical_live_combined_roots.rs'), str(REPO / 'crates/mpk-cli/tests/go_practical_live_export_projection.rs')],
        'rust-original-projection': rust + [RUST_IMAGE, 'cargo', 'test', '--offline', '--locked', '-p', 'mpk-cli', '--test', 'go_practical_live_export_projection', '--', '--nocapture'],
    }
    host_env = {'CARGO_TARGET_DIR': '/private/tmp/mpk-go-source-root-clippy-target', 'CARGO_NET_OFFLINE': 'true', 'CARGO_INCREMENTAL': '0', 'CARGO_PROFILE_DEV_DEBUG': '0'}
    write(out / 'inputs.json', {'selection_reason': 'Compile, vet and lint the changed capture coordination and both-root owner, then test exact original snapshots and coherent negative projections before the simultaneous live observation.', 'inputs': source, 'archive': seal(out / 'inputs.tar.gz'), 'retained_evidence': evidence, 'registered_sdk': sdk, 'commands': commands, 'go_environment': genv, 'rust_environment': renv, 'host_rust_environment': host_env})
    result = {'status': 'running', 'native_Linux_release_acceptance': False, 'full_T_gate': 'Deferred to final T02-W04: ./scripts/check-fast.sh'}
    write(out / 'result.json', result)
    checks = {}
    start = time.monotonic()
    for name, command in commands.items():
        with (out / (name + '.stdout')).open('xb') as stdout, (out / (name + '.stderr')).open('xb') as stderr:
            code = subprocess.run(command, cwd=REPO, env={**os.environ, **host_env} if name == 'rust-clippy' else os.environ, stdout=stdout, stderr=stderr, check=False).returncode
        okay = code == 0 and (name != 'go-format' or (out / (name + '.stdout')).read_bytes() == b'')
        checks[name] = {'status': 'passed' if okay else 'failed', 'exit_code': code, 'stdout': seal(out / (name + '.stdout')), 'stderr': seal(out / (name + '.stderr'))}
        print(json.dumps({'check': name, 'status': checks[name]['status'], 'seconds': round(time.monotonic() - start)}), flush=True)
    unchanged = source == [seal(p) for p in sorted(paths)] and evidence == [seal(p) for p in retained] and sdk == sdk_files()
    passed = unchanged and all(c['status'] == 'passed' for c in checks.values())
    result.update(status='passed' if passed else 'failed', seconds=time.monotonic() - start, original_inputs_unchanged=unchanged, checks=checks)
    write(out / 'result.json', result)
    print(json.dumps({'status': result['status'], 'original_inputs_unchanged': unchanged}), flush=True)
    raise SystemExit(not passed)


if __name__ == '__main__':
    main()
