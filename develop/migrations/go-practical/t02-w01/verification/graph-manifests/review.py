"""Unchanged-rule scoped review of actual dependency-manifest source epochs and originals."""
from pathlib import Path
import base64, hashlib, json, re, subprocess, sys, tarfile, types, urllib.parse

REPO = Path.cwd()
ROOT = Path(__file__).resolve().parent
BASIS = '105b91f991d8d86ee25d2a181c332fb67b27b363'
CACHE = {}
# The evidence inspector must not shadow Python's standard inspect module when
# the unchanged scanner imports dataclasses.
sys.path = [value for value in sys.path if Path(value or '.').resolve() != ROOT]

def git(*args):
    return subprocess.check_output(['/usr/bin/git', *args])

def seal(path):
    path = Path(path)
    before = path.stat()
    key = (str(path), before.st_dev, before.st_ino, before.st_size,
           before.st_mtime_ns, before.st_ctime_ns, before.st_mode)
    if key not in CACHE:
        raw = path.read_bytes()
        after = path.stat()
        assert (before.st_dev, before.st_ino, before.st_size, before.st_mtime_ns,
                before.st_ctime_ns, before.st_mode) == (after.st_dev, after.st_ino,
                after.st_size, after.st_mtime_ns, after.st_ctime_ns, after.st_mode)
        CACHE[key] = {'path': str(path), 'bytes': len(raw),
                      'sha256': hashlib.sha256(raw).hexdigest(), 'mode': before.st_mode & 0o777}
    return CACHE[key]

def logical(path):
    path = Path(path)
    return str(path.relative_to(REPO)) if path.is_absolute() else str(path)

