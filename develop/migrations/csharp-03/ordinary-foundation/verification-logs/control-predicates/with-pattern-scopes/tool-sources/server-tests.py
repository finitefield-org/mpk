"""Run the affected pattern scope tests and lint on an exact Linux snapshot.

The three predicate modes exercise predecessor compatibility and strict imports.
The scope unit test exhausts its Boolean composition and binder-limit fallback.
Unchanged parser, measure and data/control/exception emitters were covered by the
preceding checkpoint and are not repeated. The full T06 gate is deferred to W12.
"""
import argparse
from datetime import datetime, timezone
from hashlib import sha256
import json
import os
from pathlib import Path
import re
import subprocess
import tarfile
import time


def now():
    return datetime.now(timezone.utc).isoformat()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--repo', type=Path, required=True)
    parser.add_argument('--reports', type=Path, required=True)
    parser.add_argument('--manifest', type=Path, required=True)
    parser.add_argument('--sources', type=Path, required=True)
    parser.add_argument('--pins', type=Path, required=True)
    parser.add_argument('--target', type=Path, required=True)
    args = parser.parse_args()
    args.reports.mkdir(parents=True, exist_ok=True)
    state = {'status': 'preparing', 'supervisor_pid': os.getpid(),
             'started_at': now(), 'stages': []}

    def persist():
        temporary = args.reports / 'status.tmp'
        temporary.write_text(json.dumps(state, indent=2, sort_keys=True) + '\n')
        os.replace(temporary, args.reports / 'status.json')

    persist()
    try:
        manifest = json.loads(args.manifest.read_bytes())
        head = subprocess.check_output(['/usr/bin/git', 'rev-parse', 'HEAD'], cwd=args.repo, text=True).strip()
        assert head == manifest['base_head'], head
        with tarfile.open(args.sources) as archive:
            assert sorted(archive.getnames()) == sorted(manifest['changed_sources'])
            for member in archive.getmembers():
                assert member.isfile() and not Path(member.name).is_absolute()
                assert '..' not in Path(member.name).parts
                (args.repo / member.name).write_bytes(archive.extractfile(member).read())
        pins = args.repo / 'develop/migrations/csharp-03/ordinary-foundation/control-predicates/with-pattern-scopes'
        pins.mkdir(parents=True, exist_ok=True)
        with tarfile.open(args.pins) as archive:
            assert sorted(archive.getnames()) == sorted(manifest['fixture_hashes'])
            for member in archive.getmembers():
                assert member.isfile() and Path(member.name).name == member.name
                (pins / member.name).write_bytes(archive.extractfile(member).read())

        def verify():
            for name, digest in manifest['source_hashes'].items():
                assert sha256((args.repo / name).read_bytes()).hexdigest() == digest, name
            for name, digest in manifest['fixture_hashes'].items():
                assert sha256((pins / name).read_bytes()).hexdigest() == digest, name

        verify()
        (args.reports / 'source-manifest.json').write_bytes(args.manifest.read_bytes())
        state.update(base_head=head, verified_source_hashes=len(manifest['source_hashes']),
                     verified_pin_hashes=len(manifest['fixture_hashes']))
        env = os.environ.copy()
        env.update(CARGO_TARGET_DIR=str(args.target), CARGO_BUILD_JOBS='2', CARGO_INCREMENTAL='0',
                   CARGO_PROFILE_TEST_OPT_LEVEL='2', CARGO_PROFILE_TEST_DEBUG='0',
                   CARGO_PROFILE_TEST_DEBUG_ASSERTIONS='true', CARGO_PROFILE_TEST_OVERFLOW_CHECKS='true')
        jobs = [
            ('predicate-regressions', ['test', '-p', 'mpk-vc', '--test', 'csharp_practical_vc', 'csharp_03_t06_w09_control_predicates_', '--', '--nocapture', '--test-threads=1']),
            ('scope-composition', ['test', '-p', 'mpk-vc', '--lib', 'pattern_scopes::tests', '--', '--nocapture', '--test-threads=1']),
            ('clippy', ['clippy', '-p', 'mpk-vc', '--lib', '--test', 'csharp_practical_vc', '--', '-D', 'warnings']),
            ('format', ['fmt', '-p', 'mpk-vc', '--', '--check']),
        ]
        binaries = {}
        for label, arguments in jobs:
            state.update(status='running', stage=label)
            started = now()
            start = time.monotonic()
            log = args.reports / f'{label}.log'
            with log.open('wb') as out:
                process = subprocess.Popen(['/usr/bin/cargo', *arguments], cwd=args.repo, env=env,
                                           stdin=subprocess.DEVNULL, stdout=out, stderr=subprocess.STDOUT,
                                           start_new_session=True)
                state['process_pid'] = process.pid
                persist()
                code = process.wait()
            state['stages'].append({'stage': label, 'command': ['/usr/bin/cargo', *arguments],
                                    'exit_code': code, 'started_at': started, 'finished_at': now(),
                                    'elapsed_seconds': round(time.monotonic() - start, 3),
                                    'log_sha256': sha256(log.read_bytes()).hexdigest()})
            persist()
            assert code == 0, (label, code)
            if label in ('predicate-regressions', 'scope-composition'):
                text = log.read_text()
                count = 3 if label == 'predicate-regressions' else 1
                assert f'test result: ok. {count} passed' in text, label
                for path in re.findall(r'Running [^\n]*\(([^\n]+)\)', text):
                    binaries[path] = sha256(Path(path).read_bytes()).hexdigest()
            verify()
        state.update(status='passed_targeted_linux_tests_lint_format', finished_at=now(),
                     test_binaries=binaries, unique_tests_passed=4,
                     application_proofs='pending', full_t_gate='deferred to T06-W12')
        persist()
    except BaseException as error:
        state.update(status='failed', finished_at=now(), error=repr(error))
        persist()
        raise


if __name__ == '__main__':
    main()
