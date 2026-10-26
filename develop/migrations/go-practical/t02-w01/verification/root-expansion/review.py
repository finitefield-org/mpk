"""Unchanged-rule scoped review of actual dependency-manifest source epochs and originals."""
from pathlib import Path
import base64, hashlib, json, re, subprocess, sys, tarfile, types, urllib.parse

REPO = Path.cwd()
ROOT = Path(__file__).resolve().parent
BASIS = 'a65e6c529cf03b3efc4d9beb92a50084690f7618'
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
    failed_path = ROOT / 'review-initial-failure.json'
    failed = json.loads(failed_path.read_bytes())
    assert failed['status'] == 'failed' and failed['exit_code'] == 1
    assert 'NoneType' in failed['complete_stderr'] and not failed['runtime_code_or_test_results_changed']
    original_prelaunch = json.loads((ROOT / failed['prelaunch']).read_bytes())
    for row in original_prelaunch['inputs']:
        data=bytes.fromhex(row['raw_hex'])
        assert len(data)==row['bytes'] and hashlib.sha256(data).hexdigest()==row['sha256']
    for name in configured | fixtures:
        assert (REPO / name).read_bytes() == git('show', BASIS + ':' + name)
    protected = {}
    prior_reviews = []
    for name in ['module-manifests/review.json', 'module-graphs/review.json', 'graph-manifests/review.json', 'graph-inputs/review.json', 'graph-import/review.json']:
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
        if not name.endswith('.json'):
            return
        basename = Path(name).name
        if basename == 'coherent-original-unselected-manifest-control.json':
            value = json.loads(raw)
            data = bytes.fromhex(value['complete_changed_raw_hex']); obj = value['changed_object']
            assert len(data) == obj['bytes'] and hashlib.sha256(data).hexdigest() == obj['raw_sha256']
            scan('go-tools/go2vir/original-graph-content-controls/' + obj['path'], data); decoded += 1
            for field in ['changed_snapshot_hex','changed_capture_hex','changed_export_hex','changed_module_hex']:
                packet = json.loads(bytes.fromhex(value[field]))
                for payload in ['metadata_hex','diagnostics_hex']:
                    if payload in packet:
                        scan('go-tools/go2vir/original-graph-content-controls/' + payload + '.raw', bytes.fromhex(packet[payload])); decoded += 1
            return
        if basename in ['coherent-original-graph-controls.json','coherent-final-root-controls.json']:
            for row in json.loads(raw)['cases']:
                for payload in ['metadata_hex','diagnostics_hex']:
                    if payload in row['complete_changed_graph']:
                        scan('go-tools/go2vir/original-graph-claim-controls/' + payload + '.raw', bytes.fromhex(row['complete_changed_graph'][payload])); decoded += 1
            return
        if basename == 'complete-original-root-expansion.json':
            for case in json.loads(raw)['cases']:
                for row in [case['main'], *case['sources']]:
                    data=bytes.fromhex(row['raw_hex']); obj=row['object']
                    assert len(data)==obj['bytes'] and hashlib.sha256(data).hexdigest()==obj['raw_sha256']
                    scan('go-tools/go2vir/original-expansion-manifests/'+obj['path'],data);decoded+=1
                for field in ['metadata_hex','diagnostics_hex']:
                    scan('go-tools/go2vir/original-expansion-native/'+field+'.raw',bytes.fromhex(case[field]));decoded+=1
            return
        if not any(token in basename for token in ['graph-', 'complete-original-module-', 'complete-original-export-', 'expansion', 'ordering']):
            return
        value = json.loads(raw)
        for field in ['export_inputs_hex', 'module_observations_hex', 'module_graph_inputs_hex']:
            if field in value:
                packet = json.loads(bytes.fromhex(value[field]))
                for payload in ['metadata_hex', 'diagnostics_hex']:
                    if payload in packet:
                        scan('go-tools/go2vir/original-graph-native/' + payload + '.raw', bytes.fromhex(packet[payload]))
                        decoded += 1
        for field in ['stdout', 'stderr', 'module_stdout', 'module_stderr', 'whole_candidate_output', 'whole_output', 'candidate_output', 'candidate_diagnostics']:
            if isinstance(value.get(field), str):
                scan('go-tools/go2vir/original-graph-native/' + field + '.raw', base64.b64decode(value[field], validate=True))
                decoded += 1
        for phase in ['initial', 'final']:
            for row in value.get(phase, []):
                if row['source'] is not None:
                    source = row['source']
                    data = base64.b64decode(source['raw'], validate=True)
                    obj = source['object']
                    assert len(data) == obj['bytes'] and hashlib.sha256(data).hexdigest() == obj['raw_sha256']
                    scan('go-tools/go2vir/original-graph-manifests/' + obj['path'], data)
                    decoded += 1
        if 'objects' in value and 'raw' in value:
            for obj in value['objects']:
                encoded = value['raw'][obj['id']]
                if encoded is None:
                    assert obj['bytes'] == 0 and obj['raw_sha256'] == hashlib.sha256(b'').hexdigest()
                    data = b''
                else:
                    data = base64.b64decode(encoded, validate=True)
                assert len(data) == obj['bytes'] and hashlib.sha256(data).hexdigest() == obj['raw_sha256']
                scan('go-tools/go2vir/original-graph-captured/' + obj['path'], data)
                decoded += 1
        for row in value.get('whole_original_entries', []):
            if not row.get('directory'):
                data = base64.b64decode(row['raw'], validate=True)
                assert hashlib.sha256(data).hexdigest() == row['sha256']
                scan('go-tools/go2vir/original-graph-generated/' + row['path'], data)
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
    assert inspection['status'] == 'passed' and inspection['affected_rust_roots'] == 8 and inspection['coherent_graph_claim_controls'] == 22
    assert len(inspection['whole_original_owners']) == 2 and inspection['whole_original_owners'][1]['objects'] == 5697
    assert not inspection['algorithm_correctness_proved'] and not inspection['installed_context_authority'] and not inspection['proofs_replayed_in_this_checkpoint']
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
              'manual_review': 'Independent private graph import derives pruned final roots from original main requirements rather than native main outgoing edges. Go1.25 bootstrap redundant/main-path normalization precedes source reads; discarded lower .mod files need not exist. Identity/pruning-state loading, original replacements/go.sum/lax facts and serial selected-version computation feed iterative root upgrades. Intermediate original manifests are captured before native execution and retained independently even when absent from both endpoint inventories. Pinned Go unstable sorting preserves equal-label permutations independently from inventory raw-byte tiebreaks. All18 native expansion cases,225 unbounded real-version comparisons and84 exact sort permutations agree with current Rust; a missing transient manifest rejects. Eight targeted Rust roots, all seven affected native graph roots and the full182-package/5697-object versioned owner pass. Existing full local/versioned owners and a fresh producer/consumer roundtrip retain unused HTTP/source/init and all original source/package/proof/error scopes. Twenty-two coherent graph claim controls and the original coherent source-content checksum control reject. Initial duplicate-consistent native failure and later new-test formatting failure retain full input/output epochs; no original case is dropped or reclassified. Current production/integration Clippy, vet and changed formats pass. Unchanged32 proof streams retain prior evidence without new proof replay. No review matcher/seal/exclusion, source/type/error/proof universe, premise, math rule or kernel cap changes. General algorithm/source correctness, native-read/transport-resource, compiler/root/context/registry/source/program/foundation/native Linux/T02-T11 authority remain due; W01 remains open and profile inactive.',

              'preserved_failed_review': seal(failed_path), 'exact_prior_original_rechecks': original_rechecks, 'prior_complete_original_reviews': prior_reviews,
              'full_T_gate': 'Deferred to final T02-W04: ./scripts/check-fast.sh', 'whole_repository_gate_run': False}
    output.write_text(json.dumps(result, indent=2) + '\n')
    print(json.dumps({'status': 'passed', 'epochs': len(epochs), 'archive_members': members,
                      'current_files': len(current), 'new_findings': len(findings), 'links': len(links)}))

if __name__ == '__main__':
    main()
