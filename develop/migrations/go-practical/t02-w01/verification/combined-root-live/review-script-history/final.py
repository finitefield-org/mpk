"""Review both-root binding and every retained original archive member."""
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
    output = ROOT / (epoch + '.json')
    findings_path = ROOT / (epoch + '-archived-findings.json.hex')
    assert not output.exists() and not findings_path.exists()
    scanner = REPO / 'scripts/check-no-active-gir.sh'
    code = scanner.read_text().split("<<'PY'\n", 1)[1].rsplit('\nPY', 1)[0].rsplit('raise SystemExit(main())', 1)[0]
    module = types.ModuleType('go_combined_live_roots_review')
    module.__file__ = str(scanner)
    sys.modules[module.__name__] = module
    sys.argv = [str(scanner), str(REPO), 'strict']
    exec(compile(code, str(scanner), 'exec'), module.__dict__)
    matchers = module.load_matchers()
    manifest, _, _, _ = module.validate_manifest()
    module.run_fixture_self_test(manifest, matchers)
    assert (len(matchers), len(module.SCOPES), len(manifest['sealed_records']), len(module.active_retained_record_exclusions(manifest))) == (127, 4, 39, 51)
    paths = {p for p in ROOT.rglob('*') if p.is_file()}
    paths.update(REPO / name for name in [
        'go-tools/go2vir/practical_compiler_inputs_test.go',
        'go-tools/go2vir/practical_live_combined_roots_test.go',
        'crates/mpk-cli/src/go_practical_capture_binding.rs',
        'crates/mpk-cli/examples/go_practical_live_combined_roots.rs',
        'crates/mpk-cli/tests/go_practical_live_export_projection.rs',
        'develop/migrations/go-practical/t02-w01/README.md',
        'develop/migrations/go-practical/t02-w01/requirements.json',
        'develop/migrations/go-practical/t02-w01/verification.json',
    ])
    active, archived, rows = [], [], []
    counts = {'archive_members': 0, 'zip_members': 0, 'recovered_chunked_archives': 0, 'recovered_incident_objects': 0}

    def detect(name, raw, target):
        target.extend(vars(item) for item in module.detect(name, raw.decode(errors='ignore'), matchers))

    def zip_data(name, raw):
        if not raw.startswith((b'PK\x03\x04', b'PK\x05\x06')) or not zipfile.is_zipfile(io.BytesIO(raw)):
            return
        with zipfile.ZipFile(io.BytesIO(raw)) as archive:
            for member in archive.infolist():
                data = archive.read(member)
                assert len(data) == member.file_size
                detect(name + '!/' + member.filename, data, archived)
                counts['zip_members'] += 1

    def tar_data(name, raw):
        with tarfile.open(fileobj=io.BytesIO(raw), mode='r:*') as archive:
            for member in archive:
                if not member.isfile():
                    continue
                data = archive.extractfile(member).read()
                assert len(data) == member.size
                inside = name + '!/' + member.name
                detect(inside, data, archived)
                counts['archive_members'] += 1
                zip_data(inside, data)
                if member.name.endswith(('.tar', '.tar.gz')):
                    tar_data(inside, data)

    for path in sorted(paths):
        raw = path.read_bytes()
        name = str(path.relative_to(REPO))
        rows.append(seal(path))
        detect(name, raw, active)
        if path.name.endswith(('.tar', '.tar.gz')):
            tar_data(name, raw)
        else:
            zip_data(name, raw)
        if path.suffix == '.py':
            compile(raw, name, 'exec')
        if path.suffix == '.json':
            json.loads(raw)
    for epoch in ['live-volume', 'live-retirement']:
        storage = json.loads((ROOT / epoch / 'evidence-storage.json').read_bytes())
        blocks = []
        for chunk in storage['chunks']:
            path = ROOT / epoch / chunk['path']
            assert seal(path) == chunk['encoded']
            block = bytes.fromhex(path.read_text())
            assert len(block) == chunk['raw_bytes'] and hashlib.sha256(block).hexdigest() == chunk['raw_sha256']
            blocks.append(block)
        raw = b''.join(blocks)
        assert len(raw) == storage['archive']['bytes'] and hashlib.sha256(raw).hexdigest() == storage['archive']['sha256']
        tar_data(epoch + '/evidence-storage!decoded-archive', raw)
        counts['recovered_chunked_archives'] += 1
    incident = json.loads((ROOT / 'review-initial-storage.json').read_bytes())
    assert incident['status'] == 'passed' and incident['original_storage_paths_losslessly_encoded']
    for row in incident['originals']:
        encoded = (REPO / row['encoded_path']).read_bytes()
        assert len(encoded) == row['encoded_bytes'] and hashlib.sha256(encoded).hexdigest() == row['encoded_sha256']
        raw = bytes.fromhex(encoded.decode('ascii'))
        assert len(raw) == row['original_bytes'] and hashlib.sha256(raw).hexdigest() == row['original_sha256']
        name = row['original_path'] + '!decoded-original'
        detect(name, raw, archived)
        if row['original_path'].endswith(('.tar', '.tar.gz')):
            tar_data(name, raw)
        counts['recovered_incident_objects'] += 1
    encoded = (json.dumps(archived, indent=2) + '\n').encode()
    findings_path.write_bytes(encoded.hex().encode())
    assert bytes.fromhex(findings_path.read_text()) == encoded
    result = {'status': 'passed' if not active else 'failed', 'selection_reason': 'Review the both-root binding, live observer, all controls and records; reconstruct and scan every member of the complete losslessly stored live evidence using unchanged scanner rules.', 'inputs': rows, 'active_findings': active, 'archived_findings': {'count': len(archived), 'raw_bytes': len(encoded), 'raw_sha256': hashlib.sha256(encoded).hexdigest(), 'encoded': seal(findings_path)}, **counts, 'matcher_count': len(matchers), 'scopes': len(module.SCOPES), 'seals': len(manifest['sealed_records']), 'exclusions': len(module.active_retained_record_exclusions(manifest)), 'scanner_rules_changed': False, 'native_Linux_release_acceptance': False, 'full_T_gate': 'Deferred to final T02-W04: ./scripts/check-fast.sh'}
    output.write_text(json.dumps(result, indent=2) + '\n')
    print(json.dumps({'status': result['status'], 'files': len(rows), **counts, 'active_findings': len(active), 'archived_findings': len(archived)}), flush=True)
    raise SystemExit(bool(active))


if __name__ == '__main__':
    main()
