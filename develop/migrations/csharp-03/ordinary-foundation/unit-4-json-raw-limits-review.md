# W09 raw canonical JSON limits — review in progress

The ordinary scanner reads bounded byte storage directly. Its finite state
retains byte cursor,node count,nesting depth,string/escape/primitive status,
a closed-string decision and sticky failure. Nodes include containers and
primitive values;quoted keys are excluded by inspecting the immediately
following colon required by canonical JSON. String contents and escaped quotes
cannot start nodes or change nesting. Closed value strings count on the next
byte,or during finalization at EOF. Container/primitive/string-value depth is
checked before entering or counting the value;empty containers at depth32 are
admitted,while their children at depth33 are rejected. Counts saturate at262145.
The final predicate also checks closed strings/containers and document length.
This is conditional on separate canonical syntax and UTF8 validity;the scanner
is not a grammar parser or AcceptInput proof.

The first64-byte block passes23 full64-bit state observations. Its whole scan
had109234terms/368declarations/8510transformers. To fit the existing compound
parser's budget,the final block is32 bytes,eight blocks per guarded Step256,
4096 outer steps:exactly1MiB coverage. Actual final scan generation reports
58023terms/223declarations/4275transformers;final runtime tests are still running.
All intermediate results retain sticky failure and guarded composition,with
ordinary state sealing borrowed from the existing UTF8 scan construction.
The32-byte tests retain key/escape state across block boundaries and check
inclusive count/depth cutoffs. High-address tests read the final physical byte
with lengths1048575/1048576 and test EOF pending-string count separately.

Source/import/mutation checks use the existing68 original document-source
contexts (three nonempty boundary programs). Complete document tests include
valid JSON strings/keys/arrays at depth and byte-block boundaries. A copied test
module was initially registered at the wrong parent;registration was corrected.
An early helper syntax error and two equivalent byte-slice lint warnings were
also corrected. Source/runtime evidence must be matched to final generated
bytes before publication;no previous-version result is silently reused.

No new literal Std namespace consumer is present in this file. An attempted
inventory-update precheck detected that fact before any mutation,so all current
fingerprints and total4947 remain unchanged. Full affected inventory verification
is running. Complete1MiB runtime,raw+typed integration,same-byte checking and
full remaining W09 source/proof/assembly scope remain open.

Final32-byte block tests now pass23 complete64-bit state observations. The
scan generation/high-address test failed only at the final helper lookup:
`Finished` was a circuit label,not its emitted definition ID. The metadata now
retains the exact internal finished-definition reference and the test uses it;
no new certificate definition or admission rule was added. Final high-address
and pending-string EOF checks are rerunning against that exact reference.

The earlier source suite passed68 contexts/3 nonempty programs in79.86s and
complete-document suite passed21 trees/2 overlength cases in97.25s. Retained
metadata proves they executed the prior64-byte scanner,so they are explicitly
historical;final32-byte runs have been launched. Full consumer closure passes
15.84s with unchanged fingerprints;affected lint passes and scoped formatting
passes. Current source/scan/dual-checker acceptance remains incomplete.

Final32-byte verification is now complete for the selected standalone scopes:
68 source contexts/three nonempty programs passed49.23s;21 validJSON trees and
two overlength cases passed60.25s;scan generation,final physical byte and EOF
pending-string cutoffs passed3.63s. Five candidates are retained under
`json-raw-limits/`,with each source certificate hash verified against its
metadata. Same-byte checker agreement is running. Complete1MiB sequential
runtime and remaining W09 scope are still unproven. Raw-and-typed integration
now has a separate progress/review record;these results are not an application
proof or a W09 completion receipt.

All5 retained raw-scanner candidates now pass both unchanged checkers,zero axioms and hash mutations in440.608s;current candidate hashes and exact PASS set reconciled.
