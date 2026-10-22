"""Update scoped records after every original full-stream handle is terminal."""
from pathlib import Path
import json
import re
ROOT=Path(__file__).resolve().parent
v=json.loads((ROOT/'verification.json').read_bytes())
assert v['status']=='scoped_checks_passed' and v['all_actual_validation_handles_terminal']
assert (v['distinct_full_streams_checked_in_both_cores'],v['normal_original_streams'],v['genuine_core_type_mismatches'])==(100,92,7)
stats=v['N70'];epochs=v['source_epochs'];inputs=v['original_input_rows'];go_inputs=v['Go_complete_prelaunch_inputs']
checks=[
 'The catalogue still owns the complete original source Error selection, all source types/cases and exact method declarations/body/receiver paths/signatures, every caller/call/receiver/method-expression site and private preparation identity. No caller comparator, target table, arbitrary root oracle or reduced U is admitted.',
 'ReceiverTag is exactly Eq Word64 Tag(e)=the selected original source tag, or the frozen nil tag. Its domain is the complete same-U receiver carrier with every active/inactive payload. No source-validity or caller truth predicate is assumed.',
 'Every MatchByTag, suffix, SelectedByTag and nil/non-nil equation quantifies the same whole receiver and exact tag premise. Matching uses old checked same/different-tag transfer; nil rows prove every actual source tag differs from the frozen nil tag.',
 'CopyReceiverTag composes the checked TagOfCopy equation with the original premise. Copy selection and non-nil equations instantiate the same generic receiver theorems, preserving every payload rather than rebuilding a smaller value.',
 'ConstructedReceiverTag uses actual old TagOfBox or TagOfNil proof for the original full value. Constructed selection, non-nil and copied selection instantiate the new conditional theorems with that real witness. This checks that the premise can actually be established for the intended values.',
 'The private builder split closes the original selector row before appending receiver laws. All original selector/source/case/constructor/zero/nil/copy/comparison type/body/proof identities remain; all92 old selector streams replay with literal whole-byte equality.',
 'The independent frozen observer compares the complete ReceiverTag type and body and every complete conditional/constructed statement alongside the old source/selector structure and original Go physical spans/target tables. A separate lazy evaluator reads the actual decoded original/copied selector and nil observations.',
 'Six small premise/foreign-tag/target/nil/copy controls and one last-of70 wrong-target control reject by real core type mismatch in both cores. Coherently changed named ordinal math remains accepted, but exact import and frozen target-body comparison reject it. Different accepted callsite bytes fail expected-owner comparison even with identical complete U/targets.',
 'Both cores check92 complete normal streams, including all70 concrete cases plus nil, and8 complete control streams. Strict scoped Clippy/format pass; every source epoch and Go prelaunch original, executable mode, complete raw output and actual driver is retained. No observation timeout restarted a native test.',
 'No new axiom, kernel/conversion rule, native cap, source language, case/root/helper/parameter cap or public activation. Receiver evaluation once, nil safety at the call, callee run/preconditions/results/effects/frame, source validity, canonical capture/context/program binding and full T02-T11 requirements remain open.'
]
review={'status':'code_review_and_scoped_validation_passed','reviewer':'primary agent; no delegation','sources':v['production_sources'],'checks':checks,'verification':'verification.json','full_repository_gate_passed':False,'complete_R01_R06_W01':False,'remaining':v['remaining']}
(ROOT/'manual-review.json').write_text(json.dumps(review,indent=2)+'\n')
(ROOT/'README.md').write_text(f'''# Source-error receiver tag premises

Entry: `5fbad134bc14cd702477db3d5f756169a18b0cdc`. Go/Rust only.
[Scoped verification](verification.json), [manual review](manual-review.json),
[input review](input-review.json). R01/R06/W01 stay open; the public practical
profile remains inactive.

The complete original source Error selection, all same-U types/payloads,
exact methods and original call/receiver sites remain. ReceiverTag is the
ordinary equality Tag(e)=the selected source tag (or the frozen nil tag),
quantified over an arbitrary full receiver. Checked match/suffix/final
selection, nil/non-nil and copy equations consume that exact premise.
Actual Box and Nil witnesses establish it, then instantiate the conditional
selection/non-nil/copied-selection equations. No caller-supplied truth
predicate or source-validity assumption is introduced.

Both full cores agree on100 distinct whole streams:93 zero-axiom mathematical
acceptances and7 genuine type rejections. Normal92 comprise21 small-site
rows and all71 large rows (70 concrete types and nil). The independent
observer compares92 complete receiver predicate definitions and11,157 whole
receiver statements, alongside10,336 selector definitions/15,274 selector
statements,9,975 prior case statements,11,348 old operation statements and
327,808 actual tag bits. Separate actual decoded DAG evaluation checks all92
original selections and276 copied-selection/nil observations. All92 previous
selector rows replay with literal whole-byte equality after the private
builder split. Strict scoped Clippy and format pass.

Six small and one large false predicate/target/nil/copy claim reject by actual
core type mismatch. A coherent changed named ordinal remains valid math but
fails exact import and the frozen source-target body. Different accepted
callsite bytes fail the requested owner even with the same complete U and
methods. Those mathematical acceptances are not negative core evidence.
Large maxima{stats['maximum_declarations']:,} declarations/{stats['maximum_terms']:,}
terms/{stats['maximum_bytes']:,} bytes fit unchanged4096/131072/2097152/1024
native caps. All{epochs} source epochs/{inputs:,} original rows and{go_inputs}
Go prelaunch originals, bytes/modes/drivers/raw outputs remain. No observer
timeout restarted a native run; any retained failed epoch stays separately
classified in verification.json.

A tag equality establishes these ordinary target/nil/copy equations only.
Receiver evaluation once, nil safety at source calls, callee preconditions/
execution/results/effects/frame, arbitrary-input/inactive-case validity,
canonical capture/module/U/tag/target/context binding and full56 foundation/
private registry/native Linux/all169/T02-T11 proofs remain due. Earlier
historical-scope scanner findings remain for the final strict gate. The
cumulative T02 gate stays final W04: `./scripts/check-fast.sh`.
''')
p=Path('develop/migrations/go-practical/t02-w01/requirements.json');d=json.loads(p.read_bytes())
r01=next(r for r in d['requirements'] if r['id']=='T02-W01-R01');r07=next(r for r in d['requirements'] if r['id']=='T02-W01-R07')
scope=f'Complete same-U source receiver tag equality conditions checked match/suffix/target/nil/copy equations for arbitrary full receivers. Actual original Box/Nil tag witnesses instantiate them; all prior source/case/selector/payload identities remain. Both cores agree on100 distinct complete streams:93 zero-axiom mathematical acceptances and7 genuine type rejections;92 normal rows include all70 large cases plus nil. Independent comparison checks92 receiver predicate definitions/11157 complete receiver statements, all original selector/source statements,327808 actual tag bits and368 actual decoded original/copied target/nil observations. All92 old selector streams replay with literal full-byte equality; scoped Clippy/format pass. Coherent changed named ordinal and different accepted callsite bytes fail exact import/frozen owner despite valid math. Large maxima{stats["maximum_declarations"]} declarations/{stats["maximum_terms"]} terms/{stats["maximum_bytes"]} bytes fit unchanged native caps; no reduced U or case/root/helper/parameter cap. All{epochs} source epochs/{inputs} input rows/{go_inputs} current Go prelaunch originals retained with no observation-timeout restart. These are ordinary conditional equations, not source receiver-once/nil-safety/callee/precondition/result/effect/frame validity, canonical capture/context or full56 foundation/private registry/nativeLinux/all169/T02-T11 completion.'
checkpoint={'evidence':'verification/source-error-receiver/verification.json','executor':'crates/mpk-vc/src/go_practical_ordinary/source_error_receiver.rs','scope':scope,'not_application_verification_closure':True,'public_profile_activated':False,'R01_complete':False,'R06_complete':False}
r01['latest_ordinary_checkpoint']=checkpoint;r01['preparatory_source_error_receiver_checkpoint']=dict(checkpoint)
if r07['latest_scoped_checks']['evidence']!=checkpoint['evidence']:r07['prior_scoped_checkpoints'].insert(0,r07['latest_scoped_checks'])
r07['latest_scoped_checks']={'evidence':checkpoint['evidence'],'language_scope':['Go','Rust'],'paired_current_byte_streams':100,'zero_axiom_positives':93,'genuine_core_type_mismatches':7,'normal_receiver_rows':92,'large_concrete_receiver_rows':70,'large_nil_receiver_rows':1,'receiver_predicate_definitions':92,'complete_receiver_theorem_statements':11157,'selector_definitions':10336,'complete_selector_theorem_statements':15274,'actual_case_tag_bit_observations':321920,'actual_nil_tag_bit_observations':5888,'actual_decoded_original_copied_target_nil_observations':368,'literal_current_old_selector_streams':92,'source_epochs':epochs,'original_input_rows':inputs,'Go_complete_prelaunch_inputs':go_inputs,'coherent_wrong_ordinal_and_wrong_accepted_callsite_rejected':True,'original_native_handles_preserved_without_observation_restart':True,'strict_clippy_and_targeted_format_passed':True,'all_actual_validation_handles_terminal':True,'unrelated_prior_suites_repeated':False,'full_repository_gate_passed':False,'full_T02_gate':v['full_T02_gate']}
p.write_text(json.dumps(d,indent=2)+'\n')
p=Path('develop/docs/go-practical-implementation-traceability-ledger.md');s=p.read_text()
section=f'''
## 166. T02-W01 source-error receiver tag premises

The [receiver checkpoint](../migrations/go-practical/t02-w01/verification/source-error-receiver/README.md)
adds ordinary Tag(e)=source-tag (or nil-tag) premises for arbitrary complete
same-U receiver values. Checked match/suffix/selection/nil/copy equations use
that exact premise. Original Box/Nil witnesses establish it and instantiate
the constructed-value equations. Every source/case/selector/payload proof
remains; no caller truth predicate, source-validity assumption, reduced U or
new admission cap is introduced.

Both full cores agree on100 whole streams:93 zero-axiom mathematical
acceptances and7 genuine type rejections. Normal92 include21 small-site
rows and all70 large concrete types plus nil. Independent comparison covers
92 complete receiver predicate definitions/11,157 receiver statements, all
old selector/source/case/operation statements,327,808 tag bits and368 actual
decoded original/copied target/nil observations. All92 prior selector rows
replay literally; scoped Clippy/format pass. Coherent wrong ordinal math and
different accepted callsite rows fail exact import/frozen ownership. Large
maxima{stats['maximum_declarations']:,} declarations/{stats['maximum_terms']:,} terms/
{stats['maximum_bytes']:,} bytes fit unchanged native caps. All{epochs} source
epochs/{inputs:,} input rows/{go_inputs} current Go prelaunch originals remain,
without observation-timeout restarts.

Source receiver-once/nil-safety/callee/precondition/results/effects/frame,
input validity, canonical capture/module/U/tag/target/context binding, full56
foundation/private registry/nativeLinux/all169/T02-T11 remain due. Earlier
historical-scope scanner findings remain for the final strict gate. Go/Rust
only; R01/R06/W01 stay open, the public practical profile stays inactive, and
the full T02 gate stays final W04: `./scripts/check-fast.sh`.
'''
pattern=r'(?ms)^## 166\..*?(?=^## [0-9]+\.|\Z)'
if re.search(pattern,s):s=re.sub(pattern,lambda _:section.lstrip('\n'),s)
else:s+=section
p.write_text(s)
print(json.dumps({'records_updated':True,'source_epochs':epochs,'input_rows':inputs,'Go_prelaunch_inputs':go_inputs}))
