"""Observe one original intended-positive candidate with the unchanged Go core."""
from pathlib import Path
import datetime
import hashlib
import json
import subprocess
import sys
import time

root = Path(__file__).resolve().parent.relative_to(Path.cwd().resolve())
attempt = sys.argv[1]
assert attempt.replace('-', '').isalnum()
out = root / attempt / 'go-candidate'
out.mkdir(exist_ok=False)
def seal(path):
    path = Path(path)
    raw = path.read_bytes()
    return {'path': str(path), 'sha256': hashlib.sha256(raw).hexdigest(), 'bytes': len(raw)}
previous_path = root.parent / 'core-interfaces/paired/result.json'
previous = json.loads(previous_path.read_text())
binary = previous['go_binary']
assert seal(binary['path']) == binary
for source in previous['go_production_sources']:
    assert seal(source['path']) == source
certificate = root / attempt / 'rust-construction/certificates/word-division-foundation.mpcert'
rust_path = certificate.with_suffix('.rust.json')
rust = json.loads(rust_path.read_text())
command = [binary['path'], 'verify', str(certificate)]
start = time.monotonic()
run = subprocess.run(command, stdout=subprocess.PIPE, stderr=subprocess.PIPE, check=False, timeout=60)
(out / 'stdout.log').write_bytes(run.stdout)
(out / 'stderr.log').write_bytes(run.stderr)
go = json.loads(run.stdout)
assert not run.stderr and run.returncode == (0 if go['verdict'] == 'accepted' else 1)
record = {
    'status': 'direct_candidate_observed',
    'observed_at_utc': datetime.datetime.now(datetime.timezone.utc).isoformat(),
    'command': command, 'exit_code': run.returncode, 'seconds': time.monotonic() - start,
    'certificate': seal(certificate), 'original_Rust_report': seal(rust_path),
    'unchanged_go_binary': binary, 'unchanged_go_source_build_evidence': seal(previous_path),
    'stdout': seal(out / 'stdout.log'), 'stderr': seal(out / 'stderr.log'),
    'Rust_observation': rust, 'Go_observation': go,
    'semantic_negative_counted': False, 'complete_paired_success_claimed': False,
    'resource_or_Linux_release_claimed': False,
    'scope': 'One original intended-positive word-division certificate. Preserve both direct outcomes; full paired acceptance still requires both unchanged checkers to accept the same final bytes. A format/fuel/resource failure supplies no semantic negative.',
}
(out / 'result.json').write_text(json.dumps(record, indent=2) + '\n')
print(json.dumps({'status': record['status'], 'Go_verdict': go['verdict'], 'seconds': record['seconds']}))
