"""Retain the actual Go observation of the original failed complete stream."""
from pathlib import Path
import hashlib
import json
import subprocess
import time

ROOT = Path(__file__).resolve().parent


def seal(path):
    raw = Path(path).read_bytes()
    return {'path': str(path), 'bytes': len(raw), 'sha256': hashlib.sha256(raw).hexdigest()}


prior_path = ROOT.parent/'core-interfaces/paired/result.json'
prior = json.loads(prior_path.read_bytes())
binary = prior['go_binary']
assert seal(binary['path']) == binary
for source in prior['go_production_sources']:
    assert seal(source['path']) == source
rust_path = ROOT/'first/result.json'
rust = json.loads(rust_path.read_bytes())
assert rust['status'] == 'failed' and rust['whole_source_inputs_unchanged']
certificate = ROOT/'first/certificates/error-values.mpcert'
report = certificate.with_suffix('.rust.json')
out = ROOT/'first-go-observation'
out.mkdir(exist_ok=False)
driver = out/'driver.py'
driver.write_bytes(Path(__file__).read_bytes())
command = [binary['path'], 'verify', str(certificate)]
record = {'status': 'running', 'actual_driver_prelaunch': seal(driver),
          'original_failed_Rust_execution': seal(rust_path), 'certificate': seal(certificate),
          'rust_report': seal(report), 'actual_Go_build_evidence': seal(prior_path),
          'go_binary': binary, 'go_production_sources': prior['go_production_sources'],
          'command': command, 'classification': 'Original resource/conversion failure; never a semantic negative or passing proof.'}
result = out/'result.json'
result.write_text(json.dumps(record, indent=2)+'\n')
start = time.monotonic()
completed = subprocess.run(command, stdout=subprocess.PIPE, stderr=subprocess.PIPE, check=False)
(out/'stdout.log').write_bytes(completed.stdout)
(out/'stderr.log').write_bytes(completed.stderr)
go = json.loads(completed.stdout)
record.update(status='observed', exit_code=completed.returncode, seconds=time.monotonic()-start,
              stdout=seal(out/'stdout.log'), stderr=seal(out/'stderr.log'),
              original_rust_observation=json.loads(report.read_bytes()), original_go_observation=go)
result.write_text(json.dumps(record, indent=2)+'\n')
assert seal(certificate) == record['certificate'] and seal(report) == record['rust_report']
assert seal(binary['path']) == binary
for source in prior['go_production_sources']:
    assert seal(source['path']) == source
assert completed.returncode != 0 and go['verdict'] == 'rejected'
assert 'fuel' in go['error_detail']
print(json.dumps({'status': 'observed', 'go': go}))
