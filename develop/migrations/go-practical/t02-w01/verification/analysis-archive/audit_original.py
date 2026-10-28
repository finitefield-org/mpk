"""Inspect every original analysis ZIP member and retained vendor file."""
from pathlib import Path
import base64
import hashlib
import json
import zipfile

ROOT = Path(__file__).resolve().parent
REPO = ROOT.parents[5]
PACKAGE = REPO / 'go-tools/go2vir'
PINS = [('golang.org/x/mod', 'v0.27.0', 'h1:kb+q2PyFnEADO2IEF935ehFUXlWiNjJWtRNgBLSfbxQ='), ('golang.org/x/sync', 'v0.16.0', 'h1:ycBJEhp9p4vXvUZNszeOq0kGTPghopOL8q0fq3vstxw='), ('golang.org/x/tools', 'v0.36.0', 'h1:kWS0uv/zsvHEle1LbV5LE8QujrxB3wfQyxHfhOk0Qkg=')]


def seal(path):
    data = path.read_bytes()
    return {'path': str(path.resolve()), 'bytes': len(data), 'sha256': hashlib.sha256(data).hexdigest(), 'mode': path.stat().st_mode & 0o777}


def main():
    prelaunch = json.loads((ROOT / 'prelaunch/result.json').read_bytes())
    assert prelaunch['status'] == 'passed' and prelaunch['inputs_unchanged']
    root = PACKAGE / 'vendor'
    source_files = [p for p in sorted(root.rglob('*')) if p.is_file() and p != root / 'modules.txt']
    paths = {Path(__file__).resolve(), ROOT / 'prelaunch/result.json', PACKAGE / 'go.mod', PACKAGE / 'go.sum', root / 'modules.txt', *source_files}
    paths.update(PACKAGE / 'testdata/practical-analysis-cache' / name / '@v' / f'{version}{suffix}' for name, version, _ in PINS for suffix in ['.mod', '.info', '.zip', '.ziphash'])
    before = [seal(path) for path in sorted(paths)]
    sums = set((PACKAGE / 'go.sum').read_text().splitlines())
    members, archives = {}, []
    for name, version, expected in PINS:
        folder = PACKAGE / 'testdata/practical-analysis-cache' / name / '@v'
        archive = folder / f'{version}.zip'
        assert f'{name} {version} {expected}' in sums and (folder / f'{version}.ziphash').read_text().strip() == expected
        with zipfile.ZipFile(archive) as zipped:
            entries = zipped.infolist()
            assert len({entry.filename for entry in entries}) == len(entries)
            hashes = []
            for entry in entries:
                data = zipped.read(entry)
                assert len(data) == entry.file_size and entry.filename.startswith(f'{name}@{version}/')
                digest = hashlib.sha256(data).hexdigest()
                hashes.append((entry.filename, digest))
                relative = entry.filename.removeprefix(f'{name}@{version}/')
                if relative and not entry.is_dir():
                    members[(name, relative)] = {'bytes': len(data), 'sha256': digest}
            canonical = ''.join(f'{digest}  {path}\n' for path, digest in sorted(hashes)).encode()
            actual = 'h1:' + base64.b64encode(hashlib.sha256(canonical).digest()).decode()
            assert actual == expected
            archives.append({'module': name, 'version': version, 'checksum': actual, 'zip': seal(archive), 'entries': len(entries)})
    packages = {line for line in (root / 'modules.txt').read_text().splitlines() if line and not line.startswith('#')}
    seen, rows, differences = set(), [], []
    for file in source_files:
        logical = file.relative_to(root).as_posix()
        name = next((name for name, _, _ in PINS if logical.startswith(name + '/')), None)
        assert name is not None
        relative = logical.removeprefix(name + '/')
        member = members[(name, relative)]
        actual = seal(file)
        equal = (actual['bytes'], actual['sha256']) == (member['bytes'], member['sha256'])
        if file.suffix == '.go':
            assert not file.name.endswith('_test.go') and file.parent.relative_to(root).as_posix() in packages
            seen.add(file.parent.relative_to(root).as_posix())
        else:
            assert file.name in ['LICENSE', 'PATENTS', 'TODO']
        if not equal:
            assert file.name == 'TODO'
            differences.append({'path': logical, 'actual': actual, 'archive_member': member})
        rows.append({'path': logical, 'actual': actual, 'archive_member': member, 'equal': equal})
    assert seen == packages and len(packages) == 29 and len(differences) == 1
    unchanged = before == [seal(path) for path in sorted(paths)]
    assert unchanged
    result = {'status': 'passed', 'selection_reason': 'Independently read every actual pinned module ZIP member and every original vendor file before implementing any source/archive binding; retain the full original manifest and explicit uncompiled TODO difference.', 'driver': seal(Path(__file__).resolve()), 'inputs': before, 'archives': archives, 'vendor_files': rows, 'vendor_packages': len(packages), 'go_files': sum(row['path'].endswith('.go') for row in rows), 'other_files': sum(not row['path'].endswith('.go') for row in rows), 'differences': differences, 'inputs_unchanged': unchanged, 'source_to_binary_correspondence_claimed': False, 'registered_context_claimed': False, 'native_Linux_release_acceptance': False, 'full_T_gate': 'Deferred to final T02-W04: ./scripts/check-fast.sh'}
    output = ROOT / 'original-audit.json'
    assert not output.exists()
    output.write_text(json.dumps(result, indent=2) + '\n')
    print(json.dumps({'status': result['status'], 'archives': len(archives), 'archive_members': sum(row['entries'] for row in archives), 'vendor_files': len(rows), 'go_files': result['go_files'], 'other_files': result['other_files'], 'vendor_packages': len(packages), 'differences': len(differences)}))


if __name__ == '__main__':
    main()
