"""Finalize the exact rebuilt Rust bundles using independent file copies."""
from pathlib import Path
import json
import shutil
import sys
import time
import traceback

ROOT = Path(__file__).resolve().parent
REPO = ROOT.parents[5]
VOLUME = Path('/private/tmp/mpk-go-combined-recovery-20261029-fs')
WORK = VOLUME / 'registered-rust-reproduction'
OUTPUT = VOLUME / 'registered-rust-finalized'
COPIES = VOLUME / 'registered-rust-independent-copies'
OUT = ROOT / 'rust-finalization'
sys.dont_write_bytecode = True
sys.path.insert(0, str(ROOT.parent / 'installed-compiler'))
from run_scoped import seal, write
sys.path.insert(0, str(REPO / 'scripts'))
import successor_release_bundles as release


def main():
    assert Path.cwd() == REPO and VOLUME.is_mount()
    assert not OUTPUT.exists() and not COPIES.exists() and not OUT.exists()
    prior = json.loads((ROOT / 'rust-reproduction/result.json').read_bytes())
    assert prior['status'] == 'failed' and prior['source_inputs_unchanged']
    assert prior['error'] == "SuccessorReleaseFailure('BUNDLE_REPRODUCIBILITY_MISMATCH')"
    assert next(row for row in prior['phases'] if row['name'] == 'original-frozen-rust-frontend-build')['status'] == 'passed'
    _, candidates = release.validate_release_models()
    candidate = candidates['rust']
    staging = WORK / 'original-build-inputs'
    target = WORK / 'writable/target/x86_64-unknown-linux-gnu/release'
    rows = []
    for kind in ['frontend_bundles', 'toolchain_bundles']:
        for item in candidate[kind][0]['inventory']['files']:
            relative = item['path']
            source = (target / Path(relative).name if kind == 'frontend_bundles' else
                      staging / relative if relative.startswith('native-runtime/') else
                      staging / 'toolchain' / relative)
            assert source.is_file() and not source.is_symlink()
            observed = seal(source)
            assert observed['bytes'] == item['size_bytes'] and observed['sha256'] == item['sha256']
            rows.append({'kind': kind, 'entry': item, 'source': observed, 'links': source.stat().st_nlink})
    assert len(rows) == 362
    assert [(r['entry']['path'], r['links']) for r in rows if r['links'] != 1] == [
        ('bin/rust2vir', 2), ('bin/rust2vir-driver', 2)]
    files = [Path(__file__).resolve(), ROOT / 'rust-reproduction/result.json']
    before = [seal(path) for path in files]
    OUT.mkdir()
    COPIES.mkdir()
    write(OUT / 'inputs.json', {'selection_reason': 'The original full build succeeded and all 362 '
          'registered Rust file bytes match. Its two frontend build outputs have Cargo hardlinks, '
          'so retain those originals and copy their exact bytes to independent inodes as in the '
          'previous successful finalization before applying unchanged release validation.',
          'inputs': before, 'complete_original_files': rows})
    report = {'status': 'running', 'native_Linux_release_acceptance': False,
              'full_T_gate': 'Deferred to GO-PRAC-T02-W04: ./scripts/check-fast.sh'}
    started = time.monotonic()
    try:
        copies = []
        for row in rows:
            item = row['entry']
            source = Path(row['source']['path'])
            if row['links'] != 1:
                copy = COPIES / Path(item['path']).name
                shutil.copy2(source, copy)
                actual = seal(copy)
                assert copy.stat().st_nlink == 1
                assert all(actual[key] == row['source'][key] for key in ['bytes', 'sha256', 'mode'])
                copies.append(actual)
                source = copy
            destination = OUTPUT / ('frontend' if row['kind'] == 'frontend_bundles' else 'toolchain') / item['path']
            release.copy_described(source, destination, item)
        for name, bundle in [('frontend', candidate['frontend_bundles'][0]), ('toolchain', candidate['toolchain_bundles'][0])]:
            release.normalize_directories(OUTPUT / name)
            release.validate_root(OUTPUT / name, bundle)
        assert before == [seal(path) for path in files]
        assert all(seal(Path(row['source']['path'])) == row['source'] and Path(row['source']['path']).stat().st_nlink == row['links'] for row in rows)
        report.update(status='passed', source_inputs_unchanged=True, independent_frontend_copies=copies,
                      frontend=str(OUTPUT / 'frontend'), toolchain=str(OUTPUT / 'toolchain'),
                      registered_frontend=candidate['frontend_bundles'][0]['inventory'],
                      registered_toolchain=candidate['toolchain_bundles'][0]['inventory'])
    except BaseException as error:
        report.update(status='failed', error=repr(error), traceback=traceback.format_exc())
        raise
    finally:
        report['seconds'] = time.monotonic()-started
        write(OUT / 'result.json', report)
    print(json.dumps({'status': report['status'], 'seconds': report['seconds'],
                      'source_inputs_unchanged': report['source_inputs_unchanged']}), flush=True)


if __name__ == '__main__':
    main()
