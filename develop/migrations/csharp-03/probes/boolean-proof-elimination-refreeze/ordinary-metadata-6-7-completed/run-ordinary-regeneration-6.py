import hashlib
import json
import os
import pathlib
import re
import subprocess
import time

base = pathlib.Path(__file__).parent
repo = pathlib.Path('/private/tmp/mpk-w09-refreeze-metadata-integration')
out = base/'ordinary-regeneration-6'
out.mkdir(exist_ok=False)
goldens = out/'goldens'
goldens.mkdir()
manifest = json.loads((base/'ordinary-source-manifest-6.json').read_bytes())
digest = lambda path:hashlib.sha256(path.read_bytes()).hexdigest()
state = {'status':'running','stages':[],'selection_reason':manifest['selection_reason'],
    'full_t01_gate':'deferred to renewed T01-W10','full_t06_gate':'deferred to T06-W12'}
def save():
    (out/'status.json').write_text(json.dumps(state,sort_keys=True,indent=2)+'\n')
def verify():
    for path, sha in manifest['hashes'].items():
        assert digest(repo/path) == sha, path
jobs = [('binding-guards', 'csharp_03_t06_w09_binding_guards_original_source_certificates', 'MPK_W09_BINDING_GUARDS_OUT', 'binding-guards', 'mpk-vc', 'csharp_practical_vc'), ('binding-orders', 'csharp_03_t06_w09_binding_orders_original_source_certificates', 'MPK_W09_BINDING_ORDERS_OUT', 'binding-orders', 'mpk-vc', 'csharp_practical_vc'), ('binding-reconstruction', 'csharp_03_t06_w09_binding_reconstruction_original_sources', 'MPK_W09_BINDING_RECONSTRUCTION_OUT', 'binding-reconstruction', 'mpk-vc', 'csharp_practical_vc'), ('collection-operations', 'csharp_03_t06_w09_collections_original_source_certificates', 'MPK_W09_COLLECTIONS_OUT', 'collection-operations', 'mpk-vc', 'csharp_practical_vc'), ('literal-definitions', 'csharp_03_t06_w09_literals_original_source_certificates_and_semantics', 'MPK_W09_LITERALS_OUT', 'literal-definitions', 'mpk-vc', 'csharp_practical_vc'), ('transition-operations', 'csharp_03_t06_w09_transition_operations_original_source_certificates', 'MPK_W09_TRANSITION_OPERATIONS_OUT', 'transition-operations', 'mpk-vc', 'csharp_practical_vc'), ('transition-snapshots', 'csharp_03_t06_w09_transition_snapshots_original_sources', 'MPK_W09_TRANSITION_SNAPSHOTS_OUT', 'transition-snapshots', 'mpk-vc', 'csharp_practical_vc'), ('transition-integrated', 'csharp_03_t06_w09_transition_operations_integrated_definition_closure', 'MPK_W09_TRANSITION_INTEGRATED_OUT', 'transition-integrated', 'mpk-vc', 'csharp_practical_vc'), ('public-defaults', 'csharp_03_t06_w09_public_defaults_source_conditions', 'MPK_W09_PUBLIC_DEFAULTS_OUT', 'public-defaults', 'mpk-vc', 'csharp_practical_vc'), ('boundary-documents', 'csharp_03_t06_w09_boundary_documents_original_sources_and_storage', 'MPK_W09_BOUNDARY_DOCUMENTS_OUT', 'boundary-documents', 'mpk-vc', 'csharp_practical_vc'), ('boundary-fragments', 'csharp_03_t06_w09_boundary_fragments_original_sources', 'MPK_W09_BOUNDARY_FRAGMENTS_OUT', 'boundary-fragments', 'mpk-vc', 'csharp_practical_vc'), ('boundary-utf8', 'csharp_03_t06_w09_boundary_utf8_original_sources', 'MPK_W09_BOUNDARY_UTF8_OUT', 'boundary-utf8', 'mpk-vc', 'csharp_practical_vc')]
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
        if label == 'binding-orders':
            env['MPK_W09_BINDING_GUARDS_OUT'] = str(goldens/'binding-guards')
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
