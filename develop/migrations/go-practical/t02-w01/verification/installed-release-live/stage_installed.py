"""Stage the complete real registered namespace for an emulated local test."""
from pathlib import Path
import hashlib
import json
import shutil
import sys
import tarfile
import time
import traceback

ROOT = Path(__file__).resolve().parent
REPO = ROOT.parents[5]
VOLUME = Path('/private/tmp/mpk-go-combined-recovery-20261029-fs')
RUST = VOLUME / 'registered-rust-finalized'
GO = Path('/private/tmp/mpk-installed-compiler-205-original-go-output')
INSTALLED = VOLUME / 'installed-live'
QEMU = VOLUME / 'qemu-restored'
OUT = ROOT / 'installed-stage'
PRIOR = ROOT.parent / 'installed-compiler'
sys.dont_write_bytecode = True
sys.path.insert(0, str(PRIOR))
from run_scoped import seal, write
sys.path.insert(0, str(REPO / 'scripts'))
import successor_release_bundles as release


def main():
    assert Path.cwd() == REPO and VOLUME.is_mount()
    assert not INSTALLED.exists() and not QEMU.exists() and not OUT.exists()
    build = json.loads((ROOT / 'observer-final/result.json').read_bytes())
    rust = json.loads((ROOT / 'rust-finalization/result.json').read_bytes())
    retained = json.loads((PRIOR / 'qemu-retained-environment/result.json').read_bytes())
    assert build['status'] == rust['status'] == retained['status'] == 'passed'
    assert build['source_inputs_unchanged'] and rust['source_inputs_unchanged'] and retained['inputs_unchanged']
    binary = Path(build['executable']['path'])
    archive = Path(retained['archive']['path'])
    assert seal(binary) == build['executable'] and seal(archive) == retained['archive']
    registry, candidates = release.validate_release_models()
    roots = {candidates['go']['frontend_bundles'][0]['bundle_id']: GO / 'frontend',
             candidates['go']['toolchain_bundles'][0]['bundle_id']: GO / 'toolchain',
             candidates['rust']['frontend_bundles'][0]['bundle_id']: RUST / 'frontend',
             candidates['rust']['toolchain_bundles'][0]['bundle_id']: RUST / 'toolchain'}
    bundles = {item['bundle_id']: item for kind in ['frontend_bundles', 'toolchain_bundles'] for item in registry[kind]}
    assert set(roots) == set(bundles)
    OUT.mkdir()
    files = [Path(__file__).resolve(), binary, archive, ROOT / 'observer-final/result.json',
             ROOT / 'rust-finalization/result.json', release.REGISTRY_PATH, release.SEMANTIC_REGISTRY_PATH]
    before = [seal(path) for path in files]
    write(OUT / 'inputs.json', {'inputs': before, 'roots': {key: str(value) for key, value in roots.items()},
          'selection_reason': 'Stage the original complete Go and Rust registered bundles and '
          'the pinned registries beside the test observer. Restore the retained QEMU runtime '
          'byte for byte. This exercises the actual loader in emulation, not a native Linux '
          'release or production CLI/launcher.'})
    report = {'status': 'running', 'native_Linux_release_acceptance': False,
              'full_T_gate': 'Deferred to GO-PRAC-T02-W04: ./scripts/check-fast.sh'}
    write(OUT / 'result.json', report)
    started = time.monotonic()
    try:
        for identifier, root in roots.items():
            release.validate_root(root, bundles[identifier])
        print(json.dumps({'phase': 'complete_original_bundles_validated', 'seconds': round(time.monotonic()-started)}), flush=True)
        QEMU.mkdir()
        with tarfile.open(archive) as source:
            members = source.getmembers()
            assert {member.name for member in members} == set(retained['inputs']) and len(members) == len(retained['inputs'])
            for member in members:
                expected = retained['inputs'][member.name]
                assert member.isfile() and member.mode == expected['mode']
                raw = source.extractfile(member).read()
                assert len(raw) == expected['bytes'] and hashlib.sha256(raw).hexdigest() == expected['sha256']
                destination = QEMU / member.name
                destination.parent.mkdir(parents=True, exist_ok=True)
                destination.write_bytes(raw)
                destination.chmod(member.mode)
                restored = seal(destination)
                assert all(restored[key] == expected[key] for key in ['bytes', 'sha256', 'mode'])
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
            release.validate_root(roots[identifier], bundle)
        installed_binary = seal(INSTALLED / 'bin/mpk')
        assert all(installed_binary[key] == build['executable'][key] for key in ['bytes', 'sha256', 'mode'])
        assert before == [seal(path) for path in files]
        report.update(status='passed', original_inputs_unchanged=True,
                      installed_root=str(INSTALLED), qemu_root=str(QEMU),
                      installed_executable=installed_binary, complete_bundle_ids=sorted(roots),
                      restored_qemu_members=len(retained['inputs']))
    except BaseException as error:
        report.update(status='failed', error=repr(error), traceback=traceback.format_exc())
        raise
    finally:
        report['seconds'] = time.monotonic()-started
        write(OUT / 'result.json', report)
    print(json.dumps({'status': report['status'], 'seconds': round(report['seconds']),
                      'original_inputs_unchanged': report['original_inputs_unchanged']}), flush=True)


if __name__ == '__main__':
    main()
