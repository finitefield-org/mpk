"""Record the production source-only handoff and its actual targeted evidence."""
from pathlib import Path
import hashlib,json
ROOT=Path(__file__).resolve().parent;REPO=ROOT.parents[5]
def seal(p):
 b=Path(p).read_bytes();return {'path':str(p),'bytes':len(b),'sha256':hashlib.sha256(b).hexdigest()}
checks=['native','native-replacement-corrected','derived','rust-controls','lint','go-vet','final-format','final-go-format','original-comparison'];results={n:json.loads((ROOT/n/'result.json').read_bytes()) for n in checks};assert all(r['status']=='passed' for r in results.values())
assert json.loads((ROOT/'rust/result.json').read_bytes())['status']=='failed' and json.loads((ROOT/'native-replacement-original/result.json').read_bytes())['status']=='failed'
assert (ROOT/'final-go-format/stdout.log').read_bytes()==b''
comparison=results['original-comparison'];assert comparison['whole_prior_proof_streams_replayed']==32 and comparison['all_four_Rust_roots_verified_across_affected_epochs']
archives=json.loads((ROOT/'native-original-relocations.json').read_bytes());assert archives['status']=='passed';files=sum(r['files'] for r in archives['groups']);total=sum(r['bytes'] for r in archives['groups'])
lint=json.loads((ROOT/'lint/inputs.json').read_bytes());records={r['path']:r for r in lint['sources']}
changed=['crates/mpk-vc/src/go_practical_type_keys.rs','crates/mpk-vc/src/go_practical_type_keys/source_inputs.rs','crates/mpk-vc/tests/go_practical_source_inputs.rs','go-tools/go2vir/practical_source_inputs.go','go-tools/go2vir/practical_source_inputs_test.go','go-tools/go2vir/practical_source_string_operation_test.go','go-tools/go2vir/practical_source_invocation_test.go','go-tools/go2vir/practical_storage_test.go']
for name in changed:
 r=records[name];assert seal(name)=={k:r[k] for k in ['path','bytes','sha256']} and Path(name).stat().st_mode&0o777==r['mode']
summary=('The intact original-package owner now produces source-only analysis inputs in production code. '
'Every non-intrinsic original export package contributes its complete active original Go source set and exact module manifest, '
'including unselected files and bodies. Returned bytes and objects are independently owned, remain intact after owner/image/preparation retirement, '
'and cannot mutate the original universe. A private identity couples the complete image/compiler/main/requested-package provenance to the source component. '
'The existing native source-unit test helper delegates to this producer. The Rust source-only importer requires the complete package/source set '
'and the fixed logical Bool seed, rejecting native declarations/type/alias/underlying hints. Actual source types are independently derived without '
'a new source type/argument/result/case/root limit.')
evidence=('Actual pinned original Go compilation and immutable original-package reconstruction pass for both unchanged zero/70-error-case units and a real '
'local replacement selected package whose own module differs from the main module. The replacement retains its unselected channel/callback and init-panic bodies; '
'no application initializer executes. The complete production input bytes for both original units equal the prior source-only packets. '
'Independent source analysis derives25/95 actual types, all65 argument positions,16 results and complete zero/70-case universes. '
'All32 entire original proof certificate byte streams replay literally, preserving the previously checked complete named-union proof evidence. '
'The current32 native call/caller/group/signature observations also equal the prior complete observations. '
'All four new Rust roots pass across the original three successful roots and the isolated corrected control rerun. '
'Coherent generic-valid native uint16 data rejects at the source-only boundary; incomplete package roots already reject at the shared Reference boundary, '
'and missing/changed original bytes reject. Closed/incomplete Go owners and mutations of every returned source byte/object reject or remain isolated. '
'Scoped strict Clippy, Go vet and changed-file Rust/Go format checks pass. Unchanged kernels and large prior proof matrices are not repeated.')
failure=('The first complete replacement fixture produced a valid intact original owner, then exposed an accidental main-module-only restriction in the new handoff. '
'The corrected handoff retains the separately owned main manifest and the requested package\'s actual original module without introducing root admission. '
'The complete original failed Go inputs/export/SDK outputs and exact source remain. The first Rust suite passed source derivation/full32-byte replay, replacement and '
'raw-source mutation roots; its remaining control incorrectly expected missing package roots to pass the shared generic reader. '
'Only that expectation was corrected to the actual shared Reference rejection, and only the affected control root reran. '
'All original runtime logs, inputs and drivers remain; no semantic/source/case/root is pruned, no new cap or axiom is introduced, and no live verification restarts on an observation timeout. '
 f'All three native original trees, {files} files/{total} bytes and original modes, remain losslessly archived and physically retained under unique no-overwrite paths.')
