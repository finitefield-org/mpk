"""Audit and preserve the tested private construction type checkpoint."""
from datetime import datetime, timezone
from hashlib import sha256
import json
from pathlib import Path
import shutil

repo = Path('/private/tmp/mpk-w09-packed-pattern-proofs')
root = Path('/tmp/mpk-w09-construction-storage-types')
base = repo / 'develop/migrations/csharp-03/ordinary-foundation/verification-logs/construction-storage-types'
assert not base.exists()
state = json.loads((root / 'attempt-2/status.json').read_bytes())
assert state['status'] == 'passed_targeted_tests_lint_format'
assert state['unique_tests_passed'] == 4
assert [s['stage'] for s in state['stages']] == ['storage-type-proofs', 'private-storage-semantics', 'original-type-proofs', 'original-type-pins', 'clippy', 'format', 'support-format']
for stage in state['stages']:
    assert stage['exit_code'] == 0
    assert sha256((root / 'attempt-2' / (stage['stage'] + '.log')).read_bytes()).hexdigest() == stage['log_sha256']
for binary, digest in state['test_binaries'].items():
    assert sha256(Path(binary).read_bytes()).hexdigest() == digest
manifest_bytes = (root / 'source-manifest.json').read_bytes()
manifest = json.loads(manifest_bytes)
assert manifest_bytes == (root / 'attempt-2/source-manifest.json').read_bytes()
assert (len(manifest['source_hashes']), len(manifest['fixture_hashes'])) == (747, 173)
for group in ('source_hashes', 'fixture_hashes'):
    for name, digest in manifest[group].items():
        assert sha256((repo / name).read_bytes()).hexdigest() == digest, name
old_manifest = json.loads((root / 'attempt-1/source-manifest.json').read_bytes())
assert old_manifest['fixture_hashes'] == manifest['fixture_hashes']
assert old_manifest['source_hashes'].keys() == manifest['source_hashes'].keys()
changed = [n for n in manifest['source_hashes'] if old_manifest['source_hashes'][n] != manifest['source_hashes'][n]]
assert changed == ['crates/mpk-vc/tests/support/csharp_practical_ordinary_construction_type_tests.rs']
def catalog(folder):
    return {p.name: sha256(p.read_bytes()).hexdigest() for p in folder.iterdir() if p.is_file()}
expected = json.loads((root / 'expected-certificates.json').read_bytes())
assert len(expected) == 92
assert expected == catalog(root / 'attempt-1/certificates') == catalog(root / 'attempt-2/certificates')
assert {'certificates/' + n: d for n, d in expected.items()} == state['exported_certificate_hashes']
prior = base.parent / 'concrete-type-proofs/attempt-3/certificates'
legacy = catalog(root / 'attempt-2/legacy-certificates')
assert len(legacy) == 91
assert legacy == state['legacy_certificate_hashes'] == catalog(prior)
metadata = [json.loads(p.read_bytes()) for p in (root / 'attempt-2/certificates').glob('*.json')]
assert len(metadata) == 45
assert sum(len(m['proofs']) for m in metadata) == 87
assert sum(len(m['pending_type_instances']) for m in metadata) == 0
assert sum(len(m['pending_proof_ids']) for m in metadata) == 987
domains = [d for m in metadata for d in m.get('construction_storage_domains', [])]
assert len(domains) == 6 and all(d['private_storage_only'] and d['ownership_pending'] for d in domains)
base.mkdir()
(base / 'tool-sources').mkdir()
path_map = {}
for attempt in ('attempt-1', 'attempt-2'):
    for old in sorted((root / attempt).rglob('*')):
        if not old.is_file():
            continue
        rel = old.relative_to(root)
        dest = base / rel
        if old.suffix == '.log':
            dest = dest.with_name(dest.name + '.txt')
        dest.parent.mkdir(parents=True, exist_ok=True)
        shutil.copyfile(old, dest)
        assert old.read_bytes() == dest.read_bytes()
        path_map[str(rel)] = str(dest.relative_to(base))
for name in ('source-manifest.json', 'expected-certificates.json'):
    shutil.copyfile(root / name, base / name)
for name in ('run-targeted.py', 'check.py', 'prepare-receipts.py'):
    shutil.copyfile(root / name, base / 'tool-sources' / name)
