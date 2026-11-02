"""Review retained test executable, recovery drivers and completed receipts."""
from pathlib import Path
import hashlib
import json
import sys
import tarfile
import types

ROOT = Path(__file__).resolve().parent
REPO = ROOT.parents[5]


def seal(path):
    path = Path(path)
    data = path.read_bytes()
    return {'path':str(path.resolve()), 'bytes':len(data), 'sha256':hashlib.sha256(data).hexdigest(), 'mode':path.stat().st_mode & 0o777}


def main():
    scanner = REPO/'scripts/check-no-active-gir.sh'
    code = scanner.read_text().split("<<'PY'\n", 1)[1].rsplit('\nPY', 1)[0].rsplit('raise SystemExit(main())', 1)[0]
    module = types.ModuleType('installed_go_additional_review')
    module.__file__ = str(scanner)
    sys.modules[module.__name__] = module
    sys.argv = [str(scanner), str(REPO), 'strict']
    exec(compile(code, str(scanner), 'exec'), module.__dict__)
    matchers = module.load_matchers()
    manifest, _, _, _ = module.validate_manifest()
    module.run_fixture_self_test(manifest, matchers)
    assert (len(matchers), len(module.SCOPES), len(manifest['sealed_records']), len(module.active_retained_record_exclusions(manifest))) == (127, 4, 39, 51)
    paths = {ROOT/name for name in ['review-partial.json', 'verify_partial_epochs.py', 'verified-partial-epochs.json', 'verified-partial-epochs-final.json', 'fetch_llvm_ranges.py', 'seed_rust_crate_cache.py', 'restore_rust_retry.py', 'review_additions.py']}
    for name in ['linux-test-bound-final/test-executable.elf', 'linux-test-bound-final/retained-executable.json']:
        paths.add(ROOT/name)
    for name in ['partial-epoch-check-failure', 'rust-case-sensitive-reproduction', 'rust-crate-cache-seed']:
        paths.update(p for p in (ROOT/name).rglob('*') if p.is_file())
    rows, findings, members = [], [], 0
    for path in sorted(paths):
        data = path.read_bytes()
        name = str(path.relative_to(REPO))
        rows.append(seal(path))
        if name.endswith('.tar.gz'):
            with tarfile.open(path) as archive:
                for member in archive:
                    if member.isfile():
                        raw = archive.extractfile(member).read()
                        assert len(raw) == member.size
                        findings.extend(vars(item) for item in module.detect(member.name, raw.decode(errors='ignore'), matchers))
                        members += 1
        else:
            findings.extend(vars(item) for item in module.detect(name, data.decode(errors='ignore'), matchers))
        if path.suffix == '.py':
            compile(data, name, 'exec')
    receipt = json.loads((ROOT/'rust-case-sensitive-reproduction/inputs.json').read_bytes())
    archive_row = receipt['archive']
    assert seal(archive_row['path']) == archive_row
    entries = {str(Path(row['path']).relative_to(REPO)):row for row in receipt['inputs']}
    with tarfile.open(archive_row['path']) as archive:
        files = archive.getmembers()
        assert len(files) == len(entries) and {f.name for f in files} == set(entries)
        for file in files:
            data = archive.extractfile(file).read()
            row = entries[file.name]
            assert file.isfile() and file.mode == row['mode'] and len(data) == row['bytes'] and hashlib.sha256(data).hexdigest() == row['sha256']
    assert json.loads((ROOT/'review-partial.json').read_bytes())['status'] == 'passed'
    assert json.loads((ROOT/'verified-partial-epochs-final.json').read_bytes())['status'] == 'passed'
    assert json.loads((ROOT/'rust-case-sensitive-reproduction/result.json').read_bytes())['status'] == 'failed'
    assert json.loads((ROOT/'rust-crate-cache-seed/result.json').read_bytes())['status'] == 'passed'
    result = {'status':'passed' if not findings else 'failed', 'driver':seal(__file__), 'matcher_count':len(matchers), 'scopes':len(module.SCOPES), 'seals':len(manifest['sealed_records']), 'exclusions':len(module.active_retained_record_exclusions(manifest)), 'inputs':rows, 'archive_members':members, 'findings':findings, 'installed_root_verified':False, 'whole_repository_gate_run':False, 'full_T_gate':'Deferred to final T02-W04: ./scripts/check-fast.sh'}
    (ROOT/'review-additions.json').write_text(json.dumps(result, indent=2)+'\n')
    print(json.dumps({'status':result['status'], 'inputs':len(rows), 'archive_members':members, 'findings':len(findings)}))
    raise SystemExit(bool(findings))


if __name__ == '__main__':
    main()
