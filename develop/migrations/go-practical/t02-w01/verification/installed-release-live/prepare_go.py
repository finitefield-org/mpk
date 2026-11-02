"""Copy the complete original Go bundles into registered release modes."""
from pathlib import Path
import json
import sys
import time
import traceback

ROOT = Path(__file__).resolve().parent
REPO = ROOT.parents[5]
SOURCE = Path('/private/tmp/mpk-installed-compiler-205-original-go-output')
DESTINATION = Path('/private/tmp/mpk-go-combined-recovery-20261029-fs/registered-go-prepared')
OUT = ROOT / 'go-prepared'
sys.dont_write_bytecode = True
sys.path.insert(0, str(ROOT.parent / 'installed-compiler'))
from run_scoped import seal, write
sys.path.insert(0, str(REPO / 'scripts'))
import successor_release_bundles as release


def main():
    assert Path.cwd() == REPO and not DESTINATION.exists() and not OUT.exists()
    failed = json.loads((ROOT / 'installed-stage/result.json').read_bytes())
    assert failed['status'] == 'failed' and failed['error'] == "SuccessorReleaseFailure('BUNDLE_REPRODUCIBILITY_MISMATCH')"
    _, candidates = release.validate_release_models()
    candidate = candidates['go']
    rows, directories = [], []
    for name, field in [('frontend', 'frontend_bundles'), ('toolchain', 'toolchain_bundles')]:
        root = SOURCE / name
        bundle = candidate[field][0]
        expected = {item['path']: item for item in bundle['inventory']['files']}
        paths = list(root.rglob('*'))
        assert not any(path.is_symlink() for path in paths)
        assert {path.relative_to(root).as_posix() for path in paths if path.is_file()} == set(expected)
        directories.extend({'path': str(path), 'mode': path.stat().st_mode & 0o777}
                           for path in [root] + [path for path in paths if path.is_dir()])
        for relative, item in expected.items():
            source = root / relative
            actual = seal(source)
            assert source.stat().st_nlink == 1 and actual['bytes'] == item['size_bytes']
            assert actual['sha256'] == item['sha256'] and bool(actual['mode'] & 0o111) == item['executable']
            rows.append({'bundle': name, 'entry': item, 'source': actual})
    assert len(rows) == 5578
    files = [Path(__file__).resolve(), ROOT / 'installed-stage/result.json']
    before = [seal(path) for path in files]
    OUT.mkdir()
    write(OUT / 'inputs.json', {'selection_reason': 'The original Go frontend directory has writable '
          'directory modes. Keep it untouched and copy every original Go frontend and SDK file into '
          'a separate registered release layout with fixed modes, validating every inventory entry.',
          'inputs': before, 'original_directories': directories, 'complete_original_files': rows})
    report = {'status': 'running', 'native_Linux_release_acceptance': False,
              'full_T_gate': 'Deferred to GO-PRAC-T02-W04: ./scripts/check-fast.sh'}
    started = time.monotonic()
    try:
        for row in rows:
            item = row['entry']
            release.copy_described(Path(row['source']['path']), DESTINATION / row['bundle'] / item['path'], item)
        for name, field in [('frontend', 'frontend_bundles'), ('toolchain', 'toolchain_bundles')]:
            release.normalize_directories(DESTINATION / name)
            release.validate_root(DESTINATION / name, candidate[field][0])
        assert before == [seal(path) for path in files]
        assert all(seal(Path(row['source']['path'])) == row['source'] for row in rows)
        assert all(Path(row['path']).stat().st_mode & 0o777 == row['mode'] for row in directories)
        report.update(status='passed', original_inputs_unchanged=True, root=str(DESTINATION),
                      original_files=len(rows), original_directories=len(directories))
    except BaseException as error:
        report.update(status='failed', error=repr(error), traceback=traceback.format_exc())
        raise
    finally:
        report['seconds'] = time.monotonic()-started
        write(OUT / 'result.json', report)
    print(json.dumps({'status': report['status'], 'original_files': report['original_files'],
                      'original_inputs_unchanged': report['original_inputs_unchanged']}), flush=True)


if __name__ == '__main__':
    main()
