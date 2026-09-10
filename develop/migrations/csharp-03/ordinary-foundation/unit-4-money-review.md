# W09 unit 4 Money component review (partial unit verification)

This review covers the new ordinary Money generator, its exports, decimal-emitter
visibility, original string/enum source captures, generated pins and targeted
observers. It is not the final review of internal unit 4 or W09. Unit 3 still
has deep semantic tests and its final review open; original units 4-8 retain
all their deliverables.

## Findings corrected

1. The round selector attempted to call a word-equality helper absent from the
   ordinary fold vocabulary. Complete unsigned Less comparisons in both
   directions now implement the selector without an added trusted primitive.
2. The round selector re-emitted already-defined D9 helpers. The common builder
   correctly rejected duplicate globals. The emitter now reuses those helpers.
   Original-source generation failed before correction and passed afterward;
   two sources now produce three concrete instances and all 30 operations.
3. The semantic observer could compare `None` from an unnamed oracle error with
   the absence of a generated failure. It now requires a named semantic error.
   A separate audit runs the exact same 78+22 case functions with an oracle-only
   observer and confirms every expected failure has a name. Both matrices pass.
   This audit is supplemental test-input evidence, not generated-term execution.

## Reviewed contracts

All ten operation signatures, equation strings, return types and ordered error
labels are reconstructed from the validated foundation instance, including
uninvoked operations. Storage uses the frozen amount/currency field order.
Create retains the exact original decimal representation and requires an explicit
ordinary currency-predicate parameter; source-body justification remains a later
binding/assembly obligation. Add/subtract reject currency mismatch before
arithmetic overflow. Multiply/divide validate scale and rounding before decimal
failures, perform checked arithmetic before explicit rounding, and retain the
original currency. Decimal comparison is by numeric value; structural order is
currency first. All definitions assume the applicable operand domains.

The shared source checks two distinct currency carriers and all 20 operations in
one program. Generation and exact pin replay passed for both sources. The old
Money<string> pin and complete metadata row remain byte-identical when extending
the corpus. The source-capture harness checks deterministic two-run output under
frozen Linux inputs, with no network and a read-only repository.

## Verification boundary

The binding-vc-money pin passed both unchanged checkers with zero axioms and
hash-corruption rejection. The shared string/enum pin also passed both checkers
with zero axioms and corruption rejection.
The shared string/enum semantic run passed all 22 observations in 3,950.06 seconds.
The original 78-case run subsequently passed in 18,164.08 seconds. These jobs began
before the named-error guard change; the separate current oracle audit verifies
the added guard on the identical inputs and expected results without replacing
those generated-term observations. Production generator bytes are unchanged.
Final lint, formatting, exact request-byte replay and the oracle-case audit pass;
the five inventory tests also pass. See `unit-4-money-progress.json` for exact
commands, version scopes, log hashes and pending handles.

Both completed ordinary executions, the exact current oracle audit and the
checker results have now been inspected. The production generator hash remains
identical to the recorded source/pins. No additional actionable issue was found
within this finite Money component; no complete unit-4 review is claimed.
Bindings, canonical codec/literal/JSON relations, native/control
and transition relations, proof assembly, final W09 acceptance and unit review
remain required. The full T06 gate stays at W12. Nothing from this component has
yet been committed or pushed.
