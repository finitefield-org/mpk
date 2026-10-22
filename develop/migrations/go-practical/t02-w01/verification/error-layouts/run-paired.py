"""Check each whole original frozen error-layout stream in the independent Go core."""
from pathlib import Path
import datetime
import hashlib
import json
import re
import subprocess
import sys
import time

ROOT = Path(__file__).resolve().parent
epoch = sys.argv[1]
assert epoch.replace('-', '').isalnum()


def seal(path):
    raw = Path(path).read_bytes()
    return {'path': str(path), 'bytes': len(raw), 'sha256': hashlib.sha256(raw).hexdigest()}


execution = ROOT/epoch/'result.json'
rust_run = json.loads(execution.read_bytes())
assert rust_run['status'] == 'passed' and rust_run['whole_source_inputs_unchanged']
assert '4 passed; 0 failed' in (execution.parent/'stdout.log').read_text()
prior_path = ROOT.parent/'core-interfaces/paired/result.json'
prior = json.loads(prior_path.read_bytes())
binary = prior['go_binary']
assert seal(binary['path']) == binary
for source in prior['go_production_sources']:
    assert seal(source['path']) == source
inputs_path = Path(rust_run['source_receipt']['path'])
assert seal(inputs_path) == rust_run['source_receipt']
inputs = json.loads(inputs_path.read_bytes())
for row in inputs['sources']:
    assert seal(row['path']) == {k:row[k] for k in ['path', 'bytes', 'sha256']}
certificates = sorted((execution.parent/'certificates').glob('*.mpcert'))
assert rust_run['certificates'] == [seal(p) for p in certificates]
positives = {f'layout-N{n}' for n in [0,3,70]} | {f'matrix-N{n}' for n in range(71)} | {'honest-extension'}
negatives = {'wrong-tag', 'empty-carrier-extra-slot', 'wrong-zero-tag'}
negatives |= {f'dropped-N{n}-payload{i}' for n,i in [(3,0),(3,2),(70,0),(70,64),(70,69)]}
negatives |= {f'dropped-N{n}-from-view-tag' for n in [0,3,70]}
assert (len(positives),len(negatives)) == (75,11)
assert {p.stem for p in certificates} == positives | negatives
out = ROOT/(epoch+'-paired')
out.mkdir(exist_ok=False)
driver = out/'driver.py'
driver.write_bytes(Path(__file__).read_bytes())
record = {'status': 'running', 'actual_driver_prelaunch': seal(driver),
          'terminal_Rust_execution': seal(execution), 'original_Rust_inputs': seal(inputs_path),
          'go_binary': binary, 'go_production_sources': prior['go_production_sources'],
          'actual_unchanged_Go_build_evidence': seal(prior_path), 'cases': [],
          'selection_reason': 'Both unchanged independent cores consume exactly the same complete streams. Exact frozen balanced error layouts0..70, empty-U handling, canonical nil and complete payload adapters, both comparison transfers, wrong carrier/zero/tag/payload/adapter definitions. No prefix, evaluator oracle, kernel change, axiom, widened budget or public/source proof claim.',
          'full_T_gate': 'Deferred to final GO-PRAC-T02-W04: ./scripts/check-fast.sh',
          'started_at_utc': datetime.datetime.now(datetime.timezone.utc).isoformat()}


def save():
    (out/'result.json').write_text(json.dumps(record, indent=2)+'\n')


save()
try:
    for certificate in certificates:
        name = certificate.stem
        rust_path = certificate.with_suffix('.rust.json')
        before, rust_before = seal(certificate), seal(rust_path)
        rust = json.loads(rust_path.read_bytes())
        command = [binary['path'], 'verify', str(certificate)]
        started = time.monotonic()
        completed = subprocess.run(command, stdout=subprocess.PIPE, stderr=subprocess.PIPE, check=False)
        stdout, stderr = out/(name+'.stdout.log'), out/(name+'.stderr.log')
        stdout.write_bytes(completed.stdout)
        stderr.write_bytes(completed.stderr)
        go = json.loads(completed.stdout)
        row = {'name': name, 'certificate': before, 'rust_report': rust_before,
               'command': command, 'exit_code': completed.returncode, 'seconds': time.monotonic()-started,
               'stdout': seal(stdout), 'stderr': seal(stderr), 'rust_observation': rust, 'go_observation': go}
        record['cases'].append(row)
        save()
        assert seal(certificate) == before and seal(rust_path) == rust_before and not completed.stderr
        if name in positives:
            assert completed.returncode == 0 and rust['verdict'] == go['verdict'] == 'accepted'
            assert rust['axiom_count'] == 0 and not rust['axiom_report']['entries']
            assert rust['hashes'] == go['hashes'] and rust['declaration_count'] == go['declaration_count']
            assert go['hashes']['axiom_report'] == '0ebc281c3a8d37e2d1a9ce033773e2865f96a13186a6364cb3446204c6a990d5'
        else:
            assert completed.returncode != 0 and rust['verdict'] == go['verdict'] == 'rejected'
            assert rust['error_code'] == 'KERNEL_CORE_CHECK' and go['error_kind'] == 'core_check'
            detail = json.loads(rust['error_detail'])
            assert detail['code'] == 'CORE_TYPE_MISMATCH'
            if detail['details']['kind'] == 'check_type_mismatch':
                inferred, expected = [detail['details'][key] for key in ['inferred_kind', 'expected_kind']]
                assert re.fullmatch(r'term [0-9]+ inferred '+re.escape(inferred)+' but expected '+re.escape(expected), go['error_detail'])
            else:
                assert detail['details']['kind'] == 'lambda_domain_mismatch' and go['error_detail'] == 'lambda domain mismatch'
        print(json.dumps({'name': name, 'verdict': go['verdict'], 'seconds': row['seconds']}), flush=True)
    assert seal(binary['path']) == binary
    for source in prior['go_production_sources']:
        assert seal(source['path']) == source
    for source in inputs['sources']:
        assert seal(source['path']) == {k:source[k] for k in ['path', 'bytes', 'sha256']}
    record.update(status='passed', same_byte_streams=len(certificates), zero_axiom_positives=len(positives),
                  genuine_core_type_mismatches=len(certificates)-len(positives),
                  finished_at_utc=datetime.datetime.now(datetime.timezone.utc).isoformat())
    save()
except BaseException:
    record['status'] = 'failed'
    save()
    raise
