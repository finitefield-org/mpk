"""Final evidence generation on the approved host after all ten owner results.

Run only in a separate checkout. Original live checkouts remain immutable.
"""
import argparse
import copy
import datetime
import hashlib
import importlib.util
import json
import pathlib
import platform
import shutil
import subprocess


parser = argparse.ArgumentParser()
parser.add_argument('--repository', type=pathlib.Path, required=True)
parser.add_argument('--source-commit', required=True)
parser.add_argument('--output', type=pathlib.Path, required=True)
args = parser.parse_args()
assert platform.system() == 'Linux', 'Server-only generation.'
assert pathlib.Path('/root/mpk-w09-server-e89c2a0f').is_dir()
r = args.repository.resolve()
assert str(r).startswith('/root/mpk-w09-final-freeze-')
assert len(args.source_commit) == 40
assert subprocess.run(['/usr/bin/git', '-C', str(r), 'rev-parse', 'HEAD'], check=True,
                      stdout=subprocess.PIPE, text=True).stdout.strip() == args.source_commit
assert not subprocess.run(['/usr/bin/git', '-C', str(r), 'status', '--porcelain'], check=True,
                          stdout=subprocess.PIPE, text=True).stdout
out = args.output.resolve()
assert str(out).startswith('/root/mpk-w09-final-freeze-')
assert not out.exists(), 'Retain previous generation outputs unchanged.'
h = lambda p: hashlib.sha256(p.read_bytes()).hexdigest()
read = lambda p: json.loads(p.read_bytes())
canonical = lambda value: json.dumps(value, sort_keys=True, separators=(',', ':'), ensure_ascii=True).encode()
root = r / 'develop/migrations/csharp-03'
checkpoint = root / 'probes/boolean-proof-elimination-refreeze'

# Do not release the final freeze while any complete original owner is missing.
semantic = checkpoint / 'full-semantic-source-owner-checkpoint'
semantic_receipt = read(semantic / 'receipt.json')
assert semantic_receipt['status'] == 'passed_all_ten_complete_original_scalar_codec_source_owners_and_exact_current_pin_equivalence'
assert semantic_receipt['original_owner_tests_passed'] == 10
assert semantic_receipt['historical_original_owners_passed'] == 6
assert semantic_receipt['new_server_original_owners_passed'] == 4
assert semantic_receipt['newly_promoted_fixture_files'] == 0
for path, sha in read(semantic / 'file-manifest.json')['files'].items():
    assert h(semantic / path) == sha, path
for row in semantic_receipt['compared_files']:
    assert h(r / row['path']) == row['raw_sha256'], row['path']
for path, sha in semantic_receipt['semantic_owner_owned_production_sources_exactly_equal'].items():
    assert h(r / path) == sha, path
for name, files, key, stages in [
    ('pattern-consumer-checkpoint', 126, 'current_consumer_fresh_matching_checker_acceptances', 42),
    ('wrapper-consumer-checkpoint', 288, 'fresh_matching_zero_axiom_checker_acceptances', 230),
]:
    folder = checkpoint / name
    receipt = read(folder / 'receipt.json')
    assert receipt[key] == stages
    assert len(receipt['changes']) == files
    for path, sha in read(folder / 'file-manifest.json')['files'].items():
        assert h(folder / path) == sha, path
    for row in receipt['changes']:
        assert h(r / row['path']) == row['raw_sha256'], row['path']

registration_path = checkpoint / 'registration-order-checkpoint/receipt.json'
linux_path = checkpoint / 'registration-order-linux-checkpoint/linux-final-audit.json'
assert h(registration_path) == '4e18b0cbc1b5bc89c242595c38b6b3e193e27c8ccde573aebc00f0dbf2ef0c3c'
assert h(linux_path) == '8bb4f17bb701f623231fa5c857f21b23c870a69b1ad99ccbc0df7b2d12b6f1ec'
registration, linux = read(registration_path), read(linux_path)
assert registration['checker_stages'] == linux['checker_stages'] == 68
assert registration['rust_tests'] == 56 and linux['rust_selected_tests'] == 61
index = checkpoint / 'go-term-interning-checkpoint'
review_path = index / 'mpk-w09-go-term-interning-7505badc-reports/source-correspondence-review.json'
audit_path = index / 'mpk-w09-go-term-interning-7505badc-reports/independent-final-audit.json'
assert h(review_path) == 'f40b4d5025f3d6cdeb32bebfa17bcd2f1b700b9671a22f1ae661dc85f3c1c080'
assert h(audit_path) == '18b68f0871177a26e89a30355ea3605a6b74c7807ede68b2f67ebf1b26c8e346'
review, audit = read(review_path), read(audit_path)
assert review['exact_reconstruction_of_prior_source']
assert audit['indexed_go_core_fixture_results'] == 34
assert audit['indexed_go_capacity_recursor_results'] == 54
assert audit['quality_stages'] == 4 and audit['paired_pattern_certificates'] == 6
assert h(r / review['changed_existing_file']['path']) == review['changed_existing_file']['raw_sha256']
assert h(r / review['new_regression_test']['path']) == review['new_regression_test']['raw_sha256']

