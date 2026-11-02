"""Reproduce the two complete registered Rust bundles from their frozen inputs.

This repeats the existing byte-reproduction recipe locally. It is not a native
Linux release gate and does not alter a registry, cache namespace or source.
"""
from pathlib import Path
import json
import os
import shutil
import subprocess
import sys
import time
import traceback

ROOT = Path(__file__).resolve().parent
REPO = ROOT.parents[5]
VOLUME = Path('/private/tmp/mpk-go-combined-recovery-20261029-fs')
DOWNLOADS = VOLUME / 'original-recovery-system/downloads'
WORK = VOLUME / 'registered-rust-reproduction'
OUTPUT = VOLUME / 'registered-rust-original'
OUT = ROOT / 'rust-reproduction'
PRIOR = ROOT.parent / 'installed-compiler'
sys.dont_write_bytecode = True
sys.path.insert(0, str(PRIOR))
from run_scoped import seal, write
sys.path.insert(0, str(REPO / 'scripts'))
import rust_build_inputs as rust
import successor_release_bundles as release


def main():
    assert Path.cwd() == REPO and VOLUME.is_mount()
    assert not WORK.exists() and not OUTPUT.exists() and not OUT.exists()
    retrieved = json.loads((ROOT / 'recovery-downloads-system/result.json').read_bytes())
    assert retrieved['status'] == 'passed' and retrieved['input_files_unchanged']
    for row in retrieved['rust_archives']:
        assert seal(Path(row['file']['path'])) == row['file']
    assert seal(Path(retrieved['complete_llvm']['path'])) == retrieved['complete_llvm']
    earlier = json.loads((PRIOR / 'rust-retry-reproduction/inputs.json').read_bytes())
    selected = [row for row in earlier['inputs'] if '/rust-tools/rust2vir/' in row['path']
                or row['path'].endswith(('rust_build_inputs.py', 'successor_release_bundles.py',
                                         'release_bundles.py', 'rust-build-inputs-v0.json', 'build-inputs.json'))]
    assert len(selected) == 276
    assert all(seal(Path(row['path'])) == row for row in selected)
    files = [Path(__file__).resolve(), PRIOR / 'run_scoped.py', ROOT / 'recovery-downloads-system/result.json']
    before = [seal(path) for path in files]
    WORK.mkdir()
    OUT.mkdir()
    write(OUT / 'inputs.json', {'selection_reason': 'Reproduce the complete four-bundle installed '
          'namespace with the unchanged frozen Rust source, toolchain, dependencies, release '
          'inventories and original offline frontend build. No original program or input is replaced.',
          'inputs': before, 'original_source_epoch': selected,
          'verified_download_result': seal(ROOT / 'recovery-downloads-system/result.json')})
    report = {'status': 'running', 'native_Linux_release_acceptance': False,
              'full_T_gate': 'Deferred to GO-PRAC-T02-W04: ./scripts/check-fast.sh'}
    write(OUT / 'result.json', report)
    phases = []
    started = time.monotonic()

    def phase(name, action):
        row = {'name': name, 'status': 'running'}
        phases.append(row)
        write(OUT / 'phases.json', phases)
        print(json.dumps({'phase': name, 'status': 'running', 'seconds': round(time.monotonic()-started)}), flush=True)
        try:
            value = action()
            row['status'] = 'passed'
            return value
        except BaseException as error:
            row.update(status='failed', error=repr(error))
            raise
        finally:
            write(OUT / 'phases.json', phases)
            print(json.dumps({'phase': name, 'status': row['status'], 'seconds': round(time.monotonic()-started)}), flush=True)

    try:
        vector = rust.load_vector()
        rust.validate_project_templates(vector)
        registry, candidates = release.validate_release_models()
        candidate = candidates['rust']
        staging, scratch = WORK / 'original-build-inputs', WORK / 'work'
        staging.mkdir()
        scratch.mkdir()
        phase('complete-frozen-rust-distribution', lambda: rust.materialize_rust_distribution(vector, DOWNLOADS, staging, scratch))
        phase('frozen-llvm-linker-and-support', lambda: rust.materialize_llvm(vector, DOWNLOADS, staging, scratch))
        phase('complete-native-development-sysroot', lambda: rust.materialize_native_sysroot(vector, staging, scratch))
        phase('complete-frozen-original-sources-and-vendor', lambda: rust.materialize_sources_and_vendor(vector, DOWNLOADS, staging, scratch))
        rust.materialize_cargo_home(vector, staging)
        frontend = WORK / 'original-frontend'
        inventory = rust.capture_frontend_project(frontend)
        writable = WORK / 'writable'
        writable.mkdir()
        dirs = {}
        for name in ['home', 'cargo-home', 'tmp', 'target', 'work']:
            dirs[name] = writable / name
            dirs[name].mkdir()
        shutil.copyfile(staging / 'cargo-home-seed/config.toml', dirs['cargo-home'] / 'config.toml')
        command = [rust.docker_path(), 'run', '--rm', '--pull=never', '--platform=linux/amd64',
                   '--network=none', '--read-only', '--cap-drop=ALL', '--security-opt=no-new-privileges',
                   f'--user={os.getuid()}:{os.getgid()}']
        mounts = [(staging / 'toolchain', '/mpk/toolchain', True),
                  (staging / 'native-sysroot', '/mpk/native-sysroot', True),
                  (staging / 'vendor', '/mpk/vendor', True), (frontend, '/mpk/frontend', True)]
        mounts.extend((path, f'/mpk/{name}', False) for name, path in dirs.items())
        for source, destination, readonly in mounts:
            command.append(f'--mount=type=bind,src={source},dst={destination}' + (',readonly' if readonly else ''))
        command.extend(['--workdir=/mpk/frontend', rust.RUNTIME_IMAGE, '/usr/bin/env', '-i',
                        *rust.docker_build_environment(vector), *release.RUST_BUILD_ARGUMENTS])
        write(OUT / 'build-command.json', {'command': command})

        def build():
            with (OUT / 'build.stdout').open('xb') as stdout, (OUT / 'build.stderr').open('xb') as stderr:
                code = subprocess.run(command, cwd=REPO, stdout=stdout, stderr=stderr, check=False).returncode
            assert code == 0, f'original offline frontend build exited {code}'
        phase('original-frozen-rust-frontend-build', build)
        assert inventory == rust.current_frontend_inventory()
        view = phase('complete-original-runtime-image', lambda: rust.docker_image_view(rust.RUNTIME_IMAGE, scratch / 'runtime-image'))
        projection = vector['production_projection']['native_runtime_projection']
        for item in candidate['toolchain_bundles'][0]['inventory']['files']:
            if not item['path'].startswith('native-runtime/'):
                continue
            relative = item['path'].removeprefix('native-runtime/').replace('libstdcxx.so.6', 'libstdc++.so.6')
            if relative == 'lib64/ld-linux-x86-64.so.2':
                source = projection['interpreter_source'].lstrip('/')
            else:
                matches = []
                for directory in projection['soname_source_directories']:
                    name = f"{directory.lstrip('/')}/{Path(relative).name}"
                    try:
                        matches.append((name, view.resolve(name)))
                    except rust.RustBuildFailure:
                        continue
                assert matches and all(rust.same_file_bytes(matches[0][1][1], other[1][1]) for other in matches[1:])
                source = matches[0][0]
            view.copy_file(source, staging / 'native-runtime' / relative, executable=item['executable'])
        rust.normalize_portable_cpp_runtime(staging, dirs['target'])
        target = dirs['target'] / 'x86_64-unknown-linux-gnu/release'
        for item in candidate['frontend_bundles'][0]['inventory']['files']:
            release.copy_described(target / Path(item['path']).name, OUTPUT / 'frontend' / item['path'], item)
        for item in candidate['toolchain_bundles'][0]['inventory']['files']:
            relative = item['path']
            source = staging / relative if relative.startswith('native-runtime/') else staging / 'toolchain' / relative
            release.copy_described(source, OUTPUT / 'toolchain' / relative, item)
        for name, bundle in [('frontend', candidate['frontend_bundles'][0]), ('toolchain', candidate['toolchain_bundles'][0])]:
            release.normalize_directories(OUTPUT / name)
            release.validate_root(OUTPUT / name, bundle)
        report.update(status='passed', frontend=str(OUTPUT / 'frontend'), toolchain=str(OUTPUT / 'toolchain'),
                      registered_frontend=candidate['frontend_bundles'][0]['inventory'],
                      registered_toolchain=candidate['toolchain_bundles'][0]['inventory'])
    except BaseException as error:
        report.update(status='failed', error=repr(error), traceback=traceback.format_exc())
        raise
    finally:
        report.update(seconds=time.monotonic()-started,
                      source_inputs_unchanged=(before == [seal(path) for path in files]
                                               and all(seal(Path(row['path'])) == row for row in selected)),
                      phases=phases)
        write(OUT / 'result.json', report)
    print(json.dumps({'status': report['status'], 'seconds': report['seconds'],
                      'source_inputs_unchanged': report['source_inputs_unchanged']}), flush=True)


if __name__ == '__main__':
    main()
