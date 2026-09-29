"""Audit exact original pattern scopes without treating definitions as proofs."""
import argparse
from collections import Counter
from datetime import datetime, timezone
from hashlib import sha256
import json
from pathlib import Path


def sha(data):
    return sha256(data).hexdigest()


def certificate_hash(data):
    return sha(b'MPK-MODULE-CERT-0.1\0' + data)


def read(path):
    return json.loads(path.read_bytes())


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--repo', type=Path, required=True)
    parser.add_argument('--checks', type=Path, required=True)
    args = parser.parse_args()
    root = Path(__file__).resolve().parents[1]
    foundation = args.repo / 'develop/migrations/csharp-03/ordinary-foundation'
    pins = foundation / 'control-predicates/with-pattern-scopes'
    local = read(root / 'local-verification.json')
    assert local['status'] == 'passed_targeted_tests_lint_format'
    assert local['unique_tests_passed'] == 4 and local['test_runs_passed'] == 5
    for name, digest in local['source_hashes'].items():
        assert sha((args.repo / name).read_bytes()) == digest, name
    for name, digest in local['fixture_hashes'].items():
        assert sha((pins / name).read_bytes()) == digest, name
    for row in local['stages']:
        assert row['exit_code'] == 0
        assert sha((root / f"{row['stage']}.log").read_bytes()) == row['log_sha256']
    for name, digest in local['binaries'].items():
        assert sha(Path(name).read_bytes()) == digest, name
    coverage, operations, missing = Counter(), Counter(), Counter()
    certificates = {}
    for path in sorted(pins.glob('*.json')):
        case = path.stem
        p = read(path)
        prior = read(foundation / 'control-predicates/with-execution' / path.name)
        native_path = foundation / 'control-edges' / path.name
        native = read(native_path)
        data = bytes.fromhex(path.with_suffix('.hex').read_text())
        certificates[case] = data
        assert p['application_scope_pending'] is True
        assert p['certificate_sha256'] == certificate_hash(data)
        for key in ('source_ir_sha256', 'foundation_sha256', 'control_vc_sha256',
                    'native_source_program_sha256', 'native_source_certificate_sha256',
                    'measures', 'sequents', 'unresolved_regions'):
            assert p[key] == prior[key], (case, key)
        assert p['native_source_program_sha256'] == sha(native_path.read_bytes())
        assert p['native_source_certificate_sha256'] == certificate_hash(bytes.fromhex(native_path.with_suffix('.hex').read_text()))
        coverage['contexts'] += 1
        coverage['changed_certificates'] += int(data != bytes.fromhex((foundation / f'control-predicates/with-execution/{case}.hex').read_text()))
        for scope in p.get('pattern_scopes', []):
            coverage['original_pattern_goals'] += 1
            assert scope['original_pattern_predicate_pending'] is True
            sequent = next(s for s in p['sequents'] if s['source']['id'] == scope['source_sequent_id'])
            source = sequent['source']
            step = scope['source_step']
            assert source['function_id'] == scope['function_id']
            assert source['region_id'] == scope['pattern_id']
            assert source['source_node_id'] == step['entry_node_id']
            assert source['target_node_id'] == step['exit_node_id']
            assert not source['assumptions'] and len(source['goals']) == 1
            assert sequent['goals'][0]['definition'] is None
            flow = next(f for f in native['functions'] if f['source']['function_id'] == scope['function_id'])
            graph = flow['source']['source_graph']
            node = next(n for n in graph['nodes'] if n['id'] == step['source_node_id'])
            for field, original in [('source_ordinal', 'source_ordinal'), ('operation', 'operation'),
                                    ('source_inputs', 'inputs'), ('source_result', 'result'),
                                    ('source_slot', 'slot'), ('successor_source_ids', 'successors')]:
                assert step[field] == node[original], (case, field)
            if step['source_ordinal'] is not None:
                operation = graph['operations'][step['source_ordinal']]
                assert step['source_kind'] == operation['kind']
                assert step['source_traits'] == operation['traits']
            operations[step['operation'] or 'control'] += 1
            assert scope['excluded_unreachable'] == (step['source_node_id'] in flow.get('excluded_unreachable_source_node_ids', []))
            expected = [e for e in flow['source_executions'] if e['source_node_id'] == step['source_node_id']]
            assert len(scope['executions']) == len(expected)
            for execution, original in zip(scope['executions'], expected):
                coverage['native_paths'] += 1
                assert execution['source_execution'] == original
                edge = next(e for e in flow['edges'] if e['source']['id'] == original['edge_id'])
                assert execution['edge_kind'] == edge['source']['kind']
                coverage[f"{execution['edge_kind']}_paths"] += 1
                count = original['argument_count']
                assert execution['native_argument_indices'] == list(range(count))
                coverage['native_arguments'] += count
                coverage['scope_arguments'] += len(execution['arguments'])
                coverage['fully_mapped_paths'] += int(not execution['pending_observation_binding_indices'])
                assert len(execution['observations']) == len(source['goals'][0]['bindings'])
                pending = []
                for i, (observation, binding) in enumerate(zip(execution['observations'], source['goals'][0]['bindings'])):
                    coverage['observation_occurrences'] += 1
                    assert observation['source'] == binding
                    assert execution['arguments'][observation['observation_argument_index']] == binding
                    point = observation['native_definition_point']
                    assert point['kind'] == binding['kind'] == 'ssa'
                    assert point['type_id'] == binding['type_id'] and point['value_id'] == binding['value_id']
                    index = observation['native_argument_index']
                    role = f'observation_transport:{i}'
                    relations = [c for c in execution['components'] if c['role'] == role]
                    if index is None:
                        assert point not in execution['arguments'][:count]
                        assert not relations
                        pending.append(i)
                        missing[('governing' if i == 0 else 'result') + ':' + execution['edge_kind']] += 1
                    else:
                        assert 0 <= index < count and execution['arguments'][index] == point
                        assert len(relations) == 1
                        assert relations[0]['argument_indices'] == [observation['observation_argument_index'], index]
                        coverage['physical_transport_pairs'] += 1
                assert execution['pending_observation_binding_indices'] == pending
                coverage['pending_observation_links'] += len(pending)
                for component in execution['components']:
                    assert all(0 <= i < len(execution['arguments']) for i in component['argument_indices'])
                if original['definition'] is not None:
                    assert execution['components'][0] == {
                        'role': 'native_source_execution', 'definition': original['definition'],
                        'argument_indices': list(range(count))}
                assert execution['definition'] is not None and not execution['pending_definition_reasons']
                coverage['defined_native_scopes'] += 1
    assert dict(coverage) == {'contexts': 18, 'changed_certificates': 7,
                             'original_pattern_goals': 102, 'native_paths': 113,
                             'normal_paths': 112, 'exception_paths': 1,
                             'native_arguments': 9814, 'scope_arguments': 9955,
                             'fully_mapped_paths': 27, 'observation_occurrences': 194,
                             'physical_transport_pairs': 108, 'pending_observation_links': 86,
                             'defined_native_scopes': 113}, coverage
    assert dict(missing) == {'governing:normal': 85, 'governing:exception': 1}
    stages = read(args.checks / 'stages.json')
    retained = read(args.checks / 'retained-stages.json')
    expected = {f'{case}-{mutation}-{backend}' for case in certificates
                for mutation in ('positive', 'hash') for backend in ('go', 'rust')}
    assert set(stages) <= expected and len(expected) == 72
    assert len(retained) == 44
    for stem, row in stages.items():
        case, mutation, backend = stem.rsplit('-', 2)
        data = certificates[case]
        if mutation == 'hash':
            data = data[:-1] + bytes([data[-1] ^ 1])
        assert row['exit_code'] == (0 if mutation == 'positive' else 1)
        assert row['certificate_sha256'] == certificate_hash(data)
        assert (args.checks / f'{stem}.mpcert').read_bytes() == data
        assert sha((args.checks / f'{stem}.json').read_bytes()) == row['report_sha256']
        assert sha((args.checks / f'{stem}.stderr').read_bytes()) == row['stderr_sha256']
        report = read(args.checks / f'{stem}.json')
        assert report['verdict'] == ('accepted' if mutation == 'positive' else 'rejected')
        if mutation == 'positive':
            if backend == 'go':
                assert bytes(report['report']['CertificateHash']).hex() == certificate_hash(data)
                assert report['report']['AxiomCount'] == 0
                assert all(v == 0 for v in report['report']['AxiomReport']['Summary'].values())
            else:
                assert report['hashes']['certificate'] == certificate_hash(data)
                assert report['axiom_count'] == 0 and report['error_code'] is None
                assert all(v == 0 for v in report['axiom_report']['summary'].values())
        elif backend == 'go':
            assert report['error_kind'] == 'hash_mismatch' and report['certificate'] == certificate_hash(data)
        else:
            assert report['error_code'] == 'KERNEL_HASH_MISMATCH'
            assert report['hashes']['certificate'] == certificate_hash(data)
    fully_checked = set(stages) == expected and (args.checks / 'verification.json').exists()
    if fully_checked:
        receipt = read(args.checks / 'verification.json')
        assert receipt['status'] == 'passed' and receipt['stages'] == stages
        assert receipt['newly_executed_stages'] == 28 and receipt['retained_stage_count'] == 44
    linux_root = root / 'server-linux'
    linux = read(linux_root / 'status.json') if (linux_root / 'status.json').exists() else None
    if linux is not None and linux['status'] == 'passed_targeted_linux_tests_lint_format':
        manifest = read(linux_root / 'source-manifest.json')
        assert manifest['source_hashes'] == local['source_hashes']
        assert manifest['fixture_hashes'] == local['fixture_hashes']
        assert linux['unique_tests_passed'] == 4
        for row in linux['stages']:
            assert row['exit_code'] == 0
            assert sha((linux_root / f"{row['stage']}.log").read_bytes()) == row['log_sha256']
    transfer = read(root / 'server-transfer.json') if (root / 'server-transfer.json').exists() else None
    result = {'status': 'passed_local_scope_checkpoint', 'recorded_at': datetime.now(timezone.utc).isoformat(),
              'coverage': dict(coverage), 'source_operations': dict(operations),
              'pending_observation_roles': dict(missing), 'local': local, 'linux': linux,
              'server_transfer': transfer,
              'dual_checkers': {'expected_stages': 72, 'observed_terminal_stages': len(stages),
                                'retained_stages': 44, 'newly_observed_stages': len(stages) - 44,
                                'all_stages_and_report_audit_complete': fully_checked,
                                'pending_stages': sorted(expected - set(stages)), 'stages': stages},
              'application_proofs_pending': 987, 'original_pattern_predicates_pending': 102,
              'unit_5_complete': False, 'w09_complete': False, 'full_t_gate': 'deferred to T06-W12'}
    (root / 'verification.json').write_text(json.dumps(result, indent=2, sort_keys=True) + '\n')
    print('AUDITED', dict(coverage), 'checker stages', len(stages), '/', len(expected))


if __name__ == '__main__':
    main()