paths = [
    'develop/migrations/csharp-03/probes/boolean-proof-elimination.json',
    'develop/probes/csharp-03/profile_freeze.py',
    'develop/migrations/csharp-03/freeze/profile-freeze.json',
    'develop/migrations/csharp-03/freeze/profile-freeze-vectors.json',
    'develop/migrations/csharp-03/probes/checker-capacity.json',
    'develop/migrations/csharp-03/probes/recursor-feasibility.json',
]
expected = [
    'bd6439d06370d7caa58e36410f41375e1998acdd0ccdb0ee08f8d87cd93cbd92',
    'ef376560921bb8122b20573993a5b495dea51384e00ada08e0f16fa0dbd80c5a',
    '8bf6ac1f0c5b008fd431d904bb8c33af96932a404d747a48f947aa7369fdafd7',
    '4c8d46b78cb7f3bad906fa959bd7fb3f97bba1debf75fffca3ee63c9fa64411c',
    'd7523f38e1ec6323b5eb78909b221d9dbc6d638eb2f8ff724df98621a691bf78',
    'fbeffc3190703e30c0706315477ae2800019d8ebfe544ad8c23da5c4c2410ff9',
]
for path, sha in zip(paths, expected):
    assert h(r / path) == sha, path
records = index / 'mpk-w09-go-term-interning-7505badc-capacity-recursor/candidate-records-2'
new_records = [
    (records / 'checker-capacity.json', paths[4], 'e45cf6c9e05d90ba7de3dd99939ab27081427b713d8e458fd2fedf26b7c9f968'),
    (records / 'recursor-feasibility.json', paths[5], '98150898d27578a5ffa61e86720a600e467c185eb3aac97c39b0e3b9818df317'),
]
for candidate, path, sha in new_records:
    assert h(candidate) == sha, path
    value = read(candidate)
    assert value['probe'] == read(r / path)['probe']
    assert value['source_inventory_sha256'] == hashlib.sha256(canonical(value['source_inventory']) + b'\n').hexdigest()
    for row in value['source_inventory']:
        assert h(r / row['path']) == row['raw_sha256'], row['path']
assert read(new_records[1][0])['capacity_measurement']['raw_sha256'] == new_records[0][2]
generator = pathlib.Path(__file__).with_name('final-rule-and-index-freeze-generator-candidate.py')
assert generator.is_file()
assert h(generator) == '8a9e0250bd6eef04fed76fd0f39d6da7f0f5b8095c308cf484a9420a5ef74f2f'
old_proof, old_freeze, old_vectors = read(r / paths[0]), read(r / paths[2]), read(r / paths[3])
assert len(old_vectors['vectors']) == 709
new = copy.deepcopy(old_proof)
new.pop('content_sha256')
good = {
    'right-identity', 'constructor-false', 'constructor-true', 'open-motive',
    'conjunction-left', 'conjunction-right', 'legacy-prior-constructor-value-levels',
    'legacy-prior-theorem-proof-levels', 'unused-constructor-levels', 'prior-other-family-levels',
}
rows = []
for path in sorted((r / 'fixtures/core-bool-cases').glob('*.hex')):
    data = bytes.fromhex(path.read_text())
    rows.append({'id': path.stem, 'path': str(path.relative_to(r)), 'raw_sha256': h(path),
                 'certificate_sha256': hashlib.sha256(data).hexdigest(),
                 'expected': 'accepted' if path.stem in good else 'core_rejected'})
assert len(rows) == 31 and sum(row['expected'] == 'accepted' for row in rows) == 10
by_id = {row['id']: row for row in rows}
for row in old_proof['cases']:
    assert row == by_id[row['id']], row['id']
source_paths = set(old_proof['core_source_hashes']) | {'go-tools/mpk-checker-ref/core_check.go'}
assert len(source_paths) == 36
new.update(source_commit=args.source_commit, accepted_cases=10, rejected_cases=21, cases=rows,
           core_source_hashes={path: h(r / path) for path in sorted(source_paths)})
new['proof_interface']['universe_arguments'] = 'empty at inference, reduction and cases registration of all preceding reachable checked types/definition values/theorem proofs'
new['local'] = {'receipt_path': str(registration_path.relative_to(r)), 'receipt_raw_sha256': h(registration_path),
                'rust_tests': 56, 'go_top_level_tests': 16, 'checker_stages': 68,
                'actual_source_commit': 'fd99c03c8e3535bcc31f0b9d1441d4b74de80ebf',
                'execution_provenance': 'Completed before the server-only instruction; original source/binary/host preserved.'}
