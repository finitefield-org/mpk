import hashlib
import json
import os
import pathlib
import re
import subprocess
import time

base = pathlib.Path(__file__).parent
repo = pathlib.Path('/private/tmp/mpk-w09-refreeze-consumer-continuation')
out = base/'ordinary-regeneration-12'
out.mkdir(exist_ok=False)
goldens = out/'goldens'
goldens.mkdir()
manifest = json.loads((base/'ordinary-source-manifest-12.json').read_bytes())
digest = lambda path:hashlib.sha256(path.read_bytes()).hexdigest()
dependency = base/'ordinary-regeneration-10/goldens/control-predicates/with-execution'
scope_dependency = base/'ordinary-regeneration-11/goldens/control-predicates/with-pattern-scopes'
dependency_hashes = {str(p):digest(p) for directory in [dependency,scope_dependency] for p in sorted(directory.glob('*')) if p.is_file()}
(out/'dependency-manifest.json').write_text(json.dumps(dependency_hashes,sort_keys=True,indent=2)+'\n')
state = {'status':'running','stages':[],'selection_reason':manifest['selection_reason'],
    'full_t01_gate':'deferred to renewed T01-W10','full_t06_gate':'deferred to T06-W12'}
def save():
    (out/'status.json').write_text(json.dumps(state,sort_keys=True,indent=2)+'\n')
def verify():
    for path, sha in dependency_hashes.items():
        assert digest(pathlib.Path(path)) == sha, path
    for path, sha in manifest['hashes'].items():
        assert digest(repo/path) == sha, path
jobs = [('control-predicates--with-pattern-captures', 'csharp_03_t06_w09_control_predicates_with_pattern_captures_original_sources', 'MPK_W09_CONTROL_PATTERN_CAPTURE_OUTPUT', 'control-predicates/with-pattern-captures', 'mpk-vc', 'csharp_practical_vc')]
save()
try:
    verify()
    for label, selector, variable, destination, package, target in jobs:
        env = os.environ.copy()
        for key in list(env):
            if key.startswith(('MPK_T06_', 'MPK_W09_')) or key == 'MPK_W14_REQUEST_OUTPUT':
                env.pop(key)
        env['CARGO_TARGET_DIR'] = '/private/tmp/mpk-w09-refreeze-consumer-target'
        env['MPK_W09_CONTROL_PREDICATE_EXECUTION_INPUT'] = str(base/'ordinary-regeneration-10/goldens/control-predicates/with-execution')
        env['MPK_W09_CONTROL_PATTERN_SCOPE_INPUT'] = str(scope_dependency)
        generated = goldens/destination if destination else None
        if generated:
            generated.parent.mkdir(parents=True,exist_ok=True)
            env[variable] = str(generated)
        command = ['/Users/kazuyoshitoshiya/.cargo/bin/cargo','test','-p',package,
            '--test',target,selector,'--','--nocapture','--test-threads=1']
        state['stage'] = label
        started = time.monotonic()
        with (out/(label+'.log.txt')).open('wb') as log:
            process = subprocess.Popen(command,cwd=repo,env=env,stdout=log,stderr=subprocess.STDOUT)
            state['process_pid'] = process.pid
            save()
            code = process.wait()
        text = (out/(label+'.log.txt')).read_text()
        counts = re.findall(r'test result: ok\. (\d+) passed; 0 failed;',text)
        row = {'stage':label,'selector':selector,'destination':destination,'exit_code':code,
            'test_counts':counts,'log_sha256':digest(out/(label+'.log.txt')),
            'elapsed_seconds':time.monotonic()-started}
        state['stages'].append(row)
        save()
        print(label+': '+str(code),flush=True)
        assert code == 0 and counts == ['1'] and (generated is None or generated.is_dir()), label
        if generated:
            row['generated_files'] = {str(p.relative_to(generated)):digest(p) for p in generated.rglob('*') if p.is_file()}
        verify()
    state['status'] = 'passed_selected_source_bound_ordinary_owners'
except BaseException as error:
    state['status'] = 'failed'
    state['error'] = repr(error)
    raise
finally:
    save()
