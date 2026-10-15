"""Verify source operands, represented slot snapshots and affected consumers.

The new regression fixes the 18 original contexts and 102 pattern obligations.
The capture regression reconstructs the historical native/execution/scope pins;
the slot regression covers consumers of the shared nullable storage helper.
Unchanged parser, scalar, codec and theorem checkers are not repeated.
The full T06 gate is deferred to T06-W12.
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
                 original_pattern_predicates_pending=102, full_t_gate='deferred to T06-W12')

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
        for name in ('MPK_W09_PATTERN_OBSERVATION_OUTPUT', 'MPK_W09_CONTROL_PATTERN_CAPTURE_OUTPUT',
                     'MPK_W09_CONTROL_SLOT_OUTPUT'):
            env.pop(name, None)
        test = ['test', '-p', 'mpk-vc', '--test', 'csharp_practical_vc']
        tail = ['--', '--nocapture', '--test-threads=1']
        jobs = [
            ('observations', test + ['csharp_03_t06_w09_pattern_observations_original_sources'] + tail, 1),
            ('capture-compatibility', test + ['csharp_03_t06_w09_control_predicates_with_pattern_captures_original_sources'] + tail, 1),
            ('slot-consumers', test + ['csharp_03_t06_w09_control_slot_source_relations'] + tail, 1),
            ('clippy', ['clippy', '-p', 'mpk-vc', '--lib', '--test', 'csharp_practical_vc', '--', '-D', 'warnings'], 0),
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
                text = log.read_text()
                assert f'test result: ok. {count} passed; 0 failed;' in text, label
                for name in re.findall(r'Running [^\n]*\(([^\n]+)\)', text):
                    binaries[name] = sha256(Path(name).read_bytes()).hexdigest()
            verify()
        state.update(status='passed_targeted_tests_lint_format', finished_at=now(),
                     unique_tests_passed=3, test_binaries=binaries)
        persist()
    except BaseException as error:
        state.update(status='failed', finished_at=now(), error=repr(error))
        persist()
        raise


if __name__ == '__main__':
    main()
