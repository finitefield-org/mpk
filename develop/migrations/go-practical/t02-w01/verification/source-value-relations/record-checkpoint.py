"""Record only actual terminal source-value relation results; W01 stays open."""
from pathlib import Path
import json
ROOT=Path(__file__).resolve().parent
v=json.loads((ROOT/'verification.json').read_bytes())
assert v['normal_final_streams']==83 and v['all_distinct_paired_streams']==97 and not v['R01_complete']
c=v['independent_counts'];maximum=v['maxima']
scope=(f"Closed source-derived complete encoding relations cover every original type: all word bits, encoded array padding, full256 string bytes, complete slice nil/owner/path/offset/length/capacity, all struct fields and every same-U error field. Checked observations/equivalence/equality-lift/zero, whole-field assembly/projections, copy/repeated-copy/nil/box/active transfers retain all prior source-specific identities. Final83 rows include5 small, all71 large and7 heterogeneous rows. Both cores agree on97 distinct streams including5 original pilot rows:89 zero-axiom math acceptances and8 genuine type mismatches. Independent comparison checks{c['closed_definitions']} closed definitions/{c['closed_type_and_field_law_statements']} type/field law statements/{c['whole_error_row_statements']} whole-error statements and{c['actual_encoded_scalar_leaf_changes_and_identical_zero_observations']} actual changed/identical encoded leaf observations. Only the unary nil-only error adapter changes among100 source-type choices. Isolated fixed sources pass small/failed-first-control checks; four fresh complete-library/boundary streams replay literally and final worktree bytes match the tested isolated closure. The original failed suite retains its actual two passed control roots; only its setup-failed first root reruns. Scoped Clippy/format pass. Large maxima{maximum['declarations']} declarations/{maximum['terms']} terms/{maximum['bytes']} bytes fit unchanged native caps. No Go comparison, source/input validity, function extensionality, axiom, reduced U or admission cap. Source/runtime/program/context, full56/private-registry/nativeLinux/all169/T02-T11 remain due.")
rpath=ROOT.parent.parent/'requirements.json';requirements=json.loads(rpath.read_bytes())
r01=next(r for r in requirements['requirements'] if r['id']=='T02-W01-R01')
r01.setdefault('prior_ordinary_checkpoints',[]).insert(0,r01['latest_ordinary_checkpoint'])
r01['latest_ordinary_checkpoint']={'evidence':'verification/source-value-relations/verification.json','executor':'crates/mpk-vc/src/go_practical_ordinary/source_value_relations.rs','scope':scope,'not_application_verification_closure':True,'public_profile_activated':False,'R01_complete':False,'R06_complete':False}
r07=next(r for r in requirements['requirements'] if r['id']=='T02-W01-R07')
r07.setdefault('prior_scoped_checkpoints',[]).insert(0,r07['latest_scoped_checks'])
r07['latest_scoped_checks']={'evidence':'verification/source-value-relations/verification.json','language_scope':['Go','Rust'],'paired_current_byte_streams':97,'zero_axiom_positives':89,'genuine_core_type_mismatches':8,'normal_final_source_relation_rows':83,'normal_large_rows':71,'heterogeneous_rows':7,'additional_original_pilot_rows':5,'independent_counts':c,'source_epochs':len(v['source_epochs']),'original_input_rows':v['source_preimages'],'Go_complete_prelaunch_inputs':v['Go_prelaunch_rows'],'closed_adapter_choices':100,'only_nil_only_error_changes':True,'fresh_unchanged_boundary_byte_replay_streams':4,'actual_isolated_source_worktree_binding':True,'original_setup_failure_and_two_passed_control_roots_preserved':True,'only_failed_first_control_root_rerun':True,'original_native_handles_preserved_without_observation_restart':True,'strict_clippy_and_targeted_format_passed':True,'all_actual_validation_handles_terminal':True,'unrelated_prior_suites_repeated':False,'full_repository_gate_passed':False,'full_T02_gate':'Deferred to final GO-PRAC-T02-W04: ./scripts/check-fast.sh'}
rpath.write_text(json.dumps(requirements,indent=2)+'\n')
text=f'''

## 168. T02-W01 complete original-source value relations and Go/Rust routing

The [source-value checkpoint](../migrations/go-practical/t02-w01/verification/source-value-relations/README.md)
derives closed observations from every original same-U source type. Words,
array padding, all string bytes, complete slice descriptors/ghost fields,
ordered structs and every error payload remain. Equivalence, equality lifts,
zero, whole-field assembly/projections and copy/nil/box/active transfers are
ordinary zero-axiom proofs. All prior source-specific identities remain;
unchanged call-site libraries retain their own complete old proofs. No source
case/root/helper/parameter cap or function extensionality is introduced.

Both cores agree on97 distinct full streams:89 mathematical acceptances and8
genuine type mismatches. Final83 normal rows cover5 small,71 large and7
heterogeneous rows;5 original pilot rows remain separate. Independent checks
cover{c['closed_definitions']} closed definitions/{c['closed_type_and_field_law_statements']} type/field statements/
{c['whole_error_row_statements']} whole-error statements/{c['actual_encoded_scalar_leaf_changes_and_identical_zero_observations']} actual encoded leaf changes
and identical-zero observations. Complete tag observations remain. A coherent
constant byte view is valid mathematics but fails exact source import and the
independent identity-body oracle.

The missing-projection, whole-product fuel, nil-only function-type and first
control fixture failures retain actual inputs/outputs. The adapter audit
checks100 source-type choices; only nil-only error changes. Fixed isolated
sources pass the affected small/first-control checks, four complete-library/
boundary byte replays and literal final worktree binding. The initial control
suite's two passed roots remain; only its failed first root reruns. Scoped
Clippy/format pass. Maxima{maximum['declarations']} declarations/{maximum['terms']} terms/
{maximum['bytes']} bytes fit unchanged caps. All{len(v['source_epochs'])} source epochs/
{v['source_preimages']} original source rows/{v['Go_prelaunch_rows']} Go prelaunch rows
remain. Native verification is never restarted after an observation timeout.

The remaining active multi-language roadmap routing is retired. Complete old
plan bytes and the intervening retirement notice are preserved as historical
records; current routing leads only to Go/Rust. No other-language test or
workflow runs.

These relations are not Go comparison, source/input validity or program
correspondence. Full validity/execution/callee/result/effect/frame/context,
full56/private registry/nativeLinux/all169/T02-T11 remain due. Prior historical
scope findings and semantic binary handling remain for the final strict gate.
Go/Rust only; R01/R06/W01 stay open, the practical profile stays inactive and
the cumulative T02 gate remains final W04: `./scripts/check-fast.sh`.
'''
ledger=Path('develop/docs/go-practical-implementation-traceability-ledger.md');assert '## 168.' not in ledger.read_text();ledger.write_text(ledger.read_text()+text)
readme=ROOT.parent.parent/'README.md';readme.write_text(readme.read_text()+text.replace('## 168. T02-W01 complete original-source value relations and Go/Rust routing','## Complete original-source value relations').replace('../migrations/go-practical/t02-w01/verification/source-value-relations/README.md','verification/source-value-relations/README.md'))
print(json.dumps({'status':'checkpoint168_recorded_W01_remains_open','source_epochs':len(v['source_epochs']),'R01_complete':False,'R06_complete':False}))
