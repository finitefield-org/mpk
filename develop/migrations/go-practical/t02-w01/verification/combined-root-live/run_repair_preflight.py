"""Recheck only the Rust failures while retaining the passed Go preflight."""
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
    old = ROOT / 'preflight-original'
    prior = json.loads((old / 'result.json').read_bytes())
    inputs = json.loads((old / 'inputs.json').read_bytes())
    assert prior['status'] == 'failed' and prior['original_inputs_unchanged']
    assert all(prior['checks'][name]['status'] == 'passed' for name in ['go-test-vet', 'go-format', 'rust-format'])
    assert prior['checks']['rust-clippy']['status'] == 'failed' and prior['checks']['rust-original-projection']['status'] == 'failed'
    prior_test = (old / 'rust-original-projection.stdout').read_text()
    assert 'test live_export_projection_matches_complete_original_compiler_owner ... ok' in prior_test
    assert 'test result: FAILED. 3 passed; 1 failed; 0 ignored;' in prior_test
    changed = [row['path'] for row in inputs['inputs'] if seal(Path(row['path'])) != row]
    assert changed == [str(REPO / 'crates/mpk-cli/examples/go_practical_live_combined_roots.rs')]
    paths = {Path(row['path']) for row in inputs['inputs']}
    paths.add(Path(__file__).resolve())
    source = [seal(path) for path in sorted(paths)]
    retained = inputs['retained_evidence']
    sdk = inputs['registered_sdk']
    assert retained == [seal(Path(row['path'])) for row in retained] and sdk == sdk_files()
    assert seal(Path(inputs['archive']['path'])) == inputs['archive']
    out = ROOT / epoch
    out.mkdir(exist_ok=False)
    with tarfile.open(out / 'inputs.tar.gz', 'x:gz', dereference=True) as archive:
        for path in sorted(paths):
            archive.add(path, arcname=str(path.relative_to(REPO)), recursive=False)
    env = {'CARGO_HOME': '/cargo', 'CARGO_TARGET_DIR': '/target', 'CARGO_NET_OFFLINE': 'true', 'CARGO_INCREMENTAL': '0', 'CARGO_PROFILE_DEV_DEBUG': '0', 'HOME': '/tmp', 'MPK_GO_LIVE_EXPORT_BINDING_OUTPUT': '/out'}
    rust = ['/usr/local/bin/docker', 'run', '--pull=never', '--rm', '--platform', 'linux/arm64', '--network', 'none', '--read-only', '--cap-drop', 'ALL', '--security-opt', 'no-new-privileges', '--user', f'{os.getuid()}:{os.getgid()}', '--tmpfs', '/tmp:rw,exec,nosuid,nodev,size=3g,mode=1777', '--mount', f'type=bind,src={REPO},dst=/workspace,readonly', '--mount', f'type=bind,src={out},dst=/out', '--mount', f'type=bind,src={CARGO},dst=/cargo', '--mount', f'type=bind,src={REGISTRY},dst=/cargo/registry,readonly', '--mount', f'type=bind,src={TARGET},dst=/target', '--workdir', '/workspace']
    for key, value in env.items():
        rust += ['--env', f'{key}={value}']
    commands = {
        'rust-clippy': ['cargo', 'clippy', '--offline', '--locked', '-p', 'mpk-cli', '--test', 'go_practical_live_export_projection', '--example', 'go_practical_live_combined_roots', '--target', 'x86_64-unknown-linux-gnu', '--', '-D', 'warnings'],
        'rust-format': ['rustfmt', '--edition', '2021', '--check', '--config', 'skip_children=true', str(REPO / 'crates/mpk-cli/examples/go_practical_live_combined_roots.rs')],
        'rust-native-names': rust + [IMAGE, 'cargo', 'test', '--offline', '--locked', '-p', 'mpk-cli', '--test', 'go_practical_live_export_projection', '--', 'go_practical_source_roots::tests::native_names_and_types_are_compared_without_utf8_conversion', '--nocapture'],
    }
    host_env = {'CARGO_TARGET_DIR': '/private/tmp/mpk-go-source-root-clippy-target', 'CARGO_NET_OFFLINE': 'true', 'CARGO_INCREMENTAL': '0', 'CARGO_PROFILE_DEV_DEBUG': '0'}
    write(out / 'inputs.json', {'selection_reason': 'Retain the passed Go checks and all three passed original Rust projection tests. Fix the duplicate lint attribute, compile the observer with complete SDK import, and rerun the single failed native-name test on a case-sensitive Linux temporary filesystem; no existing case is removed.', 'inputs': source, 'archive': seal(out / 'inputs.tar.gz'), 'retained_evidence': retained, 'registered_sdk': sdk, 'previous_inputs': seal(old / 'inputs.json'), 'previous_result': seal(old / 'result.json'), 'previous_archive': inputs['archive'], 'changed_previous_inputs': changed, 'commands': commands, 'rust_environment': env, 'host_rust_environment': host_env})
    result = {'status': 'running', 'native_Linux_release_acceptance': False, 'full_T_gate': 'Deferred to final T02-W04: ./scripts/check-fast.sh'}
    write(out / 'result.json', result)
    checks = {}
    started = time.monotonic()
    for name, command in commands.items():
        with (out / (name + '.stdout')).open('xb') as stdout, (out / (name + '.stderr')).open('xb') as stderr:
            code = subprocess.run(command, cwd=REPO, env={**os.environ, **host_env} if name == 'rust-clippy' else os.environ, stdout=stdout, stderr=stderr, check=False).returncode
        checks[name] = {'status': 'passed' if code == 0 else 'failed', 'exit_code': code, 'stdout': seal(out / (name + '.stdout')), 'stderr': seal(out / (name + '.stderr'))}
        print(json.dumps({'check': name, 'status': checks[name]['status'], 'seconds': round(time.monotonic() - started)}), flush=True)
    unchanged = source == [seal(path) for path in sorted(paths)] and retained == [seal(Path(row['path'])) for row in retained] and sdk == sdk_files()
    passed = unchanged and all(item['status'] == 'passed' for item in checks.values())
    result.update(status='passed' if passed else 'failed', seconds=time.monotonic() - started, original_inputs_unchanged=unchanged, previous_go_checks_passed=True, checks=checks)
    write(out / 'result.json', result)
    print(json.dumps({'status': result['status'], 'original_inputs_unchanged': unchanged}), flush=True)
    raise SystemExit(not passed)


if __name__ == '__main__':
    main()
