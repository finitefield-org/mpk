"""Find a Clippy invocation scoped to the test-configured observer target."""
from pathlib import Path
import json
import os
import subprocess
import sys
import tarfile
import time

ROOT = Path(__file__).resolve().parent
REPO = ROOT.parents[5]
sys.dont_write_bytecode = True
sys.path.insert(0, str(ROOT.parent / 'installed-compiler'))
from run_scoped import seal, write


def main():
    epoch = sys.argv[1]
    assert epoch.replace('-', '').isalnum() and Path.cwd() == REPO
    previous = ROOT / 'preflight-original'
    prior = json.loads((previous / 'result.json').read_bytes())
    inputs = json.loads((previous / 'inputs.json').read_bytes())
    assert prior['status'] == 'failed' and prior['original_inputs_unchanged']
    assert prior['checks']['rust-clippy-test']['status'] == 'failed'
    assert all(prior['checks'][name]['status'] == 'passed' for name in ['rust-clippy-normal', 'rust-format', 'rust-test-observer-build'])
    assert inputs['inputs'] == [seal(Path(row['path'])) for row in inputs['inputs']]
    paths = {Path(row['path']) for row in inputs['inputs']}
    paths.add(Path(__file__).resolve())
    source = [seal(path) for path in sorted(paths)]
    out = ROOT / epoch
    out.mkdir(exist_ok=False)
    with tarfile.open(out / 'inputs.tar.gz', 'x:gz', dereference=True) as archive:
        for path in sorted(paths):
            archive.add(path, arcname=str(path.relative_to(REPO)), recursive=False)
    driver = subprocess.run(['rustup', 'which', 'clippy-driver'], cwd=REPO, capture_output=True, text=True, check=False)
    assert driver.returncode == 0 and not driver.stderr and Path(driver.stdout.strip()).is_absolute()
    commands = {
        'clippy-no-deps': ['cargo', 'clippy', '--no-deps', '--offline', '--locked', '-p', 'mpk-cli', '--example', 'go_practical_live_combined_roots', '--target', 'x86_64-unknown-linux-gnu', '--', '--cfg', 'test', '-Aexplicit_builtin_cfgs_in_flags', '-D', 'warnings'],
        'cargo-rustc-clippy-wrapper': ['cargo', 'rustc', '--offline', '--locked', '-p', 'mpk-cli', '--example', 'go_practical_live_combined_roots', '--target', 'x86_64-unknown-linux-gnu', '--', '--cfg', 'test', '-Aexplicit_builtin_cfgs_in_flags', '-Dwarnings', '--emit=metadata'],
    }
    common = {'CARGO_NET_OFFLINE': 'true', 'CARGO_INCREMENTAL': '0', 'CARGO_PROFILE_DEV_DEBUG': '0'}
    envs = {'clippy-no-deps': {**common, 'CARGO_TARGET_DIR': '/private/tmp/mpk-go-source-root-clippy-target'}, 'cargo-rustc-clippy-wrapper': {**common, 'CARGO_TARGET_DIR': '/private/tmp/mpk-go-source-root-clippy-wrapper-target', 'RUSTC_WORKSPACE_WRAPPER': driver.stdout.strip()}}
    write(out / 'inputs.json', {'selection_reason': 'Avoid applying test cfg to unrelated dependency crates while still linting the actual test-only observer; preserve the previous full diagnostic and all original source inputs.', 'inputs': source, 'archive': seal(out / 'inputs.tar.gz'), 'previous_inputs': seal(previous / 'inputs.json'), 'previous_result': seal(previous / 'result.json'), 'previous_archive': inputs['archive'], 'commands': commands, 'environments': envs, 'clippy_driver': driver.stdout.strip()})
    result = {'status': 'running', 'native_Linux_release_acceptance': False, 'full_T_gate': 'Deferred to final T02-W04: ./scripts/check-fast.sh'}
    write(out / 'result.json', result)
    checks = {}
    started = time.monotonic()
    for name, command in commands.items():
        with (out / (name + '.stdout')).open('xb') as stdout, (out / (name + '.stderr')).open('xb') as stderr:
            code = subprocess.run(command, cwd=REPO, env={**os.environ, **envs[name]}, stdout=stdout, stderr=stderr, check=False).returncode
        checks[name] = {'status': 'passed' if code == 0 else 'failed', 'exit_code': code, 'stdout': seal(out / (name + '.stdout')), 'stderr': seal(out / (name + '.stderr'))}
        print(json.dumps({'check': name, 'status': checks[name]['status'], 'seconds': round(time.monotonic() - started)}), flush=True)
    unchanged = source == [seal(path) for path in sorted(paths)]
    passed = unchanged and any(item['status'] == 'passed' for item in checks.values())
    result.update(status='passed' if passed else 'failed', seconds=time.monotonic() - started, original_inputs_unchanged=unchanged, checks=checks)
    write(out / 'result.json', result)
    print(json.dumps({'status': result['status'], 'original_inputs_unchanged': unchanged}), flush=True)
    raise SystemExit(not passed)


if __name__ == '__main__':
    main()
