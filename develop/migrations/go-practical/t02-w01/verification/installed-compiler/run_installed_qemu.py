"""Exercise the real installed-root reader with all registered bundle bytes.

Local Linux emulation is a scoped integration check, not a release gate.
"""
from pathlib import Path
import hashlib
import json
import os
import shutil
import subprocess
import sys
import time
import traceback

ROOT = Path(__file__).resolve().parent
REPO = ROOT.parents[5]
VOLUME = Path('/private/tmp/mpk-go-installed-205-fs')
GO = Path('/private/tmp/mpk-installed-compiler-205-original-go-output')
RUST = VOLUME / 'originals/registered'
INSTALLED = VOLUME / 'installed-qemu-binding'
EMULATOR = VOLUME / 'qemu-backports/qemu-x86_64'
RUNTIME = VOLUME / 'qemu-amd64-runtime'
IMAGE = 'sha256:7cf1e580ef5539f03b58560753e8ab84c8c360960d99dff714004aa98f203977'
sys.dont_write_bytecode = True
sys.path.insert(0, str(ROOT))
from run_scoped import seal, write
sys.path.insert(0, str(REPO / 'scripts'))
import successor_release_bundles as release


def main():
    out = ROOT / 'installed-qemu-binding'
    out.mkdir(exist_ok=False)
    linux = ROOT / 'linux-owner-serial/native-originals/versioned-owned'
    darwin = ROOT.parent / 'compiler-inputs/native/native-originals/versioned-owned'
    build = json.loads((ROOT / 'linux-test-diagnostic/result.json').read_bytes())
    assert build['status'] == 'passed' and build['consumed_inputs_unchanged']
    binary = Path(build['executable']['path'])
    assert seal(binary) == build['executable']
    registry, candidates = release.validate_release_models()
    roots = {
        candidates['go']['frontend_bundles'][0]['bundle_id']: GO / 'frontend',
        candidates['go']['toolchain_bundles'][0]['bundle_id']: GO / 'toolchain',
        candidates['rust']['frontend_bundles'][0]['bundle_id']: RUST / 'frontend',
        candidates['rust']['toolchain_bundles'][0]['bundle_id']: RUST / 'toolchain',
    }
    bundles = {b['bundle_id']: b for field in ['frontend_bundles', 'toolchain_bundles'] for b in registry[field]}
    assert set(roots) == set(bundles)
    for identifier, root in roots.items():
        release.validate_root(root, bundles[identifier])
    inputs = {Path(__file__).resolve(), ROOT / 'run_scoped.py', REPO / 'scripts/successor_release_bundles.py', REPO / 'scripts/release_bundles.py', REPO / 'scripts/rust_build_inputs.py', ROOT / 'linux-test-diagnostic/result.json', binary}
    inputs.add(EMULATOR)
    inputs.update(p for p in RUNTIME.rglob('*') if p.is_file())
    inputs.update([ROOT / 'qemu-backports-openat2/result.json', ROOT / 'qemu-amd64-runtime/result.json', ROOT / 'installed-qemu-diagnostic/result.json'])
    inputs.update(p for root in roots.values() for p in root.rglob('*') if p.is_file())
    inputs.update(p for root in [linux, darwin] for p in root.rglob('*') if p.is_file())
    inputs.update(p for p in (REPO / 'release/bundles').rglob('*.json'))
    before = [seal(p) for p in sorted(inputs)]
    write(out / 'inputs.json', {'inputs': before, 'selection_reason': 'Materialize all four complete registered Go/Rust frontend/toolchain bundles and both fixed registry transports, with unique actual file bytes. Execute the unchanged installed-root reader and bind the complete original Go owner. Negative checks alter one selected SDK byte, rehash a conflicting host, and cross the complete original Darwin owner. Local QEMU user-mode emulation on Linux arm64 is not a native Linux release gate; no practical profile or source/program theorem is activated.'})
    report = {'status': 'running', 'inputs': seal(out / 'inputs.json'), 'native_Linux_release_acceptance': False, 'full_T_gate': 'Deferred to final T02-W04: ./scripts/check-fast.sh'}
    write(out / 'result.json', report)
    start = time.monotonic()
    calls = []

    def execute(name, arguments, expected, *, installed=True):
        command = ['/usr/local/bin/docker', 'run', '--pull=never', '--rm', '--platform', 'linux/arm64', '--network', 'none', '--read-only', '--cap-drop', 'ALL', '--security-opt', 'no-new-privileges', '--user', f'{os.getuid()}:{os.getgid()}', '--tmpfs', '/tmp:rw,nosuid,nodev,size=128m,mode=1777', '--mount', f'type=bind,src={INSTALLED},dst=/installed,readonly', '--mount', f'type=bind,src={linux},dst=/linux-owner,readonly', '--mount', f'type=bind,src={darwin},dst=/darwin-owner,readonly', '--mount', f'type=bind,src={out},dst=/controls,readonly']
        command.extend(['--mount', f'type=bind,src={EMULATOR},dst=/qemu,readonly', '--mount', f'type=bind,src={RUNTIME},dst=/x86root,readonly'])
        if not installed:
            command.extend(['--mount', f'type=bind,src={binary},dst=/observer,readonly'])
        command.extend([IMAGE, '/qemu', '-L', '/x86root', '/installed/bin/mpk' if installed else '/observer', *arguments])
        result = subprocess.run(command, cwd=REPO, capture_output=True, check=False)
        stdout, stderr = out / f'{name}.stdout', out / f'{name}.stderr'
        stdout.write_bytes(result.stdout)
        stderr.write_bytes(result.stderr)
        row = {'name': name, 'command': command, 'exit_code': result.returncode, 'stdout': seal(stdout), 'stderr': seal(stderr)}
        calls.append(row)
        write(out / 'commands.json', calls)
        assert result.returncode == 0 and not result.stderr, f'{name}: {result.stderr.decode(errors="replace")}'
        observed = json.loads(result.stdout)
        assert all(observed.get(key) == value for key, value in expected.items()), (name, observed)
        row['observation'] = {key: observed.get(key) for key in expected}
        write(out / 'commands.json', calls)
        return observed

    try:
        assert not INSTALLED.exists()
        INSTALLED.mkdir()
        (INSTALLED / 'bin').mkdir()
        (INSTALLED / 'share/mpk').mkdir(parents=True)
        destination = INSTALLED / 'libexec/mpk/bundles'
        destination.mkdir(parents=True)
        shutil.copyfile(binary, INSTALLED / 'bin/mpk')
        (INSTALLED / 'bin/mpk').chmod(0o555)
        for path in [release.REGISTRY_PATH, release.SEMANTIC_REGISTRY_PATH]:
            shutil.copyfile(path, INSTALLED / 'share/mpk' / path.name)
            (INSTALLED / 'share/mpk' / path.name).chmod(0o444)
        for identifier, source in roots.items():
            shutil.copytree(source, destination / identifier, copy_function=shutil.copy2)
        release.normalize_directories(INSTALLED)
        for identifier, bundle in bundles.items():
            release.validate_root(destination / identifier, bundle)
        execute('uninstalled-executable', ['--snapshot'], {'status': 'rejected', 'phase': 'installed', 'error': 'Release'}, installed=False)
        snapshot = execute('original-snapshot', ['--snapshot'], {'status': 'accepted', 'phase': 'installed'})
        go_descriptor = bundles[candidates['go']['toolchain_bundles'][0]['bundle_id']]
        assert snapshot['inventory'] == go_descriptor['inventory']
        bound = execute('original-binding', ['--bind', '/linux-owner', 'original'], {'status': 'accepted', 'phase': 'bound', 'files': 5577, 'bytes': 166354960, 'original_objects': 5697, 'original_packages': 182})
        assert bound['compiler_record']['inventory'] == go_descriptor['inventory']
        wire = bound['compiler_record']
        wire['host'] = 'darwin_arm64'
        wire.pop('content_sha256')
        wire['content_sha256'] = hashlib.sha256(b'MPK-PRIVATE-GO-COMPILER-IMAGE-INPUTS-1.0\0' + release.canonical(wire)).hexdigest()
        (out / 'changed-host.json').write_bytes(release.canonical(wire))
        execute('coherent-wrong-host', ['--bind', '/linux-owner', '/controls/changed-host.json'], {'status': 'rejected', 'phase': 'binding', 'error': 'Inputs'})
        execute('crossed-darwin-owner', ['--bind', '/darwin-owner', 'original'], {'status': 'rejected', 'phase': 'binding', 'error': 'Inputs'})
        target = destination / go_descriptor['bundle_id'] / 'go/VERSION'
        original = target.read_bytes()
        assert len(original) > 0
        try:
            target.chmod(0o644)
            target.write_bytes(bytes([original[0] ^ 1]) + original[1:])
            target.chmod(0o444)
            execute('changed-installed-sdk-byte', ['--snapshot'], {'status': 'rejected', 'phase': 'installed', 'error': 'Release'})
        finally:
            target.chmod(0o644)
            target.write_bytes(original)
            target.chmod(0o444)

        # The complete Rust namespace is part of this installed release even
        # though the Go selection snapshots only the Go pair.
        rust_frontend = candidates['rust']['frontend_bundles'][0]['bundle_id']
        target = destination / rust_frontend / 'bin/rust2vir'
        held = VOLUME / 'held-rust2vir-original'
        assert not held.exists()
        try:
            target.parent.chmod(0o755)
            target.rename(held)
            target.parent.chmod(0o555)
            execute('missing-rust-namespace-member', ['--snapshot'], {'status': 'rejected', 'phase': 'installed', 'error': 'Release'})
        finally:
            target.parent.chmod(0o755)
            if held.exists():
                held.rename(target)
            target.parent.chmod(0o555)

        # Construct a fully rehashed, independently valid alternate registry,
        # together with the matching changed Go executable, while retaining all
        # bundle members. The build-pinned authority must still reject it.
        registry_file = INSTALLED / 'share/mpk/bundle-registry.json'
        original_registry = registry_file.read_bytes()
        binary_file = destination / go_descriptor['bundle_id'] / 'go/bin/go'
        original_binary = binary_file.read_bytes()
        changed_binary = original_binary[:-1] + bytes([original_binary[-1] ^ 1])
        changed_hash = hashlib.sha256(changed_binary).hexdigest()
        alternate = json.loads(original_registry)
        changed_toolchain = next(t for t in alternate['toolchain_bundles'] if t['bundle_id'] == go_descriptor['bundle_id'])
        next(f for f in changed_toolchain['inventory']['files'] if f['path'] == 'go/bin/go')['sha256'] = changed_hash
        next(c for c in changed_toolchain['components'] if c['name'] == 'go')['binary_sha256'] = changed_hash
        changed_toolchain['distribution_sha256'] = hashlib.sha256(release.CONTENT_DOMAIN + release.canonical(changed_toolchain['inventory'])).hexdigest()
        alternate.pop('registry_sha256')
        alternate['registry_sha256'] = hashlib.sha256(release.REGISTRY_DOMAIN + release.canonical(alternate)).hexdigest()
        alternate_transport = release.canonical_line(alternate)
        (out / 'alternate-registry.json').write_bytes(alternate_transport)
        execute('alternate-registry-validates', ['--validate-registry', '/controls/alternate-registry.json'], {'status': 'accepted', 'phase': 'registry-validation', 'registry_sha256': alternate['registry_sha256']})
        try:
            registry_file.chmod(0o644)
            registry_file.write_bytes(alternate_transport)
            registry_file.chmod(0o444)
            binary_file.chmod(0o755)
            binary_file.write_bytes(changed_binary)
            binary_file.chmod(0o555)
            execute('alternate-valid-registry-pin', ['--snapshot'], {'status': 'rejected', 'phase': 'installed', 'error': 'Release'})
        finally:
            registry_file.chmod(0o644)
            registry_file.write_bytes(original_registry)
            registry_file.chmod(0o444)
            binary_file.chmod(0o755)
            binary_file.write_bytes(original_binary)
            binary_file.chmod(0o555)
        for identifier, bundle in bundles.items():
            release.validate_root(destination / identifier, bundle)
        assert registry_file.read_bytes() == original_registry
        report.update(status='passed', complete_bundle_ids=sorted(roots), checks=[row['name'] for row in calls], installed_registry_and_sdk_restored=True)
    except BaseException as error:
        report.update(status='failed', error=repr(error), traceback=traceback.format_exc())
        raise
    finally:
        report.update(seconds=time.monotonic()-start, original_inputs_unchanged=before == [seal(p) for p in sorted(inputs)], commands=calls)
        write(out / 'result.json', report)
    print(json.dumps({key: report[key] for key in ['status', 'seconds', 'original_inputs_unchanged', 'checks']}))


if __name__ == '__main__':
    main()
