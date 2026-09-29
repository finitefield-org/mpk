"""Targeted Linux lint retry for the native/predicate source snapshot."""
from datetime import datetime, timezone
from hashlib import sha256
import json
import os
from pathlib import Path
import subprocess
import tarfile
import time

REPO = Path('/root/mpk-w09-control-predicates')
ROOT = Path('/root/mpk-w09-control-predicates-integration-lint')
MANIFEST = Path('/root/server-source-manifest.json')
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
    with tarfile.open('/root/server-source.tar') as archive:
        assert sorted(archive.getnames()) == sorted(manifest['changed_sources'])
        for member in archive.getmembers():
            assert member.isfile() and not Path(member.name).is_absolute()
            assert '..' not in Path(member.name).parts
            data = archive.extractfile(member).read()
            (REPO / member.name).write_bytes(data)
    for name, digest in manifest['source_hashes'].items():
        assert sha256((REPO / name).read_bytes()).hexdigest() == digest, name
    (ROOT / 'source-manifest.json').write_bytes(MANIFEST.read_bytes())
    STATE.update(base_head=head, verified_source_hashes=len(manifest['source_hashes']))
    env = os.environ.copy()
    env.update(CARGO_TARGET_DIR='/root/mpk-w09-control-predicates-target',
               CARGO_BUILD_JOBS='2', CARGO_INCREMENTAL='0')
    jobs = [
        ('clippy', ['clippy', '-p', 'mpk-vc', '--lib', '--test', 'csharp_practical_vc', '--', '-D', 'warnings']),
        ('format', ['fmt', '-p', 'mpk-vc', '--', '--check']),
    ]
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
    STATE.update(status='passed_targeted_linux_lint_format', finished_at=now(),
                 full_t_gate='deferred to T06-W12', application_proofs='pending')
    persist()


if __name__ == '__main__':
    try:
        main()
    except BaseException as error:
        STATE.update(status='failed', error=repr(error), finished_at=now())
        persist()
        raise
