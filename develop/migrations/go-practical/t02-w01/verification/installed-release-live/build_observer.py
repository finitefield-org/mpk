"""Build and lint the staged installed-root observer using the pinned Linux image."""
from pathlib import Path
import json
import os
import shutil
import subprocess
import sys
import tarfile
import time

ROOT = Path(__file__).resolve().parent
REPO = ROOT.parents[5]
PRIOR = ROOT.parent / 'installed-combined'
OUT = ROOT / 'observer-build'
CARGO = Path('/private/tmp/mpk-installed-release-live-cargo')
TARGET = Path('/private/tmp/mpk-installed-release-live-target')
REGISTRY = Path.home() / '.cargo/registry'
IMAGE = 'sha256:d617f1b5c2a18778ba357ba0528e973bd24992b0dd18000dfbca059f4a15ebda'
sys.dont_write_bytecode = True
sys.path.insert(0, str(ROOT.parent / 'installed-compiler'))
from run_scoped import seal, write


def main():
    assert Path.cwd() == REPO and not OUT.exists() and not CARGO.exists() and not TARGET.exists()
    prior = json.loads((PRIOR / 'scoped-final/inputs.json').read_bytes())
    prior_result = json.loads((PRIOR / 'scoped-final/result.json').read_bytes())
    assert prior_result['status'] == 'passed' and prior_result['original_inputs_unchanged']
    assert all(seal(Path(row['path'])) == row for row in prior['inputs'])
    paths = {Path(row['path']) for row in prior['inputs']}
    paths.update([Path(__file__).resolve(), REPO / 'crates/mpk-cli/examples/go_practical_live_installed_roots.rs'])
    before = [seal(path) for path in sorted(paths)]
    OUT.mkdir()
    (CARGO / 'registry/src').mkdir(parents=True)
    TARGET.mkdir()
    with tarfile.open(OUT / 'inputs.tar.gz', 'x:gz', dereference=True) as archive:
        for path in sorted(paths):
            archive.add(path, arcname=str(path.relative_to(REPO)), recursive=False)
    env = {'CARGO_HOME': '/cargo', 'CARGO_TARGET_DIR': '/target', 'CARGO_NET_OFFLINE': 'true',
           'CARGO_INCREMENTAL': '0', 'CARGO_PROFILE_DEV_DEBUG': '0', 'HOME': '/tmp'}
    command = ['/usr/local/bin/docker', 'run', '--pull=never', '--rm', '--platform', 'linux/amd64',
               '--network', 'none', '--read-only', '--cap-drop', 'ALL', '--security-opt',
               'no-new-privileges', '--user', f'{os.getuid()}:{os.getgid()}',
               '--tmpfs', '/tmp:rw,exec,nosuid,nodev,size=1g,mode=1777',
               '--mount', f'type=bind,src={REPO},dst=/workspace,readonly',
               '--mount', f'type=bind,src={CARGO},dst=/cargo',
               '--mount', f'type=bind,src={REGISTRY / "cache"},dst=/cargo/registry/cache,readonly',
               '--mount', f'type=bind,src={REGISTRY / "index"},dst=/cargo/registry/index,readonly',
               '--mount', f'type=bind,src={TARGET},dst=/target', '--workdir', '/workspace']
    for key, value in env.items():
        command += ['--env', f'{key}={value}']
    command += [IMAGE]
    checks = {
        'build': command + ['cargo', 'rustc', '--offline', '--locked', '-p', 'mpk-cli',
                            '--example', 'go_practical_live_installed_roots', '--', '-Dwarnings'],
        'clippy': command + ['cargo', 'clippy', '--offline', '--locked', '-p', 'mpk-cli',
                             '--example', 'go_practical_live_installed_roots', '--', '-Dwarnings'],
        'format': ['rustfmt', '--edition', '2021', '--check', '--config', 'skip_children=true',
                   str(REPO / 'crates/mpk-cli/examples/go_practical_live_installed_roots.rs')],
    }
    write(OUT / 'inputs.json', {'selection_reason': 'Compile and lint the new staged installed-root '
          'observer and retain all previously selected linked source inputs; the existing '
          'test-only observer, original sources and production authority are unchanged.',
          'sources': before, 'source_archive': seal(OUT / 'inputs.tar.gz'),
          'previous_result': seal(PRIOR / 'scoped-final/result.json'),
          'commands': checks, 'environment': env})
    started = time.monotonic()
    rows = {}
    for name, args in checks.items():
        with (OUT / f'{name}.stdout').open('xb') as stdout, (OUT / f'{name}.stderr').open('xb') as stderr:
            code = subprocess.run(args, cwd=REPO, stdout=stdout, stderr=stderr, check=False).returncode
        rows[name] = {'exit_code': code, 'stdout': seal(OUT / f'{name}.stdout'),
                      'stderr': seal(OUT / f'{name}.stderr')}
        print(json.dumps({'check': name, 'exit_code': code, 'seconds': round(time.monotonic()-started)}), flush=True)
        if code:
            break
    original = TARGET / 'debug/examples/go_practical_live_installed_roots'
    binary = OUT / 'observer.elf'
    if original.is_file():
        shutil.copyfile(original, binary)
        binary.chmod(0o555)
    unchanged = before == [seal(path) for path in sorted(paths)]
    passed = unchanged and len(rows) == len(checks) and all(row['exit_code'] == 0 for row in rows.values())
    write(OUT / 'result.json', {'status': 'passed' if passed else 'failed', 'checks': rows,
          'seconds': time.monotonic()-started, 'source_inputs_unchanged': unchanged,
          'executable': seal(binary) if binary.exists() else None,
          'native_Linux_release_acceptance': False,
          'full_T_gate': 'Deferred to GO-PRAC-T02-W04: ./scripts/check-fast.sh'})
    print(json.dumps({'status': 'passed' if passed else 'failed', 'source_inputs_unchanged': unchanged}), flush=True)
    raise SystemExit(not passed)


if __name__ == '__main__':
    main()
