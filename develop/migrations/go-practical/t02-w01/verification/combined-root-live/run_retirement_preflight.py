"""Validate the exact-cache retirement check before repeating the live capture."""
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
CACHE = Path('/private/tmp/mpk-go-registered-analysis-vet-cache')
IMAGE = 'docker.io/library/golang@sha256:ebd54034f076819b3054f155db53660ded951612bb4dfd277f933e62059e5d5a'
sys.dont_write_bytecode = True
sys.path.insert(0, str(ROOT.parent / 'installed-compiler'))
from run_scoped import seal, sdk_files, write


def main():
    epoch = sys.argv[1]
    assert epoch.replace('-', '').isalnum() and Path.cwd() == REPO
    previous = ROOT / 'preflight-control'
    prior = json.loads((previous / 'result.json').read_bytes())
    inputs = json.loads((previous / 'inputs.json').read_bytes())
    downstream = json.loads((ROOT / 'downstream-original/result.json').read_bytes())
    live = json.loads((ROOT / 'live-volume/result.json').read_bytes())
    retirement = json.loads((ROOT / 'live-volume/retirement-check.json').read_bytes())
    assert prior['status'] == downstream['status'] == live['status'] == retirement['status'] == 'passed'
    assert all(row['original_inputs_unchanged'] for row in [prior, downstream, live])
    assert retirement['all_paths_absent_after_run'] and not retirement['original_volume_modified']
    changed = {row['path'] for row in inputs['inputs'] if seal(Path(row['path'])) != row}
    assert changed == {str(REPO / 'crates/mpk-cli/examples/go_practical_live_combined_roots.rs'), str(REPO / 'go-tools/go2vir/practical_compiler_inputs_test.go'), str(REPO / 'go-tools/go2vir/practical_live_combined_roots_test.go')}
    paths = {Path(row['path']) for row in inputs['inputs']}
    paths.add(Path(__file__).resolve())
    source = [seal(path) for path in sorted(paths)]
    retained = inputs['retained_evidence']
    sdk = inputs['registered_sdk']
    assert retained == [seal(Path(row['path'])) for row in retained] and sdk == sdk_files()
    assert seal(Path(inputs['archive']['path'])) == inputs['archive']
    out = ROOT / epoch
    out.mkdir(exist_ok=False)
    CACHE.mkdir(exist_ok=True)
    with tarfile.open(out / 'inputs.tar.gz', 'x:gz', dereference=True) as archive:
        for path in sorted(paths):
            archive.add(path, arcname=str(path.relative_to(REPO)), recursive=False)
    env = {'GOROOT': '/sdk/go', 'GOTOOLCHAIN': 'local', 'GOENV': 'off', 'GOWORK': 'off', 'GOFLAGS': '-mod=vendor', 'GOPROXY': 'off', 'GOSUMDB': 'off', 'CGO_ENABLED': '0', 'GOOS': 'linux', 'GOARCH': 'amd64', 'GOAMD64': 'v1', 'GOMAXPROCS': '1', 'GOCACHE': '/cache', 'HOME': '/tmp'}
    go = ['/usr/local/bin/docker', 'run', '--pull=never', '--rm', '--platform', 'linux/amd64', '--network', 'none', '--read-only', '--cap-drop', 'ALL', '--security-opt', 'no-new-privileges', '--user', f'{os.getuid()}:{os.getgid()}', '--tmpfs', '/tmp:rw,exec,nosuid,nodev,size=2g,mode=1777', '--mount', f'type=bind,src={REPO},dst=/workspace,readonly', '--mount', f'type=bind,src={SDK},dst=/sdk,readonly', '--mount', f'type=bind,src={CACHE},dst=/cache', '--workdir', '/workspace/go-tools/go2vir']
    for key, value in env.items():
        go += ['--env', f'{key}={value}']
    commands = {
        'go-retirement-test-vet': go + [IMAGE, '/sdk/go/bin/go', 'test', '-json', '-count=1', '-timeout=20m', '-run', '^TestPracticalLiveCombinedExportRetirement$'],
        'go-format': go + [IMAGE, '/sdk/go/bin/gofmt', '-l', 'practical_compiler_inputs_test.go', 'practical_live_combined_roots_test.go'],
        'rust-clippy': ['cargo', 'clippy', '--offline', '--locked', '-p', 'mpk-cli', '--example', 'go_practical_live_combined_roots', '--target', 'x86_64-unknown-linux-gnu', '--', '-D', 'warnings'],
        'rust-format': ['rustfmt', '--edition', '2021', '--check', '--config', 'skip_children=true', str(REPO / 'crates/mpk-cli/examples/go_practical_live_combined_roots.rs')],
    }
    host_env = {'CARGO_TARGET_DIR': '/private/tmp/mpk-go-source-root-clippy-target', 'CARGO_NET_OFFLINE': 'true', 'CARGO_INCREMENTAL': '0', 'CARGO_PROFILE_DEV_DEBUG': '0'}
    write(out / 'inputs.json', {'selection_reason': 'Check that the exact previously held export cache path is absent at owner handoff, including existing directory, file, dangling-link and relative-path controls; lint and format the affected Go and Rust observation code.', 'inputs': source, 'archive': seal(out / 'inputs.tar.gz'), 'retained_evidence': retained, 'registered_sdk': sdk, 'previous_inputs': seal(previous / 'inputs.json'), 'previous_result': seal(previous / 'result.json'), 'previous_archive': inputs['archive'], 'previous_live_result': seal(ROOT / 'live-volume/result.json'), 'previous_retirement_check': seal(ROOT / 'live-volume/retirement-check.json'), 'previous_downstream_result': seal(ROOT / 'downstream-original/result.json'), 'changed_previous_inputs': sorted(changed), 'commands': commands, 'go_environment': env, 'host_rust_environment': host_env})
    result = {'status': 'running', 'native_Linux_release_acceptance': False, 'full_T_gate': 'Deferred to final T02-W04: ./scripts/check-fast.sh'}
    write(out / 'result.json', result)
    checks = {}
    started = time.monotonic()
    for name, command in commands.items():
        with (out / (name + '.stdout')).open('xb') as stdout, (out / (name + '.stderr')).open('xb') as stderr:
            code = subprocess.run(command, cwd=REPO, env={**os.environ, **host_env} if name == 'rust-clippy' else os.environ, stdout=stdout, stderr=stderr, check=False).returncode
        okay = code == 0 and (name != 'go-format' or (out / (name + '.stdout')).read_bytes() == b'')
        checks[name] = {'status': 'passed' if okay else 'failed', 'exit_code': code, 'stdout': seal(out / (name + '.stdout')), 'stderr': seal(out / (name + '.stderr'))}
        print(json.dumps({'check': name, 'status': checks[name]['status'], 'seconds': round(time.monotonic() - started)}), flush=True)
    unchanged = source == [seal(path) for path in sorted(paths)] and retained == [seal(Path(row['path'])) for row in retained] and sdk == sdk_files()
    passed = unchanged and all(item['status'] == 'passed' for item in checks.values())
    result.update(status='passed' if passed else 'failed', seconds=time.monotonic() - started, original_inputs_unchanged=unchanged, previous_control_and_downstream_passed=True, checks=checks)
    write(out / 'result.json', result)
    print(json.dumps({'status': result['status'], 'original_inputs_unchanged': unchanged}), flush=True)
    raise SystemExit(not passed)


if __name__ == '__main__':
    main()
