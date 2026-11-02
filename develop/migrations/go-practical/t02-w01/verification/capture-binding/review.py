"""Review changed source and retained verification inputs without a whole gate."""
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
    digest = hashlib.sha256(); size = 0
    with path.open('rb') as incoming:
        for block in iter(lambda: incoming.read(1024 * 1024), b''):
            digest.update(block); size += len(block)
    return {'path': str(path.resolve()), 'bytes': size, 'sha256': digest.hexdigest(), 'mode': path.stat().st_mode & 0o777}


def main():
    scanner = REPO / 'scripts/check-no-active-gir.sh'
    code = scanner.read_text().split("<<'PY'\n", 1)[1].rsplit('\nPY', 1)[0].rsplit('raise SystemExit(main())', 1)[0]
    module = types.ModuleType('go_capture_binding_review'); module.__file__ = str(scanner); sys.modules[module.__name__] = module; sys.argv = [str(scanner), str(REPO), 'strict']; exec(compile(code, str(scanner), 'exec'), module.__dict__)
    matchers = module.load_matchers(); manifest, _, _, _ = module.validate_manifest(); module.run_fixture_self_test(manifest, matchers)
    assert (len(matchers), len(module.SCOPES), len(manifest['sealed_records']), len(module.active_retained_record_exclusions(manifest))) == (127, 4, 39, 51)
    paths = {p for p in ROOT.rglob('*') if p.is_file() and p.name not in ('review.json', 'archived-findings.json.hex')}
    paths.update(REPO / name for name in ['crates/mpk-cli/src/lib.rs', 'crates/mpk-cli/src/go_practical_capture_binding.rs', 'crates/mpk-cli/src/go_practical_source_roots.rs', 'crates/mpk-cli/tests/go_practical_capture_binding.rs', 'crates/mpk-vc/src/go_practical_type_keys/snapshot_inputs.rs', 'develop/docs/go-practical-implementation-traceability-ledger.md', 'develop/migrations/go-practical/t02-w01/README.md', 'develop/migrations/go-practical/t02-w01/requirements.json', 'develop/migrations/go-practical/t02-w01/verification.json'])
    active = []; archived = []; rows = []; archive_members = 0; zip_members = 0
    def detect(name, raw, destination):
        destination.extend(vars(item) for item in module.detect(name, raw.decode(errors='ignore'), matchers))
    def inspect_archive(path, raw):
        nonlocal archive_members, zip_members
        with tarfile.open(fileobj=io.BytesIO(raw), mode='r:gz') as archive:
            for item in archive:
                if not item.isfile(): continue
                data = archive.extractfile(item).read(); assert len(data) == item.size
                name = str(path.relative_to(REPO)) + '!/' + item.name
                detect(name, data, archived); archive_members += 1
                if data.startswith((b'PK\x03\x04', b'PK\x05\x06')) and zipfile.is_zipfile(io.BytesIO(data)):
                    with zipfile.ZipFile(io.BytesIO(data)) as nested:
                        for member in nested.infolist():
                            contents = nested.read(member); assert len(contents) == member.file_size
                            detect(name + '!/' + member.filename, contents, archived); zip_members += 1
    for path in sorted(paths):
        raw = path.read_bytes(); name = str(path.relative_to(REPO)); rows.append(seal(path)); detect(name, raw, active)
        if path.name.endswith('.tar.gz'): inspect_archive(path, raw)
        if path.suffix == '.py': compile(raw, name, 'exec')
        if path.suffix == '.json': json.loads(raw)
    prior = ROOT.parent / 'source-root-live/review.json'; reviewed = json.loads(prior.read_bytes()); assert reviewed['status'] == 'passed' and not reviewed['active_findings'] and reviewed['archived_findings']['count'] == 0
    prior_rows = {row['path']: row for row in reviewed['inputs']}
    chunks = sorted((ROOT.parent / 'source-root-live/live-original/evidence-storage').glob('*.hex')); assert len(chunks) == 24
    for chunk in chunks: assert seal(chunk) == prior_rows[str(chunk.resolve())]
    for name in ['linux-check', 'fixture-original', 'test-compile-original', 'roots-original', 'binding-original', 'binding-after-attribute', 'clippy-after-attribute']:
        assert json.loads((ROOT / name / 'result.json').read_bytes())['status'] == 'passed', name
    for name in ['format.json', 'format-after-clippy.json', 'root-fixture-audit.json']:
        assert json.loads((ROOT / name).read_bytes())['status'] == 'passed', name
    assert json.loads((ROOT / 'root-fixture-initial.json').read_bytes())['status'] == 'failed'
    assert json.loads((ROOT / 'clippy-original/result.json').read_bytes())['status'] == 'failed'
    encoded = ROOT / 'archived-findings.json.hex'; assert not encoded.exists()
    raw_findings = (json.dumps(archived, indent=2) + '\n').encode(); encoded.write_bytes(raw_findings.hex().encode()); assert bytes.fromhex(encoded.read_text()) == raw_findings
    result = {'status': 'passed' if not active else 'failed', 'selection_reason': 'Review all changed Rust files, targeted-test records and every member of retained input archives. Recheck all 24 original evidence chunks against the earlier complete 16880-file and ZIP review; keep the scanner rules and exclusions unchanged.', 'inputs': rows, 'active_findings': active, 'archived_findings': {'count': len(archived), 'raw_bytes': len(raw_findings), 'raw_sha256': hashlib.sha256(raw_findings).hexdigest(), 'encoded': seal(encoded)}, 'archive_members': archive_members, 'zip_members': zip_members, 'original_evidence_chunks_unchanged': len(chunks), 'earlier_review': seal(prior), 'matcher_count': len(matchers), 'scopes': len(module.SCOPES), 'seals': len(manifest['sealed_records']), 'exclusions': len(module.active_retained_record_exclusions(manifest)), 'native_Linux_release_acceptance': False, 'full_T_gate': 'Deferred to final T02-W04: ./scripts/check-fast.sh'}
    (ROOT / 'review.json').write_text(json.dumps(result, indent=2) + '\n')
    print(json.dumps({'status': result['status'], 'files': len(rows), 'archive_members': archive_members, 'zip_members': zip_members, 'active_findings': len(active), 'archived_findings': len(archived)}), flush=True)
    raise SystemExit(bool(active))


if __name__ == '__main__': main()
