"""Inspect original bytes, native graph correspondence and private owner linkage.

Projection checks here are independent of the Go transport serializer, not an
independent ParseLax/MVS implementation or a source/program proof.
"""
from pathlib import Path
import base64, hashlib, json

ROOT = Path(__file__).resolve().parent
INPUTS = {}

def seal(path):
    path = Path(path)
    raw = path.read_bytes()
    return {'path': str(path), 'bytes': len(raw), 'sha256': hashlib.sha256(raw).hexdigest(),
            'mode': path.stat().st_mode & 0o777}

def raw(path):
    path = Path(path)
    INPUTS[str(path)] = seal(path)
    return path.read_bytes()

def read(path):
    return json.loads(raw(path))

def canonical(value):
    return json.dumps(value, sort_keys=True, ensure_ascii=False, separators=(',', ':')).encode()

def digest(domain, value):
    return hashlib.sha256(domain.encode() + b'\0' + canonical(value)).hexdigest()

def object_bytes(obj, data):
    assert len(data) == obj['bytes'] and hashlib.sha256(data).hexdigest() == obj['raw_sha256']
    body = {key: value for key, value in obj.items() if key != 'id'}
    assert digest('MPK-GO-SOURCE-OBJECT-1.0', body) == obj['id']

def packet_part(packet, field, domain):
    encoded = bytes.fromhex(packet[field])
    value = json.loads(encoded)
    assert encoded == canonical(value)
    body = {key: item for key, item in value.items() if key != 'content_sha256'}
    assert digest(domain, body) == value['content_sha256']
    return value

def native_events(epoch):
    return [json.loads(line) for line in raw(ROOT / epoch / 'stdout.log').splitlines()]

def observation(path):
    value = read(path)
    for field in ['initial', 'final']:
        for summary in value.get(field, []):
            source = summary['source']
            if source is None:
                assert summary['identity']['Path'] in ['go', 'toolchain']
            else:
                assert summary['identity']['Path'] not in ['go', 'toolchain']
                object_bytes(source['object'], base64.b64decode(source['raw'], validate=True))
    return value

