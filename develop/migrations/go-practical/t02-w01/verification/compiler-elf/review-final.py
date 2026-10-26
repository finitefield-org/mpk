"""Scoped original ELF/compressed-section review with unchanged repository rules."""
from pathlib import Path
import base64, hashlib, json, re, subprocess, sys, tarfile, types, urllib.parse

REPO = Path.cwd()
ROOT = Path(__file__).resolve().parent
BASIS = '133d465e86f455e0bf5c5436ba43a87b2bb02c68'
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
    output = ROOT / 'review-final.json'
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
    for name in ['go-tools/go2vir/preflight.go','go-tools/go2vir/bundle_selection.go','go-tools/go2vir/practical_compiler.go','go-tools/go2vir/practical_replay.go','crates/mpk-vc/src/release_bundle.rs','go-tools/go2vir/canonical_json.go','go-tools/go2vir/hash.go','crates/mpk-vc/src/canonical_json.rs','crates/mpk-vc/src/hash.rs','crates/mpk-kernel/src/limits.rs']:
        path=REPO/name
        if path.exists():assert path.read_bytes()==git('show',BASIS+':'+name)
    for name in configured | fixtures:
        assert (REPO / name).read_bytes() == git('show', BASIS + ':' + name)
    protected = {}
    prior_reviews = []
    for name in ['module-manifests/review.json', 'module-graphs/review.json', 'graph-manifests/review.json', 'graph-inputs/review.json', 'graph-import/review.json', 'root-expansion/review.json', 'observation-json/review.json', 'compiler-inputs/review.json', 'compiler-build-info/review.json', 'compiler-source-roles/review.json', 'standard-module/review.json']:
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
        if '/build-wiring/' in name and basename in ['original-inputs.json','original-collectors.json','inputs.json']:
            value=json.loads(raw)
            for row in value.get('inputs',[]):
                if 'raw_hex' not in row:continue
                data=bytes.fromhex(row['raw_hex']);assert len(data)==row['bytes'] and hashlib.sha256(data).hexdigest()==row['sha256']
                scan(logical(row['path']),data);decoded+=1
            return
        if basename=='complete-original-zstandard.json':
            value=json.loads(raw);assert value['go_version']=='go1.25.0' and len(value['complete_original_streams'])==3
            for row in value['complete_original_streams']:
                for field,digest in [('raw_hex','raw_sha256'),('decoded_hex','decoded_sha256')]:
                    data=bytes.fromhex(row[field]);assert hashlib.sha256(data).hexdigest()==row[digest]
                    scan('go-tools/go2vir/original-elf-zstandard/'+row['name']+'/'+field+'.raw',data);decoded+=1
                scan('go-tools/go2vir/original-elf-zstandard/'+row['name']+'/original.elf',bytes.fromhex(row['original_elf_hex']));decoded+=1
            return
        if basename=='complete-original-strict-modules.json':
            value=json.loads(raw)
            assert value['prior_original_cases']==656 and len(value['cases'])==1427
            for row in value['cases']:
                data=bytes.fromhex(row['raw_hex']);obj=row['source_object']
                assert len(data)==obj['bytes'] and hashlib.sha256(data).hexdigest()==obj['raw_sha256']
                scan('go-tools/go2vir/original-strict-standard-modules/'+row['name']+'/go.mod',data);decoded+=1
            return
        if basename.startswith('coherent-standard-module-'):
            value=json.loads(raw)
            assert value['all_original_application_source_and_Root_contract_bytes_unchanged'] and value['original_objects']==5697 and value['original_packages']==182 and value['original_compiler_files']==5577
            data=bytes.fromhex(value['changed_raw_hex']);obj=value['changed_object']
            assert len(data)==obj['bytes'] and hashlib.sha256(data).hexdigest()==obj['raw_sha256']
            scan('go-tools/go2vir/original-strict-standard-module-controls/'+obj['path'],data);decoded+=1
            for payload in ['metadata_hex','diagnostics_hex']:
                scan('go-tools/go2vir/original-strict-standard-module-controls/'+payload+'.raw',bytes.fromhex(value['exports'][payload]));decoded+=1
            return
        if basename.startswith('coherent-unused-sdk-role-'):
            value=json.loads(raw)
            assert value['whole_original_source_bytes_and_Root_contract_unchanged'] and value['error']=='SourceRole'
            assert value['original_object']['raw_sha256']==value['changed_object']['raw_sha256']==value['original_bytes_sha256']
            assert value['original_object']['path']==value['changed_object']['path']
            for field in ['changed_snapshot_hex','changed_capture_hex','changed_export_hex']:
                packet=json.loads(bytes.fromhex(value[field]))
                for payload in ['metadata_hex','diagnostics_hex']:
                    if payload in packet:
                        scan('go-tools/go2vir/original-sdk-source-role-controls/'+payload+'.raw',bytes.fromhex(packet[payload]));decoded+=1
            return
        if basename=='original-sdk-source-roles.json':
            value=json.loads(raw)
            for row in value['probes']:
                data=bytes.fromhex(row['raw_hex'])
                assert len(data)==row['bytes'] and hashlib.sha256(data).hexdigest()==row['raw_sha256']
                scan('go-tools/go2vir/original-sdk-role-probes/'+row['path'],data);decoded+=1
            return
        if basename=='source-byte-transport.json':
            value=json.loads(raw)
            assert value['complete_original_raw_sdk_unchanged'] and value['raw_source_objects']==11004
            prepared=ROOT.parent/'compiler-inputs/native/native-originals/versioned-owned/original-compiler-files'
            for row in value['sdk_sources']:
                directory=prepared if row['storage']=='prepared-compiler' else ROOT/'original-source-bytes'
                data=(directory/row['raw_file']).read_bytes()
                assert len(data)==row['bytes'] and hashlib.sha256(data).hexdigest()==row['raw_sha256']
                scan('go-tools/go2vir/original-sdk-source-corpus/'+row['path'],data);decoded+=1
            return
        if basename == 'inputs.json' and '/review-interrupted/' in name:
            value=json.loads(raw)
            assert value['exit_code']==143
            for row in value['sources']:
                data=(ROOT/'review-interrupted'/row['raw_file']).read_bytes()
                assert len(data)==row['bytes'] and hashlib.sha256(data).hexdigest()==row['sha256']
                scan(logical(row['path']),data);decoded+=1
            return
        if basename.startswith('coherent-executable-'):
            value=json.loads(raw)
            assert value['whole_original_source_and_Root_contract_unchanged'] and value['error']=='BuildInfo'
            for payload in ['metadata_hex','diagnostics_hex']:
                data=bytes.fromhex(value['exports'][payload]);scan('go-tools/go2vir/original-executable-controls/'+payload+'.raw',data);decoded+=1
            return
        if basename == 'coherent-original-sdk-content-control.json':
            value=json.loads(raw); data=bytes.fromhex(value['changed_raw_hex']); obj=value['changed_object']
            assert len(data)==obj['bytes'] and hashlib.sha256(data).hexdigest()==obj['raw_sha256']
            scan('go-tools/go2vir/original-compiler-sdk-control/'+obj['path'],data);decoded+=1
            for field in ['changed_snapshot_hex','changed_capture_hex','changed_export_hex']:
                packet=json.loads(bytes.fromhex(value[field]))
                for payload in ['metadata_hex','diagnostics_hex']:
                    if payload in packet:
                        scan('go-tools/go2vir/original-compiler-sdk-control/'+payload+'.raw',bytes.fromhex(packet[payload]));decoded+=1
            return
        if basename == 'coherent-original-compiler-controls.json':
            value=json.loads(raw)
            for row in value['cases']:
                assert row['whole_original_capture_and_Root_contract_preserved'] and row['error']
            return
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
        if basename.endswith('.canonical.json') and 'observation-json' in name:
            value=json.loads(raw)
            if value.get('schema') in ['mpk.go.practical.source_export_inputs.v1','mpk.go.practical.source_module_observations.v1','mpk.go.practical.source_module_graph_inputs.v2']:
                for field in ['metadata_hex','diagnostics_hex']:
                    data=bytes.fromhex(value[field])
                    scan('go-tools/go2vir/original-observation-byte-probes/'+field+'.raw',data);decoded+=1
            return
        if basename.endswith('.original.raw') and 'observation-json' in name:
            scan('go-tools/go2vir/original-observation-byte-probes/'+basename,raw);decoded+=1
            return
        if not any(token in basename for token in ['graph-', 'complete-original-module-', 'complete-original-export-']):
            return
        value = json.loads(raw)
        for field in ['export_inputs_hex', 'module_observations_hex', 'module_graph_inputs_hex', 'compiler_inputs_hex']:
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

    # Inspect the complete immutable upstream package, not only rewritten files.
    upstream = ROOT / 'dependencies/ruzstd-0.9.0.crate'
    provenance = json.loads((ROOT/'dependencies/provenance.json').read_bytes())
    assert seal(upstream)['sha256']==provenance['sha256']
    with tarfile.open(upstream) as package:
        for entry in package:
            if entry.isfile():
                data=package.extractfile(entry).read()
                relative=str(Path(entry.name).relative_to('ruzstd-0.9.0'))
                scan('third_party/ruzstd-go125/upstream/'+relative,data);members+=1

    # Supplemental dependency/source-mirror checks preserve the exact earlier
    # reader review and every original preimage before documentation was extended.
    original_reader_review=ROOT/'review.json';earlier=json.loads(original_reader_review.read_bytes())
    assert earlier['status']=='passed' and not earlier['new_findings']
    old_preimages={r['path']:r for r in json.loads((ROOT/'inspection-prelaunch.json').read_bytes())['inputs']}
    for row in earlier['current_inputs']:
        if row!=seal(row['path']):
            original=old_preimages[row['path']];data=bytes.fromhex(original['raw_hex'])
            assert len(data)==row['bytes'] and hashlib.sha256(data).hexdigest()==row['sha256']
            assert Path(row['path']).suffix in ['.md','.json']
    wiring_inspection=ROOT/'build-wiring/inspection.json';wiring=json.loads(wiring_inspection.read_bytes())
    assert wiring['status']=='passed' and wiring['complete_locked_graphs']==7 and len(wiring['actual_observer_builds'])==2 and not wiring['native_Linux_release_acceptance'] and not wiring['proofs_replayed']
    for name in ['native-linux','shared-native']:
        receipt=json.loads((ROOT/'build-wiring'/name/'inputs.json').read_bytes())
        archive=Path(receipt['complete_archive']['path']);assert receipt['complete_archive']==seal(archive);archives.add(archive)
        source_rows={logical(r['path']):r for r in receipt['inputs']}
        with tarfile.open(archive) as packed:
            entries=packed.getmembers();assert len(entries)==len(source_rows) and {m.name for m in entries}==set(source_rows)
            for member in entries:
                row=source_rows[member.name];data=packed.extractfile(member).read()
                assert member.isfile() and member.mode==row['mode'] and len(data)==row['bytes'] and hashlib.sha256(data).hexdigest()==row['sha256']
                scan(member.name,data);decode_originals(member.name,data);members+=1

    inspection_path = ROOT / 'inspection.json'
    inspection = json.loads(inspection_path.read_bytes())
    assert inspection['status'] == 'passed' and inspection['affected_rust_roots'] == 8 and inspection['coherent_compiler_claim_controls'] == 30 and inspection['coherent_binary_content_controls'] == 3 and inspection['coherent_source_role_controls']==3 and inspection['coherent_standard_module_controls']==4
    assert inspection['native_cases']==425 and inspection['complete_prior_cases']==187 and inspection['complete_original_zstandard_streams']==3
    assert len(inspection['whole_original_owners']) == 1 and inspection['whole_original_owners'][0]['objects'] == 5697 and inspection['whole_original_owners'][0]['compiler_files'] == 5577
    assert not inspection['installed_context_authority'] and not inspection['proofs_replayed_in_this_checkpoint']
    for row in inspection['inputs']:
        assert row == seal(Path(row['path']))
    prelaunch = json.loads((ROOT / 'inspection-prelaunch-final.json').read_bytes())
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
              'independent_inspection': seal(inspection_path), 'build_wiring_inspection':seal(wiring_inspection),'original_reader_review':seal(original_reader_review), 'current_inputs': current,
              'new_findings': findings, 'links': links, 'raw_outputs_preserved': outputs,
              'manual_review': 'Retain all187 earlier original executable/layout/text cases and9 actual SDK tools; all425 original/native/Rust observations agree. Read all ELF names before buildinfo selection, exact32/64-bit/both-endian compressed headers, signed offsets/lengths, extended counts and name indices; independently interpret complete ZLIB/Zstandard section-data prefixes with native Go1.25 dictionary/window/block/frame/checksum and unread-tail behavior. All3 complete original SDK Zstandard streams and both4320033-byte expanded tables remain intact. The complete upstream0.9.0 package archive, original sources and MIT license remain; only a private std-only manifest, block facade and3 integration/guard source changes differ. Seven full independent locked graphs and actual pinned local Linux/owned-mirror observer builds pass, with complete65-file codec source receipts/copying; original locks/collectors, failed full lock states, setup/cache failures and every command output remain. Existing codec block/history conditions are format semantics, not new source/bundle/kernel/certificate limits. All8 affected Rust roots, both native roots, scoped Clippy/vet/formats and document preflight pass; original no_std import compilation failure remains. The actual whole5577-file/163457770-byte/5480-source SDK and182-package/5697-object source/Root owner stay intact, including30 coherent claims,3 executable,3 source-role,4 standard-module and original content controls. Real original SDK tools remain their original formats; synthetic ELF observations never replace them or imply application proofs. No frozen source/type/error/proof universe, scanner rule/seal/exclusion, premise, math rule, public profile or kernel/certificate budget changes. General reader/codec edge-case equivalence and full installed compiler/root/capture/context/descriptor/registry/launcher/read/source/program/foundation/native Linux/T02-T11 work remain due; W01 and R01/R03/R04/R05/R06 stay open and profile inactive.',

              'exact_prior_original_rechecks': original_rechecks, 'prior_complete_original_reviews': prior_reviews,
              'full_T_gate': 'Deferred to final T02-W04: ./scripts/check-fast.sh', 'whole_repository_gate_run': False}
    output.write_text(json.dumps(result, indent=2) + '\n')
    print(json.dumps({'status': 'passed', 'epochs': len(epochs), 'archive_members': members,
                      'current_files': len(current), 'new_findings': len(findings), 'links': len(links)}))

if __name__ == '__main__':
    main()
