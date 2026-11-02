"""Recheck saved prelaunch source archives and complete original input bytes."""
from pathlib import Path
import hashlib
import json
import sys
import tarfile
import traceback

ROOT = Path(__file__).resolve().parent
REPO = ROOT.parents[5]
sys.dont_write_bytecode = True
sys.path.insert(0, str(ROOT))
from run_scoped import seal, write
from review_partial import COMPLETED_DIRS


def main():
    result_path = ROOT / 'verified-partial-epochs-final.json'
    assert not result_path.exists()
    result = {'status': 'running'}
    write(result_path, result)
    try:
        archives = []
        checked_originals = {}
        for name in COMPLETED_DIRS:
            directory = ROOT / name
            inputs_path = directory / 'inputs.json'
            if not inputs_path.exists():
                continue
            inputs = json.loads(inputs_path.read_bytes())
            archive_row = inputs.get('source_archive') or inputs.get('archive')
            source_rows = inputs.get('sources') or (inputs.get('inputs') if archive_row else None)
            if archive_row and source_rows:
                archive_path = Path(archive_row['path'])
                assert seal(archive_path) == archive_row
                entries = {str(Path(row['path']).relative_to(REPO)): row for row in source_rows}
                assert len(entries) == len(source_rows)
                with tarfile.open(archive_path, 'r:gz') as archive:
                    members = archive.getmembers()
                    assert len(members) == len(entries) and {m.name for m in members} == set(entries)
                    for member in members:
                        data = archive.extractfile(member).read()
                        expected = entries[member.name]
                        assert member.isfile() and member.mode == expected['mode'] and len(data) == expected['bytes'] and hashlib.sha256(data).hexdigest() == expected['sha256']
                archives.append({'epoch': name, 'archive': archive_row, 'members': len(members)})
            for item in inputs.get('complete_originals', []):
                old = checked_originals.setdefault(item['path'], item)
                assert old == item
            for item in inputs.get('inputs', []):
                if 'raw_hex' in item:
                    data = bytes.fromhex(item['raw_hex'])
                    assert len(data) == item['bytes'] and hashlib.sha256(data).hexdigest() == item['sha256']
            result_file = directory / 'result.json'
            if result_file.exists():
                record = json.loads(result_file.read_bytes())
                if isinstance(record.get('inputs'), dict):
                    assert record['inputs'] == seal(inputs_path)
                for stream in ['stdout', 'stderr']:
                    item = record.get(stream)
                    if isinstance(item, dict):
                        assert item == seal(item['path'])
        for path, expected in checked_originals.items():
            assert seal(path) == expected
        source_receipt = json.loads((ROOT / 'original-go-source.json').read_bytes())
        base = Path('/private/tmp/mpk-installed-compiler-205-go-source')
        entries = {str(Path(item['path']).relative_to(base)): item for item in source_receipt['source_files']}
        with tarfile.open(ROOT / 'original-go-source.tar') as archive:
            members = [entry for entry in archive if entry.isfile()]
            assert {entry.name for entry in members} == set(entries)
            for entry in members:
                data = archive.extractfile(entry).read()
                item = entries[entry.name]
                # git archive recorded a group-write bit; the retained extracted
                # source has it cleared. Preserve and check both actual modes.
                assert entry.mode == (item['mode'] | 0o020) and len(data) == item['bytes'] and hashlib.sha256(data).hexdigest() == item['sha256']
        retained = json.loads((ROOT / 'linux-test-bound-final/retained-executable.json').read_bytes())
        actual = seal(retained['path'])
        assert actual['bytes'] == retained['bytes'] and actual['sha256'] == retained['sha256']
        result.update(status='passed', driver=seal(__file__), archives=archives, original_files=len(checked_originals), original_go_source_files=len(members), original_go_archive_mode_note='Original git archive modes 0664/0775; extracted receipts 0644/0755. All bytes and executable bits match.', retained_test_executable=actual, full_T_gate='Deferred to final T02-W04: ./scripts/check-fast.sh')
    except BaseException as error:
        result.update(status='failed', error=repr(error), traceback=traceback.format_exc())
        raise
    finally:
        write(result_path, result)
    print(json.dumps({k:result[k] for k in ['status','original_files','original_go_source_files']}))


if __name__ == '__main__':
    main()
