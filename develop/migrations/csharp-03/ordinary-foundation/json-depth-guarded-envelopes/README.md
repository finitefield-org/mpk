# Depth-guarded input envelope candidates

Sixteen original source contexts passed30complete original document packets and
32rejected document mutations. The packet-mask helper passed1024complete bits,
including a depth failure in a later field and original parse failure. All17
module hashes were recomputed before publication; per-source metadata and rows
agree, and the attachment source IDs match the original accepted corpus.

The source maxima are145,171terms,2,213declarations and7,362static transformers.
The original parser definitions remain unchanged; the new parse entry applies
canonical typed-depth checks to every decoded/defaulted field at root depth1.
The unchanged Rust/Go checkers are checking these17identical byte sequences,
matching reports, zero axioms and rejection of corrupted hashes.

See `../unit-4-json-depth-guard-progress.json` for actual evidence. Additional
compound/deep source coverage, raw/canonical node bounds, source conditions,
reconstruction, output and remaining W09 proof/assembly work are still required.
