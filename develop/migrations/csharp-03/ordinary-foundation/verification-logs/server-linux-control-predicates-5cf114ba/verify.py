"""Detached targeted Linux validation of the pushed W04 predicate checkpoint."""
from datetime import datetime, timezone
from hashlib import sha256
import json
import os
from pathlib import Path
import re
import subprocess
import time

REPO = Path('/root/mpk-w09-control-predicates')
EVIDENCE = REPO / 'develop/migrations/csharp-03/ordinary-foundation/verification-logs/server-linux-control-predicates-5cf114ba'
TARGET = Path('/root/mpk-w09-control-predicates-target')
HEAD = '5cf114ba63a85d6dec31ffa4b4f9df233d79ccff'
STATE = {'status': 'preparing', 'expected_head': HEAD, 'stages': []}


def now():
    return datetime.now(timezone.utc).isoformat()


def persist():
    path = EVIDENCE / 'status.json'
    tmp = path.with_suffix('.json.tmp')
    tmp.write_text(json.dumps(STATE, indent=2, sort_keys=True) + '\n')
    os.replace(tmp, path)


def main():
    EVIDENCE.mkdir(parents=True, exist_ok=True)
    STATE['supervisor_pid'] = os.getpid()
    STATE['started_at'] = now()
    persist()
    actual = subprocess.check_output(['/usr/bin/git', 'rev-parse', 'HEAD'], cwd=REPO, text=True).strip()
    assert actual == HEAD, actual
    prior = json.loads((REPO / 'develop/migrations/csharp-03/ordinary-foundation/verification-logs/control-predicates/verification.json').read_bytes())
    for name, digest in prior['source_hashes'].items():
        assert sha256((REPO / name).read_bytes()).hexdigest() == digest, name
    STATE['verified_source_hashes'] = len(prior['source_hashes'])
    STATE['head'] = actual
    env = os.environ.copy()
    env.update(CARGO_TARGET_DIR=str(TARGET), CARGO_BUILD_JOBS='2',
               CARGO_PROFILE_TEST_OPT_LEVEL='2', CARGO_PROFILE_TEST_DEBUG='0',
               CARGO_PROFILE_TEST_DEBUG_ASSERTIONS='true',
               CARGO_PROFILE_TEST_OVERFLOW_CHECKS='true', CARGO_INCREMENTAL='0')
    jobs = [
        ('measures', ['test', '-p', 'mpk-vc', '--lib', 'control_predicates::tests', '--', '--nocapture', '--test-threads=1']),
        ('original-sources', ['test', '-p', 'mpk-vc', '--test', 'csharp_practical_vc', 'csharp_03_t06_w09_control_predicates_original_sources', '--', '--nocapture', '--test-threads=1']),
        ('clippy', ['clippy', '-p', 'mpk-vc', '--lib', '--test', 'csharp_practical_vc', '--', '-D', 'warnings']),
        ('format', ['fmt', '-p', 'mpk-vc', '--', '--check']),
    ]
    for label, args in jobs:
        STATE.update(status='running', stage=label)
        started = now()
        start = time.monotonic()
        log = EVIDENCE / f'{label}.log'
        with log.open('wb') as out:
            process = subprocess.Popen(['/usr/bin/cargo', *args], cwd=REPO, env=env,
                                       stdin=subprocess.DEVNULL, stdout=out, stderr=subprocess.STDOUT,
                                       start_new_session=True)
            STATE['process_pid'] = process.pid
            persist()
            code = process.wait()
        row = {'stage': label, 'command': ['/usr/bin/cargo', *args], 'exit_code': code,
               'started_at': started, 'finished_at': now(), 'elapsed_seconds': round(time.monotonic() - start, 3),
               'log_sha256': sha256(log.read_bytes()).hexdigest()}
        STATE['stages'].append(row)
        persist()
        assert code == 0, (label, code)
    for label in ('measures', 'original-sources'):
        data = (EVIDENCE / f'{label}.log').read_text()
        expected = 'test result: ok. 2 passed' if label == 'measures' else 'test result: ok. 1 passed'
        assert expected in data, label
    binaries = {}
    for log in (EVIDENCE / 'measures.log', EVIDENCE / 'original-sources.log'):
        for path in re.findall(r'Running [^\n]*\(([^\n]+)\)', log.read_text()):
            binary = Path(path)
            binaries[str(binary)] = sha256(binary.read_bytes()).hexdigest()
    assert len(binaries) == 2, binaries
    STATE.update(status='passed_scoped_targeted_linux_tests', finished_at=now(),
                 test_binaries=binaries, full_t_gate='deferred to T06-W12',
                 application_proofs='pending; predicate definitions only')
    persist()


if __name__ == '__main__':
    try:
        main()
    except BaseException as error:
        STATE.update(status='failed', error=repr(error), finished_at=now())
        persist()
        raise
