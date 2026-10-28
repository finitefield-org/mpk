"""Recheck the existing installed-compiler/root consumer against original data."""
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
    previous = ROOT / 'preflight-control'
    prior = json.loads((previous / 'result.json').read_bytes())
    inputs = json.loads((previous / 'inputs.json').read_bytes())
    assert prior['status'] == 'passed' and prior['original_inputs_unchanged']
    assert inputs['inputs'] == [seal(Path(row['path'])) for row in inputs['inputs']]
    assert seal(Path(inputs['archive']['path'])) == inputs['archive']
    paths = {Path(row['path']) for row in inputs['inputs']}
    paths.update([Path(__file__).resolve(), REPO / 'crates/mpk-cli/tests/go_practical_capture_binding.rs', ROOT.parent / 'capture-binding/restore_fixture.py', ROOT.parent / 'source-root-live/restore_evidence.py'])
    source = [seal(path) for path in sorted(paths)]
    original = ROOT.parent / 'source-root-live/live-original'
    storage = json.loads((original / 'evidence-storage.json').read_bytes())
    evidence_paths = [original / 'evidence-storage.json', original / 'original-evidence-manifest.json', original / 'coord/snapshot-ready.json']
    evidence_paths += [original / row['path'] for row in storage['chunks']]
    evidence = [seal(path) for path in evidence_paths]
    sdk = inputs['registered_sdk']
    assert sdk == sdk_files()
    out = ROOT / epoch
    out.mkdir(exist_ok=False)
    with tarfile.open(out / 'inputs.tar.gz', 'x:gz', dereference=True) as archive:
        for path in sorted(paths):
            archive.add(path, arcname=str(path.relative_to(REPO)), recursive=False)
    env = {'CARGO_HOME': '/cargo', 'CARGO_TARGET_DIR': '/target', 'CARGO_NET_OFFLINE': 'true', 'CARGO_INCREMENTAL': '0', 'CARGO_PROFILE_DEV_DEBUG': '0', 'HOME': '/tmp'}
    rust = ['/usr/local/bin/docker', 'run', '--pull=never', '--rm', '--platform', 'linux/arm64', '--network', 'none', '--read-only', '--cap-drop', 'ALL', '--security-opt', 'no-new-privileges', '--user', f'{os.getuid()}:{os.getgid()}', '--tmpfs', '/tmp:rw,exec,nosuid,nodev,size=3g,mode=1777', '--mount', f'type=bind,src={REPO},dst=/workspace,readonly', '--mount', f'type=bind,src={CARGO},dst=/cargo', '--mount', f'type=bind,src={REGISTRY},dst=/cargo/registry,readonly', '--mount', f'type=bind,src={TARGET},dst=/target', '--workdir', '/workspace']
    for key, value in env.items():
        rust += ['--env', f'{key}={value}']
    commands = {
        'rust-clippy': ['cargo', 'clippy', '--offline', '--locked', '-p', 'mpk-cli', '--test', 'go_practical_capture_binding', '--target', 'x86_64-unknown-linux-gnu', '--', '-D', 'warnings'],
        'rust-original-consumer': rust + [IMAGE, 'cargo', 'test', '--offline', '--locked', '-p', 'mpk-cli', '--test', 'go_practical_capture_binding', '--', '--nocapture'],
    }
    host_env = {'CARGO_TARGET_DIR': '/private/tmp/mpk-go-source-root-clippy-target', 'CARGO_NET_OFFLINE': 'true', 'CARGO_INCREMENTAL': '0', 'CARGO_PROFILE_DEV_DEBUG': '0'}
    write(out / 'inputs.json', {'selection_reason': 'Recheck the pre-existing installed compiler and non-export-root consumer affected by the binding module change, including its complete original fixtures and coherent controls.', 'inputs': source, 'archive': seal(out / 'inputs.tar.gz'), 'retained_evidence': evidence, 'registered_sdk': sdk, 'previous_inputs': seal(previous / 'inputs.json'), 'previous_result': seal(previous / 'result.json'), 'commands': commands, 'rust_environment': env, 'host_rust_environment': host_env})
    result = {'status': 'running', 'native_Linux_release_acceptance': False, 'full_T_gate': 'Deferred to final T02-W04: ./scripts/check-fast.sh'}
    write(out / 'result.json', result)
    checks = {}
    started = time.monotonic()
    for name, command in commands.items():
        with (out / (name + '.stdout')).open('xb') as stdout, (out / (name + '.stderr')).open('xb') as stderr:
            code = subprocess.run(command, cwd=REPO, env={**os.environ, **host_env} if name == 'rust-clippy' else os.environ, stdout=stdout, stderr=stderr, check=False).returncode
        checks[name] = {'status': 'passed' if code == 0 else 'failed', 'exit_code': code, 'stdout': seal(out / (name + '.stdout')), 'stderr': seal(out / (name + '.stderr'))}
        print(json.dumps({'check': name, 'status': checks[name]['status'], 'seconds': round(time.monotonic() - started)}), flush=True)
    unchanged = source == [seal(path) for path in sorted(paths)] and evidence == [seal(path) for path in evidence_paths] and sdk == sdk_files()
    passed = unchanged and all(item['status'] == 'passed' for item in checks.values())
    result.update(status='passed' if passed else 'failed', seconds=time.monotonic() - started, original_inputs_unchanged=unchanged, checks=checks)
    write(out / 'result.json', result)
    print(json.dumps({'status': result['status'], 'original_inputs_unchanged': unchanged}), flush=True)
    raise SystemExit(not passed)


if __name__ == '__main__':
    main()