selection=('Test selection follows the production handoff, the changed source-unit helper and independent source-only reader. '
'The existing full original native invocation fixture directly exercises the new producer and ownership boundaries. '
'The distinct replacement root targets the actual observed restriction; the failed Rust control alone reruns after its test-only correction. '
'Existing execution matrices, unchanged mathematical kernels and unrelated Go/Rust suites are not repeated. '
'The complete prior32 proof bytes and native call observations supply direct consumer equivalence, without inventing new kernel reports. '
'No intermediate broad gate or decomposed equivalent is executed.')
remaining=('These are privately owned source-only candidates and provenance identities, not authenticated installed capture contexts, source admission, '
'callee execution/precondition/result/effect/frame proofs, operand-once/order proofs or source/IR/program correspondence. '
'Complete context/descriptor/private registry/capture and foundation assembly, all169 theorem demands, native Linux release gates and remaining T02-T11 work remain due. '
'R01/R06/W01 stay open and the practical profile stays inactive. The cumulative T02 gate remains final W04: ./scripts/check-fast.sh. '
'Go/Rust only; no removed-language implementation/test/release prerequisite is added.')
record={'schema':'mpk.go_practical.source_inputs_checkpoint.v1','status':'scoped_source_handoff_checks_passed_complete_W01_due','basis_commit':'7aa3006df1b86ed465c123236ef7b0dcfc974778','scope':summary,'verification':evidence,'original_failure_retention':failure,'test_selection':selection,'remaining':remaining,'whole_prior_proof_streams_replayed':32,'actual_source_types':[25,95],'actual_error_cases':[0,70],'native_original_files':files,'native_original_bytes':total,'checks':{n:seal(ROOT/n/'result.json') for n in checks},'retained_failed_epochs':{n:seal(ROOT/n/'result.json') for n in ['native-replacement-original','rust']},'kernel_matrix_repeated':False,'all32_individual_wrappers_kernel_accepted_claimed':False,'T02_W01_complete':False,'R01_complete':False,'R06_complete':False,'public_profile_activated':False,'source_program_or_context_proof_authority':False,'native_Linux_release_claimed':False,'full_T02_gate':'deferred to final W04: ./scripts/check-fast.sh'}
(ROOT/'verification.json').write_text(json.dumps(record,indent=2)+'\n')
(ROOT/'README.md').write_text('# Owned original-source input handoff\n\n'+summary+'\n\n'+evidence+'\n\n'+failure+'\n\n'+selection+'\n\n'+remaining+'''

- [Actual scoped checks](verification.json)
- [Complete original/native/prior-proof comparison](original-comparison/result.json)
- [All32 literal proof replays](rust/observations/original-proof-replay.json)
- [Corrected replacement owner](native-replacement-corrected/result.json)
- [Isolated control rerun](rust-controls/result.json)
- [Complete original preservation](native-original-relocations.json)
- [Scoped input review](input-review.json)
''')
p=ROOT.parents[1]/'requirements.json';r=json.loads(p.read_bytes())
for item in r['requirements']:
 if item['id'] in ['T02-W01-R01','T02-W01-R06']:
  item['source_only_input_handoff_checkpoint']={'evidence':'verification/source-inputs/verification.json','scope':summary,'verification':evidence,'remaining':remaining,'R01_complete':False,'R06_complete':False,'W01_complete':False,'public_profile_activated':False}
p.write_text(json.dumps(r,ensure_ascii=False,indent=2)+'\n')
p=REPO/'develop/docs/go-practical-implementation-traceability-ledger.md';assert '## 175.' not in p.read_text();p.write_text(p.read_text()+'\n\n## 175. T02-W01 owned original-source input handoff\n\nThe [source-input checkpoint](../migrations/go-practical/t02-w01/verification/source-inputs/README.md) moves complete source-only unit production out of a fixture and into the intact original-package owner.\n\n'+summary+'\n\n'+evidence+'\n\n'+failure+'\n\n'+remaining+'\n')
p=ROOT.parents[1]/'README.md';p.write_text(p.read_text()+'\n\n## Owned original-source input handoff\n\nThe [source-input checkpoint](verification/source-inputs/README.md) connects the intact native package owner to the strict independent source reader.\n\n'+summary+'\n\n'+evidence+'\n\n'+remaining+'\n')
print(json.dumps({'status':'recorded','whole_prior_proof_streams_replayed':32,'native_original_files':files,'native_original_bytes':total,'W01_complete':False}))
