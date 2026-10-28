"""Probe a test-only Rust observer build without altering installed authority."""
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
from run_scoped import seal, write


def main():
    epoch = sys.argv[1]
    assert epoch.replace('-', '').isalnum() and Path.cwd() == REPO
    prior_root = ROOT.parent / 'combined-root-live/preflight-retirement'
    prior = json.loads((prior_root / 'result.json').read_bytes())
    inputs = json.loads((prior_root / 'inputs.json').read_bytes())
    assert prior['status'] == 'passed' and prior['original_inputs_unchanged']
    source = inputs['inputs']
    assert source == [seal(Path(row['path'])) for row in source]
    paths = {Path(row['path']) for row in source}
    paths.add(Path(__file__).resolve())
    before = [seal(path) for path in sorted(paths)]
    out = ROOT / epoch
    out.mkdir(exist_ok=False)
    with tarfile.open(out / 'inputs.tar.gz', 'x:gz', dereference=True) as archive:
        for path in sorted(paths):
            archive.add(path, arcname=str(path.relative_to(REPO)), recursive=False)
    env = {'CARGO_HOME': '/cargo', 'CARGO_TARGET_DIR': '/target', 'CARGO_NET_OFFLINE': 'true', 'CARGO_INCREMENTAL': '0', 'CARGO_PROFILE_DEV_DEBUG': '0', 'HOME': '/tmp'}
    command = ['/usr/local/bin/docker', 'run', '--pull=never', '--rm', '--platform', 'linux/arm64', '--network', 'none', '--read-only', '--cap-drop', 'ALL', '--security-opt', 'no-new-privileges', '--user', f'{os.getuid()}:{os.getgid()}', '--tmpfs', '/tmp:rw,exec,nosuid,nodev,size=1g,mode=1777', '--mount', f'type=bind,src={REPO},dst=/workspace,readonly', '--mount', f'type=bind,src={CARGO},dst=/cargo', '--mount', f'type=bind,src={REGISTRY},dst=/cargo/registry,readonly', '--mount', f'type=bind,src={TARGET},dst=/target', '--workdir', '/workspace']
    for key, value in env.items():
        command += ['--env', f'{key}={value}']
    command += [IMAGE, 'cargo', 'rustc', '--offline', '--locked', '-p', 'mpk-cli', '--example', 'go_practical_live_combined_roots', '--', '--cfg', 'test', '-Aexplicit_builtin_cfgs_in_flags']
    write(out / 'inputs.json', {'selection_reason': 'Check whether the existing private test-only SDK constructor can be used by a local live observer without enabling any production constructor or installed-root claim.', 'inputs': before, 'archive': seal(out / 'inputs.tar.gz'), 'previous_result': seal(prior_root / 'result.json'), 'command': command, 'environment': env})
    result = {'status': 'running', 'native_Linux_release_acceptance': False, 'full_T_gate': 'Deferred to final T02-W04: ./scripts/check-fast.sh'}
    write(out / 'result.json', result)
    started = time.monotonic()
    with (out / 'probe.stdout').open('xb') as stdout, (out / 'probe.stderr').open('xb') as stderr:
        code = subprocess.run(command, cwd=REPO, stdout=stdout, stderr=stderr, check=False).returncode
    unchanged = before == [seal(path) for path in sorted(paths)]
    result.update(status='passed' if code == 0 and unchanged else 'failed', exit_code=code, seconds=time.monotonic() - started, original_inputs_unchanged=unchanged, stdout=seal(out / 'probe.stdout'), stderr=seal(out / 'probe.stderr'))
    write(out / 'result.json', result)
    print(json.dumps({'status': result['status'], 'exit_code': code, 'original_inputs_unchanged': unchanged}), flush=True)
    raise SystemExit(result['status'] != 'passed')


if __name__ == '__main__':
    main()
