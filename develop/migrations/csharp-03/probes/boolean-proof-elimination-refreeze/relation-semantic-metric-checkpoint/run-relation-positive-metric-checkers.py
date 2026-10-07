import hashlib
import json
import pathlib
import subprocess
import time

b = pathlib.Path(__file__).parent
r = pathlib.Path('/private/tmp/mpk-w09-refreeze-metadata-integration')
out = b / 'relation-positive-metric-checkers'
out.mkdir(exist_ok=False)
h = lambda p: hashlib.sha256(p.read_bytes()).hexdigest()
read = lambda p: json.loads(p.read_bytes())
source = b / 'ordinary-relations-metrics-regeneration-2/goldens/relations/construction-vc-positive_constructor.hex'
data = bytes.fromhex(source.read_text())
expected = 'e7183e2eb0a8b5f03b4c4eb70295750e5a31d2efb3f6ddb0b6b1ad9c26ed8e99'
assert hashlib.sha256(b'MPK-MODULE-CERT-0.1\0' + data).hexdigest() == expected
input_path = out / 'positive-constructor.mpcert'
input_path.write_bytes(data)
plan = read(b / 'ordinary-checker-plan-1-final.json')
selection = ('Select only the positive-constructor certificate whose complete metrics alias '
             'adds the already-current Bool carrier and four real semantic observations. '
             'All21 regenerated certificates exactly match current pins;20 unchanged '
             'relation cases require no repeated checker run. Independently confirm this '
             'current78-declaration certificate on both immutable source-free binaries, '
             'with matching module/certificate/export/axiom hashes and zero axioms. '
             'Whole gates stay final T01-W10/T06-W12; no application proof is discharged.')
state = {'status': 'running', 'selection_reason': selection, 'stages': [],
         'application_proof_ids_pending': 987, 'binaries': plan['binaries']}
def save():
    (out / 'status.json').write_text(json.dumps(state, sort_keys=True, indent=2) + '\n')
def verify():
    for path, sha in plan['source_hashes'].items():
        assert h(r / path) == sha, path
    for binary in plan['binaries'].values():
        assert h(pathlib.Path(binary['path'])) == binary['sha256']
save()
try:
    verify()
    results = []
    for kind, binary in plan['binaries'].items():
        label = 'positive-constructor-' + kind
        report = out / (label + '.json')
        stderr = out / (label + '.stderr.txt')
        start = time.monotonic()
        with report.open('wb') as stdout, stderr.open('wb') as err:
            proc = subprocess.Popen([binary['path'], 'check' if kind == 'rust' else 'verify', str(input_path)], stdout=stdout, stderr=err)
            state['stage'] = label
            state['process_pid'] = proc.pid
            save()
            code = proc.wait()
        result = read(report)
        state['stages'].append({'stage': label, 'backend': kind, 'exit_code': code,
            'verdict': result['verdict'], 'input_sha256': h(input_path),
            'report_sha256': h(report), 'stderr_sha256': h(stderr),
            'elapsed_seconds': time.monotonic() - start})
        save()
        assert code == 0 and result['verdict'] == 'accepted'
        assert result['hashes']['certificate'] == expected
        if kind == 'rust':
            assert all(value == 0 for value in result['axiom_report']['summary'].values())
        results.append({key: result[key] for key in ['module', 'hashes']})
        verify()
    assert results[0] == results[1]
    state['status'] = 'passed_positive_constructor_current_certificate_both_checkers_matching_zero_axiom_reports'
except BaseException as error:
    state['status'] = 'failed'
    state['error'] = repr(error)
    raise
finally:
    save()
print(state['status'], flush=True)
