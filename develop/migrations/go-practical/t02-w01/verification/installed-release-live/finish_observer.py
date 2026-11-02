"""Finish the scoped checks after the Linux image lacked a Clippy component."""
from pathlib import Path
import json
import os
import subprocess
import sys
import time

ROOT = Path(__file__).resolve().parent
REPO = ROOT.parents[5]
OUT = ROOT / 'observer-final'
CARGO = Path('/private/tmp/mpk-installed-release-live-clippy-cargo')
TARGET = Path('/private/tmp/mpk-installed-release-live-clippy-target')
REGISTRY = Path.home() / '.cargo/registry'
sys.dont_write_bytecode = True
sys.path.insert(0, str(ROOT.parent / 'installed-compiler'))
from run_scoped import seal, write


def main():
    assert Path.cwd() == REPO and not OUT.exists() and not CARGO.exists() and not TARGET.exists()
    build = json.loads((ROOT / 'observer-build/result.json').read_bytes())
    sources = json.loads((ROOT / 'observer-build/inputs.json').read_bytes())['sources']
    assert build['status'] == 'failed' and build['source_inputs_unchanged']
    assert build['checks']['build']['exit_code'] == 0 and build['checks']['clippy']['exit_code'] != 0
    assert b'cargo-clippy' in (ROOT / 'observer-build/clippy.stderr').read_bytes()
    assert sources == [seal(Path(row['path'])) for row in sources]
    binary = Path(build['executable']['path'])
    assert seal(binary) == build['executable']
    OUT.mkdir()
    (CARGO / 'registry/src').mkdir(parents=True)
    (CARGO / 'registry/cache').symlink_to(REGISTRY / 'cache', target_is_directory=True)
    (CARGO / 'registry/index').symlink_to(REGISTRY / 'index', target_is_directory=True)
    TARGET.mkdir()
    environment = {'CARGO_HOME': str(CARGO), 'CARGO_TARGET_DIR': str(TARGET), 'CARGO_NET_OFFLINE': 'true',
                   'CARGO_INCREMENTAL': '0', 'CARGO_PROFILE_DEV_DEBUG': '0'}
    commands = {'clippy': ['cargo', 'clippy', '--offline', '--locked', '-p', 'mpk-cli', '--example',
                           'go_practical_live_installed_roots', '--target', 'x86_64-unknown-linux-gnu',
                           '--', '-D', 'warnings'],
                'format': ['rustfmt', '--edition', '2021', '--check', '--config', 'skip_children=true',
                           str(REPO / 'crates/mpk-cli/examples/go_practical_live_installed_roots.rs')]}
    write(OUT / 'inputs.json', {'selection_reason': 'Retain the successful Linux x86 build and its failed '
          'missing-Clippy-component diagnostic. Run real Clippy for that same Linux target on the '
          'host toolchain and check the new observer format, with exact original sources and lockfile.',
          'driver': seal(Path(__file__).resolve()), 'original_build': seal(ROOT / 'observer-build/result.json'),
          'source_archive': seal(ROOT / 'observer-build/inputs.tar.gz'), 'source_inputs': sources,
          'executable': build['executable'], 'commands': commands, 'environment': environment})
    started = time.monotonic()
    rows = {}
    for name, command in commands.items():
        with (OUT / f'{name}.stdout').open('xb') as stdout, (OUT / f'{name}.stderr').open('xb') as stderr:
            code = subprocess.run(command, cwd=REPO, env={**os.environ, **environment}, stdout=stdout, stderr=stderr, check=False).returncode
        rows[name] = {'exit_code': code, 'stdout': seal(OUT / f'{name}.stdout'), 'stderr': seal(OUT / f'{name}.stderr')}
        print(json.dumps({'check': name, 'exit_code': code, 'seconds': round(time.monotonic()-started)}), flush=True)
        if code:
            break
    unchanged = sources == [seal(Path(row['path'])) for row in sources] and seal(binary) == build['executable']
    passed = unchanged and len(rows) == len(commands) and all(row['exit_code'] == 0 for row in rows.values())
    write(OUT / 'result.json', {'status': 'passed' if passed else 'failed', 'checks': rows,
          'seconds': time.monotonic()-started, 'source_inputs_unchanged': unchanged,
          'executable': build['executable'], 'linux_build_exit_code': 0,
          'initial_missing_clippy_retained': seal(ROOT / 'observer-build/clippy.stderr'),
          'native_Linux_release_acceptance': False,
          'full_T_gate': 'Deferred to GO-PRAC-T02-W04: ./scripts/check-fast.sh'})
    print(json.dumps({'status': 'passed' if passed else 'failed', 'source_inputs_unchanged': unchanged}), flush=True)
    raise SystemExit(not passed)


if __name__ == '__main__':
    main()
