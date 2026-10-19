# Actual ordinary full-width word circuits

This W01 component compiles all8/16/32/64-bit word circuits as ordinary checked
definitions. It adds1000 definitions and144 universal basic laws to the retained
Bool/Nat/Int/range basis. The complete zero-axiom certificate has1892 declarations,
31,104 terms and433,285 bytes, with certificate hash
`c00e978e19b88963b0c756314b400411b1dee28f35830022eca09a3e50a62a7e`.
All748 preceding range declarations retain their exact complete component bytes
and hash. Every255 frozen W04 design input remains unchanged. No prototype
certificate, context, graph or receipt is promoted.

The [circuits](../../../../../../crates/mpk-vc/src/go_practical_ordinary/word.rs)
retain all actual bits in Bool-indexed functions with3/4/5/6 address arguments.
Addresses are most-significant first; integer bit0 is least significant. Exposed
Pi types remain at binders/results, while ordinary Carrier members retain the
complete shape. Core carrier equality does not authorize Go nominal assignments.
Source types, original constants and their representability/nominal bindings
remain separate requirements.

Definitions cover bitwise operations, wrapping addition/subtraction/negation/
multiplication, signed/unsigned comparisons, three shifts, division/remainder
and all signed-source/unsigned-source width conversions. Restoring division
includes the previous high remainder bit in its decision. Signed minimum/-1
wraps and remainder follows the dividend sign. A zero divisor has an explicit
model totalizer; DivideDefined must hold before claiming Go execution. Shifts
consume the entire unsigned64-bit count and saturate at the word width; signed
right shift uses sign fill. SignedCountDefined rejects negative sign-extended
counts. Go count-type conversion/source binding is still separate work.

The [proof constructor](../../../../../../crates/mpk-vc/src/go_practical_ordinary/word_proofs.rs)
generates ordinary Eq.congr/trans/symm computations for constructor majors.
Four local Boolean laws and every carry-zero lemma support pointwise AddZero,
XorZero, AndOnes and NotNot for arbitrary whole words and every address. Only
local Boolean flags/address selectors are eliminated, never all word operands.
No function extensionality, arithmetic opcode, axiom, new reduction or fuel
rule enters either checker. Named construction syntax now owns binder names
to support capture-avoiding substitution; unchanged range bytes verify the
retained producer behavior. Exact word import regenerates every complete byte.

The [actual certificate-DAG interpreter](../../../../../../crates/mpk-vc/tests/support/go_word_eval.rs)
executes decoded ordinary terms. Its only value primitives are Bool constructors/
recursors and externally assigned input-word bits. Arithmetic names supply no
model shortcut. All1640 unchanged frozen query inputs agree with an independent
finite integer oracle:1312 full word values,288 Booleans and40 modeled-domain
failures. No integer value is narrowed through a JSON number. The interpreter
executes592,995,280 term/application steps in total, with18,693,095 for the largest
query, within its private observation guard. This is finite comparison evidence,
not general numeric or source correctness.

The [fresh native observation](native/result.json) compiles the original independent
Go observer with Go1.25.0 in an empty build cache and an isolated exact prepared
GOROOT. All5577 original prepared file bytes/modes/namespace,163,457,770 bytes,
are checked before and after compilation/execution. Two native output streams
are byte identical. Every actual-DAG result/guard and integer-oracle result
agrees with [fresh native typed Go](native-comparison/result.json), including
signed extrema, high remainder/carry cases, full64-bit counts and conversions.
Native expected panics remain distinct from proof/configuration/resource errors.
The prepared image is no immutable application replay or Linux release claim.

The [same-byte current-checker matrix](paired-final/result.json) has20 final
candidates: three zero-axiom acceptances with identical certificate/export/axiom
hashes and17 core type rejections. Twelve false carry/add-identity/divisor-guard
claims, four Boolean-as-word shift counts and one typed wrong full-adder
definition reject. No decoding/hash/resource error is counted as a type failure.
The unchanged current Go binary and every checker source are rehashed against
their preceding build. Strict [Clippy](final/clippy/result.json) and
[complete changed-file format](reviewed/format/result.json) pass. No C# tests run;
unchanged checker/legacy unit matrices are retained without repetition.

The first [two-root smoke](initial/rust-tests/result.json) passes the complete
all-width component and retained range hash. The larger
[execution](reviewed/rust-tests/result.json) passes all1640 model/oracle cases,
complete component/import and typed-control roots, then stops one false-claim
root on a noncanonical hyphenated helper name before core checking. Its original
candidate/report/source/diagnostic remain. The corrected
[focused root](final/rust-claims/result.json) reruns only those twelve claims and
requires actual core type mismatches; the other four passing roots are retained.

Native preparation first compares the full original SDK against the smaller
complete prepared-image namespace. The second attempt incorrectly equates42
original executable source/support-script modes with their frozen non-executable
prepared modes. Both [namespace](initial/native-image-scope-failure/result.json)
and [mode](initial/native-mode-scope-failure/result.json) failures retain exact
scripts, original/reproduced raw diagnostics, partial inputs and workspaces.
No compilation occurred in those attempts. All original file bytes match.
The final preparer copies exactly the pinned files, requires the actual command/
tool binaries to be executable and applies the unchanged prepared permissions.
No compiler, stdlib or observer source bytes are rewritten and no inventory
member/namespace check is weakened.

The [verification record](verification.json) retains commands, exact inputs,
outcomes, scopes and times. The input review keeps every original failed byte
and three exact libtest final-blank-line observations, with unchanged strict
scanner/seal policy and no source/JSON/directory or blanket whitespace exemption.
The cumulative T02 gate stays deferred to W04: `check-fast.sh --skip-csharp`.

General unbounded Nat/balanced-Int interpretation/conservation, multiplication/
division/shift/source correctness, captured nominal source-operation binding,
bounded definedness, storage/lifetime/program laws and the complete56-family
foundation/descriptor/private registry/context remain due. W01 stays In progress;
all169 application theorem demands remain unelaborated. No public practical
profile, source-program proof or native Linux acceptance is claimed.
