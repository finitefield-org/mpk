"""Revalidate the changed scope composition and promoted pins locally.

The observed candidate run covers all three predicate modes. The promoted-pin
run checks exact final JSON/hex bytes; the unit test exhausts native-premise and
observation conjunction and the combined binder limit. Existing measure/parser/
data/control/exception emitters are unchanged and were checked at 82992f91.
"""
import argparse
from datetime import datetime, timezone
from hashlib import sha256
import json
import os
from pathlib import Path
import re
import subprocess
import time


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--repo', type=Path, required=True)
    parser.add_argument('--reports', type=Path, required=True)
    parser.add_argument('--target', type=Path, required=True)
    parser.add_argument('--candidate-exit', type=int, required=True,
                        help='Actual exit observed from the completed candidate process')
    parser.add_argument('--candidate-session', required=True)
    args = parser.parse_args()
    assert args.candidate_exit == 0
    manifest = json.loads((args.reports / 'source-manifest.json').read_bytes())

    def verify():
        for name, digest in manifest['source_hashes'].items():
            assert sha256((args.repo / name).read_bytes()).hexdigest() == digest, name
        pins = args.repo / 'develop/migrations/csharp-03/ordinary-foundation/control-predicates/with-pattern-scopes'
        for name, digest in manifest['fixture_hashes'].items():
            assert sha256((pins / name).read_bytes()).hexdigest() == digest, name

    verify()
    candidate = args.reports / 'predicate-regressions.log'
    assert 'test result: ok. 3 passed' in candidate.read_text()
    stages = [{'stage': 'predicate-regressions',
               'command': ['cargo', 'test', '-p', 'mpk-vc', '--test', 'csharp_practical_vc',
                           'csharp_03_t06_w09_control_predicates', '--', '--nocapture', '--test-threads=1'],
               'exit_code': args.candidate_exit, 'observed_exec_session': args.candidate_session,
               'output_environment': {'MPK_W09_CONTROL_PATTERN_SCOPE_OUTPUT': str(args.reports)},
               'tests_passed': 3, 'log_sha256': sha256(candidate.read_bytes()).hexdigest()}]
    env = os.environ.copy()
    env.update(CARGO_TARGET_DIR=str(args.target), CARGO_BUILD_JOBS='2', CARGO_INCREMENTAL='0',
               CARGO_PROFILE_TEST_OPT_LEVEL='2', CARGO_PROFILE_TEST_DEBUG='0',
               CARGO_PROFILE_TEST_DEBUG_ASSERTIONS='true', CARGO_PROFILE_TEST_OVERFLOW_CHECKS='true')
    for name in ('MPK_W09_CONTROL_PATTERN_SCOPE_OUTPUT', 'MPK_W09_CONTROL_PREDICATE_OUTPUT',
                 'MPK_W09_CONTROL_PREDICATE_INTEGRATED_OUTPUT'):
        env.pop(name, None)
    jobs = [
        ('scope-composition', ['test', '-p', 'mpk-vc', '--lib', 'pattern_scopes::tests', '--', '--nocapture', '--test-threads=1'], 1),
        ('pinned-scopes', ['test', '-p', 'mpk-vc', '--test', 'csharp_practical_vc', 'csharp_03_t06_w09_control_predicates_with_pattern_scopes_original_sources', '--', '--nocapture', '--test-threads=1'], 1),
        ('clippy', ['clippy', '-p', 'mpk-vc', '--lib', '--test', 'csharp_practical_vc', '--', '-D', 'warnings'], 0),
        ('format', ['fmt', '-p', 'mpk-vc', '--', '--check'], 0),
    ]
    for label, arguments, count in jobs:
        log = args.reports / f'{label}.log'
        start = time.monotonic()
        started = datetime.now(timezone.utc).isoformat()
        with log.open('wb') as out:
            code = subprocess.call(['cargo', *arguments], cwd=args.repo, env=env,
                                   stdin=subprocess.DEVNULL, stdout=out, stderr=subprocess.STDOUT)
        stages.append({'stage': label, 'command': ['cargo', *arguments], 'exit_code': code,
                       'started_at': started, 'elapsed_seconds': round(time.monotonic() - start, 3),
                       'tests_passed': count, 'log_sha256': sha256(log.read_bytes()).hexdigest()})
        (args.reports / 'selected-stages.json').write_text(json.dumps(stages, indent=2, sort_keys=True) + '\n')
        print(label, code, flush=True)
        assert code == 0, (label, code)
        if count:
            assert f'test result: ok. {count} passed' in log.read_text()
        verify()
    binaries = {}
    for row in stages:
        text = (args.reports / f"{row['stage']}.log").read_text()
        for name in re.findall(r'Running [^\n]*\(([^\n]+)\)', text):
            binaries[name] = sha256(Path(name).read_bytes()).hexdigest()
    result = {'status': 'passed_targeted_tests_lint_format', 'unique_tests_passed': 4,
              'test_runs_passed': 5, 'stages': stages, 'binaries': binaries,
              'source_hashes': manifest['source_hashes'], 'fixture_hashes': manifest['fixture_hashes'],
              'selection_reason': manifest['selection_reason'], 'full_t_gate': 'deferred to T06-W12'}
    (args.reports / 'local-verification.json').write_text(json.dumps(result, indent=2, sort_keys=True) + '\n')


if __name__ == '__main__':
    main()
