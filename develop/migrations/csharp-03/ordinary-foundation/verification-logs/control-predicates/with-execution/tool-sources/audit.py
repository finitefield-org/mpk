"""Audit the partial native/predicate checkpoint without inferring checker exits."""
import argparse
from collections import Counter
from datetime import datetime, timezone
from hashlib import sha256
import json
from pathlib import Path

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--repo', type=Path, required=True)
parser.add_argument('--checks', type=Path, required=True)
args = parser.parse_args()
root = Path(__file__).resolve().parents[1]
foundation = args.repo / 'develop/migrations/csharp-03/ordinary-foundation'
pins = foundation / 'control-predicates/with-execution'
native = foundation / 'control-edges'


def sha(data):
    return sha256(data).hexdigest()


def certificate_hash(data):
    return sha(b'MPK-MODULE-CERT-0.1\0' + data)


def read(path):
    return json.loads(path.read_bytes())


local = read(root / 'local-verification.json')
assert local['status'] == 'passed_targeted_tests_lint_format' and local['passed_tests'] == 10
for name, digest in local['source_hashes'].items():
    assert sha((args.repo / name).read_bytes()) == digest, name
for row in local['tests']:
    assert sha((root / f"{row['stage']}.log").read_bytes()) == row['log_sha256']
for name, digest in local['binaries'].items():
    assert sha(Path(name).read_bytes()) == digest, name
coverage = Counter()
certificates = {}
for path in sorted(pins.glob('*.json')):
    case = path.stem
    program = read(path)
    original = read(foundation / 'control-predicates' / path.name)
    source = read(native / path.name)
    data = bytes.fromhex(path.with_suffix('.hex').read_text())
    certificates[case] = data
    assert program['application_scope_pending'] is True
    assert certificate_hash(data) == program['certificate_sha256']
    assert sha((native / path.name).read_bytes()) == program['native_source_program_sha256']
    assert certificate_hash(bytes.fromhex((native / f'{case}.hex').read_text())) == program['native_source_certificate_sha256']
    assert program['control_vc_sha256'] == original['control_vc_sha256']
    assert len(program['sequents']) == len(original['sequents'])
    coverage['contexts'] += 1
    for sequent, prior in zip(program['sequents'], original['sequents']):
        assert sequent['source'] == prior['source']
        coverage['sequents'] += 1
        coverage['implications'] += int(sequent['logical_implication_definition'] is not None)
        for predicate, old in zip(sequent['assumptions'] + sequent['goals'], prior['assumptions'] + prior['goals']):
            assert predicate['source'] == old['source']
            assert [sequent['arguments'][i] for i in predicate['argument_indices']] == predicate['source']['bindings']
            coverage['predicates'] += 1
            coverage['defined'] += int(predicate['definition'] is not None)
            coverage['pending'] += int(predicate['definition'] is None)
            if predicate['definition'] is None:
                assert predicate['pending_constant_names']
                assert all(n.startswith('Mpk.CSharp.Control.PatternStep.') for n in predicate['pending_constant_names'])
            if dependency := predicate.get('native_guard_dependency'):
                flow = next(f for f in source['functions'] if f['source']['function_id'] == dependency['function_id'])
                edge = next(e for e in flow['edges'] if e['source']['id'] == dependency['edge_id'])
                assert edge['source']['guard'] == predicate['source']
                assert edge['source']['source_node_id'] == sequent['source']['source_node_id']
                assert edge['source']['target_node_id'] == sequent['source']['target_node_id']
                assert edge['guard_definition'] == predicate['definition'] == dependency['guard_definition']
                assert edge.get('ownership') == dependency.get('ownership')
                coverage['native_guards'] += 1
                coverage['ownership_guards'] += int(dependency.get('ownership') is not None)
assert dict(coverage) == {'contexts': 18, 'sequents': 215, 'implications': 113,
                          'predicates': 500, 'defined': 398, 'pending': 102,
                          'native_guards': 100, 'ownership_guards': 4}, coverage
linux = {}
for label, status in [('lint', 'passed_targeted_linux_lint_format'), ('tests', 'passed_targeted_linux_tests')]:
    directory = root / f'server-linux-{label}'
    report = read(directory / 'status.json')
    assert report['status'] == status, report
    manifest = read(directory / 'source-manifest.json')
    assert manifest['source_hashes'] == local['source_hashes']
    for row in report['stages']:
        assert row['exit_code'] == 0
        assert sha((directory / f"{row['stage']}.log").read_bytes()) == row['log_sha256']
    if label == 'tests':
        counts = {'predicate-regressions': 2, 'measures': 2, 'parser': 1,
                  'data-handoffs': 2, 'control-handoffs': 2, 'exception-handoffs': 1}
        assert len(report['stages']) == len(counts)
        for name, count in counts.items():
            assert f'test result: ok. {count} passed' in (directory / f'{name}.log').read_text()
        assert len(report['test_binaries']) == 2
        for name, digest in manifest['fixture_hashes'].items():
            assert sha((pins / name).read_bytes()) == digest
    linux[label] = report
stages = read(args.checks / 'stages.json')
expected = {f'{case}-{mutation}-{backend}' for case in certificates
            for mutation in ('positive', 'hash') for backend in ('go', 'rust')}
assert set(stages) <= expected and len(expected) == 72
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
    if mutation == 'positive' and backend == 'rust':
        assert report['hashes']['certificate'] == certificate_hash(data)
        assert report['axiom_count'] == 0 and all(n == 0 for n in report['axiom_report']['summary'].values())
fully_audited = False
if set(stages) == expected and (args.checks / 'verification.json').exists():
    checker_receipt = read(args.checks / 'verification.json')
    assert checker_receipt['status'] == 'passed'
    assert checker_receipt['stage_count'] == 72 and checker_receipt['case_count'] == 18
    assert checker_receipt['stages'] == stages
    fully_audited = True
output = {'status': ('passed_targeted_local_linux_runtime_lint_and_dual_checkers' if fully_audited
                     else 'passed_targeted_local_linux_runtime_lint; dual_checkers_in_progress'),
          'recorded_at': datetime.now(timezone.utc).isoformat(), 'coverage': dict(coverage),
          'local': local, 'linux': linux,
          'dual_checkers': {'expected_stages': 72, 'observed_terminal_stages': len(stages),
                            'all_stages_and_report_audit_complete': fully_audited,
                            'distinct_certificates': len(set(certificates.values())),
                            'pending_stages': sorted(expected - set(stages)), 'stages': stages},
          'application_proofs_pending': 987, 'unit_5_complete': False, 'w09_complete': False,
          'full_t_gate': 'deferred to T06-W12'}
(root / 'verification.json').write_text(json.dumps(output, indent=2, sort_keys=True) + '\n')
print('AUDITED', dict(coverage), 'checker stages', len(stages), '/', len(expected))