def main():
    assert git('rev-parse', 'HEAD').decode().strip() == BASIS
    output = ROOT / 'review.json'
    assert not output.exists()
    scanner = REPO / 'scripts/check-no-active-gir.sh'
    code = scanner.read_text().split("<<'PY'\n", 1)[1].rsplit('\nPY', 1)[0].rsplit('raise SystemExit(main())', 1)[0]
    module = types.ModuleType('original_graph_scoped_review')
    module.__file__ = str(scanner)
    sys.modules[module.__name__] = module
    sys.argv = [str(scanner), str(REPO), 'strict']
    exec(compile(code, str(scanner), 'exec'), module.__dict__)
    matchers = module.load_matchers()
    manifest, configured, fixtures, _ = module.validate_manifest()
    module.run_fixture_self_test(manifest, matchers)
    assert (len(matchers), len(module.SCOPES), len(manifest['sealed_records']),
            len(module.active_retained_record_exclusions(manifest))) == (127, 4, 39, 51)
    for name in configured | fixtures:
        assert (REPO / name).read_bytes() == git('show', BASIS + ':' + name)
    protected = {}
    prior_reviews = []
    for name in ['module-manifests/review.json', 'module-graphs/review.json']:
        previous_path = ROOT.parent / name
        assert previous_path.read_bytes() == git('show', BASIS + ':' + logical(previous_path))
        previous = json.loads(previous_path.read_bytes())
        assert previous['status'] == 'passed' and not previous['new_findings']
        protected.update({row['path']: row for row in previous['current_inputs']})
        prior_reviews.append(seal(previous_path))
    previous_path = ROOT.parent / 'module-graphs/review.json'
    old_receipt = ROOT.parent / 'module-manifests/rust-toolchain/inputs.json'
    assert old_receipt.read_bytes() == git('show', BASIS + ':' + logical(old_receipt))
    protected.update({row['path']: row for row in json.loads(old_receipt.read_bytes())['sources']})

    findings = []
    scan_cache = {}
    scanned = 0
    decoded = 0
    exact_rules = [rule for rule in matchers if rule['scope'] == 'exact-path']
    def scan(name, raw):
        nonlocal scanned
        key = (hashlib.sha256(raw).hexdigest(), tuple(module.path_matches(name, rule['path']) for rule in exact_rules))
        if key not in scan_cache:
            scan_cache[key] = [vars(row) for row in module.detect(name, raw.decode(errors='ignore'), matchers)]
        findings.extend({**row, 'path': name, 'excerpt_hex': row['excerpt'].encode().hex()} for row in scan_cache[key])
        scanned += 1

    def decode_originals(name, raw):
        nonlocal decoded
        if name.endswith('/complete-original-graph-manifests.json'):
            for row in json.loads(raw)['cases']:
                data = bytes.fromhex(row['raw_hex']);obj = row['object']
                assert len(data) == obj['bytes'] and hashlib.sha256(data).hexdigest() == obj['raw_sha256']
                scan('go-tools/go2vir/original-dependency-manifests/' + row['case'] + '/go.mod', data)
                decoded += 1
        if name.endswith('.json') and Path(name).name in ['complete-original-export-inputs.json', 'complete-original-module-observations.json', 'complete-original-module-graph-inputs.json']:
            packet = json.loads(raw)
            for field in ['export_inputs_hex', 'module_observations_hex', 'module_graph_inputs_hex']:
                if field in packet:
                    body = json.loads(bytes.fromhex(packet[field]))
                    for payload in ['metadata_hex', 'diagnostics_hex']:
                        if payload in body:
                            scan('go-tools/go2vir/original-dependency-native/' + payload + '.raw', bytes.fromhex(body[payload]))
                            decoded += 1

    archives = set()
    members = 0
    original_rechecks = 0
    def archive_inputs(receipt, archive_root, scan_prefix):
        nonlocal members, original_rechecks
        archive = Path(receipt['complete_archive']['path'])
        assert receipt['complete_archive'] == seal(archive)
        archives.add(archive)
        sources = {str(Path(row['path']).relative_to(archive_root)): row for row in receipt['sources']}
        preserved = set(receipt.get('preserved_original_paths', []))
        for name in preserved:
            row = sources[str(Path(name).relative_to(archive_root))]
            assert row == seal(Path(name))
            prior = protected.get(name)
            if prior is None:
                data = git('show', BASIS + ':' + logical(name))
                assert len(data) == row['bytes'] and hashlib.sha256(data).hexdigest() == row['sha256']
            else:
                assert row == prior
            original_rechecks += 1
        packed_names = set(sources) - {str(Path(name).relative_to(archive_root)) for name in preserved}
        with tarfile.open(archive, 'r:gz') as packed:
            entries = packed.getmembers()
            assert len(entries) == len(packed_names) and {entry.name for entry in entries} == packed_names
            for entry in entries:
                row = sources[entry.name]
                data = packed.extractfile(entry).read()
                assert entry.isfile() and entry.mode == row['mode']
                assert len(data) == row['bytes'] and hashlib.sha256(data).hexdigest() == row['sha256']
                name = scan_prefix + entry.name
                scan(name, data)
                decode_originals(name, data)
                members += 1

    # Reuse the complete unchanged native SDK archive already reviewed in194.
    # Every actual original SDK file is still compared with its prelaunch seal.
    sdk_path = ROOT.parent / 'module-graphs/original-sdk/inputs.json'
    sdk = json.loads(sdk_path.read_bytes())
    assert sdk_path.read_bytes() == git('show', BASIS + ':' + logical(sdk_path))
    for row in sdk['sources']:
        assert row == seal(Path(row['path']))
    sdk_archive = Path(sdk['complete_archive']['path'])
    assert sdk['complete_archive'] == seal(sdk_archive)
    prior_sdk = json.loads(previous_path.read_bytes())['complete_original_SDKs']
    assert any(row['inputs'] == seal(sdk_path) and row['files'] == len(sdk['sources']) for row in prior_sdk)
    sdks = [{'inputs': seal(sdk_path), 'files': len(sdk['sources']), 'bytes': sum(row['bytes'] for row in sdk['sources']), 'unchanged_archive': sdk['complete_archive'], 'review_reused_from': seal(previous_path)}]

    verification = json.loads((ROOT / 'verification.json').read_bytes())
    epochs = []
    outputs = []
    for epoch in verification['source_epochs']:
        path = ROOT / epoch / 'inputs.json'
        receipt = json.loads(path.read_bytes())
        result = json.loads(path.with_name('result.json').read_bytes())
        assert result['inputs'] == seal(path)
        assert result['status'] in ['passed', 'failed']
        if result['status'] == 'failed':
            assert epoch in verification['preserved_failed_epochs']
        assert result['consumed_inputs_unchanged'] or result['expected_formatter_mutation']
        assert result['whole_sdk_unchanged']
        assert receipt['whole_original_sdk'] == seal(Path(receipt['whole_original_sdk']['path']))
        archive_inputs(receipt, REPO, '')
        for field in ['stdout', 'stderr']:
            row = result[field]
            assert row == seal(Path(row['path']))
            outputs.append(row)
        epochs.append({'result': seal(path.with_name('result.json')), 'status': result['status'],
                       'command': result['command'], 'all_inputs': len(receipt['sources'])})

    inspection_path = ROOT / 'inspection.json'
    inspection = json.loads(inspection_path.read_bytes())
    assert inspection['status'] == 'passed' and inspection['affected_rust_roots'] == 11 and inspection['native_cases'] == 779
    assert inspection['actual_owner_packages'] == 181 and inspection['actual_owner_objects'] == 5667
    assert not inspection['independent_MVS_or_graph_transport_import'] and not inspection['proofs_replayed_in_this_checkpoint']
    for row in inspection['inputs']:
        assert row == seal(Path(row['path']))
    prelaunch = json.loads((ROOT / 'inspection-prelaunch.json').read_bytes())
    for row in prelaunch['inputs']:
        data = bytes.fromhex(row['raw_hex'])
        assert len(data) == row['bytes'] and hashlib.sha256(data).hexdigest() == row['sha256']
        assert Path(row['path']).read_bytes() == data

    names = set((git('diff', '--name-only', '-z', 'HEAD') + git('ls-files', '--others', '--exclude-standard', '-z')).decode().split('\0')) - {'', logical(output)}
    current = []
    links = []
    for name in sorted(names):
        path = REPO / name
        assert path.is_file() and not path.is_symlink()
        current.append(seal(path))
        if path not in archives:
            data = path.read_bytes()
            scan(name, data)
            decode_originals(name, data)
            if 'original-source-objects' in path.parts:
                scan('go-tools/go2vir/original-graph-objects/' + path.name, data)
        if path.suffix == '.md':
            for target in re.findall(r'\]\(([^)]+)\)', path.read_text()):
                target = target.strip('<>').split('#', 1)[0]
                if not target or re.match(r'^[a-zA-Z][a-zA-Z0-9+.-]*:', target):
                    continue
                resolved = path.parent / re.sub(r':\d+$', '', urllib.parse.unquote(target))
                links.append({'source': name, 'target': target, 'exists': resolved.exists()})
    assert not findings and all(row['exists'] for row in links)
    for path in ROOT.glob('*.py'):
        compile(path.read_bytes(), str(path), 'exec')
    result = {'status': 'passed', 'basis_commit': BASIS, 'driver': seal(Path(__file__).resolve()),
              'matcher_count': 127, 'scopes': 4, 'seals': 39, 'exclusions': 51, 'rules_unchanged': True,
              'previous_complete_original_review': seal(previous_path), 'complete_original_SDKs': sdks,
              'actual_source_epochs': epochs, 'complete_new_archive_members_verified_and_scanned': members,
              'logical_source_preimages_scanned': scanned, 'decoded_native_originals': decoded,
              'independent_inspection': seal(inspection_path), 'current_inputs': current,
              'new_findings': findings, 'links': links, 'raw_outputs_preserved': outputs,
              'manual_review': 'Independent byte-owned dependency manifest parsing preserves native lax syntax/dispositions, unknown and main-only ignored blocks, optional unchecked module paths, exact Go regexp/fallback/semantic version distinctions, ordered require/retract/ignore rows and original raw bytes. All779 native/Rust verdicts/full facts agree, including every earlier656 source/mode row and123 new cases. The exact eight original parser-library files match the complete unchanged pinned SDK. Strict defaults still pass their complete656-case comparison and original source/export/module consumers. Actual initial/final source-only graph requirements keep excludes, implicit go edges, ordering, duplicates, old/empty unselected manifests and the entire181-package/5667-object HTTP owner. Eleven affected Rust roots, the native oracle, strict Clippy/vet/formats and only the two strengthened graph/owner comparisons pass. Every source epoch/output and all immutable prior originals are retained without a dropped source/package/type/error/proof domain. Unchanged32 proof streams and43-root/zero-or70-error universes retain prior evidence without new replay claims. No scanner rule, fixture, seal, exclusion, premise, math rule, kernel cap or language scope changes. This parser component supplies no complete graph transport, MVS/root selection, transient native trace, compiler/root/context/registry/source/program/native Linux authority; remaining full foundation/T02-T11 obligations stay open.',
              'exact_prior_original_rechecks': original_rechecks, 'prior_complete_original_reviews': prior_reviews,
              'full_T_gate': 'Deferred to final T02-W04: ./scripts/check-fast.sh', 'whole_repository_gate_run': False}
    output.write_text(json.dumps(result, indent=2) + '\n')
    print(json.dumps({'status': 'passed', 'epochs': len(epochs), 'archive_members': members,
                      'current_files': len(current), 'new_findings': len(findings), 'links': len(links)}))

if __name__ == '__main__':
    main()
