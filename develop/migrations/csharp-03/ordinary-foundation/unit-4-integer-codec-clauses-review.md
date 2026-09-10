# Integer codec contract adapter review (W09 internal unit 4)

Status: integer codec contract component targeted verification passed; no W09 or
original internal-unit completion claim.

## Scope and implementation review

The adapter accepts the ten integer/Duration/Instant codec IDs. It reconstructs
the fixed scale/rounding record and validates it with BoundaryCodec; unknown
configuration, an incompatible nominal value type, or a noncanonical format mode
cannot select a helper. Parse requires String input and the exact closed
Result<Value, ParseError> metadata and output layout. Format requires String
output and registers the separate W03 definedness symbol.

The standalone integer emitters were extracted without changing loop order,
helper sharing, type/width checks, or operation bodies. The contract compiler
owns a per-program family cache and delegates to those emitters, so repeated
recipes share one family. Failed generation publishes no partial program.
The formatter condition reads the complete 32-bit generated length and compares
it with the inclusive 16384 bound. It introduces neither an assumed truth nor
an input-domain proof. Input domains and application VC proofs remain separate.

## Verification selection

Selected the 54 existing format and 54 parse certificate pins because both
standalone generators now use the extracted emitter. They passed byte-for-byte;
their earlier extensive standalone arithmetic tests are not rerun. New original
source contracts and direct recipe calls exercise the new connection, including
full parser result bits, formatter characters/padding, nested format/parse
round trips, and W03 definedness. A dedicated length predicate test covers
16384, 16385, high bits and u32::MAX, so a truncated or constant guard fails.
Compiler nominal/binder regression, inventory and lint/format cover the shared
compiler/cache additions. All ten same-byte dual-checker candidates passed, with zero axioms, agreeing
reports and actual hash corruption rejection (168.783 seconds).

## Corrections during verification

- Fixed the adapter's re-export visibility before successful library checking.
- Exposed existing test oracle helpers at their common test ancestor.
- Corrected the test artifact's codec_format field order.
- Replaced the test's invalid String sequence_length expression with a
  format/parse round trip; bounded_sequence operations do not accept String.
- Moved the new test module after production items to satisfy clippy.

These corrections preserve the production codec arithmetic and prior pins.
Remaining decimal, hex and calendar contract adapters are outside this component.
No application theorem, W09 completion or permission to advance W10 is inferred.
The full check-fast.sh gate remains deferred to T06-W12.

## Final review and evidence

Ten original-source contexts passed 76 contract conditions, 40 directly aliased
format cases and 140 directly aliased parse cases (915.96 seconds). All 20 codec
recipe aliases and complete transitive definition closures match the standalone
programs. Exact import and changed-result-metadata rejection passed. The prior
54 format and 54 parse pins remain byte-identical. Eight full-length bound cases,
nominal/binder regression, inventory, clippy and format also passed.

Final direct review found no remaining actionable issue in this component.
The ten checker PASS subtest names match the complete pinned corpus. Current
module hashes/sizes, metadata hashes, source-tested output copies and retained
log hashes were reconciled. Capture JSON pretty-printing preserved all original
facts; the raw frontend response transport is retained with its hash.

The working tree remains dirty on main at aaf39bf. Original unit 4 is incomplete,
so no component commit or push was made. The eight-unit plan and W09/W10 ledger
are unchanged.