def main():
    output = ROOT / 'inspection.json'
    assert not output.exists()
    verification = read(ROOT / 'verification.json')
    epochs = {path.parent.name: read(path) for path in ROOT.glob('*/result.json')}
    assert sorted(epochs) == verification['source_epochs']
    assert sorted(name for name, row in epochs.items() if row['status'] == 'failed') == ['native-initial']
    for name, row in epochs.items():
        assert row['consumed_inputs_unchanged'] or row['expected_formatter_mutation']
        assert row['whole_sdk_unchanged']
        if name != 'native-initial':
            assert row['status'] == 'passed'
        assert row['inputs'] == seal(Path(row['inputs']['path']))
        for field in ['stdout', 'stderr']:
            assert row[field] == seal(Path(row[field]['path']))
    latest = {}
    for epoch in ['native-initial', 'affected-initial', 'edges-final', 'replacements-final', 'toolchains-final']:
        for event in native_events(epoch):
            name = event.get('Test', '')
            if name and '/' not in name and event['Action'] in ['pass', 'fail']:
                latest[name] = event['Action']
    assert len(latest) == 20 and set(latest.values()) == {'pass'}
    initial_log = ''.join(event.get('Output', '') for event in native_events('native-initial'))
    assert 'file exists' in initial_log and 'original offline Go module discovery failed' in initial_log

    # Complete original initial/final module-source snapshots, including the
    # unselected .mod-only leaves and every selected ZIP/cache original.
    snapshots = []
    for epoch in ['native-initial', 'edges-final', 'replacements-final', 'toolchains-final']:
        for path in sorted((ROOT / epoch / 'native-originals').glob('*.graph-*-inputs.json')):
            value = read(path)
            for obj in value['objects']:
                object_bytes(obj, base64.b64decode(value['raw'][obj['id']], validate=True))
            assert len(value['objects']) == len(value['raw'])
            snapshots.append({'input': seal(path), 'objects': len(value['objects'])})
    generated = []
    for epoch in ['edges-final', 'replacements-final', 'toolchains-final']:
        for path in sorted((ROOT / epoch / 'native-originals').glob('*.graph-generated-originals.json')):
            value = read(path)
            for row in value['whole_original_entries']:
                if not row.get('directory'):
                    data = base64.b64decode(row['raw'], validate=True)
                    assert hashlib.sha256(data).hexdigest() == row['sha256']
            generated.append(seal(path))
    assert len(generated) == 6

    graph_path = next((ROOT / 'edges-final/native-originals').glob('*.graph-whole-original-observation.json'))
    graph = observation(graph_path)
    stdout = base64.b64decode(graph['stdout'], validate=True)
    assert graph['arguments'] == ['mod', 'graph']
    environment = dict(row.split('=', 1) for row in graph['environment'])
    assert all(environment[key] == value for key, value in {'GOPROXY': 'off', 'GOSUMDB': 'off', 'GOTOOLCHAIN': 'local', 'GOWORK': 'off', 'CGO_ENABLED': '0', 'GOOS': 'linux', 'GOARCH': 'amd64'}.items())
    summaries = {(s['identity']['Path'], s['identity']['Version']): s for s in graph['final']}
    assert summaries['example.com/c', 'v1.0.0']['source'] is not None
    assert b'future_directive ignored' in base64.b64decode(summaries['example.com/c', 'v1.0.0']['source']['raw'], validate=True)
    assert summaries['example.com/leaf', 'v1.0.0']['requirements'] is None
    assert ('example.com/q', 'v1.0.0') not in summaries
    assert b'example.com/leaf@v1.0.0 ' not in stdout and b'example.com/never' not in stdout
    assert stdout.count(b'example.com/a@v1.0.0 example.com/c@v1.0.0\n') == 2
    assert b'example.com/z@v1.0.0 example.com/q@v1.0.0\n' in stdout
    assert graph['selected_modules'] == 7
    controls = []
    for path in sorted((ROOT / 'edges-final/native-originals').glob('graph-*.json')):
        value = read(path)
        if 'error' in value:
            assert value['error'] != '<nil>'
            controls.append(seal(path))
    assert len(controls) == 9

    rejected = read(ROOT / 'replacements-final/native-originals/graph-readonly-root-upgrade-rejection.json')
    assert not rejected['accepted_owner']
    assert b'updates to go.mod needed' in base64.b64decode(rejected['stderr'], validate=True)
    expanded = observation(ROOT / 'replacements-final/native-originals/graph-rejected-native-root-expansion.json')
    assert not expanded['accepted_owner']
    expanded_output = base64.b64decode(expanded['stdout'], validate=True)
    assert b'example.com/app example.com/alias@v1.1.0\n' in expanded_output
    assert b'example.com/app example.com/alias@v1.0.0\n' not in expanded_output
    assert any(s['identity'] == {'Path': 'example.com/alias', 'Version': 'v1.0.0'} and s['source'] is not None for s in expanded['initial'])
    versioned = observation(next((ROOT / 'replacements-final/native-originals').glob('*.graph-whole-original-observation.json')))
    assert any(s['replacement']['Path'] == 'example.com/Actual' and '!actual' in s['source']['object']['path'] for s in versioned['final'] if s['source'] is not None)
    virtual = [observation(path) for path in (ROOT / 'toolchains-final/native-originals').glob('*.graph-virtual-observation.json')]
    assert {row['suggestion'] for row in virtual} == {'default', 'go1.25.0', 'go1.25.0-original'}
    for row in virtual:
        assert any(s['identity'] == {'Path': 'toolchain', 'Version': row['suggestion']} for s in row['initial'])
        assert all(s['source'] is None for s in row['initial'] + row['final'])

    # The owner packet includes every original source, compiler export and
    # unused HTTP package. Check full raw objects and the new acyclic hash link.
    directory = ROOT / 'native-initial/native-originals/graph-owned'
    packet = read(directory / 'complete-original-module-graph-inputs.json')
    objects = {}
    total_bytes = 0
    for row in packet['snapshot_sources']:
        obj = row['object']
        data = raw(directory / 'original-source-objects' / row['raw_file'])
        object_bytes(obj, data)
        objects[obj['id']] = obj
        total_bytes += len(data)
    assert len(objects) == 5667
    snapshot = packet_part(packet, 'snapshot_hex', 'MPK-PRIVATE-GO-SOURCE-SNAPSHOT-INPUTS-1.0')
    capture = packet_part(packet, 'capture_hex', 'MPK-PRIVATE-GO-SOURCE-CAPTURE-INPUTS-1.0')
    exports = packet_part(packet, 'export_inputs_hex', 'MPK-PRIVATE-GO-SOURCE-EXPORT-INPUTS-1.0')
    modules = packet_part(packet, 'module_observations_hex', 'MPK-PRIVATE-GO-SOURCE-MODULE-OBSERVATIONS-1.0')
    owned_graph = packet_part(packet, 'module_graph_inputs_hex', 'MPK-PRIVATE-GO-SOURCE-MODULE-GRAPH-INPUTS-1.0')
    assert snapshot['objects'] == [row['object'] for row in packet['snapshot_sources']]
    assert capture['snapshot_inputs_sha256'] == snapshot['content_sha256']
    assert exports['source_capture_inputs_sha256'] == capture['content_sha256']
    assert modules['source_export_inputs_sha256'] == exports['content_sha256']
    assert owned_graph['source_module_observations_sha256'] == modules['content_sha256']
    assert len(exports['packages']) == 181 and any(row['import_path'] == 'net/http' for row in exports['packages'])
    for phase in ['initial_summaries', 'final_summaries']:
        for row in owned_graph[phase]:
            sid = row['manifest_source_object_id']
            if sid is None:
                assert row['module']['path'] in ['go', 'toolchain']
            else:
                assert objects[sid]['kind'] == 'go_mod'
    ownership = read(directory / 'graph-original-ownership.json')
    assert ownership['detached_input_buffers'] and ownership['immutable_image_and_original_roots_retired'] and ownership['unused_http_retained']
    previous = read(ROOT.parent / 'module-manifests/inspection.json')
    assert previous['status'] == 'passed' and not previous['proofs_replayed_in_this_checkpoint']
    proofs = read(ROOT.parent / 'module-data/controls-remaining/observations/sources/original-proof-replay.json')['complete_original_call_proofs']
    assert len(proofs) == 32
    for row in proofs:
        data = raw(Path(row['original_certificate']))
        assert len(data) == row['bytes'] and hashlib.sha256(data).hexdigest() == row['raw_sha256']
    assert not raw(ROOT / 'format-virtual-check/stdout.log')
    assert epochs['legacy-final']['status'] == 'passed'
    legacy = read(ROOT / 'legacy-final/inputs.json')
    assert 'original-legacy-sdk' in legacy['whole_original_sdk']['path']
    assert all(row == seal(Path(row['path'])) for row in INPUTS.values())
    result = {'status': 'passed', 'driver': seal(Path(__file__).resolve()), 'inputs': list(INPUTS.values()),
              'source_epochs': len(epochs), 'affected_unique_native_test_roots': len(latest),
              'original_module_snapshots': snapshots, 'complete_generated_input_trees': generated,
              'graph_controls': controls, 'original_owner_packages': 181, 'original_owner_objects': len(objects),
              'original_owner_bytes': total_bytes, 'original_proof_streams_unchanged': 32,
              'proofs_replayed_in_this_checkpoint': False, 'independent_ParseLax_or_MVS': False,
              'native_intermediate_root_expansion_trace_complete': False,
              'previous_original_roots_per_unit': 43, 'previous_original_error_universes': [0, 70],
              'full_T_gate': 'Deferred to final T02-W04: ./scripts/check-fast.sh'}
    output.write_text(json.dumps(result, indent=2) + '\n')
    print(json.dumps({key: result[key] for key in ['status', 'source_epochs', 'affected_unique_native_test_roots', 'original_owner_packages', 'original_owner_objects', 'original_owner_bytes']}))

if __name__ == '__main__':
    main()
