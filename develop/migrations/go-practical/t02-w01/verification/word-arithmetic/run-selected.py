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
assert mode in ['rust-tests', 'rust-controls', 'rust-diagnostic', 'rust-construction', 'clippy', 'format']
assert attempt.replace('-', '').isalnum()
out = root / attempt / mode
out.mkdir(parents=True, exist_ok=False)

def seal(path):
    path = Path(path)
    raw = path.read_bytes()
    return {'path': str(path), 'sha256': hashlib.sha256(raw).hexdigest(), 'bytes': len(raw)}

sources = [
    'crates/mpk-vc/src/go_practical_ordinary.rs',
    *[str(p) for p in sorted(Path('crates/mpk-vc/src/go_practical_ordinary').rglob('*.rs'))],
    'crates/mpk-vc/tests/go_practical_word_arithmetic.rs',
    *[str(p) for p in sorted(Path('crates/mpk-vc/tests').glob('go_practical_word_arithmetic_diagnostic.rs'))],
    *[str(p) for p in sorted(Path('crates/mpk-vc/tests/support').glob('go_word_arithmetic_*.rs'))],
    'crates/mpk-vc/tests/support/go_word_eval.rs',
    'develop/migrations/go-practical/t01-w04/word-foundation/queries.json',
    'develop/probes/go-practical/arithmetic_observer.go',
    'develop/migrations/go-practical/t02-w01/verification/word-arithmetic/queries.json',
    'develop/migrations/go-practical/t02-w01/verification/word-arithmetic/query-origins.json',
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
if mode == 'rust-construction':
    overrides['CARGO_PROFILE_TEST_OPT_LEVEL'] = '0'
env = os.environ.copy()
env.update(overrides)
for key in ['RUST_MIN_STACK', 'MPK_CORE_MAX_STEPS', 'MPK_CORE_MAX_SECONDS']:
    env.pop(key, None)
if mode == 'rust-diagnostic':
    overrides['CARGO_PROFILE_TEST_OPT_LEVEL'] = '0'
    if 'MPK_GO_WORD_ARITHMETIC_DIAGNOSTIC_INPUT' in env:
        overrides['MPK_GO_WORD_ARITHMETIC_DIAGNOSTIC_INPUT'] = env['MPK_GO_WORD_ARITHMETIC_DIAGNOSTIC_INPUT']
    elif attempt.startswith('shared-vectors'):
        overrides['MPK_GO_WORD_ARITHMETIC_DIAGNOSTIC_INPUT'] = str(root / 'shared-vectors/rust-construction/certificates/word-arithmetic-foundation.mpcert')
    env.update(overrides)
target = 'go_practical_word_arithmetic'
if mode in ['rust-tests', 'rust-controls', 'rust-diagnostic', 'rust-construction']:
    if mode == 'rust-controls':
        overrides['MPK_GO_WORD_ARITHMETIC_CONTROL_SELECTION'] = 'zero-witness-and-final-shift'
    if mode == 'rust-diagnostic':
        target += '_diagnostic'
        overrides['MPK_GO_WORD_ARITHMETIC_DIAGNOSTIC_OUTPUT'] = str((out / 'prefix-diagnosis.json').resolve())
    overrides['MPK_GO_WORD_ARITHMETIC_TEST_OUTPUT'] = str((out / 'certificates').resolve())
    overrides['MPK_GO_WORD_ARITHMETIC_MODEL_OUTPUT'] = str((out / 'model').resolve())
    env.update(overrides)
    command = ['cargo', 'test', '--offline', '--locked', '-p', 'mpk-vc', '--test', target,
               '--', '--nocapture', '--test-threads=1', *sys.argv[3:]]
    if mode == 'rust-controls':
        command.append('false_shift_product_correspondence_and_discarded_value_claims_reject')
elif mode == 'clippy':
    command = ['cargo', 'clippy', '--offline', '--locked', '-p', 'mpk-vc', '--lib',
               '--test', target, '--', '-D', 'warnings']
else:
    command = ['rustfmt', '--check', '--edition', '2021', '--config', 'skip_children=true',
               *[p for p in sources if p.endswith('.rs')]]
record = {
    'schema': 'mpk.go_practical.word_arithmetic_check.v1', 'status': 'running',
    'started_at_utc': datetime.datetime.now(datetime.timezone.utc).isoformat(),
    'command': command, 'environment': overrides, 'sources': [seal(p) for p in sources],
    'selection_reason': 'Fresh unbounded PowerTwo/weight/digit-prefix laws, discarded-bit conservation for actual PushLow and all fixed Left0..width-1, selected addends and actual ripple carries; computed Nat witness proves full8/16/32/64-bit actual multiplication conservation with arbitrary initial Nat. Keep frozen4096-declaration/131072-term/2MiB bounds and all core fuel/rules. Exact retained2579-declaration numeric basis. Scope to four new isolated Rust roots: complete exact component/import,23 typed wrong-value/shift-premise/bit/witness/omitted-contribution controls, and actual48 original frozen Mul inputs/every1440 fixed-shift/product/carry stage. Only Boolean primitives execute; host derives finite scalar witnesses while universal Nat proofs check separately. No CSharp tests, unrelated matrices or intermediate full gate.',
    'construction_diagnostic_only': mode == 'rust-construction',
    'CSharp_tests_executed': False,
    'full_T02_gate': 'deferred to T02-W04: check-fast.sh --skip-csharp',
    'T02_W01_complete': False,
}
if mode == 'rust-controls':
    record['selection_reason'] = 'The complete all-width arithmetic/import/retained-basis root, three generic controls, and actual48-query/1440-stage Boolean model pass. One late64-bit deliberately wrong witness (drop final carry in its definition) exhausts Rust conversion fuel; preserve it as resource failure, never semantic negative. Change only this control branch to the distinctly wrong constant-zero witness at all four widths and run those four cases plus the single previously unreached64-bit shift-discard case. Retain the eighteen unaffected typed rejections and all three positive/current production/model bytes; no checker/producer/fuel/bound change or rerun of unrelated roots.'
result = out / 'result.json'
result.write_text(json.dumps(record, indent=2) + '\n')
start = time.monotonic()
with (out / 'stdout.log').open('wb') as stdout, (out / 'stderr.log').open('wb') as stderr:
    code = subprocess.run(command, cwd=repo, env=env, stdout=stdout, stderr=stderr, check=False).returncode
record.update(status='passed' if code == 0 else 'failed', exit_code=code,
              wall_seconds=time.monotonic() - start,
              finished_at_utc=datetime.datetime.now(datetime.timezone.utc).isoformat(),
              stdout=seal(out / 'stdout.log'), stderr=seal(out / 'stderr.log'))
if mode in ['rust-tests', 'rust-controls', 'rust-diagnostic', 'rust-construction']:
    record['certificates'] = [seal(p) for p in sorted((out / 'certificates').glob('*.mpcert'))]
result.write_text(json.dumps(record, indent=2) + '\n')
print(json.dumps({key: record[key] for key in ['status', 'exit_code', 'wall_seconds']}), flush=True)
raise SystemExit(0 if code == 0 else 1)
