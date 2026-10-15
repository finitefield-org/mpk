"""Audit original source conditions and exact execution receipts, not proofs."""
import argparse
from collections import Counter
from datetime import datetime, timezone
from hashlib import sha256
import json
from pathlib import Path
import re


def read(path):
    return json.loads(path.read_bytes())


def sha(data):
    return sha256(data).hexdigest()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--repo', type=Path, required=True)
    args = parser.parse_args()
    root = Path(__file__).resolve().parents[1]
    foundation = args.repo / 'develop/migrations/csharp-03/ordinary-foundation'
    manifest = read(root / 'source-manifest.json')
    for group in ('source_hashes', 'fixture_hashes'):
        for name, digest in manifest[group].items():
            assert sha((args.repo / name).read_bytes()) == digest, name
    coverage = Counter(contexts=0, original_goals=0, defined_conditions=0,
                       pending_conditions=0, changed_certificates=0)
    operations = Counter()
    pins = foundation / 'control-predicates/with-pattern-observations'
    for path in sorted(pins.glob('*.json')):
        coverage['contexts'] += 1
        program = read(path)
        control = read(foundation / 'pattern-observation-vc' / path.name)
        prior = read(foundation / 'control-predicates/with-pattern-captures' / path.name)
        data = bytes.fromhex(path.with_suffix('.hex').read_text())
        assert program['certificate_sha256'] == sha(b'MPK-MODULE-CERT-0.1\0' + data)
        assert program['application_scope_pending'] is True
        for field in ('source_ir_sha256', 'foundation_sha256', 'native_source_program_sha256',
                      'native_source_certificate_sha256', 'measures', 'unresolved_regions'):
            assert program[field] == prior[field], (path.stem, field)
        old_data = bytes.fromhex((foundation / 'control-predicates/with-pattern-captures'
                                 / path.with_suffix('.hex').name).read_text())
        coverage['changed_certificates'] += int(data != old_data)
        assert [s['source'] for s in program['sequents']] == control['sequents']
        sources = program.get('pattern_sources', [])
        assert [s['source'] for s in sources] == control.get('pattern_observations', [])
        for entry in sources:
            coverage['original_goals'] += 1
            source = entry['source']
            step = entry['source_step']
            assert entry['native_source_equivalence_proof_pending'] is True
            flow = next(f for f in control['functions'] if f['function_id'] == source['function_id'])
            operation = flow['source_graph']['operations'][step['source_ordinal']]
            assert entry['source_constant'] == operation['constant']
            assert step['source_kind'] == operation['kind']
            assert step['source_traits'] == operation['traits']
            sequent = next(s for s in program['sequents'] if s['source']['id'] == source['sequent_id'])
            if entry['definition']:
                coverage['defined_conditions'] += 1
                operations[step['operation']] += 1
                assert entry['semantic_rule'] and not entry['pending_definition_reasons']
                assert sequent['goals'][0]['definition']
                assert not sequent['goals'][0]['pending_constant_names']
                assert sequent['logical_implication_definition']
            else:
                coverage['pending_conditions'] += 1
                assert entry['pending_definition_reasons']
                assert sequent['goals'][0]['pending_constant_names'] == [entry['source_name']]
            for scope in program.get('pattern_scopes', []) + program.get('pattern_capture_scopes', []):
                if scope['source_sequent_id'] == source['sequent_id']:
                    assert scope['original_pattern_predicate_pending'] == (entry['definition'] is None)
                    for execution in scope['executions']:
                        bindings = sequent['source']['goals'][0]['bindings']
                        assert [o['source'] for o in execution['observations']] == bindings
                        for observation in execution['observations']:
                            index = observation['observation_argument_index']
                            assert execution['arguments'][index] == observation['source']
                            index = observation.get('captured_argument_index')
                            if index is not None:
                                assert execution['arguments'][index] == observation['native_definition_point']
    assert dict(coverage) == dict(contexts=18, original_goals=102, defined_conditions=61,
                                 pending_conditions=41, changed_certificates=7), coverage
    assert dict(operations) == dict(constant=17, join_value=9, load=19, store=12, pattern_bind=4)
    hosts = {}
    for host in ('local', 'server-linux'):
        report = root / host
        if not (report / 'status.json').exists():
            hosts[host] = dict(status='pending')
            continue
        state = read(report / 'status.json')
        assert state['status'] == 'passed_targeted_tests_lint_format'
        assert state['unique_tests_passed'] == 6
        assert read(report / 'source-manifest.json') == manifest
        assert [s['stage'] for s in state['stages']] == [
            'source-conditions', 'capture-compatibility', 'observations', 'scope-units', 'clippy', 'format']
        for stage in state['stages']:
            assert stage['exit_code'] == 0
            assert sha((report / f"{stage['stage']}.log").read_bytes()) == stage['log_sha256']
        output = (report / 'source-conditions.log').read_text()
        counts = re.search(r'102 goals, 61 definitions, 41 pending; (\d+) positive and (\d+) changed-value/state', output)
        assert counts
        state['source_condition_observations'] = dict(positive=int(counts[1]), changed=int(counts[2]))
        if host == 'local':
            for name, digest in state['test_binaries'].items():
                assert sha(Path(name).read_bytes()) == digest
        hosts[host] = state
    assert hosts['local']['status'] == 'passed_targeted_tests_lint_format'
    checks = root / 'checks'
    stages = read(checks / 'stages.json') if (checks / 'stages.json').exists() else {}
    result = dict(status='passed_source_condition_checkpoint', recorded_at=datetime.now(timezone.utc).isoformat(),
                  coverage=dict(coverage), defined_operations=dict(operations), hosts=hosts,
                  checker_terminal_stages=len(stages), checker_expected_stages=72,
                  checker_audit_complete=(checks / 'verification.json').exists(),
                  native_source_equivalence_proofs_pending=True,
                  application_proofs_pending=987, unit_5_complete=False, w09_complete=False,
                  full_t_gate='deferred to T06-W12')
    (root / 'verification.json').write_text(json.dumps(result, indent=2, sort_keys=True) + '\n')
    print('AUDITED', dict(coverage), {host: state['status'] for host, state in hosts.items()})


if __name__ == '__main__':
    main()
