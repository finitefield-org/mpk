# Floating contract expression connections (verification pending)

Five native-source fixtures cover all 38 single/double floating operations and
six numeric conversions. Each method contains its relevant native operations,
so contract recipes resolve against the actual operation table. The double contexts separate comparisons, arithmetic and division to preserve
existing certificate limits. The original oversized double input is retained
in limit-rejections/ and its ordinary generation is required to reject Limit.

The adapter reuses existing ordinary floating definitions, validating nominal
arguments/results, ordered checks and failure-definition counts. IEEE arithmetic
has no exception check; checked float-to-integer conversion has one overflow
condition. Every alias and complete definition dependency is compared with the
standalone ordinary program. Runtime selects 18 connection/definedness cases:
NaN, infinity, signed-zero comparison/negation, valid conversions and two
checked overflow rejections. Unchanged standalone arithmetic matrices are not
repeated. Metadata import and corruption checks remain part of each context.

Capture, all source/alias tests, the oversized rejection test and five candidate
pins are complete. Same-byte checking remains pending. See ../unit-4-floating-clauses-progress.json. This component
neither supplies native-body application proofs nor completes original unit 4
or W09. check-fast.sh remains deferred to T06-W12.
