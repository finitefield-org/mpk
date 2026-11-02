"""Lint and build the local test-only installed-owner live observer."""
from pathlib import Path
import json
import os
import subprocess
import sys
import tarfile
import time

ROOT = Path(__file__).resolve().parent
REPO = ROOT.parents[5]
CARGO = Path('/private/tmp/mpk-go-source-root-matching/cargo')
TARGET = Path('/private/tmp/mpk-go-source-root-matching/target')
REGISTRY = Path('/Users/kazuyoshitoshiya/.cargo/registry')
IMAGE = 'sha256:7cf1e580ef5539f03b58560753e8ab84c8c360960d99dff714004aa98f203977'
sys.dont_write_bytecode = True
sys.path.insert(0, str(ROOT.parent / 'installed-compiler'))
from run_scoped import seal, sdk_files, write


def main():
    epoch = sys.argv[1]
    assert epoch.replace('-', '').isalnum() and Path.cwd() == REPO
    probe = ROOT / 'cfg-original'
    prior = json.loads((probe / 'result.json').read_bytes())
    inputs = json.loads((probe / 'inputs.json').read_bytes())
    assert prior['status'] == 'passed' and prior['original_inputs_unchanged']
    changed = [row['path'] for row in inputs['inputs'] if seal(Path(row['path'])) != row]
    assert changed == [str(REPO / 'crates/mpk-cli/examples/go_practical_live_combined_roots.rs')]
    paths = {Path(row['path']) for row in inputs['inputs']}
    paths.add(Path(__file__).resolve())
    source = [seal(path) for path in sorted(paths)]
    sdk = sdk_files()
    out = ROOT / epoch
    out.mkdir(exist_ok=False)
    with tarfile.open(out / 'inputs.tar.gz', 'x:gz', dereference=True) as archive:
        for path in sorted(paths):
            archive.add(path, arcname=str(path.relative_to(REPO)), recursive=False)
    env = {'CARGO_HOME': '/cargo', 'CARGO_TARGET_DIR': '/target', 'CARGO_NET_OFFLINE': 'true', 'CARGO_INCREMENTAL': '0', 'CARGO_PROFILE_DEV_DEBUG': '0', 'HOME': '/tmp'}
    rust = ['/usr/local/bin/docker', 'run', '--pull=never', '--rm', '--platform', 'linux/arm64', '--network', 'none', '--read-only', '--cap-drop', 'ALL', '--security-opt', 'no-new-privileges', '--user', f'{os.getuid()}:{os.getgid()}', '--tmpfs', '/tmp:rw,exec,nosuid,nodev,size=1g,mode=1777', '--mount', f'type=bind,src={REPO},dst=/workspace,readonly', '--mount', f'type=bind,src={CARGO},dst=/cargo', '--mount', f'type=bind,src={REGISTRY},dst=/cargo/registry,readonly', '--mount', f'type=bind,src={TARGET},dst=/target', '--workdir', '/workspace']
    for key, value in env.items():
        rust += ['--env', f'{key}={value}']
    commands = {
        'rust-clippy-normal': ['cargo', 'clippy', '--offline', '--locked', '-p', 'mpk-cli', '--example', 'go_practical_live_combined_roots', '--target', 'x86_64-unknown-linux-gnu', '--', '-D', 'warnings'],
        'rust-clippy-test': ['cargo', 'clippy', '--offline', '--locked', '-p', 'mpk-cli', '--example', 'go_practical_live_combined_roots', '--target', 'x86_64-unknown-linux-gnu', '--', '--cfg', 'test', '-Aexplicit_builtin_cfgs_in_flags', '-D', 'warnings'],
        'rust-format': ['rustfmt', '--edition', '2021', '--check', '--config', 'skip_children=true', str(REPO / 'crates/mpk-cli/examples/go_practical_live_combined_roots.rs')],
        'rust-test-observer-build': rust + [IMAGE, 'cargo', 'rustc', '--offline', '--locked', '-p', 'mpk-cli', '--example', 'go_practical_live_combined_roots', '--', '--cfg', 'test', '-Aexplicit_builtin_cfgs_in_flags', '-Dwarnings'],
    }
    host_env = {'CARGO_TARGET_DIR': '/private/tmp/mpk-go-source-root-clippy-target', 'CARGO_NET_OFFLINE': 'true', 'CARGO_INCREMENTAL': '0', 'CARGO_PROFILE_DEV_DEBUG': '0'}
    write(out / 'inputs.json', {'selection_reason': 'Check both ordinary and test-only observer builds after adding the existing private test SDK constructor; all original Go inputs and previously passed root tests remain unchanged.', 'inputs': source, 'archive': seal(out / 'inputs.tar.gz'), 'registered_sdk': sdk, 'previous_probe_inputs': seal(probe / 'inputs.json'), 'previous_probe_result': seal(probe / 'result.json'), 'previous_probe_archive': inputs['archive'], 'changed_previous_inputs': changed, 'commands': commands, 'rust_environment': env, 'host_rust_environment': host_env})
    result = {'status': 'running', 'native_Linux_release_acceptance': False, 'full_T_gate': 'Deferred to final T02-W04: ./scripts/check-fast.sh'}
    write(out / 'result.json', result)
    checks = {}
    started = time.monotonic()
    for name, command in commands.items():
        with (out / (name + '.stdout')).open('xb') as stdout, (out / (name + '.stderr')).open('xb') as stderr:
            code = subprocess.run(command, cwd=REPO, env={**os.environ, **host_env} if name.startswith('rust-clippy') else os.environ, stdout=stdout, stderr=stderr, check=False).returncode
        checks[name] = {'status': 'passed' if code == 0 else 'failed', 'exit_code': code, 'stdout': seal(out / (name + '.stdout')), 'stderr': seal(out / (name + '.stderr'))}
        print(json.dumps({'check': name, 'status': checks[name]['status'], 'seconds': round(time.monotonic() - started)}), flush=True)
    unchanged = source == [seal(path) for path in sorted(paths)] and sdk == sdk_files()
    passed = unchanged and all(item['status'] == 'passed' for item in checks.values())
    result.update(status='passed' if passed else 'failed', seconds=time.monotonic() - started, original_inputs_unchanged=unchanged, checks=checks)
    write(out / 'result.json', result)
    print(json.dumps({'status': result['status'], 'original_inputs_unchanged': unchanged}), flush=True)
    raise SystemExit(not passed)


if __name__ == '__main__':
    main()
