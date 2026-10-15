import hashlib,json
from pathlib import Path
r=Path('/private/tmp/mpk-w09-refreeze-consumer-continuation')
checkpoint=r/'develop/migrations/csharp-03/probes/boolean-proof-elimination-refreeze/registration-order-checkpoint'
h=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
RECEIPT_SHA256='4e18b0cbc1b5bc89c242595c38b6b3e193e27c8ccde573aebc00f0dbf2ef0c3c'
SOURCE_MANIFEST_SHA256='13293a1b3089f03660a5f1fcc3e39086c4f36359b99ae8f93bee887774ad0613'
EXPECTED_PATHS=['crates/mpk-core/src/bool_cases.rs', 'crates/mpk-core/src/inductive_gen.rs', 'crates/mpk-kernel/src/bool_cases_tests.rs', 'crates/mpk-vc/tests/csharp_practical_spec.rs', 'develop/specs/CORE_V0.md', 'fixtures/core-bool-cases/README.md', 'go-tools/mpk-checker-ref/bool_cases.go', 'go-tools/mpk-checker-ref/bool_cases_test.go', 'go-tools/mpk-checker-ref/core_check.go']

def load_review():
 assert h(checkpoint/'receipt.json')==RECEIPT_SHA256
 assert h(checkpoint/'source-manifest.json')==SOURCE_MANIFEST_SHA256
 receipt=json.loads((checkpoint/'receipt.json').read_bytes())
 assert receipt['status']=='passed_reviewed_registration_order_fix_with_selected_tests_and_exact_legacy_correspondence'
 assert receipt['ordinary_interface_inspection']=={'inspected_paths':1920,'distinct_certificate_byte_sets':1443,'reserved_cases_names':0,'acceptance_claim':False}
 assert receipt['checker_stages']==68 and receipt['rust_tests']==56
 rows={x['path']:x for x in receipt['changed_existing_files']};assert set(rows)==set(EXPECTED_PATHS)
 manifest=json.loads((checkpoint/'source-manifest.json').read_bytes())
 for group in ['source_hashes','fixture_hashes','document_hashes']:
  for p,sha in manifest[group].items():assert h(r/p)==sha,p
 return rows

def verify_source_or_reviewed_registration_delta(root,path,original_sha):
 current=h(root/path)
 if current==original_sha:return 'exact_original_bytes'
 rows=load_review();assert path in rows,path
 row=rows[path];assert original_sha==row['before_raw_sha256'],path
 assert current==row['raw_sha256'],path
 return 'exact_reviewed_canonical_cases_registration_only_delta'
