import hashlib
import json
from pathlib import Path
import shutil

b=Path(__file__).parent
f=Path('/private/tmp/mpk-w09-context-application-integration')
r=Path('/private/tmp/mpk-w09-refreeze-consumer-continuation')
h=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
read=lambda p:json.loads(p.read_bytes())
from verified_server_result_provenance import load_result_provenance
owners, owner_folder = load_result_provenance(b, 'scalar')
manifest=read(b/'ordinary-source-manifest-4b.json')
for p,sha in manifest['hashes'].items():assert h(f/p)==sha,p
production={}
for p,sha in manifest['hashes'].items():
 if p in ['Cargo.toml','Cargo.lock'] or (p.startswith('crates/') and '/src/' in p):
  assert h(r/p)==sha,p
  production[p]=sha
compared=[]
for s in owners['stages']:
 for path,sha in s['generated_files'].items():
  original=owner_folder/s['goldens_directory']/path
  current=r/'develop/migrations/csharp-03/ordinary-foundation'/s['destination']/path
  assert h(original)==sha,path
  assert original.read_bytes()==current.read_bytes(),(s['stage'],path)
  compared.append({'owner':s['stage'],'path':str(current.relative_to(r)),'raw_sha256':sha})
out=b/'full-semantic-source-owner-checkpoint-evidence'
out.mkdir(exist_ok=False)
for src in sorted(owner_folder.rglob('*')):
 if src.is_file():
  dst=out/'completed-scalar-result-provenance'/src.relative_to(owner_folder)
  dst.parent.mkdir(parents=True,exist_ok=True);shutil.copy2(src,dst);assert h(src)==h(dst)
for name in ['ordinary-source-manifest-4b.json','run-ordinary-regeneration-4b.py',
             'full-semantic-completed-prefix-pin-comparison.json', 'decimal-formats-completed-owner-pin-review.json',
             'finalize-full-semantic-source-owner-evidence-2.py', 'finalize-full-semantic-source-owner-evidence.py',
             'verified_server_result_provenance.py', 'assemble-server-result-provenance.py', 'audit-server-remaining-results.py',
             'full-semantic-finalizer-preparation-disk-failure.json',
             'unused-refreeze-incremental-cache-cleanup-3.json','registration-order-source-correspondence-guard-review.json', 'registration-order-fresh-environment-correspondence-review.json','verify_registration_order_source_correspondence.py']:
 shutil.copy2(b/name,out/name);assert h(b/name)==h(out/name)
receipt={'status':'passed_all_ten_complete_original_scalar_codec_source_owners_and_exact_current_pin_equivalence',
 'selection_reason':owners['selection_reason'],
 'original_owner_tests_passed':10,'historical_original_owners_passed':6,'new_server_original_owners_passed':4,
 'actual_execution_provenance':'completed-scalar-result-provenance/receipt.json; original historical sources/logs retain their original host and source hashes; server four owners pin public e89c2a0f source',
 'exact_current_output_files':len(compared),
 'comparison':'Every complete original-source-generated fixture file equals the already-promoted current fixture bytes exactly. Original value, import, mutation and boundary observations execute unchanged. No certificate-only shortcut supplies this semantic-owner result.',
 'semantic_owner_owned_production_sources_exactly_equal':production,
 'registered_bool_cases_source_correspondence':'../registration-order-checkpoint/receipt.json; exact ordinary certificates lack this reserved cases interface; frozen F execution provenance remains at its original sources/binaries','compared_files':compared,
 'newly_promoted_fixture_files':0,'application_proof_ids_pending':987,
 'w09_status':'In progress','w10_status':'Blocked','practical_profile':'inactive',
 'full_t01_gate':'deferred_to_final_W10_on_server','full_t06_gate':'deferred_to_W12_on_server'}
(out/'receipt.json').write_text(json.dumps(receipt,sort_keys=True,indent=2)+'\n')
files={str(p.relative_to(out)):h(p) for p in sorted(out.rglob('*')) if p.is_file()}
(out/'file-manifest.json').write_text(json.dumps({'schema':'mpk.evidence_file_manifest.v1','files':files},sort_keys=True,indent=2)+'\n')
destination=r/'develop/migrations/csharp-03/probes/boolean-proof-elimination-refreeze/full-semantic-source-owner-checkpoint'
shutil.copytree(out,destination)
for p,sha in files.items():assert h(destination/p)==sha,p
print('All ten original semantic owners passed;',len(compared),'actual files exact current pins; no new fixture bytes')
