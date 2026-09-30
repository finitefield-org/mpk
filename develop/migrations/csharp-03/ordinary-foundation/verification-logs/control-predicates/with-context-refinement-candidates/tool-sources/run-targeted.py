"""Verify complete candidate generation, contextual Boolean proofs and affected consumers.

The contextual units use actual equality proof terms under free branch binders;
missing facts fail and wrong proofs reject. A small shared DAG checks generator
bounds without declaring kernel acceptance. Candidate units require every named
original refinement and an unchanged certificate prefix. The free-selector unit
and both original ownership/default consumers use the changed normalizer; their
original pins must remain identical. These are eight distinct targeted tests.
The prior fixed-source 18-context packed replay is already terminal and its
production generators are unchanged here, so it is retained. Clippy includes
library and test targets. Source/application proofs remain pending; the original
is_binding probe still cannot produce its seven required refinements. The full
T06 gate is deferred to T06-W12.
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


def now():
    return datetime.now(timezone.utc).isoformat()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--repo', type=Path, required=True)
    parser.add_argument('--reports', type=Path, required=True)
    parser.add_argument('--manifest', type=Path, required=True)
    parser.add_argument('--target', type=Path, required=True)
    parser.add_argument('--cargo', default='cargo')
    args = parser.parse_args()
    args.reports.mkdir(parents=True, exist_ok=True)
    manifest = json.loads(args.manifest.read_bytes())
    state = dict(status='preparing', supervisor_pid=os.getpid(), started_at=now(),
                 stages=[], selection_reason=__doc__, application_proofs_pending=987,
                 original_pattern_predicates_defined=102,
                 original_pattern_predicates_pending=0,
                 native_source_equivalence_proofs_pending=True,
                 full_t_gate='deferred to T06-W12')

    def persist():
        path = args.reports / 'status.tmp'
        path.write_text(json.dumps(state, indent=2, sort_keys=True) + '\n')
        os.replace(path, args.reports / 'status.json')

    def verify():
        for group in ('source_hashes', 'fixture_hashes'):
            for name, digest in manifest[group].items():
                assert sha256((args.repo / name).read_bytes()).hexdigest() == digest, name

    persist()
    try:
        verify()
        (args.reports / 'source-manifest.json').write_bytes(args.manifest.read_bytes())
        state.update(verified_source_hashes=len(manifest['source_hashes']),
                     verified_fixture_hashes=len(manifest['fixture_hashes']))
        env = os.environ.copy()
        env.update(CARGO_TARGET_DIR=str(args.target), CARGO_BUILD_JOBS='2', CARGO_INCREMENTAL='0',
                   CARGO_PROFILE_TEST_OPT_LEVEL='2', CARGO_PROFILE_TEST_DEBUG='0',
                   CARGO_PROFILE_TEST_DEBUG_ASSERTIONS='true', CARGO_PROFILE_TEST_OVERFLOW_CHECKS='true')
        for name in ('MPK_W09_CONTROL_PATTERN_ROUTE_OUTPUT', 'MPK_W09_CONTROL_PATTERN_SOURCE_OUTPUT', 'MPK_W09_PATTERN_OBSERVATION_OUTPUT',
                     'MPK_W09_CONTROL_PATTERN_CAPTURE_OUTPUT', 'MPK_W09_CONTROL_PATTERN_SCOPE_OUTPUT',
                     'MPK_W09_CONTROL_PREDICATE_INTEGRATED_OUTPUT', 'MPK_W09_CONTROL_PREDICATE_OUTPUT'):
            env.pop(name, None)
        env.pop('MPK_W09_PATTERN_PROOF_TYPES_OUTPUT', None)
        env.pop('MPK_W09_PATTERN_PROOF_TYPES_UNIT_OUTPUT', None)
        env.pop('MPK_W09_PACKED_PATTERN_OUTPUT', None)
        env.pop('MPK_W09_PACKED_PATTERN_UNIT_OUTPUT', None)
        env.pop('MPK_W09_PATTERN_CANDIDATE_UNIT_OUTPUT', None)
        env.pop('MPK_W09_CONTEXT_BOOLEAN_UNIT_OUTPUT', None)
        env.pop('MPK_W09_OWNERSHIP_PROOFS_OUT', None)
        env.pop('MPK_W09_DEFAULT_CLOSED_PROOFS_OUT', None)
        test = ['test', '-p', 'mpk-vc', '--test', 'csharp_practical_vc']
        tail = ['--', '--nocapture', '--test-threads=1']
        jobs = [
            ('context-units', ['test', '-p', 'mpk-vc', '--lib', 'context_boolean_normalization_'] + tail, 2),
            ('candidate-units', ['test', '-p', 'mpk-vc', '--lib', 'pattern_refinement_candidate_'] + tail, 3),
            ('ownership-unit', ['test', '-p', 'mpk-vc', '--lib', 'ownership_normalization_preserves_free_selector_under_binder'] + tail, 1),
            ('ownership-consumer', test + ['csharp_03_t06_w09_ownership_proof_candidates'] + tail, 1),
            ('closed-default-consumer', test + ['csharp_03_t06_w09_binding_defaults_closed_boolean_proof_candidates'] + tail, 1),
            ('clippy', ['clippy', '-p', 'mpk-vc', '--lib', '--tests', '--', '-D', 'warnings'], 0),
            ('format', ['fmt', '-p', 'mpk-vc', '--', '--check'], 0),
        ]
        binaries = {}
        for label, arguments, count in jobs:
            verify()
            state.update(status='running', stage=label)
            started = now()
            start = time.monotonic()
            log = args.reports / f'{label}.log'
            with log.open('wb') as out:
                process = subprocess.Popen([args.cargo, *arguments], cwd=args.repo, env=env,
                                           stdin=subprocess.DEVNULL, stdout=out, stderr=subprocess.STDOUT)
                state['process_pid'] = process.pid
                persist()
                code = process.wait()
            state['stages'].append(dict(stage=label, command=[args.cargo, *arguments],
                                       exit_code=code, started_at=started, finished_at=now(),
                                       elapsed_seconds=round(time.monotonic() - start, 3),
                                       log_sha256=sha256(log.read_bytes()).hexdigest()))
            persist()
            assert code == 0, (label, code)
            if count:
                output = log.read_text()
                assert f'test result: ok. {count} passed; 0 failed;' in output, label
                for name in re.findall(r'Running [^\n]*\(([^\n]+)\)', output):
                    binaries[name] = sha256(Path(name).read_bytes()).hexdigest()
            verify()
        state.update(status='passed_targeted_tests_lint_format', finished_at=now(),
                     unique_tests_passed=8, test_binaries=binaries)
        persist()
    except BaseException as error:
        state.update(status='failed', finished_at=now(), error=repr(error))
        persist()
        raise


if __name__ == '__main__':
    main()
