import hashlib
import json
import os
import pathlib
import shutil
import subprocess

root = pathlib.Path('/private/tmp/mpk-w09-bool-cases-reports/refreeze/registration-order-fix')
repo = pathlib.Path('/private/tmp/mpk-w09-bool-cases-registration-order')
out = root / 'go-local-4'
manifest = json.loads((root / 'source-manifest.json').read_text())


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def verify():
    for group in ('source_hashes', 'fixture_hashes', 'document_hashes'):
        for name, digest in manifest[group].items():
            assert sha(repo / name) == digest, name


state = dict(status='running', supervisor_pid=os.getpid(), stages=[],
             selection_reason=manifest['selection_reason'], application_scope_pending=True)


def save():
    temporary = out / 'status.json.tmp'
    temporary.write_text(json.dumps(state, indent=2, sort_keys=True) + '\n')
    temporary.replace(out / 'status.json')


save()
try:
    verify()
    env = os.environ.copy()
    env.update(GOCACHE='/private/tmp/mpk-w09-bool-cases-registration-order-go-cache',
               GOMODCACHE='/private/tmp/mpk-w09-bool-cases-go-modcache')
    go = shutil.which('go')
    assert go
    for label, command in [
        ('selected-core-defeq-tests', [go, 'test', '-count=1', '-v', '-run', '^Test(BoolCases|Defeq|Core|CheckCore)', './...']),
        ('go-checker-build', [go, 'build', '-o', str(out / 'go-checker'), './cmd/mpk-checker-ref']),
    ]:
        state['stage'] = label
        save()
        log = out / (label + '.log.txt')
        with log.open('wb') as stream:
            process = subprocess.Popen(command, cwd=repo / 'go-tools/mpk-checker-ref', env=env,
                                       stdout=stream, stderr=subprocess.STDOUT)
            state['process_pid'] = process.pid
            save()
            result = process.wait()
        state['stages'].append(dict(stage=label, command=command, exit_code=result, log_sha256=sha(log)))
        save()
        print(label + ': ' + str(result), flush=True)
        assert result == 0, label
    verify()
    state.update(status='passed_selected_go_tests_build', go_checker_sha256=sha(out / 'go-checker'))
except BaseException as error:
    state.update(status='failed', error=repr(error))
    raise
finally:
    save()
