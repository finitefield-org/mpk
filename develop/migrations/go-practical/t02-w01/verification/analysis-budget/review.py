"""Review the complete original analysis ZIP/vendor evidence and changes."""
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
    data = path.read_bytes()
    return {'path': str(path.resolve()), 'bytes': len(data), 'sha256': hashlib.sha256(data).hexdigest(), 'mode': path.stat().st_mode & 0o777}


def main():
    scanner = REPO / 'scripts/check-no-active-gir.sh'
    code = scanner.read_text().split("<<'PY'\n", 1)[1].rsplit('\nPY', 1)[0].rsplit('raise SystemExit(main())', 1)[0]
    module = types.ModuleType('go_analysis_budget_review')
    module.__file__ = str(scanner)
    sys.modules[module.__name__] = module
    sys.argv = [str(scanner), str(REPO), 'strict']
    exec(compile(code, str(scanner), 'exec'), module.__dict__)
    matchers = module.load_matchers()
    manifest, _, _, _ = module.validate_manifest()
    module.run_fixture_self_test(manifest, matchers)
    assert (len(matchers), len(module.SCOPES), len(manifest['sealed_records']), len(module.active_retained_record_exclusions(manifest))) == (127, 4, 39, 51)
    paths = {p for p in ROOT.rglob('*') if p.is_file() and p.name != 'review.json'}
    paths.update(REPO / name for name in ['crates/mpk-vc/src/go_practical_type_keys.rs', 'crates/mpk-vc/src/go_practical_type_keys/analysis_vendor.rs', 'crates/mpk-vc/src/go_practical_type_keys/analysis_vendor/archives.rs', 'crates/mpk-vc/src/go_practical_type_keys/analysis_vendor/snapshot.rs', 'crates/mpk-vc/tests/go_practical_analysis_budget.rs', 'go-tools/go2vir/practical_analysis_budget_differential_test.go', 'develop/docs/09_go_practical_subset_design-todo.md', 'develop/docs/go-practical-implementation-traceability-ledger.md', 'develop/migrations/go-practical/t02-w01/README.md', 'develop/migrations/go-practical/t02-w01/requirements.json', 'develop/migrations/go-practical/t02-w01/verification.json'])
    packet = json.loads((ROOT.parent / 'analysis-namespace/native/native.json').read_bytes())
    originals = {row['object']['id']: row for row in packet['original_sources']}
    zip_paths = {ROOT.parent / 'analysis-namespace/native/original-bytes' / originals[source_id]['raw_file'] for source_id in packet['archive_source_ids']}
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
                    if not entry.isfile():
                        continue
                    payload = archive.extractfile(entry).read()
                    assert len(payload) == entry.size
                    if entry.name.endswith('/original-go-source.tar'):
                        with tarfile.open(fileobj=io.BytesIO(payload)) as nested:
                            for member in nested:
                                if member.isfile():
                                    original = nested.extractfile(member).read()
                                    assert len(original) == member.size
                                    scan(member.name, original)
                    else:
                        scan(entry.name, payload)
        elif path in zip_paths:
            with zipfile.ZipFile(io.BytesIO(raw)) as archive:
                for entry in archive.infolist():
                    payload = archive.read(entry)
                    assert len(payload) == entry.file_size
                    scan(entry.filename, payload)
                    zip_members += 1
        else:
            findings.extend(vars(item) for item in module.detect(relative, raw.decode(errors='ignore'), matchers))
        if path.suffix == '.py':
            compile(raw, relative, 'exec')
    assert zip_members == 1751
    inspection = json.loads((ROOT / 'inspection.json').read_bytes())
    assert inspection['status'] == 'passed'
    assert inspection['original_sources'] == 213 and inspection['archive_namespace_entries'] == 2403 and not inspection['original_root_authenticated']
    for name in ['prelaunch', 'native', 'budget-tests', 'budget-clippy', 'go-format-write', 'go-checks', 'rust-format']:
        assert json.loads((ROOT / name / 'result.json').read_bytes())['status'] == 'passed'
    for name in ['develop/migrations/go-practical/t02-w01/requirements.json', 'develop/migrations/go-practical/t02-w01/verification.json']:
        json.loads((REPO / name).read_bytes())
    result = {'status': 'passed' if not findings else 'failed', 'selection_reason': 'Review the changed Rust/Go shared namespace budget, every captured original and control snapshot, all original ZIP members, affected test evidence and updated scope records. Existing scanner rules and exclusions remain unchanged; no whole T gate runs.', 'inputs': rows, 'archive_members': members, 'original_zip_members': zip_members, 'findings': findings, 'matcher_count': len(matchers), 'scopes': len(module.SCOPES), 'seals': len(manifest['sealed_records']), 'exclusions': len(module.active_retained_record_exclusions(manifest)), 'native_Linux_release_acceptance': False, 'whole_repository_gate_run': False, 'full_T_gate': 'Deferred to final T02-W04: ./scripts/check-fast.sh'}
    (ROOT / 'review.json').write_text(json.dumps(result, indent=2) + '\n')
    print(json.dumps({'status': result['status'], 'inputs': len(rows), 'archive_members': members, 'original_zip_members': zip_members, 'findings': len(findings)}))
    raise SystemExit(bool(findings))


if __name__ == '__main__':
    main()
