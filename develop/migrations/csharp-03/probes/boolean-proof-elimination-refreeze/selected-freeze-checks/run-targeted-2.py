from pathlib import Path
import os
import subprocess
import hashlib
import json
import re
import time

root = Path('/private/tmp/mpk-w09-bool-cases-reports/refreeze')
repo = Path('/private/tmp/mpk-w09-context-application-integration')
out = root / 'local-2'
out.mkdir()
manifest = json.loads((root / 'source-manifest-2.json').read_text())
state = dict(status='running', supervisor_pid=os.getpid(), stages=[],
             selection_reason=manifest['selection_reason'],
             full_t01_gate='deferred to renewed T01-W10', full_t06_gate='deferred to T06-W12')


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def save():
    temporary = out / 'status.json.tmp'
    temporary.write_text(json.dumps(state, indent=2, sort_keys=True) + '\n')
    temporary.replace(out / 'status.json')


def verify():
    for name, digest in manifest['hashes'].items():
        assert sha(repo / name) == digest, name


save()
try:
    verify()
    env = os.environ.copy()
    env['CARGO_TARGET_DIR'] = '/private/tmp/mpk-w09-bool-cases-target'
    cargo = '/Users/kazuyoshitoshiya/.cargo/bin/cargo'
    jobs = [
        ('w09-spec', [cargo, 'test', '-p', 'mpk-vc', '--test', 'csharp_practical_spec', 'csharp_03_t01_w09_', '--', '--nocapture']),
        ('registry-consumers', [cargo, 'test', '-p', 'mpk-vc', '--test', 'csharp_practical_registry', '--', '--nocapture']),
        ('foundation-consumers', [cargo, 'test', '-p', 'mpk-vc', '--test', 'csharp_practical_vir_model', 'csharp_03_t02_w02_', '--', '--nocapture']),
        ('vc-clippy', [cargo, 'clippy', '-p', 'mpk-vc', '--test', 'csharp_practical_spec', '--test', 'csharp_practical_registry', '--test', 'csharp_practical_vir_model', '--', '-D', 'warnings']),
        ('format', [cargo, 'fmt', '-p', 'mpk-vc', '--', '--check']),
    ]
    total = 0
    for label, command in jobs:
        state['stage'] = label
        save()
        log = out / (label + '.log.txt')
        started = time.monotonic()
        with log.open('wb') as stream:
            process = subprocess.Popen(command, cwd=repo, env=env, stdout=stream, stderr=subprocess.STDOUT)
            state['process_pid'] = process.pid
            save()
            code = process.wait()
        text = log.read_text()
        counts = re.findall(r'test result: ok\. (\d+) passed; 0 failed;', text)
        state['stages'].append(dict(stage=label, command=command, exit_code=code,
                                   log_sha256=sha(log), test_counts=counts,
                                   elapsed_seconds=round(time.monotonic() - started, 3)))
        save()
        print(label + ': ' + str(code), flush=True)
        assert code == 0, label
        if command[1] == 'test':
            assert len(counts) == 1 and int(counts[0]) > 0
            total += int(counts[0])
        verify()
    state.update(status='passed_selected_refreeze_specification_and_consumers', tests=total)
except BaseException as error:
    state.update(status='failed', error=repr(error))
    raise
finally:
    save()
