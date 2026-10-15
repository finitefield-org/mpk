"""Run the affected native/predicate and parser/handoff tests on Linux."""
from datetime import datetime, timezone
from hashlib import sha256
import json
import os
from pathlib import Path
import re
import subprocess
import tarfile
import time

REPO = Path('/root/mpk-w09-control-predicates')
ROOT = Path('/root/mpk-w09-control-predicates-integration-tests')
MANIFEST = Path('/root/server-test-manifest.json')
STATE = {'status': 'preparing', 'stages': []}


def now():
    return datetime.now(timezone.utc).isoformat()


def persist():
    tmp = ROOT / 'status.tmp'
    tmp.write_text(json.dumps(STATE, indent=2, sort_keys=True) + '\n')
    os.replace(tmp, ROOT / 'status.json')


def main():
    ROOT.mkdir(parents=True, exist_ok=True)
    STATE.update(supervisor_pid=os.getpid(), started_at=now())
    persist()
    manifest = json.loads(MANIFEST.read_bytes())
    head = subprocess.check_output(['/usr/bin/git', 'rev-parse', 'HEAD'], cwd=REPO, text=True).strip()
    assert head == manifest['base_head'], head
    pins = REPO / 'develop/migrations/csharp-03/ordinary-foundation/control-predicates/with-execution'
    pins.mkdir(parents=True, exist_ok=True)
    with tarfile.open('/root/server-pins.tar') as archive:
        for member in archive.getmembers():
            assert member.isfile() and Path(member.name).name == member.name
            (pins / member.name).write_bytes(archive.extractfile(member).read())
    for name, digest in manifest['source_hashes'].items():
        assert sha256((REPO / name).read_bytes()).hexdigest() == digest, name
    for name, digest in manifest['fixture_hashes'].items():
        assert sha256((pins / name).read_bytes()).hexdigest() == digest, name
    (ROOT / 'source-manifest.json').write_bytes(MANIFEST.read_bytes())
    STATE.update(base_head=head, verified_source_hashes=len(manifest['source_hashes']),
                 verified_pin_hashes=len(manifest['fixture_hashes']))
    env = os.environ.copy()
    env.update(CARGO_TARGET_DIR='/root/mpk-w09-control-predicates-target',
               CARGO_BUILD_JOBS='2', CARGO_INCREMENTAL='0',
               CARGO_PROFILE_TEST_OPT_LEVEL='2', CARGO_PROFILE_TEST_DEBUG='0',
               CARGO_PROFILE_TEST_DEBUG_ASSERTIONS='true', CARGO_PROFILE_TEST_OVERFLOW_CHECKS='true')
    jobs = [
        ('predicate-regressions', ['test', '-p', 'mpk-vc', '--test', 'csharp_practical_vc', 'csharp_03_t06_w09_control_predicates_', '--', '--nocapture', '--test-threads=1']),
        ('measures', ['test', '-p', 'mpk-vc', '--lib', 'control_predicates::tests', '--', '--nocapture', '--test-threads=1']),
        ('parser', ['test', '-p', 'mpk-vc', '--test', 'csharp_practical_vc', 'csharp_03_t06_w01_scopes_normalization_and_parser_fuzz', '--', '--nocapture', '--test-threads=1']),
        ('data-handoffs', ['test', '-p', 'mpk-vc', '--test', 'csharp_practical_vc', 'csharp_03_t06_w03_', '--', '--nocapture', '--test-threads=1']),
        ('control-handoffs', ['test', '-p', 'mpk-vc', '--test', 'csharp_practical_vc', 'csharp_03_t06_w04_', '--', '--nocapture', '--test-threads=1']),
        ('exception-handoffs', ['test', '-p', 'mpk-vc', '--test', 'csharp_practical_vc', 'csharp_03_t06_w05_original_exception_handlers_and_mutations', '--', '--nocapture', '--test-threads=1']),
    ]
    binaries = {}
    for label, args in jobs:
        STATE.update(status='running', stage=label)
        start = time.monotonic()
        started = now()
        log = ROOT / f'{label}.log'
        with log.open('wb') as out:
            process = subprocess.Popen(['/usr/bin/cargo', *args], cwd=REPO, env=env,
                                       stdin=subprocess.DEVNULL, stdout=out, stderr=subprocess.STDOUT,
                                       start_new_session=True)
            STATE['process_pid'] = process.pid
            persist()
            code = process.wait()
        STATE['stages'].append({'stage': label, 'command': ['/usr/bin/cargo', *args],
                                'exit_code': code, 'started_at': started, 'finished_at': now(),
                                'elapsed_seconds': round(time.monotonic() - start, 3),
                                'log_sha256': sha256(log.read_bytes()).hexdigest()})
        persist()
        assert code == 0, (label, code)
        data = log.read_text()
        assert 'test result: ok.' in data and '0 passed' not in data, label
        for path in re.findall(r'Running [^\n]*\(([^\n]+)\)', data):
            binaries[path] = sha256(Path(path).read_bytes()).hexdigest()
    for name, digest in manifest['source_hashes'].items():
        assert sha256((REPO / name).read_bytes()).hexdigest() == digest, name
    STATE.update(status='passed_targeted_linux_tests', finished_at=now(), test_binaries=binaries,
                 full_t_gate='deferred to T06-W12', application_proofs='pending')
    persist()


if __name__ == '__main__':
    try:
        main()
    except BaseException as error:
        STATE.update(status='failed', error=repr(error), finished_at=now())
        persist()
        raise
