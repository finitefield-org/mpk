import hashlib
import json
import os
import pathlib
import re
import subprocess
import time

base = pathlib.Path(__file__).parent
repo = pathlib.Path('/private/tmp/mpk-w09-context-application-integration')
out = base/'pinned-consumers'
out.mkdir(exist_ok=False)
goldens = out/'goldens'
goldens.mkdir()
manifest = json.loads((base/'pinned-consumer-source-manifest.json').read_bytes())
digest = lambda path:hashlib.sha256(path.read_bytes()).hexdigest()
state = {'status':'running','stages':[],'selection_reason':manifest['selection_reason'],
    'full_t01_gate':'deferred to renewed T01-W10','full_t06_gate':'deferred to T06-W12'}
def save():
    (out/'status.json').write_text(json.dumps(state,sort_keys=True,indent=2)+'\n')
def verify():
    for path, sha in manifest['hashes'].items():
        assert digest(repo/path) == sha, path
jobs = [('construction', 'csharp_03_t06_w02_actual_source_goldens_and_failing_conditions', None, None, 'mpk-vc', 'csharp_practical_vc'), ('data', 'csharp_03_t06_w03_replay_semantic_rows_and_ordering', None, None, 'mpk-vc', 'csharp_practical_vc'), ('control', 'csharp_03_t06_w04_original_control_paths_and_stable_ids', None, None, 'mpk-vc', 'csharp_practical_vc'), ('exception', 'csharp_03_t06_w05_original_exception_handlers_and_mutations', None, None, 'mpk-vc', 'csharp_practical_vc'), ('binding', 'csharp_03_t06_w06_all_binding_roles_and_closed_handoff_mutations', None, None, 'mpk-vc', 'csharp_practical_vc'), ('boundary', 'csharp_03_t06_w07_original_contracts_and_three_state_obligations', None, None, 'mpk-vc', 'csharp_practical_vc'), ('boundary-run', 'csharp_03_t06_w07_complete_run_relations_and_hostile_serializers', None, None, 'mpk-vc', 'csharp_practical_vc'), ('transition', 'csharp_03_t06_w08_original_source_programs_and_mutations', None, None, 'mpk-vc', 'csharp_practical_vc'), ('boundary-input', 'csharp_03_t05_w02_real_source_input_matrix_and_two_run_identity', None, None, 'mpk-cli', 'csharp_practical_boundary'), ('boundary-output', 'csharp_03_t05_w03_original_source_output_and_cumulative_identity', None, None, 'mpk-cli', 'csharp_practical_boundary')]
save()
try:
    verify()
    for label, selector, variable, destination, package, target in jobs:
        env = os.environ.copy()
        for key in list(env):
            if key.startswith(('MPK_T06_', 'MPK_W09_')) or key == 'MPK_W14_REQUEST_OUTPUT':
                env.pop(key)
        env['CARGO_TARGET_DIR'] = '/private/tmp/mpk-w09-bool-cases-target'
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
        assert code == 0 and counts == ['1'] and (generated is None or generated.is_file()), label
        if generated:
            row['generated_sha256'] = digest(generated)
            row['previous_sha256'] = digest(repo/'develop/migrations/csharp-03'/destination)
        verify()
    state['status'] = 'passed_normal_pinned_affected_consumer_owners'
except BaseException as error:
    state['status'] = 'failed'
    state['error'] = repr(error)
    raise
finally:
    save()
