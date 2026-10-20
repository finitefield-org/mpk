# Go practical shared artifacts v1

Scope amendment: [LANGUAGE_SUPPORT_SCOPE_V1.md](LANGUAGE_SUPPORT_SCOPE_V1.md)
limits implementation and retained contexts to Go/Rust. Four future contexts
replace earlier six-context retention statements; all Go semantics, common
artifact members, proof obligations and resource caps remain required.


Status: frozen inactive Go-owned shared-family design. This document owns
`mpk.go.practical.shared_artifacts.conformance.v1` at
`develop/specs/vectors/go-practical-shared-artifacts-v1.json`. Primary test owner:
`crates/mpk-vc/tests/go_practical_artifacts.rs`; publication checks:
`scripts/check-go-practical-spec.sh`.

The conformance set's byte-pinned catalogue/schema members are normative for
every root, nested record, tagged union, mandatory field, canonical/hash preimage,
numeric budget and cross-artifact predicate. Its identity catalogue is the
closed unreserved Go-owned family. In particular common context/registry/source/
VIR/frontend/release use v3 identities, VC/skeleton use v4, and policy/API/AI use
their exact v4 identities. Go foundation/source identities remain v1. All four
required Go/Rust target contexts share one new installed family at T07:
Go fixed, practical Go, Rust32 and Rust64. Frozen v1/v2 codecs and retired
identity namespaces cannot be widened or relabeled.

Use narrowed RFC8785 JSON: UTF8 scalar strings, safe integral metadata, exact
Boolean/null distinction, no duplicate/unknown members, exact required union
arms and preserved array order. Source full-word values use their own decimal/
bit representations. `H(D,x)=SHA256(ASCII(D)||00||canonical(x))`; each declared
preimage includes every non-own-hash field, including schema/context/typed refs.
Raw canonical references hash the entire document without a domain. Record
transports have precisely the LF rules in their schema: API/frontend/release
use one LF; policy/AI and domain preimages exclude it. No old-schema fallback,
schema alias, ignored field, Boolean-as-integer or hash-only authority is allowed.

The final source-kind enum in
`develop/migrations/go-practical/t01-w05/source-kind/catalog.json` uses `go_file`
for Go source. The preceding private candidate graph is a sealed historical
fixture, not an allowed input dialect. New producers/importers regenerate all
dependent source/context/IR/VC hashes; there is no old-tag alias or relabeling.

Independent import regenerates source capture/type/scope/nominal identities,
closed operations/contracts, region/control/effect linkage and every request.
The ordinary assembly contains full canonical context certificate bytes and
concrete source/IR/contract/representation definitions. Required theorem rows
bind request hash, declaration name, exact type term index and existing
declaration interface hash. The whole assembly hash and VC bind the actual term
tables; the old declaration hash alone does not identify a proposition.
Candidate proof terms/theorems append to an unchanged context prefix.

The acyclic proof graph is captured source/VIR/contract -> obligation program
-> assembly -> semantic binding -> materialized VC -> skeleton -> certificate
manifest -> paired checked certificate -> policy/API/AI. Unavailable elaboration
retains the exact unavailable arm and full unelaborated request IDs; it cannot
produce a skeleton. Materialized types with pending proofs are a separate state.
Every property covers the exact source/assembly/VC/statement/dependency. Go uses
zero axioms, while retained contexts dispatch their own unchanged compiled policy.
API state commits only after complete validation/encoding and binds server-owned
incarnation/revision. AI uses complete redacted batches and global aliases;
provider text cannot supply status, proof, target or source authority.

The release DAG is semantic inputs -> actual built frontends/toolchains ->
bundle registry -> built consumer -> outer image registry -> candidate -> native
gate evidence -> receipt -> atomic whole-image install/rollback. Consumers do
not embed their own outer image hash; frontends do not embed a registry hash
containing their executable hash. Generic inventory.v1 keeps exact path/raw hash/
size fields; separately bound modes remain mandatory. New bundle IDs rebind
component inventory owner scopes and all dependent hashes/library refs. Old
metadata projections and linkage-fixture hashes do not establish runtime support.

The original183-consumer catalogue remains a historical coverage index. The
scope amendment removes retired-language owners; every remaining Go/Rust
consumer and common member must preserve source/contract/operation/axiom
meaning. T02 implements additive private producers/importers and real
source/ordinary bindings. T07 performs complete native local gates, all four
required target contexts, preservation of every Go/Rust policy, two installed
replays and compare-and-swap activation/rollback. Metadata cannot grant readiness.
Historical helpers/observations stay at their original Git epoch; current
production requires fresh inputs and execution. No partial mix of families is
installed and no native resource/isolation result is inferred from emulation.
