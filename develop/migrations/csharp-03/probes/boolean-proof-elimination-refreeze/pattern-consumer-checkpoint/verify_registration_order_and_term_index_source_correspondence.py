"""Exact two-step source correspondence; hash guards only, no test execution."""
import hashlib
import json
from pathlib import Path
from verify_registration_order_source_correspondence import (
    EXPECTED_PATHS, RECEIPT_SHA256, SOURCE_MANIFEST_SHA256,
)

r = Path('/private/tmp/mpk-w09-refreeze-consumer-continuation')
h = lambda p: hashlib.sha256(p.read_bytes()).hexdigest()
read = lambda p: json.loads(p.read_bytes())
indexed_path = 'go-tools/mpk-checker-ref/core_check.go'
index_checkpoint = r / 'develop/migrations/csharp-03/probes/boolean-proof-elimination-refreeze/go-term-interning-checkpoint'
index_review_path = index_checkpoint / 'mpk-w09-go-term-interning-7505badc-reports/source-correspondence-review.json'
index_audit_path = index_checkpoint / 'mpk-w09-go-term-interning-7505badc-reports/independent-final-audit.json'

def load_index_review():
    assert h(index_review_path) == 'f40b4d5025f3d6cdeb32bebfa17bcd2f1b700b9671a22f1ae661dc85f3c1c080'
    review = read(index_review_path)
    assert review['status'] == 'passed_exact_term_index_only_source_delta_and_actual_server_validations'
    assert review['source_commit'] == '7505badc6684191a7dad9833e67d29af32529ee9'
    assert review['exact_reconstruction_of_prior_source']
    assert review['all_other_reduction_inference_fuel_declaration_and_canonical_cases_registration_source_bytes_unchanged']
    row = review['changed_existing_file']
    assert row == {
        'path': indexed_path,
        'before_raw_sha256': '20f51e083e3ee604db977990cad62021b476232a98a2a530babe7c8469d15a77',
        'raw_sha256': '19008b25758b9a00919f468d6bf0f8d8275bd918fe7678156971bd48dde097f8',
    }
    assert h(index_audit_path) == '18b68f0871177a26e89a30355ea3605a6b74c7807ede68b2f67ebf1b26c8e346'
    audit = read(index_audit_path)
    assert audit['status'] == 'passed_independent_indexed_go_full_actual_result_and_source_review'
    assert audit['source_correspondence_review_raw_sha256'] == h(index_review_path)
    assert audit['paired_pattern_certificates'] == 6
    assert audit['indexed_go_core_fixture_results'] == 34
    assert audit['indexed_go_capacity_recursor_results'] == 54
    assert audit['source_files_verified'] == 441 and audit['quality_stages'] == 4
    assert h(r / indexed_path) == row['raw_sha256']
    regression = review['new_regression_test']
    assert regression['test_functions'] == 3 and h(r / regression['path']) == regression['raw_sha256']
    return row

def load_review():
    checkpoint = r / 'develop/migrations/csharp-03/probes/boolean-proof-elimination-refreeze/registration-order-checkpoint'
    assert h(checkpoint / 'receipt.json') == RECEIPT_SHA256
    assert h(checkpoint / 'source-manifest.json') == SOURCE_MANIFEST_SHA256
    receipt = read(checkpoint / 'receipt.json')
    assert receipt['status'] == 'passed_reviewed_registration_order_fix_with_selected_tests_and_exact_legacy_correspondence'
    assert receipt['ordinary_interface_inspection'] == {'inspected_paths': 1920, 'distinct_certificate_byte_sets': 1443, 'reserved_cases_names': 0, 'acceptance_claim': False}
    assert receipt['checker_stages'] == 68 and receipt['rust_tests'] == 56
    rows = {x['path']: x for x in receipt['changed_existing_files']}
    assert set(rows) == set(EXPECTED_PATHS)
    index = load_index_review()
    assert rows[indexed_path]['raw_sha256'] == index['before_raw_sha256']
    manifest = read(checkpoint / 'source-manifest.json')
    for group in ['source_hashes', 'fixture_hashes', 'document_hashes']:
        for path, sha in manifest[group].items():
            if path == indexed_path:
                assert sha == index['before_raw_sha256'] and h(r / path) == index['raw_sha256']
            else:
                assert h(r / path) == sha, path
    return rows

def verify_source_or_reviewed_registration_delta(root, path, original_sha):
    current = h(root / path)
    if current == original_sha:
        return 'exact_original_bytes'
    rows = load_review()
    assert path in rows, path
    registration = rows[path]
    if path == indexed_path:
        index = load_index_review()
        assert original_sha in [registration['before_raw_sha256'], index['before_raw_sha256']]
        assert current == index['raw_sha256']
        return 'exact_reviewed_registration_delta_then_complete_structural_term_index_delta'
    assert original_sha == registration['before_raw_sha256'], path
    assert current == registration['raw_sha256'], path
    return 'exact_reviewed_canonical_cases_registration_only_delta'
