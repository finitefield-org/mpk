"""Scoped ordinary word/Nat conservation checks, with complete raw diagnostics."""
from pathlib import Path
import datetime
import hashlib
import json
import os
import subprocess
import sys
import time

repo = Path.cwd()
root = Path(__file__).resolve().parent.relative_to(repo.resolve())
mode, attempt = sys.argv[1:3]
assert mode in ['rust-tests', 'clippy', 'format']
assert attempt.replace('-', '').isalnum()
out = root / attempt / mode
out.mkdir(parents=True, exist_ok=False)

def seal(path):
    path = Path(path)
    raw = path.read_bytes()
    return {'path': str(path), 'sha256': hashlib.sha256(raw).hexdigest(), 'bytes': len(raw)}

sources = [
    'crates/mpk-vc/src/go_practical_ordinary.rs',
    *['crates/mpk-vc/src/go_practical_ordinary/' + part + '.rs' for part in
      ['syntax', 'natural', 'integer', 'polynomial', 'range', 'word', 'word_proofs', 'word_numeric']],
    'crates/mpk-vc/tests/go_practical_word_numeric.rs',
    'crates/mpk-vc/src/go_practical_ordinary/word_numeric/signed.rs',
    'crates/mpk-vc/tests/support/go_word_numeric_controls.rs',
    'crates/mpk-vc/tests/support/go_word_numeric_observations.rs',
    'crates/mpk-vc/tests/support/go_word_eval.rs',
    'develop/migrations/go-practical/t01-w04/word-numeric-bridge/queries.json',
    *['proofs/std/' + part + '/std-' + part + '.hex' for part in ['bool', 'eq', 'nat']],
]
for source in sources:
    dest = out / 'inputs' / source
    dest.parent.mkdir(parents=True, exist_ok=True)
    dest.write_bytes(Path(source).read_bytes())
overrides = {
    'CARGO_NET_OFFLINE': 'true', 'CARGO_PROFILE_TEST_OPT_LEVEL': '1',
    'CARGO_PROFILE_TEST_DEBUG_ASSERTIONS': 'true', 'CARGO_PROFILE_TEST_OVERFLOW_CHECKS': 'true',
    'CARGO_INCREMENTAL': '0', 'RUST_TEST_THREADS': '1',
}
env = os.environ.copy()
env.update(overrides)
for key in ['RUST_MIN_STACK', 'MPK_CORE_MAX_STEPS', 'MPK_CORE_MAX_SECONDS']:
    env.pop(key, None)
target = 'go_practical_word_numeric'
if mode == 'rust-tests':
    overrides['MPK_GO_WORD_NUMERIC_TEST_OUTPUT'] = str((out / 'certificates').resolve())
    overrides['MPK_GO_WORD_NUMERIC_MODEL_OUTPUT'] = str((out / 'model').resolve())
    env.update(overrides)
    command = ['cargo', 'test', '--offline', '--locked', '-p', 'mpk-vc', '--test', target,
               '--', '--nocapture', '--test-threads=1', *sys.argv[3:]]
elif mode == 'clippy':
    command = ['cargo', 'clippy', '--offline', '--locked', '-p', 'mpk-vc', '--lib',
               '--test', target, '--', '-D', 'warnings']
else:
    command = ['rustfmt', '--check', '--edition', '2021', '--config', 'skip_children=true',
               *[sources[i] for i in [0, 4, 6, 7, 8, 9, 10, 11, 12]]]
record = {
    'schema': 'mpk.go_practical.word_numeric_check.v1', 'status': 'running',
    'started_at_utc': datetime.datetime.now(datetime.timezone.utc).isoformat(),
    'command': command, 'environment': overrides, 'sources': [seal(p) for p in sources],
    'selection_reason': 'Fresh balanced unbounded Nat values/shared powers, pointwise value congruence, actual8/16/32/64-bit ripple-block addition conservation and conditional signed cross sums/balanced-Int addition. Preserve the complete preceding word graph and raw bytes. Scope to four isolated integration roots: exact component/import,23 fully reencoded wrong-value/actual-bit/overflow-premise/final-carry controls, and actual Boolean-circuit execution on all288 frozen numeric queries. Only local flags/addresses use Bool elimination. No prototype certificate, source/application authority, CSharp test, intermediate full gate or unaffected checker matrix.',
    'CSharp_tests_executed': False,
    'full_T02_gate': 'deferred to T02-W04: check-fast.sh --skip-csharp',
    'T02_W01_complete': False,
}
result = out / 'result.json'
result.write_text(json.dumps(record, indent=2) + '\n')
start = time.monotonic()
with (out / 'stdout.log').open('wb') as stdout, (out / 'stderr.log').open('wb') as stderr:
    code = subprocess.run(command, cwd=repo, env=env, stdout=stdout, stderr=stderr, check=False).returncode
record.update(status='passed' if code == 0 else 'failed', exit_code=code,
              wall_seconds=time.monotonic() - start,
              finished_at_utc=datetime.datetime.now(datetime.timezone.utc).isoformat(),
              stdout=seal(out / 'stdout.log'), stderr=seal(out / 'stderr.log'))
if mode == 'rust-tests':
    record['certificates'] = [seal(p) for p in sorted((out / 'certificates').glob('*.mpcert'))]
result.write_text(json.dumps(record, indent=2) + '\n')
print(json.dumps({key: record[key] for key in ['status', 'exit_code', 'wall_seconds']}), flush=True)
raise SystemExit(0 if code == 0 else 1)
