"""Reproduce complete registered Rust bytes for the installed-reader test.

This is a byte-reproduction observer, not the native Linux release/build gate.
It retains the frozen upstream archives, real source and exact build recipe.
"""
from pathlib import Path
import datetime
import hashlib
import json
import os
import shutil
import subprocess
import sys
import tarfile
import time
import traceback

ROOT = Path(__file__).resolve().parent
REPO = ROOT.parents[5]
WORK = Path('/private/tmp/mpk-installed-compiler-205-rust-originals')
sys.dont_write_bytecode = True
sys.path.insert(0, str(ROOT))
from run_scoped import seal, write
sys.path.insert(0, str(REPO / 'scripts'))
import rust_build_inputs as rust
import successor_release_bundles as bundles


def main():
    out = ROOT / 'rust-byte-reproduction'
    out.mkdir(exist_ok=False)
    WORK.mkdir(exist_ok=False)
    paths = {Path(__file__).resolve(), ROOT / 'run_scoped.py', REPO / 'scripts/rust_build_inputs.py', REPO / 'scripts/successor_release_bundles.py', REPO / 'scripts/release_bundles.py', REPO / 'release/build-inputs/rust/build-inputs.json', REPO / 'develop/specs/vectors/rust-build-inputs-v0.json'}
    paths.update(p for p in (REPO / 'rust-tools/rust2vir').rglob('*') if p.is_file())
    paths.update(p for p in (REPO / 'release/bundles').rglob('*.json'))
    inputs = [seal(p) for p in sorted(paths)]
    with tarfile.open(out / 'inputs.tar.gz', 'x:gz', dereference=True) as archive:
        for p in sorted(paths):
            archive.add(p, arcname=str(p.relative_to(REPO)), recursive=False)
    write(out / 'inputs.json', {'inputs': inputs, 'archive': seal(out / 'inputs.tar.gz'), 'scope': 'Restore every registered Rust toolchain/frontend file, with complete original frozen Rust archives and source. The production provisioning/launcher gate requires a native Linux host; this local emulated byte observer does not replace or satisfy that gate and changes no registry, profile, source, limit or production recipe.'})
    report = {'status': 'running', 'started_at_utc': datetime.datetime.now(datetime.timezone.utc).isoformat(), 'inputs': seal(out / 'inputs.json'), 'native_Linux_release_acceptance': False, 'full_T_gate': 'Deferred to final T02-W04: ./scripts/check-fast.sh'}
    write(out / 'result.json', report)
    downloads, calls, phases = [], [], []
    original_download, original_run = rust.download, subprocess.run

    def download(url, expected, destination, maximum=2_147_483_648):
        row = {'url': url, 'expected_sha256': expected, 'destination': str(destination), 'maximum': maximum, 'status': 'running'}
        downloads.append(row)
        write(out / 'downloads.json', downloads)
        try:
            original_download(url, expected, destination, maximum)
            row.update(status='passed', original_archive=seal(destination))
        except BaseException as error:
            row.update(status='failed', error=repr(error), traceback=traceback.format_exc())
            raise
        finally:
            write(out / 'downloads.json', downloads)

    def observed(*args, **kwargs):
        index = len(calls)
        row = {'command': list(map(str, args[0]))}
        calls.append(row)
        stdout, stderr = b'', b''
        try:
            result = original_run(*args, **kwargs)
            row['exit_code'] = result.returncode
            stdout, stderr = result.stdout or b'', result.stderr or b''
            return result
        except BaseException as error:
            row.update(error=repr(error), traceback=traceback.format_exc())
            raise
        finally:
            for name, data in [('stdout', stdout), ('stderr', stderr)]:
                path = out / f'command-{index:02d}-{name}.log'
                path.write_bytes(data)
                row[name] = seal(path)
            write(out / 'commands.json', calls)

    def phase(name, action):
        row = {'name': name, 'status': 'running'}
        phases.append(row)
        write(out / 'phases.json', phases)
        try:
            result = action()
            row['status'] = 'passed'
            return result
        except BaseException as error:
            row.update(status='failed', error=repr(error))
            raise
        finally:
            write(out / 'phases.json', phases)

    rust.download, subprocess.run = download, observed
    start = time.monotonic()
    try:
        vector = rust.load_vector()
        rust.validate_project_templates(vector)
        registry, candidates = bundles.validate_release_models()
        candidate = candidates['rust']
        staging, work = WORK / 'original-build-inputs', WORK / 'work'
        staging.mkdir()
        work.mkdir()
        archives = WORK / 'downloads'
        archives.mkdir()
        phase('complete-frozen-rust-distribution', lambda: rust.materialize_rust_distribution(vector, archives, staging, work))
        phase('frozen-llvm-linker-and-support', lambda: rust.materialize_llvm(vector, archives, staging, work))
        phase('complete-native-development-sysroot', lambda: rust.materialize_native_sysroot(vector, staging, work))
        phase('complete-frozen-original-sources-and-vendor', lambda: rust.materialize_sources_and_vendor(vector, archives, staging, work))
        rust.materialize_cargo_home(vector, staging)
        frontend = WORK / 'original-frontend'
        source_inventory = rust.capture_frontend_project(frontend)
        writable = WORK / 'writable'
        writable.mkdir()
        dirs = {}
        for name in ['home', 'cargo-home', 'tmp', 'target', 'work']:
            dirs[name] = writable / name
            dirs[name].mkdir()
        shutil.copyfile(staging / 'cargo-home-seed/config.toml', dirs['cargo-home'] / 'config.toml')
        command = [rust.docker_path(), 'run', '--rm', '--pull=never', '--platform=linux/amd64', '--network=none', '--read-only', '--cap-drop=ALL', '--security-opt=no-new-privileges', f'--user={os.getuid()}:{os.getgid()}']
        mounts = [(staging / 'toolchain', '/mpk/toolchain', True), (staging / 'native-sysroot', '/mpk/native-sysroot', True), (staging / 'vendor', '/mpk/vendor', True), (frontend, '/mpk/frontend', True)]
        mounts.extend((path, f'/mpk/{name}', False) for name, path in dirs.items())
        for source, destination, readonly in mounts:
            command.append(f'--mount=type=bind,src={source},dst={destination}' + (',readonly' if readonly else ''))
        command.extend(['--workdir=/mpk/frontend', rust.RUNTIME_IMAGE, '/usr/bin/env', '-i', *rust.docker_build_environment(vector), *bundles.RUST_BUILD_ARGUMENTS])
        def build():
            result = subprocess.run(command, cwd=REPO, capture_output=True, check=False)
            assert result.returncode == 0, result.stderr.decode(errors='replace')
        phase('original-frozen-rust-frontend-build', build)
        assert source_inventory == rust.current_frontend_inventory()
        runtime_view = phase('complete-original-runtime-image', lambda: rust.docker_image_view(rust.RUNTIME_IMAGE, work / 'runtime-image'))
        runtime = staging / 'native-runtime'
        # Preserve the exact old name until the unchanged registered
        # libstdcxx normalization is applied to all actual ELF files below.
        for entry in candidate['toolchain_bundles'][0]['inventory']['files']:
            if not entry['path'].startswith('native-runtime/'):
                continue
            relative = entry['path'].removeprefix('native-runtime/').replace('libstdcxx.so.6', 'libstdc++.so.6')
            runtime_view.copy_file(relative, runtime / relative, executable=entry['executable'])
        rust.normalize_portable_cpp_runtime(staging, dirs['target'])
        output = WORK / 'registered'
        for entry in candidate['frontend_bundles'][0]['inventory']['files']:
            source = dirs['target'] / 'x86_64-unknown-linux-gnu/release' / Path(entry['path']).name
            bundles.copy_described(source, output / 'frontend' / entry['path'], entry)
        for entry in candidate['toolchain_bundles'][0]['inventory']['files']:
            relative = entry['path']
            source = staging / relative if relative.startswith('native-runtime/') else staging / 'toolchain' / relative
            bundles.copy_described(source, output / 'toolchain' / relative, entry)
        for name, bundle in [('frontend', candidate['frontend_bundles'][0]), ('toolchain', candidate['toolchain_bundles'][0])]:
            bundles.normalize_directories(output / name)
            bundles.validate_root(output / name, bundle)
        report.update(status='passed', frontend=str(output / 'frontend'), toolchain=str(output / 'toolchain'), registered_complete_frontend=candidate['frontend_bundles'][0]['inventory'], registered_complete_toolchain=candidate['toolchain_bundles'][0]['inventory'])
    except BaseException as error:
        report.update(status='failed', error=repr(error), traceback=traceback.format_exc())
        raise
    finally:
        rust.download, subprocess.run = original_download, original_run
        report.update(seconds=time.monotonic() - start, consumed_inputs_unchanged=inputs == [seal(p) for p in sorted(paths)])
        write(out / 'result.json', report)
    print(json.dumps({k: report[k] for k in ['status', 'seconds', 'consumed_inputs_unchanged', 'frontend', 'toolchain']}))


if __name__ == '__main__':
    main()
