import copy
import hashlib
import json
import pathlib
import re

base = pathlib.Path(__file__).parent
requests_out = base / 'request-candidates-final'
responses_out = base / 'response-candidates-final'
requests_out.mkdir(exist_ok=False)
responses_out.mkdir(exist_ok=False)
digest = lambda raw: hashlib.sha256(raw).hexdigest()
encode = lambda value: (json.dumps(value, ensure_ascii=True, separators=(',', ':'))+'\n').encode()
old_maps = [json.loads(line) for line in (base/'snapshot-map-old.jsonl').read_text().splitlines()][1:]
new_maps = [json.loads(line) for line in (base/'snapshot-map-new.jsonl').read_text().splitlines()][1:]
assert len(old_maps) == len(new_maps) == 384
by_corpus = {}
id_map = {}
for before, after in zip(old_maps, new_maps):
    assert before['request_path'] == after['request_path'] and before['row_index'] == after['row_index']
    assert before['original_id'] == after['original_id']
    original_id = before['original_id']
    if re.fullmatch('[0-9a-f]{64}', original_id):
        assert original_id == before['snapshot_sha256']
        replacement = after['snapshot_sha256']
    else:
        replacement = original_id
    by_corpus.setdefault(before['request_path'], []).append((original_id, replacement))
    if original_id != replacement:
        if original_id in id_map:
            assert id_map[original_id] == replacement
        id_map[original_id] = replacement

def scalar_spans(text):
    decoder = json.JSONDecoder()
    spans = {}
    def whitespace(index):
        while index < len(text) and text[index].isspace():
            index += 1
        return index
    def walk(index, pointer):
        index = whitespace(index)
        if text[index] == '{':
            index = whitespace(index+1)
            if text[index] == '}':
                return index+1
            while True:
                key, end = decoder.raw_decode(text, index)
                assert isinstance(key, str)
                index = whitespace(end)
                assert text[index] == ':'
                index = whitespace(walk(index+1, pointer+(key,)))
                if text[index] == '}':
                    return index+1
                assert text[index] == ','
                index = whitespace(index+1)
        if text[index] == '[':
            index = whitespace(index+1)
            if text[index] == ']':
                return index+1
            ordinal = 0
            while True:
                index = whitespace(walk(index, pointer+(ordinal,)))
                if text[index] == ']':
                    return index+1
                assert text[index] == ','
                ordinal += 1
                index = whitespace(index+1)
        value, end = decoder.raw_decode(text, index)
        assert pointer not in spans
        spans[pointer] = (index, end, value)
        return end
    assert whitespace(walk(0, ())) == len(text)
    return spans

def patch_native_bytes(raw, desired, request=False):
    original = json.loads(raw)
    text = raw.decode()
    spans = scalar_spans(text)
    patches = []
    def walk(before, after, pointer):
        assert type(before) is type(after)
        if isinstance(before, dict):
            assert before.keys() == after.keys()
            for key in before:
                walk(before[key], after[key], pointer+(key,))
        elif isinstance(before, list):
            assert len(before) == len(after)
            for index, (a, b) in enumerate(zip(before, after)):
                walk(a, b, pointer+(index,))
        elif before != after:
            allowed = (len(pointer) == 2 and pointer[1] == 'id') or (
                len(pointer) == 5 and pointer[1:3] == ('facts', 'input_files') and
                pointer[4] in ['raw_sha256', 'size_bytes'])
            if request:
                allowed = (len(pointer) == 2 and pointer[1] == 'id') or (
                    len(pointer) == 4 and pointer[1] == 'inputs' and pointer[3] == 'utf8')
            assert allowed, pointer
            start, end, observed = spans[pointer]
            assert before == observed
            patches.append((start, end, json.dumps(after, ensure_ascii=True, separators=(',', ':'))))
    walk(original, desired, ())
    for start, end, replacement in sorted(patches, reverse=True):
        text = text[:start]+replacement+text[end:]
    assert json.loads(text) == desired
    return text.encode(), len(patches)

plan = json.loads((base/'response-candidate-plan-2.json').read_bytes())
lineage = []
for row in plan:
    request_path = row['request_path']
    requests = json.loads((base/'request-candidates-6'/request_path).read_bytes())
    for request, (original_id, replacement) in zip(requests, by_corpus[request_path]):
        assert request['id'] == original_id
        request['id'] = replacement
    destination = requests_out/request_path
    destination.parent.mkdir(parents=True, exist_ok=True)
    preserved_request_bytes, request_patches = patch_native_bytes(
        (base/'before-inputs'/request_path).read_bytes(), requests, request=True)
    destination.write_bytes(preserved_request_bytes)
    responses = json.loads((base/'response-candidates-5'/row['response_path']).read_bytes())
    for response in responses:
        response['id'] = id_map.get(response['id'], response['id'])
    native_before = (base/'before-responses'/row['response_path']).read_bytes()
    native_after, patches = patch_native_bytes(native_before, responses)
    destination = responses_out/row['response_path']
    destination.parent.mkdir(parents=True, exist_ok=True)
    destination.write_bytes(native_after)
    lineage.append({'request_path':request_path,'response_path':row['response_path'],
        'before_request_sha256':digest((base/'before-inputs'/request_path).read_bytes()),
        'after_request_sha256':digest((requests_out/request_path).read_bytes()),
        'before_response_sha256':digest(native_before),'after_response_sha256':digest(native_after),
        'changed_native_transport_scalar_fields':patches,
        'changed_request_transport_scalar_fields':request_patches,
        'all_other_original_request_transport_bytes_preserved':True,
        'allowed_response_changes':['id','facts.input_files[].raw_sha256','facts.input_files[].size_bytes'],
        'all_other_native_transport_bytes_preserved':True})
(base/'input-lineage-final.json').write_text(json.dumps(lineage, sort_keys=True, indent=2)+'\n')
(base/'snapshot-id-map.json').write_text(json.dumps(id_map, sort_keys=True, indent=2)+'\n')
print(json.dumps({'corpora':len(lineage),'snapshot_ids_rebound':len(id_map),
    'native_transport_scalar_fields_changed':sum(row['changed_native_transport_scalar_fields'] for row in lineage)}))
