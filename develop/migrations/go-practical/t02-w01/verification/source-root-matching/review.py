"""Review changed source, full retained input archives and original ZIP members."""
from pathlib import Path
import hashlib
import io
import json
import sys
import tarfile
import types
import zipfile

ROOT = Path(__file__).resolve().parent
REPO = ROOT.parents[5]


def seal(path):
    raw = path.read_bytes()
    return {'path': str(path.resolve()), 'bytes': len(raw), 'sha256': hashlib.sha256(raw).hexdigest(), 'mode': path.stat().st_mode & 0o777}


def main():
    scanner = REPO / 'scripts/check-no-active-gir.sh'
    code = scanner.read_text().split("<<'PY'\n", 1)[1].rsplit('\nPY', 1)[0].rsplit('raise SystemExit(main())', 1)[0]
    module = types.ModuleType('go_source_root_matching_review')
    module.__file__ = str(scanner)
    sys.modules[module.__name__] = module
    sys.argv = [str(scanner), str(REPO), 'strict']
    exec(compile(code, str(scanner), 'exec'), module.__dict__)
    matchers = module.load_matchers()
    manifest, _, _, _ = module.validate_manifest()
    module.run_fixture_self_test(manifest, matchers)
    assert (len(matchers), len(module.SCOPES), len(manifest['sealed_records']), len(module.active_retained_record_exclusions(manifest))) == (127, 4, 39, 51)
    paths = {p for p in ROOT.rglob('*') if p.is_file() and p.name != 'review.json'}
    paths.update(REPO / name for name in ['crates/mpk-vc/src/go_practical_type_keys/snapshot_inputs.rs', 'crates/mpk-cli/src/lib.rs', 'crates/mpk-cli/src/go_practical_source_roots.rs', 'crates/mpk-cli/tests/go_practical_source_roots.rs', 'develop/docs/09_go_practical_subset_design-todo.md', 'develop/docs/go-practical-implementation-traceability-ledger.md', 'develop/migrations/go-practical/t02-w01/README.md', 'develop/migrations/go-practical/t02-w01/requirements.json', 'develop/migrations/go-practical/t02-w01/verification.json'])
    native = ROOT.parent / 'analysis-namespace/native'
    packet = json.loads((native / 'native.json').read_bytes())
    originals = {row['object']['id']: row for row in packet['original_sources']}
    zip_paths = {native / 'original-bytes' / originals[source_id]['raw_file'] for source_id in packet['archive_source_ids']}
    paths.update(zip_paths)
    rows, findings, members, zip_members = [], [], 0, 0

    def scan(name, raw):
        nonlocal members
        findings.extend(vars(item) for item in module.detect(name, raw.decode(errors='ignore'), matchers))
        members += 1

    for path in sorted(paths):
        assert path.is_file()
        raw = path.read_bytes()
        relative = str(path.relative_to(REPO))
        rows.append(seal(path))
        if path.name.endswith('.tar.gz'):
            with tarfile.open(path) as archive:
                for entry in archive:
                    if entry.isfile():
                        data = archive.extractfile(entry).read()
                        assert len(data) == entry.size
                        scan(entry.name, data)
        elif path in zip_paths:
            with zipfile.ZipFile(io.BytesIO(raw)) as archive:
                for entry in archive.infolist():
                    data = archive.read(entry)
                    assert len(data) == entry.file_size
                    scan(entry.filename, data)
                    zip_members += 1
        else:
            findings.extend(vars(item) for item in module.detect(relative, raw.decode(errors='ignore'), matchers))
        if path.suffix == '.py':
            compile(raw, relative, 'exec')
        if path.suffix == '.json':
            json.loads(raw)
    assert zip_members == 1751
    for name in ['initial', 'final', 'expanded', 'snapshot-regression', 'clippy-linux-target', 'clippy-linux-target-expanded']:
        result = json.loads((ROOT / name / 'result.json').read_bytes())
        assert result['status'] == 'passed' and result['original_inputs_unchanged']
    inputs = json.loads((ROOT / 'expanded/inputs.json').read_bytes())['inputs']
    assert sum('/third_party/ruzstd-go125/' in row['path'] for row in inputs) == 65
    assert 'test result: ok. 2 passed;' in (ROOT / 'expanded/stdout.log').read_text()
    assert 'test result: ok. 3 passed;' in (ROOT / 'snapshot-regression/stdout.log').read_text()
    assert json.loads((ROOT / 'format.json').read_bytes())['status'] == 'passed'
    result = {'status': 'passed' if not findings else 'failed', 'selection_reason': 'Review the Linux held-root matcher, the independent snapshot view, all four retained complete input archives, original module ZIP members, affected tests and scope records. Existing scanner rules and exclusions are unchanged; the T gate remains at W04.', 'inputs': rows, 'archive_members': members, 'original_zip_members': zip_members, 'findings': findings, 'matcher_count': len(matchers), 'scopes': len(module.SCOPES), 'seals': len(manifest['sealed_records']), 'exclusions': len(module.active_retained_record_exclusions(manifest)), 'native_Linux_release_acceptance': False, 'full_T_gate': 'Deferred to final T02-W04: ./scripts/check-fast.sh'}
    (ROOT / 'review.json').write_text(json.dumps(result, indent=2) + '\n')
    print(json.dumps({'status': result['status'], 'inputs': len(rows), 'archive_members': members, 'original_zip_members': zip_members, 'findings': len(findings)}))
    raise SystemExit(bool(findings))


if __name__ == '__main__':
    main()
