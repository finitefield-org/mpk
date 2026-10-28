"""Review changed code and exact retained input archives with existing rules."""
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
    epoch = sys.argv[1]
    assert epoch.replace('-', '').isalnum()
    output = ROOT/(epoch+'.json')
    encoded_findings = ROOT/(epoch+'-archived-findings.json.hex')
    assert not output.exists() and not encoded_findings.exists()
    scanner = REPO/'scripts/check-no-active-gir.sh'
    code = scanner.read_text().split("<<'PY'\n", 1)[1].rsplit('\nPY', 1)[0].rsplit('raise SystemExit(main())', 1)[0]
    module = types.ModuleType('go_analysis_pgo_review')
    module.__file__ = str(scanner)
    sys.modules[module.__name__] = module
    sys.argv = [str(scanner), str(REPO), 'strict']
    exec(compile(code, str(scanner), 'exec'), module.__dict__)
    matchers = module.load_matchers()
    manifest, _, _, _ = module.validate_manifest()
    module.run_fixture_self_test(manifest, matchers)
    assert (len(matchers), len(module.SCOPES), len(manifest['sealed_records']), len(module.active_retained_record_exclusions(manifest))) == (127, 4, 39, 51)
    paths = {p for p in ROOT.rglob('*') if p.is_file()}
    paths.update(REPO/name for name in [
        'go-tools/go2vir/practical_analysis.go', 'go-tools/go2vir/practical_analysis_pgo_test.go',
        'crates/mpk-vc/src/go_practical_type_keys.rs', 'crates/mpk-vc/src/go_practical_type_keys/analysis_vendor.rs',
        'crates/mpk-vc/src/go_practical_type_keys/analysis_vendor/source.rs', 'crates/mpk-vc/tests/go_practical_analysis_pgo.rs',
        'crates/mpk-cli/tests/go_practical_analysis_root_pgo.rs', 'develop/docs/go-practical-implementation-traceability-ledger.md',
        'develop/migrations/go-practical/t02-w01/README.md', 'develop/migrations/go-practical/t02-w01/requirements.json',
        'develop/migrations/go-practical/t02-w01/verification.json'])
    active, archived, rows = [], [], []
    counts = {'archive_members': 0, 'zip_members': 0, 'decoded_binaries': 0, 'recovered_archives': 0}
    storage_path = ROOT/'archive-storage.json'
    storage = json.loads(storage_path.read_bytes()) if storage_path.exists() else {'archives': []}
    encoded_archives = {str(Path(row['encoded']['path']).resolve()): row for row in storage['archives']}

    def detect(name, raw, target):
        target.extend(vars(item) for item in module.detect(name, raw.decode(errors='ignore'), matchers))

    def binary(name, raw):
        data = bytes.fromhex(raw.decode('ascii'))
        detect(name+'!decoded-binary', data, archived)
        counts['decoded_binaries'] += 1

    def zip_data(name, raw):
        if not raw.startswith((b'PK\x03\x04', b'PK\x05\x06')) or not zipfile.is_zipfile(io.BytesIO(raw)):
            return
        with zipfile.ZipFile(io.BytesIO(raw)) as z:
            for member in z.infolist():
                data = z.read(member)
                assert len(data) == member.file_size
                detect(name+'!/'+member.filename, data, archived)
                counts['zip_members'] += 1

    def tar_data(name, raw):
        with tarfile.open(fileobj=io.BytesIO(raw), mode='r:*') as archive:
            for member in archive:
                if not member.isfile():
                    continue
                data = archive.extractfile(member).read()
                assert len(data) == member.size
                inside = name+'!/'+member.name
                detect(inside, data, archived)
                counts['archive_members'] += 1
                zip_data(inside, data)
                if member.name.endswith(('.tar', '.tar.gz')):
                    tar_data(inside, data)
                elif member.name.endswith('.elf.hex'):
                    binary(inside, data)

    for path in sorted(paths):
        raw = path.read_bytes()
        name = str(path.relative_to(REPO))
        rows.append(seal(path))
        detect(name, raw, active)
        if path.name.endswith(('.tar', '.tar.gz')):
            tar_data(name, raw)
        elif path.name.endswith('.tar.gz.hex'):
            item = encoded_archives[str(path.resolve())]
            data = bytes.fromhex(raw.decode('ascii'))
            assert len(data) == item['original']['bytes'] and hashlib.sha256(data).hexdigest() == item['original']['sha256']
            tar_data(name+'!decoded-archive', data)
            counts['recovered_archives'] += 1
        elif path.name.endswith('.elf.hex'):
            binary(name, raw)
        else:
            zip_data(name, raw)
        if path.suffix == '.py':
            compile(raw, name, 'exec')
        if path.suffix == '.json':
            json.loads(raw)
    findings = (json.dumps(archived, indent=2)+'\n').encode()
    encoded_findings.write_bytes(findings.hex().encode())
    assert bytes.fromhex(encoded_findings.read_text()) == findings
    result = {'status': 'passed' if not active else 'failed', 'selection_reason': 'Review each changed producer/importer/test and retained original evidence; scan every input archive and nested ZIP and each complete decoded binary using the unchanged scanner.', 'inputs': rows, 'active_findings': active, 'archived_findings': {'count': len(archived), 'raw_bytes': len(findings), 'raw_sha256': hashlib.sha256(findings).hexdigest(), 'encoded': seal(encoded_findings)}, **counts, 'matcher_count': len(matchers), 'scopes': len(module.SCOPES), 'seals': len(manifest['sealed_records']), 'exclusions': len(module.active_retained_record_exclusions(manifest)), 'scanner_rules_changed': False, 'native_Linux_release_acceptance': False, 'full_T_gate': 'Deferred to final T02-W04: ./scripts/check-fast.sh'}
    output.write_text(json.dumps(result, indent=2)+'\n')
    print(json.dumps({'status': result['status'], 'files': len(rows), **counts, 'active_findings': len(active), 'archived_findings': len(archived)}), flush=True)
    raise SystemExit(bool(active))


if __name__ == '__main__':
    main()
