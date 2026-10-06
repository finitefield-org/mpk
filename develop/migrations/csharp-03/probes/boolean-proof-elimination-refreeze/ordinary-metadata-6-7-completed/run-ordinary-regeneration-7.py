import hashlib
import json
import os
import pathlib
import re
import subprocess
import time

base = pathlib.Path(__file__).parent
repo = pathlib.Path('/private/tmp/mpk-w09-refreeze-metadata-integration')
out = base/'ordinary-regeneration-7'
out.mkdir(exist_ok=False)
goldens = out/'goldens'
goldens.mkdir()
manifest = json.loads((base/'ordinary-source-manifest-7.json').read_bytes())
digest = lambda path:hashlib.sha256(path.read_bytes()).hexdigest()
state = {'status':'running','stages':[],'selection_reason':manifest['selection_reason'],
    'full_t01_gate':'deferred to renewed T01-W10','full_t06_gate':'deferred to T06-W12'}
def save():
    (out/'status.json').write_text(json.dumps(state,sort_keys=True,indent=2)+'\n')
def verify():
    for path, sha in manifest['hashes'].items():
        assert digest(repo/path) == sha, path
jobs = [('string-basic-circuits', 'csharp_03_t06_w09_string_basic_source_linkage_rejects_substitution', 'MPK_W09_STRING_OUT', 'string-basic-circuits', 'mpk-vc', 'csharp_practical_vc'), ('string-ordinal-circuits', 'csharp_03_t06_w09_string_ordinal_source_linkage_rejects_substitution', 'MPK_W09_STRING_ORDINAL_OUT', 'string-ordinal-circuits', 'mpk-vc', 'csharp_practical_vc'), ('string-construction-circuits', 'csharp_03_t06_w09_string_construction_source_linkage_rejects_substitution', 'MPK_W09_STRING_CONSTRUCT_OUT', 'string-construction-circuits', 'mpk-vc', 'csharp_practical_vc'), ('reference-data', 'csharp_03_t06_w09_reference_data_candidates', 'MPK_W09_REFERENCE_DATA_OUT', 'reference-data', 'mpk-vc', 'csharp_practical_vc'), ('sequence-data', 'csharp_03_t06_w09_sequence_data_candidates', 'MPK_W09_SEQUENCE_DATA_OUT', 'sequence-data', 'mpk-vc', 'csharp_practical_vc'), ('option-data', 'csharp_03_t06_w09_option_data_candidates', 'MPK_W09_OPTION_DATA_OUT', 'option-data', 'mpk-vc', 'csharp_practical_vc'), ('construction-data', 'csharp_03_t06_w09_construction_data_candidates', 'MPK_W09_CONSTRUCTION_DATA_OUT', 'construction-data', 'mpk-vc', 'csharp_practical_vc'), ('ownership-equations', 'csharp_03_t06_w09_ownership_equation_candidates', 'MPK_W09_OWNERSHIP_EQUATIONS_OUT', 'ownership-equations', 'mpk-vc', 'csharp_practical_vc'), ('construction-ownership-records', 'csharp_03_t06_w09_construction_ownership_control_records', 'MPK_W09_CONSTRUCTION_RECORDS_OUT', 'construction-ownership-records', 'mpk-vc', 'csharp_practical_vc'), ('ownership-proofs', 'csharp_03_t06_w09_ownership_proof_candidates', 'MPK_W09_OWNERSHIP_PROOFS_OUT', 'ownership-proofs', 'mpk-vc', 'csharp_practical_vc'), ('ordered-folds', 'csharp_03_t06_w09_ordered_folds_source_replay_and_mutations', 'MPK_W09_ORDERED_FOLD_OUT', 'ordered-folds', 'mpk-vc', 'csharp_practical_vc'), ('control-predicates', 'csharp_03_t06_w09_control_predicates_original_sources', 'MPK_W09_CONTROL_PREDICATE_OUTPUT', 'control-predicates', 'mpk-vc', 'csharp_practical_vc'), ('control-predicates--with-execution', 'csharp_03_t06_w09_control_predicates_with_execution_original_sources', 'MPK_W09_CONTROL_PREDICATE_INTEGRATED_OUTPUT', 'control-predicates/with-execution', 'mpk-vc', 'csharp_practical_vc'), ('control-predicates--with-pattern-scopes', 'csharp_03_t06_w09_control_predicates_with_pattern_scopes_original_sources', 'MPK_W09_CONTROL_PATTERN_SCOPE_OUTPUT', 'control-predicates/with-pattern-scopes', 'mpk-vc', 'csharp_practical_vc'), ('control-predicates--with-pattern-captures', 'csharp_03_t06_w09_control_predicates_with_pattern_captures_original_sources', 'MPK_W09_CONTROL_PATTERN_CAPTURE_OUTPUT', 'control-predicates/with-pattern-captures', 'mpk-vc', 'csharp_practical_vc')]
save()
try:
    verify()
    for label, selector, variable, destination, package, target in jobs:
        env = os.environ.copy()
        for key in list(env):
            if key.startswith(('MPK_T06_', 'MPK_W09_')) or key == 'MPK_W14_REQUEST_OUTPUT':
                env.pop(key)
        env['CARGO_TARGET_DIR'] = '/private/tmp/mpk-w09-refreeze-metadata-target'
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
