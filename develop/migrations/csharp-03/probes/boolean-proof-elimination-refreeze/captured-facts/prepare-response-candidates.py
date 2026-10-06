import copy
import hashlib
import json
import pathlib

base = pathlib.Path(__file__).parent
root = pathlib.Path('/private/tmp/mpk-w09-context-application-integration')
request_root = base / 'request-candidates-6'
out = base / 'response-candidates-5'
out.mkdir(exist_ok=False)
digest = lambda raw: hashlib.sha256(raw).hexdigest()
encode = lambda value: (json.dumps(value, ensure_ascii=True, sort_keys=True, indent=2) + '\n').encode()
plan = []
request_plan = []

def contract_projection(row):
    result = copy.deepcopy(row)
    for entry in result['inputs']:
        if entry['kind'] != 'sidecar':
            continue
        value = json.loads(entry['utf8'])
        value.pop('semantic_context', None)
        value.pop('binding_set_sha256', None)
        value.pop('contract_sha256', None)
        entry['utf8'] = value
    return result

def native_projection(rows):
    result = copy.deepcopy(rows)
    for row in result:
        if isinstance(row.get('facts'), dict):
            row['facts'].pop('input_files')
    return result

for previous in json.loads((base / 'request-rebinding.json').read_bytes()):
    request_path = previous['path']
    current_requests = json.loads((request_root / request_path).read_bytes())
    original_raw = (base / 'before-inputs' / request_path).read_bytes()
    original_requests = json.loads(original_raw)
    assert len(current_requests) == len(original_requests)
    assert [contract_projection(r) for r in current_requests] == [contract_projection(r) for r in original_requests]
    source_count = 0
    by_id = {}
    for before, row in zip(original_requests, current_requests):
        for old_entry, entry in zip(before['inputs'], row['inputs']):
            assert old_entry['path'] == entry['path'] and old_entry['kind'] == entry['kind']
            if entry['kind'] == 'source':
                assert old_entry == entry
                source_count += 1
        if row['id'] in by_id:
            assert all(row[k] == by_id[row['id']][k] for k in ['compilation_id', 'inputs', 'roots'])
        by_id[row['id']] = row
    request_plan.append({'path':request_path, 'before_sha256':digest(original_raw),
        'after_sha256':digest((request_root / request_path).read_bytes()), 'unchanged_source_inputs':source_count,
        'unchanged_source_and_contract_body_projection_sha256':digest(encode([contract_projection(r) for r in current_requests]))})
    path = pathlib.Path(request_path)
    response_path = str(path.with_name(path.name.replace('requests.json', 'responses.json')))
    if request_path.endswith('control-vc/loop-requests.json'):
        response_path = 'develop/migrations/csharp-03/control-emission/loop-responses.json'
    original_responses = (base / 'before-responses' / response_path).read_bytes()
    responses = json.loads(original_responses)
    before = copy.deepcopy(responses)
    changes = 0
    facts_rows = 0
    for row in responses:
        assert row['id'] in by_id
        facts = row.get('facts')
        if not isinstance(facts, dict):
            continue
        facts_rows += 1
        inputs = {i['path']:i for i in by_id[row['id']]['inputs']}
        for entry in facts['input_files']:
            actual = inputs[entry['path']]
            assert actual['kind'] == entry['kind']
            raw = actual['utf8'].encode()
            metadata = {'raw_sha256':digest(raw), 'size_bytes':len(raw)}
            if entry['kind'] == 'source':
                assert all(entry[k] == v for k, v in metadata.items())
            elif any(entry[k] != v for k, v in metadata.items()):
                entry.update(metadata)
                changes += 1
    assert native_projection(responses) == native_projection(before)
    destination = out / response_path
    destination.parent.mkdir(parents=True, exist_ok=True)
    destination.write_bytes(encode(responses) if changes else original_responses)
    plan.append({'request_path':request_path, 'response_path':response_path,
        'before_response_sha256':digest(original_responses), 'after_response_sha256':digest(destination.read_bytes()),
        'rows':len(responses), 'facts_rows':facts_rows, 'sidecar_metadata_changes':changes,
        'unchanged_native_facts_projection_sha256':digest(encode(native_projection(responses)))})

(base / 'response-candidate-plan-2.json').write_bytes(encode(plan))
(base / 'request-candidate-plan-2.json').write_bytes(encode(request_plan))
print(json.dumps({'corpora':len(plan), 'rows':sum(r['rows'] for r in plan),
    'facts_rows':sum(r['facts_rows'] for r in plan), 'unchanged_native_source_inputs':sum(r['unchanged_source_inputs'] for r in request_plan),
    'sidecar_metadata_changes':sum(r['sidecar_metadata_changes'] for r in plan)}))
