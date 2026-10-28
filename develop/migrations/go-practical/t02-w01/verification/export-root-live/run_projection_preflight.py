"""Compile, lint and format the independent complete-owner projection test."""
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
    out = ROOT / epoch
    out.mkdir(exist_ok=False)
    test = REPO / 'crates/mpk-cli/tests/go_practical_live_export_projection.rs'
    paths = {Path(__file__).resolve(), REPO / 'Cargo.toml', REPO / 'Cargo.lock', test}
    paths.update((REPO / 'release/bundles').glob('*.json'))
    for crate in ['mpk-cli', 'mpk-api', 'mpk-cert', 'mpk-core', 'mpk-kernel', 'mpk-linux-sandbox', 'mpk-theory', 'mpk-vc']:
        base = REPO / 'crates' / crate
        paths.add(base / 'Cargo.toml')
        paths.update(p for p in (base / 'src').rglob('*') if p.is_file())
        paths.update(p for p in base.glob('build.rs') if p.is_file())
    before = [seal(p) for p in sorted(paths)]
    with tarfile.open(out / 'inputs.tar.gz', 'x:gz', dereference=True) as archive:
        for path in sorted(paths):
            archive.add(path, arcname=str(path.relative_to(REPO)), recursive=False)
    env = {'CARGO_TARGET_DIR': '/private/tmp/mpk-go-source-root-clippy-target', 'CARGO_NET_OFFLINE': 'true', 'CARGO_INCREMENTAL': '0', 'CARGO_PROFILE_DEV_DEBUG': '0'}
    commands = {
        'rust-clippy': ['cargo', 'clippy', '--offline', '--locked', '-p', 'mpk-cli', '--test', 'go_practical_live_export_projection', '--target', 'x86_64-unknown-linux-gnu', '--', '-D', 'warnings'],
        'rust-format': ['rustfmt', '--edition', '2021', '--check', '--config', 'skip_children=true', str(test)],
    }
    write(out / 'inputs.json', {'selection_reason': 'Compile and lint the new Linux integration test that binds all original Go exports and compiler inputs, and check its formatting; execution and native evidence restoration are verified separately.', 'inputs': before, 'archive': seal(out / 'inputs.tar.gz'), 'commands': commands, 'environment': env})
    result = {'status': 'running', 'native_Linux_release_acceptance': False, 'full_T_gate': 'Deferred to final T02-W04: ./scripts/check-fast.sh'}
    write(out / 'result.json', result)
    checks = {}
    start = time.monotonic()
    for name, command in commands.items():
        with (out / (name + '.stdout')).open('xb') as stdout, (out / (name + '.stderr')).open('xb') as stderr:
            code = subprocess.run(command, cwd=REPO, env={**os.environ, **env}, stdout=stdout, stderr=stderr, check=False).returncode
        checks[name] = {'status': 'passed' if code == 0 else 'failed', 'exit_code': code, 'stdout': seal(out / (name + '.stdout')), 'stderr': seal(out / (name + '.stderr'))}
        print(json.dumps({'check': name, 'status': checks[name]['status'], 'seconds': round(time.monotonic() - start)}), flush=True)
    unchanged = before == [seal(p) for p in sorted(paths)]
    passed = unchanged and all(c['status'] == 'passed' for c in checks.values())
    result.update(status='passed' if passed else 'failed', seconds=time.monotonic() - start, original_inputs_unchanged=unchanged, checks=checks)
    write(out / 'result.json', result)
    print(json.dumps({'status': result['status'], 'original_inputs_unchanged': unchanged}), flush=True)
    raise SystemExit(not passed)


if __name__ == '__main__':
    main()
