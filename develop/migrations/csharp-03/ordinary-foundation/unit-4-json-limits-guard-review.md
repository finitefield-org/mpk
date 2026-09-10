# W09 raw and typed JSON guard integration — review in progress

The new opt-in envelope mode retains the complete typed-depth and typed-node
parser,then gates its entire output packet on the original document's raw
node/depth/length predicate. An invalid document returns zero header and
arguments. Existing parser failure remains zero. The raw scanner is conditional
on canonical JSON syntax;the retained parser separately enforces its grammar.
Source/public domains,reconstruction and application proofs remain separate.

The raw scanner accepts an already-emitted document definition. Its shared
cube helpers are emitted only when absent. This prevents duplicate declarations
when the parser and scanner share one builder. The scanner's transformer metric
counts the owning builder's cumulative occurrences;all other scanner metadata
and every old definition closure must match the standalone definitions. The
original three envelope modes retain absent raw metadata via serde omission.

The new mask helper executes the actual scanner on a valid empty object and a
33-deep tree against both successful and failed synthetic predecessor packets.
All1024 packet-bit observations pass9.20s. This tests raw guard composition;
it does not establish source parser semantics by itself. Source runtime checks
for required/empty/default cases and all16 compound definition closures are
running. Combined admission costs are unproven until those checks pass.

Review corrections so far concern test integration:use the existing shared
raw emitter,pass validated VIR to standalone generation,and encode document
length at the actual padded carrier bit positions. No checker rule changed.
Affected library/integration lint passes. Import mutation coverage,predecessor
metadata preservation,combined same-byte checking and the remaining W09
source/proof/assembly obligations are pending. No clean unit review or W09
completion is claimed. T-wide check-fast remains deferred toT06-W12.

Exact integrated import and10 metadata/certificate mutations pass85.06s.
Original/depth pinned metadata and bytes pass24.02s. The import checks include
missing raw metadata and replacing the certificate with its typed predecessor;
neither can bypass reconstruction. Integration-target lint including the new
import test passes11.34s. The new mode defaults to four representative source
runtime cases;its existing modes keep their previous16-source scopes. All16
compound closures remain required. The largest of the12 completed compound
cases has241171terms and13037transformers,within unchanged practical limits;
final full compound results and same-byte checking remain pending.

All16 compound source contexts now pass334.29s. Each preserves every prior
parser/count and standalone raw-scanner definition closure,while passing the
unchanged term/declaration/binder/transformer limits. Maximum measured terms
are241171 and maximum transformers13037. Final compound certificates and the
packet-mask helper are retained with module hashes. Source runtime and combined
same-byte checking are still pending;these retained definitions remain partial
W09 evidence.

All four selected original source contexts now pass387.85s:8 accepted and15 rejected complete packets. All20 source candidates plus the raw mask helper are retained,hash-verified,and undergoing same-byte checking. The checker run encountered an unrelated source-clause compile interruption on early cases;those must be retried once the run is terminal,while preserving exact-byte PASS cases.

Follow-up:the original21-candidate checker run is terminal after15342.545s.
20 cases passed and current names/hashes/sizes/metadata reconcile. Only
empty-input.hex failed a now-corrected Rust compilation;this was not a proof
rejection. That single case is being retried (session17264),without repeating
20 successful candidates. Combined checker completion remains pending.

Final reconciliation:the sole empty-input retry passed in474.863s. Both
unchanged checkers agree on the same bytes with zero axioms,and hash corruption
rejects. The union of this result with20 retained successes covers all21
current candidates;their exact hashes and sizes reconcile. No passing case was
repeated. There are no outstanding findings in this component;original internal
unit4 and W09 acceptance remain open.
