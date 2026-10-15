import hashlib
import json
import os
import pathlib
import re
import subprocess
import time

b = pathlib.Path(__file__).parent
r = pathlib.Path('/private/tmp/mpk-w09-wrapper-owner-validation')
out = b / 'wrapper-owner-validation'
out.mkdir(exist_ok=False)
h = lambda p: hashlib.sha256(p.read_bytes()).hexdigest()
selection = (
    'Select the fourteen original wrapper owner/dependency and hostile-import tests '
    'affected by the exact288-file candidate: field/envelope/depth/count/compound '
    'and guard metadata, plus the semantic-root nested parser alias and preserved '
    'unguarded closure. All full source/native-value/mutation/structural checks '
    'and normal fixture comparisons execute without OUTPUT/SELECT overrides. '
    'Unchanged primitive/scalar/control matrices are excluded. Candidate files '
    'exist only in this isolated review checkout; dual acceptance remains separate. '
    'Whole gates stay final T01-W10 and T06-W12.'
)
paths = set(json.loads((b / 'ordinary-source-manifest-13.json').read_bytes())['hashes'])
plan = json.loads((b / 'wrapper-program-promotion-plan/plan.json').read_bytes())
paths.update(row['path'] for row in plan['changes'])
manifest = {'selection_reason': selection, 'hashes': {p: h(r / p) for p in sorted(paths)}}
(out / 'source-manifest.json').write_text(json.dumps(manifest, sort_keys=True, indent=2) + '\n')
jobs = [
    ('boundary-fields', 'csharp_03_t06_w09_json_boundary_fields_original_sources'),
    ('envelopes', 'csharp_03_t06_w09_json_envelopes_original_documents'),
    ('depth-envelopes', 'csharp_03_t06_w09_json_depth_guarded_envelopes_original_documents'),
    ('typed-depth', 'csharp_03_t06_w09_json_typed_depth_original_inputs'),
    ('depth-compound', 'csharp_03_t06_w09_json_depth_guard_compound_sources'),
    ('typed-nodes', 'csharp_03_t06_w09_json_typed_nodes_original_inputs'),
    ('typed-nodes-compound', 'csharp_03_t06_w09_json_typed_nodes_compound_inputs'),
    ('typed-envelopes', 'csharp_03_t06_w09_json_typed_guarded_envelopes_original_documents'),
    ('typed-envelopes-compound', 'csharp_03_t06_w09_json_typed_guarded_compound_definition_closures'),
    ('limits-envelopes', 'csharp_03_t06_w09_json_limits_guarded_envelopes_original_documents'),
    ('limits-envelopes-compound', 'csharp_03_t06_w09_json_limits_guarded_compound_definition_closures'),
    ('semantic-root-aliases', 'csharp_03_t06_w09_json_semantic_root_depth_envelopes'),
    ('unguarded-closure', 'csharp_03_t06_w09_unguarded_envelope_metadata_preserved'),
    ('limits-hostile-imports', 'csharp_03_t06_w09_json_limits_guarded_import_rejects_linkage_mutations'),
]
state = {
    'status': 'running', 'stages': [], 'selection_reason': selection,
    'candidate_only': True, 'original_application_proof_ids_pending': 987,
    'full_t01_gate': 'deferred_to_final_W10', 'full_t06_gate': 'deferred_to_W12',
    'supervisor_pid': os.getpid(),
}
def save():
    (out / 'status.json').write_text(json.dumps(state, sort_keys=True, indent=2) + '\n')
def verify():
    for p, sha in manifest['hashes'].items():
        assert h(r / p) == sha, p
save()
try:
    verify()
    for label, selector in jobs:
        env = os.environ.copy()
        for key in list(env):
            if key.startswith(('MPK_W09_', 'MPK_T06_')) or key == 'MPK_W14_REQUEST_OUTPUT':
                env.pop(key)
        env['CARGO_TARGET_DIR'] = '/private/tmp/mpk-w09-wrapper-owner-target'
        env['CARGO_INCREMENTAL'] = '0'
        command = ['/Users/kazuyoshitoshiya/.cargo/bin/cargo', 'test', '-p', 'mpk-vc',
                   '--test', 'csharp_practical_vc', selector, '--', '--nocapture', '--test-threads=1']
        state['stage'] = label
        start = time.monotonic()
        log = out / (label + '.log.txt')
        with log.open('wb') as stream:
            proc = subprocess.Popen(command, cwd=r, env=env, stdout=stream, stderr=subprocess.STDOUT)
            state['process_pid'] = proc.pid
            save()
            code = proc.wait()
        counts = re.findall(r'test result: ok\. (\d+) passed; 0 failed;', log.read_text())
        row = {'stage': label, 'selector': selector, 'exit_code': code,
               'test_counts': counts, 'elapsed_seconds': time.monotonic() - start,
               'log_sha256': h(log)}
        state['stages'].append(row)
        save()
        assert code == 0 and counts == ['1'], row
        verify()
        print(label + ': passed original owner with exact candidate pins', flush=True)
    state['status'] = 'passed_all_14_affected_original_wrapper_owner_dependency_and_import_tests'
except BaseException as error:
    state['status'] = 'failed'
    state['error'] = repr(error)
    raise
finally:
    save()
