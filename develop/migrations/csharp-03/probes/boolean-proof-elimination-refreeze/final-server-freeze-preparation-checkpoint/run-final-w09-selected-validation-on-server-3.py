"""Run only affected final W09 normal owners on the approved server."""
import argparse
import datetime
import hashlib
import json
import os
import pathlib
import platform
import re
import subprocess
import time


parser = argparse.ArgumentParser()
parser.add_argument('--repository', type=pathlib.Path, required=True)
parser.add_argument('--generation', type=pathlib.Path, required=True)
parser.add_argument('--output', type=pathlib.Path, required=True)
args = parser.parse_args()
assert platform.system() == 'Linux'
assert pathlib.Path('/root/mpk-w09-server-e89c2a0f').is_dir()
r, generation, out = args.repository.resolve(), args.generation.resolve(), args.output.resolve()
assert str(r).startswith('/root/mpk-w09-final-freeze-')
assert str(generation).startswith('/root/mpk-w09-final-freeze-')
assert str(out).startswith('/root/mpk-w09-final-w09-')
assert not out.exists()
h = lambda path: hashlib.sha256(path.read_bytes()).hexdigest()
read = lambda path: json.loads(path.read_bytes())
generated = read(generation / 'receipt.json')
assert generated['status'] == 'passed_server_only_final_proof_and_index_evidence_generation_exact709_vectors'
assert generated['all_709_complete_original_vector_rows_unchanged']
assert generated['non_evidence_freeze_behavior_unchanged']
assert generated['core_source_pin_count'] == 36
assert generated['indexed_go_actual_core_results'] == 34
assert generated['indexed_go_actual_capacity_recursor_results'] == 54
for path, sha in generated['generated_files'].items():
    assert h(r / path) == sha, path
checkpoint = r / 'develop/migrations/csharp-03/probes/boolean-proof-elimination-refreeze'
semantic = read(checkpoint / 'full-semantic-source-owner-checkpoint/receipt.json')
assert semantic['original_owner_tests_passed'] == 10
pattern = read(checkpoint / 'pattern-consumer-checkpoint/receipt.json')
wrapper = read(checkpoint / 'wrapper-consumer-checkpoint/receipt.json')
assert pattern['current_consumer_fresh_matching_checker_acceptances'] == 42
assert wrapper['fresh_matching_zero_axiom_checker_acceptances'] == 230
spec = r / 'develop/specs/CSHARP_PRACTICAL_SHARED_ARTIFACTS_V1.md'
assert h(spec) == '0716bf73fac436cbb20167de3e437a1fac1284662e01315bbdb5d5b911418a9c'
selection = ('The final proof evidence updates 31 Boolean cases and 36 source pins, attributes the indexed reference results to their actual source, and regenerates private evidence while preserving all709 complete vectors. Execute six W09 primary tests plus the two original capacity/recursor normal-pin owners, private freeze --check and affected spec Clippy. Completed unrelated W08/native/scalar matrices are excluded; whole gate remains final T01-W10.')
paths = set()
for directory in ['crates', 'develop/probes/csharp-03', 'go-tools/mpk-checker-ref']:
    for path in (r / directory).rglob('*'):
        if path.is_file() and path.suffix in ['.rs', '.toml', '.py', '.go'] and '__pycache__' not in path.parts:
            paths.add(path.relative_to(r).as_posix())
for path in [
    'Cargo.toml', 'Cargo.lock',
    'develop/specs/CSHARP_PRACTICAL_SHARED_ARTIFACTS_V1.md',
    'develop/specs/CSHARP_PRACTICAL_FOUNDATION_V1.md',
    'develop/specs/vectors/csharp-practical-profile-v1.json',
    'develop/specs/vectors/csharp-practical-foundation-v1.json',
    'develop/migrations/csharp-03/foundation/foundation-descriptor.json',
    'develop/migrations/csharp-03/foundation/foundation-definitions.json',
    'develop/migrations/csharp-03/probes/runtime-foundation-data.json',
    'develop/migrations/csharp-03/artifact-consumer-inventory.json',
    'proofs/std/bool/std-bool.hex', 'proofs/std/nat/std-nat.hex',
    'proofs/std/eq/std-eq.hex', 'proofs/std/logic/std-logic.hex',
]:
    paths.add(path)