new['linux'] = {'audit_path': str(linux_path.relative_to(r)), 'audit_raw_sha256': h(linux_path),
                'rust_tests': 61, 'go_top_level_tests': 16, 'checker_stages': 68, 'git_blob_files_verified': 371,
                'actual_source_commit': 'fd99c03c8e3535bcc31f0b9d1441d4b74de80ebf'}
new['indexed_reference_source_correspondence'] = {
    'review_path': str(review_path.relative_to(r)), 'review_raw_sha256': h(review_path),
    'audit_path': str(audit_path.relative_to(r)), 'audit_raw_sha256': h(audit_path),
    'actual_source_commit': '7505badc6684191a7dad9833e67d29af32529ee9',
    'execution_host': 'root@162.43.92.154', 'actual_core_fixture_results': 34,
    'actual_capacity_recursor_results': 54, 'actual_pattern_pairs': 6,
    'historical_executions_attributed_to_indexed_source': False,
}
new['previous_implementation_evidence'] = {
    'path': str((checkpoint / 'registration-order-proof-evidence-predecessor/boolean-proof-elimination.json').relative_to(r)),
    'raw_sha256': expected[0], 'source_commit': old_proof['source_commit'],
    'accepted_cases': old_proof['accepted_cases'], 'rejected_cases': old_proof['rejected_cases'],
}
new['content_sha256'] = hashlib.sha256(b'MPK-CSHARP-BOOL-PROOF-ELIMINATION-1.0\0' + canonical(new)).hexdigest()

# Retain every old input before mutating the isolated candidate checkout.
out.mkdir()
archive = out / 'before'
archive.mkdir()
for path, sha in zip(paths, expected):
    shutil.copy2(r / path, archive / pathlib.Path(path).name)
    assert h(archive / pathlib.Path(path).name) == sha
(archive / 'file-manifest.json').write_text(json.dumps({'schema': 'mpk.evidence_file_manifest.v1',
    'files': {pathlib.Path(path).name: sha for path, sha in zip(paths, expected)}}, sort_keys=True, indent=2) + '\n')
(r / paths[0]).write_bytes(canonical(new) + b'\n')
shutil.copy2(generator, r / paths[1])
for candidate, path, sha in new_records:
    shutil.copy2(candidate, r / path)
    assert h(r / path) == sha
spec = importlib.util.spec_from_file_location('server_final_profile_freeze', r / paths[1])
module = importlib.util.module_from_spec(spec)
spec.loader.exec_module(module)
fresh = module.make_freeze()
vectors = module.make_vectors(fresh)
assert {k: v for k, v in old_vectors.items() if k != 'freeze_content_sha256'} == {k: v for k, v in vectors.items() if k != 'freeze_content_sha256'}
assert {k: v for k, v in old_freeze.items() if k not in ['evidence', 'content_sha256']} == {k: v for k, v in fresh.items() if k not in ['evidence', 'content_sha256']}
changed = {k for k in old_freeze['evidence'] if old_freeze['evidence'][k] != fresh['evidence'][k]}
assert set(old_freeze['evidence']) == set(fresh['evidence'])
assert changed == {'freeze_generator_raw_sha256', 'boolean_proof_evidence_raw_sha256',
                   'boolean_proof_evidence_content_sha256', 'recursor_evidence_raw_sha256',
                   'capacity_evidence_raw_sha256', 'capacity_source_inventory_sha256'}
module.FREEZE.write_bytes(module.bytes_of(fresh))
module.VECTORS.write_bytes(module.bytes_of(vectors))
generated = out / 'generated'
generated.mkdir()
for path in paths:
    shutil.copy2(r / path, generated / pathlib.Path(path).name)
result = {
    'status': 'passed_server_only_final_proof_and_index_evidence_generation_exact709_vectors',
    'execution_host': 'root@162.43.92.154', 'source_commit': args.source_commit,
    'executed_at_utc': datetime.datetime.now(datetime.timezone.utc).isoformat(),
    'all_709_complete_original_vector_rows_unchanged': True,
    'non_evidence_freeze_behavior_unchanged': True, 'changed_freeze_evidence_fields': sorted(changed),
    'core_source_pin_count': 36, 'original_19_case_records_retained_exactly': True,
    'shared_positive_cases': 10, 'shared_negative_cases': 21, 'predecessor_cases': 3,
    'historical_fixed_checker_stages_per_host': 68, 'indexed_go_actual_core_results': 34,
    'indexed_go_actual_capacity_recursor_results': 54, 'original_application_proof_ids_pending': 987,
    'generated_files': {path: h(r / path) for path in paths},
    'complete_predecessor_files': dict(zip(paths, expected)),
    'full_t01_gate': 'deferred_to_final_W10_on_server', 'full_t06_gate': 'deferred_to_W12_on_server',
}
(out / 'receipt.json').write_text(json.dumps(result, sort_keys=True, indent=2) + '\n')
print(json.dumps({'status': result['status'], 'generated_files': len(paths)}, sort_keys=True))
