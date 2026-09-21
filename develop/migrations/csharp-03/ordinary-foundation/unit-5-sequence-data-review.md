# Published sequence SSA adapter review

The adapter validates each frozen foundation signature before reusing its
existing length, checked read, equality or eligible comparison body. It admits
only reachable bounded-sequence instance IDs and exact input/result/check
signatures. Native probes cover Length and read; uninvoked helper generation
retains the full sequence operation set. Other definitions remain explicit
pending entries, including private construction and ownership operations.

Result comparison checks the physical result carrier after binding computed
output once. Operand and result indices retain original W03 ordering. The read
failure wrapper forwards both sequence and full 32-bit index; negative indices
cannot alias a low-address cell. Only the frozen index-range exception is
registered, with original normal and exceptional SSA successor metadata.
The implication is vacuous on failure, rather than falsely certifying the
checked-read helper's unused zero result as successful.

Seven captured original C# sources cover direct Length, a called array reader,
nullable-element/product/string lengths, field reads and property reads.
Independent sparse storage avoids allocating the full array-of-strings cube.
Runtime probes use lengths 0/1/2/4095/4096, first/last cells, signed extrema,
negative indices, logical/physical limits and high index bits. All 411 result
and guard observations pass. Metadata/context/symbol/function/node/successor
and actual certificate hash mutations reject through independent regeneration.

All seven certificates pass same-byte Rust/Go verification with zero axioms
and corrupted-hash rejection. Twelve predecessor sequence certificates remain
byte-identical. Clippy, affected formatting and inventory pass. There is no
new namespace search fingerprint. No checker or evaluator acceptance rule
changed. Direct review found no remaining adapter-specific defect; construction,
public domains, ownership and native CFG proofs remain explicit work, not
claims established by these finite observations. Unit 5 and W09 remain open.