(base / 'receipt-path-map.json').write_text(json.dumps(path_map, indent=2, sort_keys=True) + '\n')
verification = dict(
    status='passed_local_targeted_verification', recorded_at=datetime.now(timezone.utc).isoformat(),
    unique_tests_passed=4, private_domain_observations=247, source_contexts=45,
    supplied_concrete_type_proofs=87, concrete_type_instances_pending=0,
    private_storage_domains=6, source_ownership_pending=True,
    application_proofs_pending=987, concrete_operations_pending=25,
    original_is_binding_refinement_proofs_pending=7, complete_application_assembly_pending=True,
    source_files_verified=747, fixture_files_verified=173,
    exported_files_verified=92, legacy_exported_files_verified=91,
    legacy_certificate_bytes_unchanged=True,
    selection_reason=state['selection_reason'], local_receipt='attempt-2/status.json',
    failed_attempt='attempt-1/status.json',
    test_only_repair=changed, proof_generator_and_production_source_unchanged_after_first_export=True,
    checker_inputs_match_final_local_exports=True,
    dual_checker_verification='pending independent final-input audit of terminal receipts',
    linux_verification='pending exact published-source terminal receipt',
    full_t_gate='deferred to T06-W12', w09_status='In progress', w10_w12_status='Blocked',
)
(base / 'verification.json').write_text(json.dumps(verification, indent=2, sort_keys=True) + '\n')
review = """# Private construction storage type equivalence proofs

This partial T06-W09 checkpoint adds an opt-in type route for the six original
sequence-construction instances. It supplies all 87 original type equivalence
proof candidates across 45 source contexts; the old route remains at 81 with
its six internal instances pending. The new route has no pending type instances,
but every private storage domain explicitly retains ownership_pending=true.
All 987 application proof IDs remain pending. W09 stays In progress and W10-W12
stay Blocked; the full T06 gate remains deferred to T06-W12.

## Domain and proof construction

The private three-field layout retains the complete 32-bit length, 16,384
element cells and initialization bitmap. Length must be at most 16,384.
Initialized cells use the existing recursive public child domain and complete
source clauses. Uninitialized cells must have all-zero physical storage; their
element need not have a default public value. Both contribute their proper
logical-cell counts, including the enclosing cell, with saturation at 65,537.
Field padding, the unused fourth role, inactive cells and bitmap tail must be
zero. The count/validity definitions enforce the complete 65,536-cell bound.

Each instance must match the exact original W06 construction type descriptor
and element dependency. Builder.resume retains every old term/declaration and
all public domains, source clauses, definitions, conditions and historical pins.
The added wrappers reconstruct every complete original concrete_type_equivalence sequent
and both original domain/definition operands. Ordinary Std.Eq reflexivity proves
these definitional equations without additional assumptions or axioms. Strict
imports regenerate the exact source-specific metadata and certificate bytes.

The domains describe private storage only. Origin/current SSA version, lifetime,
exclusive ownership, initialization/publication capabilities and default
eligibility remain separate obligations. No ownership bit, public capability,
kernel rule, foundation schema, Certificate v0 rule or acceptance behavior is
introduced. All 25 remaining operations, seven original is_binding refinements,
native execution establishment and application assembly remain pending.

## Verification and direct review

Four targeted local tests pass: all 45 source contexts and 87 supplied proofs,
six private-domain contexts with 247 complete observations, the affected legacy
type proof regression and all 45 historical type pins. The shared proof helper
and serialized type metadata changed, so both legacy consumers are included.
Other generators and operation algorithms are unchanged. Crate Clippy with
warnings denied, crate format and both changed support-file format checks pass.

The tests independently reconstruct original W06 sequents and inspect the full
theorem operands, premise shape and old certificate prefix. They exercise strict
imports, changed metadata/context, size bounds and actual unchanged Rust kernel
acceptance with zero axioms. A well-typed always-true private type definition
remains accepted without proof candidates and is rejected at core checking
when the supplied proofs are present. The earlier wrong-proof negative remains.

The semantic matrix covers full length 16,384, complete high length bits,
every padding selector branch, both ends of unused storage and inactive cells,
inactive bitmap indices and nonzero uninitialized cells. Four exact source
products containing nullable strings reach independently counted totals
65,535/65,536/65,537, checked against separately generated child public domains.

The initial semantic failure was a nominal test fixture error: the source
element is a product containing a nullable-string member, but the fixture used
the nullable value directly. The repaired test preserves the exact source
product. No production or proof-generator bytes changed after the first proof
export. The failed attempt and the later prelaunch missing-catalog failure are
retained with their actual results. The final 747-source/173-fixture audit passes,
all 92 proof exports exactly match the first passing proof stage and all 91
legacy exports match the preceding published type corpus.

Dual-checker verification awaits independent final-input audit. Requested Linux
verification remains pending an exact published-source terminal receipt.
See verification-logs/construction-storage-types/.
"""
(base.parent.parent / 'unit-7-construction-storage-types-review.md').write_text(review)
todo = repo / 'develop/docs/08_csharp_practical_subset_design-todo.md'
text = todo.read_text()
anchor = 'Owns: translation of every already-expanded concrete foundation definition in\n'
assert text.count(anchor) == 1
addition = """The opt-in private construction storage type route now adds the six original
internal type instances and supplies all 87 original type-equivalence proof
candidates across 45 source contexts. Every added domain retains private-only
storage and ownership_pending=true. Complete length/cell/bitmap/padding/tail and
recursive source child domains enforce the 16,384-element and 65,536-cell bounds.
Four targeted tests, 247 complete semantic observations, Clippy and both format
checks pass locally. The old 81-proof route, all 45 original type pins and all
91 legacy outputs remain unchanged. The tested audit verifies 747 source and
173 fixture hashes. Dual-checker final-input audit and exact published-source
Linux replay remain pending terminal receipts. All 25 remaining operations,
seven original is_binding refinements, source ownership and all 987 application
proof IDs stay pending. W09 remains In progress, W10-W12 remain Blocked and the
full T06 gate remains deferred to W12. See
`ordinary-foundation/unit-7-construction-storage-types-review.md`.

"""
todo.write_text(text.replace(anchor, addition + anchor))
print(json.dumps(dict(status='passed', local_tests=4, private_observations=247,
                      source_files=747, fixtures=173, supplied_type_proofs=87,
                      final_exports_identical_to_first=92, legacy_exports_unchanged=91)))
