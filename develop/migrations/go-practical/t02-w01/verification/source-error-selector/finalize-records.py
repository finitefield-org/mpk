"""Publish scoped records only after the terminal full-stream audit passes."""
from pathlib import Path
import json
import re
ROOT=Path(__file__).resolve().parent
v=json.loads((ROOT/'verification.json').read_bytes())
assert v['status']=='scoped_checks_passed' and v['all_actual_validation_handles_terminal']
assert (v['distinct_full_streams_checked_in_both_cores'],v['normal_original_streams'])==(99,92)
stats=v['N70'];epochs=v['source_epochs'];inputs=v['original_input_rows'];go_inputs=v['Go_complete_prelaunch_inputs']
draft=ROOT/'code-review-precompletion.json'
review=json.loads((draft if draft.exists() else ROOT/'manual-review.json').read_bytes())
review['status']='code_review_and_scoped_validation_passed'
review['sources']=v['production_sources']
review.pop('remaining_verification',None)
completion_check='All71 large original target-selection rows pass both complete cores; the independent frozen selector/statement/actual-DAG observation and large final-type/first-target false claim also pass. Current scope evidence is verified by summarize.py rather than inferred from code review.'
if completion_check not in review['checks']:review['checks'].append(completion_check)
review.update(verification='verification.json',full_repository_gate_passed=False,complete_R01_R06_W01=False,remaining=v['remaining'])
(ROOT/'manual-review.json').write_text(json.dumps(review,indent=2)+'\n')
if draft.exists():draft.unlink()
(ROOT/'README.md').write_text(f"""# Original-source error dispatch target selection

Entry: `1a0058ffe62998822209223f109276b5b6ef2f39`. Go/Rust only.
[Scoped verification](verification.json), [manual review](manual-review.json),
[input review](input-review.json). R01/R06/W01 stay open; the public practical
profile remains inactive.

The immutable original Error selection supplies every same-U concrete type,
exact method declaration/body/receiver path/signature and original call/receiver
site, including method-expression form. A private preparation identity binds
that complete table. It is not a canonical capture/compiler/context identity.
Every ordinary suffix scans the full target list, returning a named Nat ordinal
for a match and NoTarget Nat0 otherwise. Checked true/false branch equations,
old Matches/NilHasNoCase witnesses and every suffix prove the exact final target
for arbitrary full boxed payloads, or no target for nil. Every prior case-row
and source type/zero/constructor/nil/copy/payload/comparison identity is retained.
No target/case/root/helper/parameter cap or caller-supplied table/oracle is added.

Both cores agree on99 full streams:93 zero-axiom mathematical acceptances and6
genuine type mismatches. Normal92 comprise21 small-site rows and all71 large
rows (70 concrete types plus nil). Small sites include method expressions,
receiver-returning calls and a nil-only secondary root under the full global U;
boxing-only has no dispatch sites. Original Go observations compare call and
receiver physical spans and every exact target. Independent comparison checks
10,336 selector definitions,15,274 complete selector theorem statements,
9,975 prior case-row statements,11,348 old operation statements and327,808
actual tag bits. It also evaluates the ACTUAL decoded selector/Box/Zero DAG
for all92 selected-target observations. All78 prior case-row bytes replay
literally after the private builder split. Scoped Clippy/format pass.

Five small and one large wrong-target/erased-branch/forced-branch/nil claims
reject by actual core type mismatch. A coherently changed named target ordinal
is valid mathematics but fails exact import and the independent frozen target
body. A different accepted call-site row cannot replace the requested site,
even with the same U and methods. These accepted alternatives are not core
negative evidence. Large rows have at most{stats['maximum_declarations']:,}
declarations,{stats['maximum_terms']:,} terms and{stats['maximum_bytes']:,} bytes
under unchanged native caps. All{epochs} source epochs/{inputs:,} original input
rows and{go_inputs} current Go prelaunch originals, bytes/modes/drivers/raw
outputs are retained.

The initial missing-entry fixture stopped before generating proof candidates.
The first wrong-ordinal control referenced a later declaration and failed
certificate construction; its raw Nat2 replacement is correctly ordered. The
initial Clippy unused-import failure is retained and corrected. All original
versions and failures remain distinct from source counterexamples and genuine
core-negative controls; no observer timeout caused a native run restart.

NoTarget proves no receiver non-nil safety, target precondition, method-body
run, joint result/effect/frame, input validity or source/program relation.
Those complete proofs, canonical capture/module/U/tag/target binding, full56
foundation/private registry/context, native Linux and all169/T02-T11 remain
due. Earlier historical-scope scanner findings remain for the final strict
gate. The cumulative T02 gate stays final W04: `./scripts/check-fast.sh`.
""")
p=Path('develop/migrations/go-practical/t02-w01/requirements.json');d=json.loads(p.read_bytes())
r01=next(r for r in d['requirements'] if r['id']=='T02-W01-R01');r07=next(r for r in d['requirements'] if r['id']=='T02-W01-R07')
checkpoint={'evidence':'verification/source-error-selector/verification.json','executor':'crates/mpk-vc/src/go_practical_ordinary/source_error_dispatch.rs and source_error_cases.rs','scope':f'Complete original Error selection supplies every same-U case/exact method declaration/body/receiver path/signature and original caller/call/receiver/method-expression site. Private preparation names bind the complete source table; no canonical context or method-execution authority is inferred. Actual ordinary suffix/true-false transfer/value equations select named target ordinals for arbitrary full boxed payloads and NoTarget for nil, retaining every prior case-row/source type/zero/constructor/nil/copy/payload/comparison identity. Both cores agree on99 full streams:93 zero-axiom mathematical acceptances and6 genuine type rejections. Normal92 include21 small-site rows (expressions, receiver-call and multiple roots) and all70 large concrete types plus nil. Independent original Go physical call/receiver spans/target tables and frozen comparison cover10336 selector definitions/15274 complete selector statements/9975 old row statements/11348 old operation statements/327808 actual tag bits and92 actual decoded selector observations. All78 prior case rows replay literally; scoped Clippy/format pass. Named-ordinal coherent math alternative and another accepted call-site row fail exact import/frozen ownership; no such acceptance counts as core-negative evidence. Large maxima {stats["maximum_declarations"]} declarations/{stats["maximum_terms"]} terms/{stats["maximum_bytes"]} bytes under unchanged caps, no reduced U/case/root/helper/parameter cap. All{epochs} source epochs/{inputs} rows and{go_inputs} current Go prelaunch originals preserve missing-entry, forward-reference and lint failures with no observer-timeout restart. Nil safety/receiver-once/callee/precondition/joint-result/effect/frame execution, arbitrary-input validity, canonical capture/context/target binding and all remaining full56 foundation/private registry/nativeLinux/all169/T02-T11 proofs remain due.','not_application_verification_closure':True,'public_profile_activated':False,'R01_complete':False,'R06_complete':False}
r01['latest_ordinary_checkpoint']=checkpoint;r01['preparatory_source_error_target_selection_checkpoint']=dict(checkpoint)
if r07['latest_scoped_checks']['evidence']!=checkpoint['evidence']:r07['prior_scoped_checkpoints'].insert(0,r07['latest_scoped_checks'])
r07['latest_scoped_checks']={'evidence':checkpoint['evidence'],'language_scope':['Go','Rust'],'paired_current_byte_streams':99,'zero_axiom_positives':93,'genuine_core_type_mismatches':6,'normal_target_selection_rows':92,'large_concrete_target_rows':70,'large_nil_no_target_rows':1,'selector_definitions':10336,'complete_selector_theorem_statements':15274,'complete_prior_case_row_theorem_statements':9975,'actual_case_tag_bit_observations':321920,'actual_nil_tag_bit_observations':5888,'actual_decoded_target_observations':92,'literal_current_old_case_row_streams':78,'source_epochs':epochs,'original_input_rows':inputs,'Go_complete_prelaunch_inputs':go_inputs,'coherent_wrong_ordinal_and_wrong_accepted_call_site_ownership_rejected':True,'original_fixture_forward_reference_lint_failures_preserved':True,'original_native_handles_preserved_without_observation_restart':True,'strict_clippy_and_targeted_format_passed':True,'all_actual_validation_handles_terminal':True,'unrelated_prior_suites_repeated':False,'full_repository_gate_passed':False,'full_T02_gate':v['full_T02_gate']}
p.write_text(json.dumps(d,indent=2)+'\n')
p=Path('develop/docs/go-practical-implementation-traceability-ledger.md');s=p.read_text()
section=f"""
## 165. T02-W01 original-source error target selection

The [target-selection checkpoint](../migrations/go-practical/t02-w01/verification/source-error-selector/README.md)
binds every original call/receiver/method-expression site to the complete
same-U exact method declaration/body/receiver path/signature table in private
preparation names. Ordinary suffix/branch/value equations select the correct
named target for arbitrary full boxed payloads and NoTarget for nil. Every
prior case-row/source type/zero/constructor/nil/copy/payload/comparison proof
remains; no caller table/oracle, reduced U or new admission cap is introduced.

Both cores agree on99 full streams:93 zero-axiom mathematical acceptances and6
genuine type rejections. Normal92 include21 small-site rows and all70 large
concrete types plus nil. Original Go physical call/receiver spans and exact
targets agree. Independent comparison checks10,336 selector definitions,
15,274 full selector statements,9,975 old case-row statements,11,348 old
operation statements and327,808 actual tag bits;92 actual decoded selector
observations agree. All78 prior case rows replay literally. Scoped Clippy/
format pass; large maxima{stats['maximum_declarations']:,} declarations,
{stats['maximum_terms']:,} terms/{stats['maximum_bytes']:,} bytes fit unchanged
native caps. All{epochs} source epochs/{inputs:,} input rows and{go_inputs}
current Go prelaunch originals remain. Initial missing-entry, invalid forward
reference and unused-import failures remain separate from source failures and
genuine type controls. No observer timeout caused native execution restart.
A coherent wrong named ordinal and a different accepted call-site row fail
exact import and independent frozen ownership despite valid mathematics.

NoTarget is no nil safety/receiver-once/callee/precondition/joint-result/effect/
frame theorem. All source execution/validity/canonical capture/module/U/tag/
target/context, full56 foundation/private registry/native Linux/all169/T02-T11
proofs remain due. Earlier historical-scope findings remain for the final
strict gate. Go/Rust only; R01/R06/W01 stay open, the public practical profile
stays inactive and the full T02 gate stays final W04: `./scripts/check-fast.sh`.
"""
pattern=r'(?ms)^## 165\..*?(?=^## [0-9]+\.|\Z)'
if re.search(pattern,s):s=re.sub(pattern,lambda _:section.lstrip('\n'),s)
else:s+=section
p.write_text(s)
print(json.dumps({'records_updated':True,'source_epochs':epochs,'input_rows':inputs,'Go_prelaunch_inputs':go_inputs}))
