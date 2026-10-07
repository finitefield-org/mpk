import copy
import hashlib
import json
import pathlib

b = pathlib.Path(__file__).parent
r = pathlib.Path('/private/tmp/mpk-w09-refreeze-consumer-continuation')
j = pathlib.Path('/private/tmp/mpk-w09-wrapper-owner-validation')
h = lambda p: hashlib.sha256(p.read_bytes()).hexdigest()
read = lambda p: json.loads(p.read_bytes())
assert read(b / 'wrapper-owner-validation/status.json')['status'] == 'failed'
original = read(b / 'wrapper-program-promotion-plan/plan.json')
targets = read(b / 'wrapper-program-plan.json')['targets']
actual = {x['label']: x for x in read(b / 'wrapper-program-regeneration/generated/generation.json')['programs']}
identities = read(b / 'vir-hash-replay-audit.json')['identities']
out = b / 'wrapper-program-promotion-plan-2'
out.mkdir(exist_ok=False)
docs = {}
raw = {}
rows = []
paths = set()

def at(v, pointer):
    for k in pointer.split('/')[1:]:
        k = k.replace('~1', '/').replace('~0', '~')
        v = v[int(k)] if isinstance(v, list) else v[k]
    return v

for target in targets:
    label = target['label']
    source = actual[label]['source']
    assert source == target['source']
    for occurrence in target['occurrences']:
        pointer = occurrence['pointer']
        if not pointer.endswith('/program'):
            continue
        path = occurrence['file']
        candidate = b / 'wrapper-program-promotion-plan/files' / path
        if path not in docs:
            raw[path] = candidate.read_bytes() if candidate.exists() else (r / path).read_bytes()
            docs[path] = json.loads(raw[path])
        parent_pointer = pointer.rsplit('/', 1)[0]
        parent = at(docs[path], parent_pointer)
        for key in ['source_snapshot_sha256', 'source_request_id']:
            if key not in parent or parent[key] == source['id']:
                continue
            old = parent[key]
            matches = [x for x in identities if x['corpus'] == source['request_path']
                       and x['old_source_ir_sha256'] == target['old_source_ir_sha256']
                       and x['old_id'] == old and x['new_id'] == source['id']
                       and x['new_source_ir_sha256'] == source['new_source_ir_sha256']]
            assert len(matches) == 1, (label, path, old, source)
            before = b / 'before-inputs' / source['request_path']
            after = r / source['request_path']
            old_request = next(x for x in read(before) if x['id'] == old)
            new_request = next(x for x in read(after) if x['id'] == source['id'])
            assert old_request['case'] == new_request['case']
            old_sources = [x for x in old_request['inputs'] if x['kind'] == 'source']
            new_sources = [x for x in new_request['inputs'] if x['kind'] == 'source']
            assert old_sources == new_sources
            assert old_request['roots'] == new_request['roots']
            parent[key] = source['id']
            rows.append({
                'label': label, 'file': path, 'pointer': parent_pointer + '/' + key,
                'old_request_id': old, 'new_request_id': source['id'],
                'actual_source': source, 'actual_replay_identity': matches[0],
                'before_request_corpus_sha256': h(before),
                'current_request_corpus_sha256': h(after),
                'original_source_inputs_and_roots_identical': True,
            })
            paths.add(path)

assert len(rows) == 198 and len(paths) == 105
changes = {x['path']: dict(x) for x in original['changes']}
for path in paths:
    payload = json.dumps(docs[path], sort_keys=True, ensure_ascii=False, indent=2).encode() + raw[path][len(raw[path].rstrip()):]
    dst = out / 'files' / path
    dst.parent.mkdir(parents=True, exist_ok=True)
    dst.write_bytes(payload)
    if path in changes:
        assert h(r / path) == changes[path]['before_raw_sha256']
        assert h(j / path) == changes[path]['raw_sha256']
    else:
        assert (j / path).read_bytes() == (r / path).read_bytes()
        changes[path] = {'path': path, 'kind': 'metadata', 'before_raw_sha256': h(r / path)}
    changes[path]['raw_sha256'] = h(dst)
    changes[path]['actual_parent_request_identity_reconciled'] = True

for path, row in changes.items():
    src = out / 'files' / path
    if not src.exists():
        old = b / 'wrapper-program-promotion-plan/files' / path
        src.parent.mkdir(parents=True, exist_ok=True)
        src.write_bytes(old.read_bytes())
    assert h(src) == row['raw_sha256']

receipt = dict(original)
receipt.update(
    status='reviewable_exact_program_and_parent_request_identity_plan_pending_validation',
    parent_request_identity_occurrences=198,
    parent_request_identity_files=105,
    parent_request_identity_audit=rows,
    initial_original_owner_failure={'test': 'boundary-fields', 'exit_code': 101,
        'difference': 'Only /source_snapshot_sha256 differs; program/certificate/term/declaration fields are exactly equal.'},
    changes=list(changes.values()),
)
(out / 'plan.json').write_text(json.dumps(receipt, sort_keys=True, indent=2) + '\n')
for path in paths:
    (j / path).write_bytes((out / 'files' / path).read_bytes())
    assert h(j / path) == changes[path]['raw_sha256']
print('actual198 parent identity occurrences repaired in105 isolated review files; complete plan', len(changes))
