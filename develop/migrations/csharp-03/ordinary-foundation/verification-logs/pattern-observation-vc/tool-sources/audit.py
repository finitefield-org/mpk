"""Audit typed source observations and executor receipts without proof claims."""
import argparse
from collections import Counter
from datetime import datetime, timezone
from hashlib import sha256
import json
from pathlib import Path

BOOL = 'mpk.csharp.value.bool.v1'


def read(path):
    return json.loads(path.read_bytes())


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--repo', type=Path, required=True)
    args = parser.parse_args()
    root = Path(__file__).resolve().parents[1]
    manifest = read(root / 'source-manifest.json')
    for group in ('source_hashes', 'fixture_hashes'):
        for name, digest in manifest[group].items():
            assert sha256((args.repo / name).read_bytes()).hexdigest() == digest, name
    foundation = args.repo / 'develop/migrations/csharp-03/ordinary-foundation'
    coverage = Counter(contexts=0, original_goals=0, exact_operands=0, slot_snapshots=0,
                       nullable_storage_observations=0, distinct_current_updates=0)
    for path in sorted((foundation / 'pattern-observation-vc').glob('*.json')):
        coverage['contexts'] += 1
        program = read(path)
        native = read(foundation / 'control-edges' / path.name)
        assert program['source_ir_sha256'] == native['source_ir_sha256']
        observations = program.get('pattern_observations', [])
        assert len(observations) == sum(len(p['steps']) for p in program['patterns'])
        assert len({o['sequent_id'] for o in observations}) == len(observations)
        for observation in observations:
            coverage['original_goals'] += 1
            assert observation['source_semantics_pending'] is True
            pattern = next(p for p in program['patterns'] if p['id'] == observation['pattern_id'])
            step = next(s for s in pattern['steps'] if s['source_node_id'] == observation['source_node_id'])
            flow = next(f for f in program['functions'] if f['function_id'] == observation['function_id'])
            seq = next(s for s in program['sequents'] if s['id'] == observation['sequent_id'])
            assert seq['function_id'] == pattern['function_id'] == observation['function_id']
            assert seq['source_node_id'] == step['entry_node_id']
            assert seq['target_node_id'] == step['exit_node_id']
            assert not seq['assumptions'] and len(seq['goals']) == 1
            goal = seq['goals'][0]
            bindings = goal['bindings']
            prefix = observation['original_binding_count']
            assert 0 < prefix <= len(bindings) <= 256
            operands = observation['operands']
            assert len(operands) == len(step['source_inputs'])
            for i, operand in enumerate(operands):
                coverage['exact_operands'] += 1
                assert operand['source_input_index'] == i
                assert operand['source_value_id'] == step['source_inputs'][i]
                assert operand['binding_index'] == prefix + i
                producers = [n for n in flow['source_graph']['nodes']
                             if n['result'] == operand['source_value_id']]
                assert len(producers) == 1 and producers[0]['id'] == operand['producer_source_node_id']
                binding = bindings[prefix + i]
                assert binding == dict(kind='ssa', edge_id=None, node_id=step['entry_node_id'],
                                       value_id=operand['native_value']['id'], type_id=operand['native_value']['type_id'])
                if path.stem == 'guard_order' and step['operation'] == 'unary_update':
                    assert binding['value_id'] != bindings[0]['value_id']
                    coverage['distinct_current_updates'] += 1
            slot = observation['slot']
            if slot is not None:
                coverage['slot_snapshots'] += 1
                transfer = next(t for t in flow['transfers'] if t['source_node_id'] == step['source_node_id'])
                assert slot['source'] == transfer
                nominal = dict(flow['slots'])[transfer['slot']]
                native_flow = next(f for f in native['functions']
                                   if f['source']['function_id'] == observation['function_id'])
                storage = native_flow.get('slot_type_overrides', {}).get(transfer['slot'], nominal)
                assert slot['nominal_type_id'] == nominal and slot['storage_type_id'] == storage
                coverage['nullable_storage_observations'] += int(storage != nominal)
                for i, (index_name, kind) in enumerate([
                        ('before_assigned_index', 'source_entry_assigned'),
                        ('before_value_index', 'source_entry_slot'),
                        ('after_assigned_index', 'source_exit_assigned'),
                        ('after_value_index', 'source_exit_slot')]):
                    index = slot[index_name]
                    assert index == prefix + len(operands) + i
                    assert bindings[index] == dict(kind=kind, edge_id=None, value_id=transfer['slot'],
                                                   node_id=step['entry_node_id'] if i < 2 else step['exit_node_id'],
                                                   type_id=BOOL if i % 2 == 0 else storage)
            else:
                assert step['operation'] not in ('load', 'store', 'pattern_bind')
            term, arguments = goal['term'], []
            while term['form'] == 'app':
                arguments.append(term['argument'])
                term = term['function']
            assert list(reversed(arguments)) == [dict(form='var', index=i, type_id=b['type_id'])
                                                for i, b in enumerate(bindings)]
            signature = BOOL
            for binding in reversed(bindings):
                signature = f"({binding['type_id']}->{signature})"
            assert term == dict(form='const', name=f"Mpk.CSharp.Control.PatternStep.{pattern['id']}.{step['source_node_id']}",
                                type_id=signature)
    assert dict(coverage) == dict(contexts=18, original_goals=102, exact_operands=74,
                                 slot_snapshots=36, nullable_storage_observations=1, distinct_current_updates=2)
    statuses = {}
    for host in ('local', 'server-linux'):
        report = root / host
        if not (report / 'status.json').exists():
            statuses[host] = dict(status='pending')
            continue
        state = read(report / 'status.json')
        assert state['status'] == 'passed_targeted_tests_lint_format'
        assert state['unique_tests_passed'] == 3
        assert read(report / 'source-manifest.json') == manifest
        assert [s['stage'] for s in state['stages']] == [
            'observations', 'capture-compatibility', 'slot-consumers', 'clippy', 'format']
        for stage in state['stages']:
            assert stage['exit_code'] == 0
            assert sha256((report / f"{stage['stage']}.log").read_bytes()).hexdigest() == stage['log_sha256']
        if host == 'local':
            for name, digest in state['test_binaries'].items():
                assert sha256(Path(name).read_bytes()).hexdigest() == digest
        statuses[host] = state
    assert statuses['local']['status'] == 'passed_targeted_tests_lint_format'
    result = dict(status='passed_typed_observation_checkpoint', recorded_at=datetime.now(timezone.utc).isoformat(),
                  coverage=dict(coverage), hosts=statuses, source_semantics_pending=True,
                  original_pattern_predicates_pending=102, application_proofs_pending=987,
                  unit_5_complete=False, w09_complete=False, full_t_gate='deferred to T06-W12')
    (root / 'verification.json').write_text(json.dumps(result, indent=2, sort_keys=True) + '\n')
    print('AUDITED', dict(coverage), {host: state['status'] for host, state in statuses.items()})


if __name__ == '__main__':
    main()
