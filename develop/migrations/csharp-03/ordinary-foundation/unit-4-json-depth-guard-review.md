# W09 typed-depth checks on decoded input envelopes — review in progress

The depth-guarded envelope generator retains the original object parser and
adds a wrapper over its completed header/argument packet. Every field is
projected in reconstructed contract order and checked with its semantic type's
canonical JSON depth predicate at root depth1. This includes frozen defaults
and exposed missing/null values because the original parser has already
materialized them. The original validity flag and every field check must hold;
otherwise all packet bits are zero. The completed parse is bound once in a
Let, so the checks refer to that exact result.

The original parse definition remains in the certificate and is named by the
new `unguarded_parse_definition` metadata. The new entry is `parse_definition`;
its program schema explicitly identifies the depth-guarded variant. Existing
unguarded metadata omits both the new optional reference and empty depth list.
All parser/depth generation uses one owning Builder, so declaration, term,
binder and static-transformer costs are measured on the actual combined program.
Imports reconstruct the exact variant from emitted source and compare all
metadata/certificate bytes. There is no runtime Boolean admission shortcut.

Direct review checked that a failure in any field, including later fields,
participates in the final conjunction; original parse failure cannot be revived;
each field starts at depth1 even if its raw input was scalar; and the mask covers
the full header, arguments and padding. The helper exercises semantic heights31
and32 with a failing second field and complete1024-bit observations. Combined
source tests cover all16 original contexts and compare every predecessor parser
definition/dependency. They retain full original and malformed document checks.
All16contexts passed30original and32invalid complete packets. The first4
contexts completed in214.58s;the remaining12 completed in567.89s. Their16
source candidates and the packet-mask helper have been independently hash-checked
and published for identical-byte dual checking. The packet helper initially failed to
compile because its128-bit header constant used a u32 word helper. That test
fixture now emits a full128-bit constant circuit. The corrected helper passed
all1024packet bits in0.44s; production guard definitions were not changed by
this correction.

The shared standalone depth generator has been refactored to emit into the
owning Builder. Its15 original metadata/byte pins and167 core observations
passed in3.06s. Affected library/integration lint also passed. The unguarded
required-source metadata and exact certificate-byte regression passed in3.81s
without repeating its parser execution. Same-byte checking on combined source
candidates remains required. No unrelated scalar/parser suite or whole T gate
is rerun here.

This closes the implementation gap between semantic depth predicates and the
decoded arguments, subject to the pending verification. It does not establish
raw/canonical node counts, representation/source domains, reconstruction or
full AcceptInput. Additional compound/map/set/Money/transition and deep-source
coverage and the remaining W09 native/control/replay/proof/assembly acceptance
are still required. Unit4 and W09 remain incomplete.
