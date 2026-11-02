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
assert mode in ['rust-tests', 'rust-construction', 'clippy', 'format']
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
    'crates/mpk-vc/tests/go_practical_word_division_bounds.rs',
    *[str(p) for p in sorted(Path('crates/mpk-vc/tests/support').glob('go_word_division_bounds*.rs'))],
    'crates/mpk-vc/tests/support/go_word_eval.rs',
    'crates/mpk-vc/tests/support/go_word_division_dependencies.rs',
    'develop/migrations/go-practical/t01-w04/word-foundation/queries.json',
    *['proofs/std/' + p + '/std-' + p + '.hex' for p in ['bool','eq','nat']],
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
target = 'go_practical_word_division_bounds'
if mode in ['rust-tests', 'rust-construction']:
    overrides['MPK_GO_WORD_DIVISION_BOUNDS_TEST_OUTPUT'] = str((out / 'certificates').resolve())
    overrides['MPK_GO_WORD_DIVISION_BOUNDS_MODEL_OUTPUT'] = str((out / 'model').resolve())
    env.update(overrides)
    command = ['cargo', 'test', '--offline', '--locked', '-p', 'mpk-vc', '--test', target,
               '--', '--nocapture', '--test-threads=1', *sys.argv[3:]]
elif mode == 'clippy':
    command = ['cargo', 'clippy', '--offline', '--locked', '-p', 'mpk-vc', '--lib',
               '--test', target, '--', '-D', 'warnings']
else:
    command = ['rustfmt', '--check', '--edition', '2021', '--config', 'skip_children=true',
               *[p for p in sources if p.endswith('.rs')]]
record = {
    'schema': 'mpk.go_practical.word_division_bounds_check.v1', 'status': 'running',
    'started_at_utc': datetime.datetime.now(datetime.timezone.utc).isoformat(),
    'command': command, 'environment': overrides, 'sources': [seal(p) for p in sources],
    'selection_reason': 'Actual all-width quotient-half gaps and strict remainder-half bounds classify source corrections as the actual minimum dividend/negative-unit divisor exception. Prove zero remainder correction, exact optional quotient guard, negative-unit soundness/completeness, and normal minimum/-one source outputs from existing real unsigned/magnitude/refinement proofs. Retain all3897 prior named declarations and exact raw producer; eight additional actual half premises are selected from unchanged full original inputs. Only affected new target/library checks; no CSharp/unrelated suites/intermediate whole gate or final source/application completeness claim. Frozen4096/131072/2097152 ceilings remain.',
    'construction_diagnostic_only': mode == 'rust-construction',
    'CSharp_tests_executed': False,
    'full_T02_gate': 'deferred to T02-W04: check-fast.sh --skip-csharp',
    'T02_W01_complete': False,
}
result = out / 'result.json'
result.write_text(json.dumps(record, indent=2) + '\n')
start = time.monotonic()
with (out / 'stdout.log').open('wb') as stdout, (out / 'stderr.log').open('wb') as stderr:
    code = subprocess.run(command, cwd=repo, env=env, stdout=stdout, stderr=stderr, check=False).returncode
changed_inputs = [source for source in record['sources'] if seal(source['path']) != source]
record.update(status='passed' if code == 0 and not changed_inputs else 'failed', changed_inputs_during_execution=changed_inputs, exit_code=code,
              wall_seconds=time.monotonic() - start,
              finished_at_utc=datetime.datetime.now(datetime.timezone.utc).isoformat(),
              stdout=seal(out / 'stdout.log'), stderr=seal(out / 'stderr.log'))
if mode in ['rust-tests', 'rust-construction']:
    record['certificates'] = [seal(p) for p in sorted((out / 'certificates').glob('*.mpcert'))]
result.write_text(json.dumps(record, indent=2) + '\n')
print(json.dumps({key: record[key] for key in ['status', 'exit_code', 'wall_seconds']}), flush=True)
raise SystemExit(0 if record['status'] == 'passed' else 1)