paths.update(generated['generated_files'])
paths.update('fixtures/core-bool-cases/' + path.name for path in (r / 'fixtures/core-bool-cases').glob('*.hex'))
manifest = {path: h(r / path) for path in sorted(paths)}
out.mkdir()
(out / 'source-manifest.json').write_text(json.dumps({'hashes': manifest, 'selection_reason': selection}, sort_keys=True, indent=2) + '\n')
state = {'status': 'running', 'stages': [], 'execution_host': 'root@162.43.92.154',
         'source_commit': generated['source_commit'], 'supervisor_pid': os.getpid(),
         'started_at_utc': datetime.datetime.now(datetime.timezone.utc).isoformat(),
         'generation_receipt_raw_sha256': h(generation / 'receipt.json'),
         'selection_reason': selection, 'application_proof_ids_pending': 987,
         'full_t01_gate': 'deferred_to_final_W10_on_server', 'full_t06_gate': 'deferred_to_W12_on_server'}


def save():
    temporary = out / 'status.json.tmp'
    temporary.write_text(json.dumps(state, sort_keys=True, indent=2) + '\n')
    temporary.replace(out / 'status.json')


def verify():
    for path, sha in manifest.items():
        assert h(r / path) == sha, path


env = os.environ.copy()
for key in list(env):
    if key.startswith(('MPK_W09_', 'MPK_T06_')) or key == 'MPK_W14_REQUEST_OUTPUT':
        env.pop(key)
env.update(CARGO_TARGET_DIR='/root/mpk-w09-final-freeze-target', CARGO_INCREMENTAL='0',
           CARGO_BUILD_JOBS='2', CARGO_PROFILE_TEST_OPT_LEVEL='2', CARGO_PROFILE_TEST_DEBUG='0',
           CARGO_PROFILE_TEST_DEBUG_ASSERTIONS='true', CARGO_PROFILE_TEST_OVERFLOW_CHECKS='true',
           CARGO_PROFILE_DEV_OPT_LEVEL='2', CARGO_PROFILE_DEV_DEBUG='0',
           CARGO_PROFILE_DEV_DEBUG_ASSERTIONS='true', CARGO_PROFILE_DEV_OVERFLOW_CHECKS='true',
           PYTHONDONTWRITEBYTECODE='1')
commands = [
    ('private-freeze-check', ['python3', 'develop/probes/csharp-03/profile_freeze.py', '--check'], None),
    ('eight-original-w09-owners', ['/usr/bin/cargo', 'test', '-p', 'mpk-vc', '--test',
     'csharp_practical_spec', 'csharp_03_t01_w09_', '--', '--test-threads=1'], '8'),
    ('selected-spec-clippy', ['/usr/bin/cargo', 'clippy', '-p', 'mpk-vc', '--test',
     'csharp_practical_spec', '--', '-D', 'warnings'], None),
]
try:
    for stage, command, expected_count in commands:
        verify()
        state['stage'] = stage
        save()
        start = time.monotonic()
        log = out / (stage + '.log.txt')
        with log.open('wb') as stream:
            process = subprocess.Popen(command, cwd=r, env=env, stdout=stream, stderr=subprocess.STDOUT)
            state['process_pid'] = process.pid
            save()
            code = process.wait()
        counts = re.findall(r'test result: ok\. (\d+) passed; 0 failed;', log.read_text())
        row = {'stage': stage, 'command': command, 'exit_code': code,
               'elapsed_seconds': time.monotonic() - start, 'log_sha256': h(log), 'test_counts': counts}
        state['stages'].append(row)
        save()
        assert code == 0, row
        if expected_count is not None:
            assert counts == [expected_count], row
    verify()
    state.update(status='passed_server_final_w09_eight_original_normal_pin_owners_freeze_check_and_selected_clippy',
                 rust_owner_tests=8, source_or_fixture_mutations=0)
except BaseException as error:
    state.update(status='failed', error=repr(error))
    raise
finally:
    save()
