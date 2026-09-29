"""Audit original producer premises and complete scopes without proof claims."""
import argparse
from collections import Counter
from datetime import datetime, timezone
from hashlib import sha256
import json
from pathlib import Path


def read(path):
    return json.loads(path.read_bytes())


def sha(data):
    return sha256(data).hexdigest()


def cert_hash(data):
    return sha(b'MPK-MODULE-CERT-0.1\0' + data)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--repo', type=Path, required=True)
    parser.add_argument('--checks', type=Path, required=True)
    args = parser.parse_args()
    root = Path(__file__).resolve().parents[1]
    foundation = args.repo / 'develop/migrations/csharp-03/ordinary-foundation'
    pins = foundation / 'control-predicates/with-pattern-captures'
    local = read(root / 'local-verification.json')
    assert local['status'] == 'passed_targeted_tests_lint_format'
    assert local['unique_tests_passed'] == 7 and local['test_runs_passed'] == 8
    for name, digest in local['source_hashes'].items():
        assert sha((args.repo / name).read_bytes()) == digest, name
    for name, digest in local['fixture_hashes'].items():
        assert sha((pins / name).read_bytes()) == digest, name
    for row in local['stages']:
        assert row['exit_code'] == 0
        assert sha((root / f"{row['stage']}.log").read_bytes()) == row['log_sha256']
    for name, digest in local['binaries'].items():
        assert sha(Path(name).read_bytes()) == digest, name
    coverage = Counter()
    certificates = {}
    for path in sorted(pins.glob('*.json')):
        case = path.stem
        current = read(path)
        prior = read(foundation / 'control-predicates/with-pattern-scopes' / path.name)
        native_path = foundation / 'control-edges' / path.name
        native = read(native_path)
        data = bytes.fromhex(path.with_suffix('.hex').read_text())
        certificates[case] = data
        coverage['contexts'] += 1
        coverage['changed_certificates'] += int(data != bytes.fromhex(
            (foundation / f'control-predicates/with-pattern-scopes/{case}.hex').read_text()))
        assert current['application_scope_pending'] is True
        assert current['certificate_sha256'] == cert_hash(data)
        for field in ('source_ir_sha256', 'foundation_sha256', 'control_vc_sha256',
                      'native_source_program_sha256', 'native_source_certificate_sha256',
                      'measures', 'sequents', 'unresolved_regions', 'pattern_scopes'):
            assert current.get(field) == prior.get(field), (case, field)
        assert current['native_source_program_sha256'] == sha(native_path.read_bytes())
        captures = current.get('pattern_captures', [])
        for capture in captures:
            coverage['exact_producers'] += 1
            assert capture['execution_establishment_pending'] is True
            assert capture['definition'] and not capture['pending_reasons']
            flow = next(f for f in native['functions']
                        if f['source']['function_id'] == capture['function_id'])
            graph = flow['source']['source_graph']
            decisions = [n for n in graph['nodes'] if n['kind'] == 'pattern_decision'
                         and n['source_ordinal'] == capture['source_ordinal']]
            assert len(decisions) == 1
            value_id = decisions[0]['inputs'][0]
            assert capture['governing_source_value_id'] == value_id
            producers = [n for n in graph['nodes'] if n['result'] == value_id]
            assert len(producers) == 1
            assert producers[0]['id'] == capture['producer_source_node_id']
            normal, exceptional = [], []
            for execution in flow['source_executions']:
                if execution['source_node_id'] != producers[0]['id']:
                    continue
                edge = next(e for e in flow['edges'] if e['source']['id'] == execution['edge_id'])
                if edge['source']['kind'] == 'normal':
                    normal.append(execution)
                else:
                    exceptional.append(execution['edge_id'])
            assert normal and len(capture['alternatives']) == len(normal)
            assert capture['excluded_exceptional_edge_ids'] == exceptional
            point = capture['native_definition_point']
            assert point['value_id'] == capture['governing_value']['id']
            assert point['type_id'] == capture['governing_value']['type_id']
            assert capture['arguments'][capture['governing_argument_index']] == point
            for alternative, execution in zip(capture['alternatives'], normal):
                assert alternative['source_execution'] == execution
                indices = alternative['argument_indices']
                # Native metadata omits cached arguments/components. The Rust
                # source regression checks these reconstructed maps directly.
                assert len(indices) == execution['argument_count']
                assert all(0 <= i < len(capture['arguments']) for i in indices)
                gov = alternative['governing_native_argument_index']
                assert gov is not None and 0 <= gov < len(indices)
                assert capture['arguments'][indices[gov]] == point
                assert indices[gov] == capture['governing_argument_index']
                assert execution['definition']  # All seven frozen producer paths are compact.
                expected = [{'role': 'native_governing_producer_execution',
                             'definition': execution['definition'], 'argument_indices': indices}]
                assert alternative['components'] == expected and expected
        scopes = current.get('pattern_capture_scopes', [])
        old_scopes = prior.get('pattern_scopes', [])
        assert len(scopes) == len(old_scopes)
        for scope, old in zip(scopes, old_scopes):
            coverage['original_pattern_goals'] += 1
            for field in ('source_sequent_id', 'function_id', 'pattern_id', 'source_step',
                          'excluded_unreachable', 'original_pattern_predicate_pending'):
                assert scope[field] == old[field], (case, field)
            assert scope['original_pattern_predicate_pending'] is True
            assert len(scope['executions']) == len(old['executions'])
            capture = next(c for c in captures if c['pattern_id'] == scope['pattern_id'])
            for execution, previous in zip(scope['executions'], old['executions']):
                coverage['consuming_paths'] += 1
                assert execution['source_execution'] == previous['source_execution']
                assert execution['native_argument_indices'] == previous['native_argument_indices']
                assert execution['components'][0] == previous['components'][0]
                count = execution['source_execution']['argument_count']
                arguments = execution['arguments']
                assert arguments[:count] == previous['arguments'][:count]
                coverage['merged_arguments'] += len(arguments)
                assert not execution['pending_observation_binding_indices']
                dependency = execution['capture_dependency']
                assert dependency['pattern_id'] == scope['pattern_id']
                assert dependency['producer_source_node_id'] == capture['producer_source_node_id']
                assert dependency['definition'] == capture['definition']
                assert [arguments[i] for i in dependency['argument_indices']] == capture['arguments']
                assert arguments[dependency['governing_argument_index']] == capture['native_definition_point']
                governing = [c for c in execution['components'] if c['role'] == 'governing_capture']
                assert governing == [{'role': 'governing_capture', 'definition': capture['definition'],
                                      'argument_indices': dependency['argument_indices']}]
                assert len(execution['observations']) == len(previous['observations'])
                for i, (observation, before) in enumerate(zip(execution['observations'], previous['observations'])):
                    coverage['physical_observation_occurrences'] += 1
                    for field in ('source', 'native_definition_point', 'native_argument_index'):
                        assert observation[field] == before[field]
                    native_index = observation['native_argument_index']
                    captured_index = observation.get('captured_argument_index')
                    if i == 0:
                        assert captured_index == dependency['governing_argument_index']
                    else:
                        assert captured_index is None
                    target = native_index if native_index is not None else captured_index
                    assert target is not None and arguments[target] == observation['native_definition_point']
                    observed = observation['observation_argument_index']
                    assert arguments[observed] == observation['source']
                    transport = [c for c in execution['components'] if c['role'] == f'observation_transport:{i}']
                    assert len(transport) == 1 and transport[0]['argument_indices'] == [observed, target]
                    coverage['newly_linked_observations'] += int(native_index is None)
                for component in execution['components']:
                    assert all(0 <= i < len(arguments) for i in component['argument_indices'])
                if execution['definition']:
                    assert not execution['pending_definition_reasons']
                    coverage['compact_definitions'] += 1
                else:
                    assert execution['pending_definition_reasons'] == ['combined_binder_limit']
                    coverage['complete_component_fallbacks'] += 1
    assert dict(coverage) == {'contexts': 18, 'changed_certificates': 7, 'exact_producers': 7,
                             'original_pattern_goals': 102, 'consuming_paths': 113,
                             'merged_arguments': 18725, 'physical_observation_occurrences': 194,
                             'newly_linked_observations': 86, 'compact_definitions': 108,
                             'complete_component_fallbacks': 5}, coverage
    stages = read(args.checks / 'stages.json')
    retained = read(args.checks / 'retained-stages.json')
    assert len(retained) == 44 and set(retained) <= set(stages)
    expected = {f'{case}-{mutation}-{backend}' for case in certificates
                for mutation in ('positive', 'hash') for backend in ('go', 'rust')}
    assert set(stages) <= expected
    for stem, row in stages.items():
        case, mutation, backend = stem.rsplit('-', 2)
        data = certificates[case]
        if mutation == 'hash':
            data = data[:-1] + bytes([data[-1] ^ 1])
        assert row['exit_code'] == (0 if mutation == 'positive' else 1)
        assert row['certificate_sha256'] == cert_hash(data)
        assert (args.checks / f'{stem}.mpcert').read_bytes() == data
        assert sha((args.checks / f'{stem}.json').read_bytes()) == row['report_sha256']
        assert sha((args.checks / f'{stem}.stderr').read_bytes()) == row['stderr_sha256']
        report = read(args.checks / f'{stem}.json')
        assert report['verdict'] == ('accepted' if mutation == 'positive' else 'rejected')
        if mutation == 'positive':
            if backend == 'go':
                assert bytes(report['report']['CertificateHash']).hex() == cert_hash(data)
                assert report['report']['AxiomCount'] == 0
                assert all(v == 0 for v in report['report']['AxiomReport']['Summary'].values())
            else:
                assert report['hashes']['certificate'] == cert_hash(data)
                assert report['axiom_count'] == 0 and report['error_code'] is None
                assert all(v == 0 for v in report['axiom_report']['summary'].values())
        elif backend == 'go':
            assert report['error_kind'] == 'hash_mismatch'
            assert report['certificate'] == cert_hash(data)
        else:
            assert report['error_code'] == 'KERNEL_HASH_MISMATCH'
            assert report['hashes']['certificate'] == cert_hash(data)
    fully_checked = set(stages) == expected and (args.checks / 'verification.json').exists()
    if fully_checked:
        receipt = read(args.checks / 'verification.json')
        assert receipt['status'] == 'passed' and receipt['stage_count'] == 72
        assert receipt['stages'] == stages
        assert receipt['newly_executed_stages'] == 28 and receipt['retained_stage_count'] == 44
    checker = {'expected_stages': 72, 'observed_terminal_stages': len(stages),
               'all_stages_and_report_audit_complete': fully_checked,
               'newly_observed_stages': len(stages) - 44, 'retained_stages': 44,
               'pending_stages': sorted(expected - set(stages)), 'stages': stages}
    linux_path = root / 'server-linux/status.json'
    linux = read(linux_path) if linux_path.exists() else None
    if linux and linux['status'] == 'passed_targeted_linux_tests_lint_format':
        manifest = read(linux_path.parent / 'source-manifest.json')
        assert manifest['source_hashes'] == local['source_hashes']
        assert manifest['fixture_hashes'] == local['fixture_hashes']
        assert linux['unique_tests_passed'] == 7
        for row in linux['stages']:
            assert row['exit_code'] == 0
            assert sha((linux_path.parent / f"{row['stage']}.log").read_bytes()) == row['log_sha256']
    result = {'status': 'passed_local_producer_premise_checkpoint',
              'recorded_at': datetime.now(timezone.utc).isoformat(), 'coverage': dict(coverage),
              'local': local, 'linux': linux, 'dual_checkers': checker,
              'original_pattern_predicates_pending': 102, 'application_proofs_pending': 987,
              'producer_execution_establishment_pending': True,
              'unit_5_complete': False, 'w09_complete': False, 'full_t_gate': 'deferred to T06-W12'}
    (root / 'verification.json').write_text(json.dumps(result, indent=2, sort_keys=True) + '\n')
    print('AUDITED', dict(coverage), 'checker stages', len(stages), '/', 72)


if __name__ == '__main__':
    main()
