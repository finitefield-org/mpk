"""Compare both complete retained frontend source trees without relabeling them."""
from pathlib import Path
import hashlib
import json
import tarfile

ROOT = Path(__file__).resolve().parent
PREFIX = 'go-tools/go2vir/'


def load(path):
    result = {}
    with tarfile.open(path) as archive:
        for item in archive:
            if item.isfile() and item.name.startswith(PREFIX):
                raw = archive.extractfile(item).read()
                assert item.name not in result and len(raw) == item.size
                result[item.name] = {'bytes': len(raw), 'sha256': hashlib.sha256(raw).hexdigest(), 'mode': item.mode}
    return result


def main():
    before = json.loads((ROOT / 'prelaunch/result.json').read_bytes())
    assert before['status'] == 'passed' and before['inputs_unchanged']
    archive = ROOT / 'prelaunch/inputs.tar.gz'
    assert hashlib.sha256(archive.read_bytes()).hexdigest() == before['archive']['sha256']
    original = ROOT.parent / 'installed-compiler/original-go-source.tar'
    original_row = next(row for row in before['inputs'] if row['path'] == str(original))
    assert hashlib.sha256(original.read_bytes()).hexdigest() == original_row['sha256']
    old, current = load(original), load(archive)
    rows = []
    for name in sorted(set(old) | set(current)):
        a, b = old.get(name), current.get(name)
        status = 'added' if a is None else 'removed' if b is None else 'equal' if a['sha256'] == b['sha256'] and a['bytes'] == b['bytes'] else 'changed'
        rows.append({'path': name, 'status': status, 'registered_source_epoch': a, 'current_prelaunch': b})
    assert all(row['status'] in ['added', 'removed', 'equal', 'changed'] for row in rows)
    counts = {status: sum(row['status'] == status for row in rows) for status in ['equal', 'changed', 'added', 'removed']}
    changed_go = [row['path'] for row in rows if row['status'] in ['changed', 'added', 'removed'] and row['path'].endswith('.go') and not row['path'].endswith('_test.go')]
    result = {'status': 'passed', 'scope': 'Compare all files in the retained registered frontend source epoch with all files in the complete current prelaunch frontend tree. Go production-file changes are a view of this full comparison; no compiler-selected closure or source-to-binary proof is inferred.', 'registered_files': len(old), 'current_files': len(current), 'counts': counts, 'changed_non_test_go_files': changed_go, 'files': rows, 'registered_frontend': before['registered_frontend'], 'registered_source_archive_sha256': original_row['sha256'], 'current_prelaunch_archive_sha256': before['archive']['sha256'], 'source_to_binary_correspondence_claimed': False, 'native_Linux_release_acceptance': False, 'full_T_gate': 'Deferred to final T02-W04: ./scripts/check-fast.sh'}
    out = ROOT / 'generation-diff.json'
    assert not out.exists()
    out.write_text(json.dumps(result, indent=2) + '\n')
    print(json.dumps({'status': result['status'], 'registered_files': len(old), 'current_files': len(current), 'counts': counts, 'changed_non_test_go_files': len(changed_go)}))


if __name__ == '__main__':
    main()
