"""Review the changed analysis vendor reader and complete local evidence."""
from pathlib import Path
import hashlib
import io
import json
import sys
import tarfile
import types

ROOT = Path(__file__).resolve().parent
REPO = ROOT.parents[5]


def seal(path):
    data = path.read_bytes()
    return {'path': str(path.resolve()), 'bytes': len(data), 'sha256': hashlib.sha256(data).hexdigest(), 'mode': path.stat().st_mode & 0o777}


def main():
    # The evidence directory contains inspect.py; avoid shadowing Python's
    # standard-library module when the existing scanner loads dataclasses.
    sys.path = [entry for entry in sys.path if Path(entry or '.').resolve() != ROOT]
    scanner = REPO / 'scripts/check-no-active-gir.sh'
    code = scanner.read_text().split("<<'PY'\n", 1)[1].rsplit('\nPY', 1)[0].rsplit('raise SystemExit(main())', 1)[0]
    module = types.ModuleType('go_analysis_vendor_review')
    module.__file__ = str(scanner)
    sys.modules[module.__name__] = module
    sys.argv = [str(scanner), str(REPO), 'strict']
    exec(compile(code, str(scanner), 'exec'), module.__dict__)
    matchers = module.load_matchers()
    manifest, _, _, _ = module.validate_manifest()
    module.run_fixture_self_test(manifest, matchers)
    assert (len(matchers), len(module.SCOPES), len(manifest['sealed_records']), len(module.active_retained_record_exclusions(manifest))) == (127, 4, 39, 51)
    paths = {p for p in ROOT.rglob('*') if p.is_file() and p.name != 'review.json'}
    paths.update(REPO / name for name in ['crates/mpk-vc/src/go_practical_type_keys.rs', 'crates/mpk-vc/src/go_practical_type_keys/analysis_vendor.rs', 'crates/mpk-vc/tests/go_practical_analysis_vendor.rs', 'go-tools/go2vir/practical_analysis_vendor_differential_test.go', 'develop/docs/09_go_practical_subset_design-todo.md', 'develop/docs/go-practical-implementation-traceability-ledger.md', 'develop/migrations/go-practical/t02-w01/README.md', 'develop/migrations/go-practical/t02-w01/requirements.json', 'develop/migrations/go-practical/t02-w01/verification.json'])
    rows, findings, members = [], [], 0

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
        else:
            findings.extend(vars(item) for item in module.detect(relative, raw.decode(errors='ignore'), matchers))
        if path.suffix == '.py':
            compile(raw, relative, 'exec')
    inspection = json.loads((ROOT / 'inspection.json').read_bytes())
    assert inspection['status'] == 'passed' and inspection['cases'] == 36 and inspection['original_packages'] == 29
    assert inspection['maximum_packages_accepted'] == 32768 and inspection['one_over_maximum_rejected']
    assert json.loads((ROOT / 'review-failed/failure-observation.json').read_bytes())['status'] == 'failed'
    for name in ['prelaunch', 'native', 'vendor-selected', 'vendor-clippy-check', 'go-format-write', 'go-checks', 'rust-format-check']:
        assert json.loads((ROOT / name / 'result.json').read_bytes())['status'] == 'passed'
    for name in ['develop/migrations/go-practical/t02-w01/requirements.json', 'develop/migrations/go-practical/t02-w01/verification.json']:
        json.loads((REPO / name).read_bytes())
    result = {'status': 'passed' if not findings else 'failed', 'selection_reason': 'Review the new Rust/Go vendor-manifest boundary, the complete original and adversarial input bytes, targeted source/type-key tests, lint and updated scope records. Existing scanner rules and exclusions remain unchanged; no whole T gate runs.', 'inputs': rows, 'archive_members': members, 'findings': findings, 'matcher_count': len(matchers), 'scopes': len(module.SCOPES), 'seals': len(manifest['sealed_records']), 'exclusions': len(module.active_retained_record_exclusions(manifest)), 'native_Linux_release_acceptance': False, 'whole_repository_gate_run': False, 'full_T_gate': 'Deferred to final T02-W04: ./scripts/check-fast.sh'}
    (ROOT / 'review.json').write_text(json.dumps(result, indent=2) + '\n')
    print(json.dumps({'status': result['status'], 'inputs': len(rows), 'archive_members': members, 'findings': len(findings)}))
    raise SystemExit(bool(findings))


if __name__ == '__main__':
    main()
