# CSHARP-03-T06-W01: verification expression import

The verification importer consumes the complete frozen 33-tag contract union.
It parses canonical bytes independently, rejects duplicate/unknown/out-of-order
fields and unknown tags, then shares the existing concrete typing and literal
semantics. It never invokes the frontend expression parser, Roslyn, a source
method evaluator or a proof checker. Original source-side scope/attachment
owners remain responsible for reconstructing subjects, operations and bindings.

The resulting capability contains one typed ordinary var/const/app/lam/let
encoding. Let and bounded quantifier names become de Bruijn indices; `old`
selects entry-state subjects, while results exist only in the current state.
Renaming a local binder preserves its term, but the original expression hash
still retains the original bytes. Definition recipes bind exact stored members,
source constructors, specialized foundation operation IDs, semantic bindings,
codec parameters and modes, sum arms and lossless literal payloads. These are
references for later ordinary definition generation, not uninterpreted axioms
or discharged proofs. Checked arithmetic retains its ordered definedness checks
for W03. Calling an arbitrary source method or a captured partial constructor
from a contract is not admitted.

The attachment digest binds the original owner contract hash, subject scope,
old/result permissions, exception universe, operation and constructor signatures,
property and semantic binding identities, partial-callable set, and recomputed
closed roots/instances. Encoding import derives the complete expected record
again and compares exact bytes. Supplied terms, type annotations, definition
names and attachment hashes cannot substitute for that recomputation.

Original-input VIR import collects expressions through existing type/method,
loop/exception and transition attachment. Control method expressions are
recorded once by their existing loop/exception scope owner. Boundary contracts
supply input/output linkage and do not add another expression grammar. VC
capabilities retain the encodings; canonical VC foundation subjects bind their
identities and existing resource reservations include their ordinary nodes,
definitions and binder depth. Boundary and transition contract subjects route
to W07 and W08 respectively. The VIR/VC wire schemas and Certificate v0 format
are unchanged. New obligation generation and definition bodies remain with
T06-W02 through W09.

Structural limits precede expression retention: 1 MiB per expression transport,
32 expression levels, 1,024 nodes per method expression, four nested bounded
quantifiers, and 256 ordinary binders. The existing cumulative-root walk also
enforces 8,192 original contract nodes across the closure, without recounting
literal payload data as syntax or introducing a second specialization walk.
The capture and VC reservation enforce the existing ordinary-term limits.

## Evidence

- `encodings.json` retains all 33 typed tag cases, including every result type,
  ordinary term, definition recipe, canonical original bytes and attachment hash.
- `type-requests.json` and `type-responses.json` retain an actual C# type with
  construction and public invariants. The fixed local Linux/amd64 harness captures
  accepted input twice and compares bytes. Its source producer is unchanged.
- `attachments.json` retains the actual type-invariant fixture, a boundary with
  no expression clauses, the T05 combined boundary/transition fixture and the
  T03 contract-derived codec-root fixture through independent VIR and VC import.
- The target suite checks every tag/result type, specialized operation identity,
  partial and scope rejection, alpha normalization, exact encoding mutations,
  128 bounded invalid-UTF-8 parser seeds and inclusive depth/node/binder/quantifier
  boundaries. VC resource tampering reaches the resource-validation barrier.
- `verification.json` records scoped local tests, lint/format results and hashes.

`check-fast.sh` is deferred to the final W of T06 under AGENTS.md. The earlier
T05-W06 user waiver does not replace the T06 completion gate. No workspace-wide
substitute or GitHub workflow is run for this intermediate work item.
