import hashlib
import json
import os
from pathlib import Path
import subprocess
import time

root = Path('/private/tmp/mpk-w09-bool-cases-reports')
repo = Path('/private/tmp/mpk-w09-context-application-integration')
out = root / 'dual-checkers-6'
out.mkdir()
manifest = json.loads((root / 'source-manifest-4.json').read_text())
binaries = {'rust': Path('/private/tmp/mpk-w09-bool-cases-target/debug/mpk'),
            'go': root / 'go-local-4/go-checker'}


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def verify():
    for group in ('source_hashes', 'fixture_hashes', 'document_hashes'):
        for name, digest in manifest[group].items():
            assert sha(repo / name) == digest, name


state = dict(status='running', supervisor_pid=os.getpid(), stages=[],
             binaries={k: dict(path=str(v), sha256=sha(v)) for k, v in binaries.items()},
             full_t01_gate='deferred to renewed T01-W10',
             full_t06_gate='deferred to T06-W12', application_scope_pending=True)


def save():
    temporary = out / 'status.json.tmp'
    temporary.write_text(json.dumps(state, indent=2, sort_keys=True) + '\n')
    temporary.replace(out / 'status.json')


save()
try:
    verify()
    positive = {'right-identity', 'constructor-false', 'constructor-true',
                'open-motive', 'conjunction-left', 'conjunction-right'}
    cases = [(p.stem, bytes.fromhex(p.read_text()), p.stem in positive)
             for p in sorted((repo / 'fixtures/core-bool-cases').glob('*.hex'))]
    assert len(cases) == 19
    for name in ('zero-axiom', 'one-theorem'):
        p = repo / 'fixtures/cert-basic' / (name + '.hex')
        cases.append(('predecessor-' + name, bytes.fromhex(p.read_text()), True))
    p = Path('/private/tmp/mpk-w09-context-application-reports/local/certificates/true.mpcert')
    cases.append(('predecessor-std-bool', p.read_bytes(), True))
    for name, data, good in cases:
        path = out / (name + '.mpcert')
        path.write_bytes(data)
        accepted = []
        for backend, binary in binaries.items():
            label = name + '-' + backend
            state['stage'] = label
            save()
            command = [str(binary), 'check' if backend == 'rust' else 'verify', str(path)]
            started = time.monotonic()
            process = subprocess.Popen(command, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
            state['process_pid'] = process.pid
            save()
            stdout, stderr = process.communicate()
            report_path = out / (label + '.json')
            stderr_path = out / (label + '.stderr.txt')
            report_path.write_bytes(stdout)
            stderr_path.write_bytes(stderr)
            report = json.loads(stdout)
            assert process.returncode == (0 if good else 1), (label, report)
            assert report['verdict'] == ('accepted' if good else 'rejected')
            if good:
                if name != 'predecessor-one-theorem':
                    assert report.get('axiom_count', 0) == 0
                    if backend == 'rust':
                        assert all(v == 0 for v in report['axiom_report']['summary'].values())
                assert report['hashes']['certificate'] == hashlib.sha256(b'MPK-MODULE-CERT-0.1\0' + data).hexdigest()
                accepted.append({k: report[k] for k in ('module', 'hashes')})
            else:
                assert report['error_code' if backend == 'rust' else 'error_kind'] == ('KERNEL_CORE_CHECK' if backend == 'rust' else 'core_check')
            state['stages'].append(dict(stage=label, command=command, exit_code=process.returncode,
                                       verdict=report['verdict'], input_sha256=sha(path),
                                       report_sha256=sha(report_path), stderr_sha256=sha(stderr_path),
                                       elapsed_seconds=round(time.monotonic() - started, 3)))
            save()
        if good:
            assert accepted[0] == accepted[1], name
        print(name + ': matched', flush=True)
    verify()
    assert all(sha(v) == state['binaries'][k]['sha256'] for k, v in binaries.items())
    state.update(status='passed_same_byte_checker_corpus', cases=22, checker_stages=44,
                 new_accepted=6, new_rejected=13, predecessors=3,
                 source_files=len(manifest['source_hashes']), fixture_files=len(manifest['fixture_hashes']))
except BaseException as error:
    state.update(status='failed', error=repr(error))
    raise
finally:
    save()
