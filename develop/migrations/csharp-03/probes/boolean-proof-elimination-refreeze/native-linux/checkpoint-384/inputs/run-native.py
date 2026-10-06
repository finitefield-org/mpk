import hashlib
import json
import os
import pathlib
import subprocess
import time

control = pathlib.Path(__file__).parent
repo = pathlib.Path('/root/mpk-w09-bool-cases-97559877')
out = pathlib.Path('/root/mpk-w09-refreeze-native-97559877-reports')
out.mkdir(exist_ok=False)
manifest = json.loads((control/'manifest.json').read_bytes())
digest = lambda path: hashlib.sha256(path.read_bytes()).hexdigest()
state = {'status':'running','supervisor_pid':os.getpid(),'stages':[],
    'source_commit':manifest['source_commit'], 'source_files':len(manifest['source_hashes']),
    'request_counts':manifest['counts'], 'selection_reason':'Foundation descriptor/registry refreeze changes only context-bearing sidecar metadata; independently rerun both pinned native Roslyn capture harnesses for all 384 original request rows, including their internal second deterministic captures and retained CLR runs.'}
def save():
    (out/'status.json').write_text(json.dumps(state,sort_keys=True,indent=2)+'\n')
def git(*args):
    return subprocess.check_output(['/usr/bin/git','-C',str(repo),*args],text=True).strip()
def verify():
    assert git('rev-parse','HEAD') == manifest['source_commit']
    assert git('status','--porcelain','--untracked-files=all') == ''
    for name, sha in manifest['source_hashes'].items():
        assert digest(repo/name) == sha, name
    for row in manifest['requests'].values():
        assert digest(control/row['path']) == row['sha256']
save()
try:
    verify()
    for kind, option in [('data','--test-data-phase-requests'),('control','--test-control-emission-requests')]:
        state['stage'] = kind
        start = time.monotonic()
        with (control/(kind+'-requests.json')).open('rb') as inp, (out/(kind+'-responses.json')).open('wb') as stdout, (out/(kind+'.stderr.txt')).open('wb') as stderr:
            process = subprocess.Popen([str(repo/'scripts/build-csharp-practical-frontend.sh'),option],cwd=repo,
                env={'PATH':'/usr/bin:/bin','PYTHONDONTWRITEBYTECODE':'1'},stdin=inp,stdout=stdout,stderr=stderr)
            state['process_pid'] = process.pid
            save()
            code = process.wait()
        stage = {'kind':kind,'exit_code':code,'elapsed_seconds':time.monotonic()-start,
            'stdout_sha256':digest(out/(kind+'-responses.json')),'stderr_sha256':digest(out/(kind+'.stderr.txt'))}
        state['stages'].append(stage)
        save()
        assert code == 0 and (out/(kind+'.stderr.txt')).stat().st_size == 0
        assert len(json.loads((out/(kind+'-responses.json')).read_bytes())) == manifest['counts'][kind]
        verify()
    state['status'] = 'passed_pinned_native_capture_execution'
except BaseException as error:
    state['status'] = 'failed'
    state['error'] = repr(error)
    raise
finally:
    save()
print(json.dumps(state),flush=True)
