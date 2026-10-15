import hashlib
import json
import pathlib
import shutil

b = pathlib.Path(__file__).parent
r = pathlib.Path('/private/tmp/mpk-w09-refreeze-consumer-continuation')
out = b / 'capture-source-owner-checkpoint-evidence'
out.mkdir(exist_ok=False)
h = lambda p: hashlib.sha256(p.read_bytes()).hexdigest()
read = lambda p: json.loads(p.read_bytes())
owners = read(b / 'ordinary-regeneration-13/status.json')
assert owners['status'] == 'passed_all_four_affected_original_pattern_source_and_proof_type_owners'
assert len(owners['stages']) == 4
assert all(x['exit_code'] == 0 and x['test_counts'] == ['1'] for x in owners['stages'])
capture = read(b / 'capture-promotion.json')
assert capture['status'].startswith('promoted_completed_capture_owner_')
assert len(capture['changes']) == 36
equivalence = read(b / 'all-pattern-original-owner-equivalence.json')
assert len(equivalence['files']) == 126

def copy_file(src, relative):
    dst = out / relative
    dst.parent.mkdir(parents=True, exist_ok=True)
    shutil.copy2(src, dst)
    assert h(src) == h(dst)

for folder in ['control-pattern-capture-name-graphs', 'control-pattern-capture-dual-checkers',
               'historical-pattern-condition-closure-review', 'pattern-consumer-test-source-review',
               'pattern-consumer-selected-quality', 'ordinary-regeneration-12', 'ordinary-regeneration-13']:
    for p in sorted((b / folder).rglob('*')):
        if p.is_file() and p.suffix != '.mpcert' and 'goldens' not in p.relative_to(b / folder).parts:
            copy_file(p, pathlib.Path(folder) / p.relative_to(b / folder))
for name in ['capture-promotion.json', 'control-pattern-capture-changed-certificates.json',
             'control-pattern-capture-name-rebinding-audit.json', 'context-name-rebindings-10.json',
             'context-name-rebindings-11.json', 'derive-control-pattern-capture-name-rebindings.py',
             'derive-control-pattern-consumer-name-rebindings.py',
             'control-pattern-consumer-name-rebinding-audit.json',
             'run-control-pattern-capture-name-graphs.py', 'run-control-pattern-capture-dual-checkers.py',
             'run-ordinary-regeneration-12.py', 'run-ordinary-regeneration-13.py',
             'ordinary-source-manifest-12.json', 'ordinary-source-manifest-13.json',
             'pattern-checkpoint-second-review.json', 'all-pattern-original-owner-equivalence.json',
             'run-historical-pattern-condition-closure-review.py',
             'compare-historical-pattern-condition-closures.rs',
             'historical-pattern-condition-closure-compile-command.json',
             'historical-pattern-condition-closure-compile.log.txt',
             'finalize-capture-source-owner-checkpoint.py', 'live-owner-immutability-audit.json']:
    copy_file(b / name, name)

archive = r / 'develop/migrations/csharp-03/ordinary-foundation/previous-context-6-captures'
archive.mkdir(exist_ok=False)
before_manifest = {}
for row in capture['changes']:
    assert h(r / row['path']) == row['raw_sha256']
    src = b / 'capture-before' / row['path']
    assert h(src) == row['before_raw_sha256']
    relative = pathlib.Path(row['path']).relative_to('develop/migrations/csharp-03/ordinary-foundation')
    dest = archive / relative
    dest.parent.mkdir(parents=True, exist_ok=True)
    shutil.copy2(src, dest)
    before_manifest[str(relative)] = h(dest)
(archive / 'manifest.json').write_text(json.dumps({
    'status': 'immutable_complete_capture_predecessor_bytes_retained',
    'source_checkpoint_commit': '53e88e9e900f0e58a35fc9ef2764a258299b6c24', 'files': before_manifest,
}, sort_keys=True, indent=2) + '\n')

receipt = {
    'schema': 'mpk.csharp_practical.t01_w09.capture_and_original_pattern_owner_checkpoint.v1',
    'status': 'passed_promoted_capture_graphs_dual_acceptance_and_all_four_original_pattern_owners',
    'selection_reason': 'Promote only the18 completed capture programs. Three narrow test changes retain all original source/value/type/import/mutation checks; four original18-source consumer owners now pass, and all126 current primitive/route/packed output files match the independent54program generation. Remaining21 distinct current-consumer dual acceptance and whole-wrapper checks are separate pending work, so their126 fixture files are not promoted here. No unaffected scalar/native matrix is repeated.',
    'capture_source_cases': 18, 'capture_complete_changed_graph_pairs': 7,
    'capture_fresh_matching_checker_acceptances': 14,
    'capture_complete_old_fresh_pair_reuses': 11,
    'four_original_pattern_owner_tests_passed': 4,
    'actual54program_equivalent_original_owner_files': 126,
    'unpacked_original_proof_types': '113paths703premises108refinements664projections5binderlimitedpaths',
    'packed_original_proof_types': '113paths703completepremises/projections37598originalfield-bitobservations',
    'historical61condition_complete_typed_closures_preserved': 18,
    'three_test_changes': 'SHA-pinned actual historical-name map and two optional exact predecessor input paths; every other original test byte preserved.',
    'selected_vc_test_clippy_and_format': 'passed_exact_current_test_sources',
    'accepted_parent_reports': '../standalone-execution-checkpoint/',
    'remaining_current_consumer_fixture_promotion': 'pending21freshcertificatepairsbothcheckers',
    'remaining_wrapper_fixture_promotion': 'pending115certificatepairsbothcheckersand14originalowner/dependencytests',
    'application_proof_ids_pending': 987,
    'w09_status': 'In progress', 'w10_status': 'Blocked', 'practical_profile': 'inactive',
    'full_t01_gate': 'deferred_to_final_W10', 'full_t06_gate': 'deferred_to_W12',
    'changes': capture['changes'],
}
(out / 'receipt.json').write_text(json.dumps(receipt, sort_keys=True, indent=2) + '\n')
manifest = {str(p.relative_to(out)): h(p) for p in sorted(out.rglob('*')) if p.is_file()}
(out / 'file-manifest.json').write_text(json.dumps({'schema': 'mpk.evidence_file_manifest.v1',
                                                  'files': manifest}, sort_keys=True, indent=2) + '\n')
dest = r / 'develop/migrations/csharp-03/probes/boolean-proof-elimination-refreeze/capture-and-original-owner-checkpoint'
shutil.copytree(out, dest)
assert all(h(dest / path) == sha for path, sha in manifest.items())
print('capture36fixturepaths, fouroriginalowners and exact126producer outputs verified; evidence', len(manifest))
