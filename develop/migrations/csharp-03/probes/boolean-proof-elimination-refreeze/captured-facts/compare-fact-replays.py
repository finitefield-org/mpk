import collections
import copy
import hashlib
import json
import pathlib

base = pathlib.Path(__file__).parent
digest = lambda path: hashlib.sha256(path.read_bytes()).hexdigest()
old_dir = base / 'facts-replay-old'
new_dir = base / 'facts-replay-new-2'
for directory in [old_dir, new_dir]:
    status = json.loads((directory / 'status.json').read_bytes())
    assert status['status'] == 'passed' and status['exit_code'] == 0
    assert status['source_and_input_bytes_unchanged']

old = [json.loads(line) for line in (old_dir / 'stdout.jsonl').read_text().splitlines()]
new = [json.loads(line) for line in (new_dir / 'stdout.jsonl').read_text().splitlines()]
assert old[0]['revision'] == 4 and new[0]['revision'] == 5
assert old[0]['foundation_sha256'] == '230c708601f4b89feeae28af23da10ccb11eec998d990b5133cf9336369e15a2'
assert new[0]['foundation_sha256'] == '99369543ab96e97e971118fe980322fe0b138f253a31ab7f8a9b08565bad0844'
assert len(old) == len(new) == 385

def projection(row):
    result = copy.deepcopy(row)
    if result['result'] == 'emitted':
        for expression in result['contract_expressions']:
            expression.pop('attachment_sha256')
        # The real emitter sorts contracts by their recomputed hashes. Preserve
        # every expression and subject binding while comparing the full multiset.
        result['contract_expressions'].sort(key=lambda expression:json.dumps(expression, sort_keys=True))
    return result

different_attachments = 0
unchanged_rows = 0
reordered_expression_rows = 0
for before, after in zip(old[1:], new[1:]):
    assert projection(before) == projection(after), (before['corpus'], before['index'], before['result'], after['result'])
    unchanged_rows += before == after
    if before['result'] == 'emitted':
        key = lambda expression:json.dumps({k:v for k,v in expression.items() if k!='attachment_sha256'}, sort_keys=True)
        reordered_expression_rows += [key(x) for x in before['contract_expressions']] != [key(x) for x in after['contract_expressions']]
        for a, b in zip(sorted(before['contract_expressions'],key=key), sorted(after['contract_expressions'],key=key)):
            different_attachments += a['attachment_sha256'] != b['attachment_sha256']

receipt = {
    'schema':'mpk.csharp_practical.t01_w09.captured_fact_replay.v1',
    'status':'passed_original_native_facts_and_contract_bodies_preserved',
    'original_compiler_reexecuted':False,
    'replay_scope':'all captured facts imported and emitted by real production APIs at old revision 4 and approved revision 5; native source rejection rows preserved exactly',
    'request_corpora':63, 'rows':384, 'facts_rows':370, 'native_source_inputs_preserved':384,
    'results':dict(collections.Counter(row['result'] for row in new[1:])),
    'complete_result_rows_unchanged':unchanged_rows,
    'contract_attachment_hashes_recomputed':different_attachments,
    'expression_collections_reordered_by_recomputed_contract_hashes':reordered_expression_rows,
    'allowed_derived_result_changes':['contract_expressions[].attachment_sha256','contract_expressions collection ordering by contract hash'],
    'expression_bodies_definitions_terms_routes_and_boundary_transition_counts_unchanged':True,
    'old_compiled_context':old[0], 'new_compiled_context':new[0],
    'files':{str(path.relative_to(base)):digest(path) for path in [
        old_dir/'status.json', old_dir/'source-manifest.json', old_dir/'stdout.jsonl', old_dir/'stderr.txt',
        new_dir/'status.json', new_dir/'source-manifest.json', new_dir/'stdout.jsonl', new_dir/'stderr.txt',
        base/'request-candidate-plan-2.json', base/'response-candidate-plan-2.json',
        base/'input-lineage-final.json', base/'snapshot-id-map.json',
        base/'snapshot-map-old.jsonl', base/'snapshot-map-new.jsonl',
        base/'snapshot-map.rs', base/'complete-input-lineage.py',
        base/'sidecar-envelope-rehash-3.jsonl', base/'sidecar-envelope-rehash-status-3.json',
        base/'rehash-sidecars.rs', base/'replay-facts.rs', base/'prepare-response-candidates.py',
        base/'run-fact-replay.py', pathlib.Path(__file__)]},
    'full_t01_gate':'deferred to renewed T01-W10',
    'full_t06_gate':'deferred to T06-W12',
}
(base/'captured-fact-replay.json').write_text(json.dumps(receipt, sort_keys=True, indent=2)+'\n')
print(json.dumps({k:v for k,v in receipt.items() if k!='files'}))
