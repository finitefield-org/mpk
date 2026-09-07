# CSHARP-03-T05-W01 boundary attachment

The private data/control emitter now attaches captured `mpk.csharp.boundary.v1`
contracts to the selected original application method. It checks the frozen
root and nested field records, profiles, hashes, semantic context, compilation,
source identity, exact logical signature and a captured total method contract.
The ordinary VIR importer repeats attachment from the captured sidecars. The
VIR, source manifest and source artifact set retain the boundary contract link.

Input fields correspond to logical arguments in order (including the receiver
first for an instance method). A non-void result has one complete typed output
field; void has none. Products remain complete field-ordered values inside that
field. Field IDs and JSON names are unique within each direction. Wire names
are UTF-16 sequences, including lossless lone surrogates. No source parameter,
constructor, stored member or invariant can be invented by a boundary sidecar.
The profile fixes 256 fields per direction, 32 value edges, 1 MiB documents and
65,536 total value cells; the frozen root has no caller-controlled limit fields.

`required`, `nullable` and the three frozen missing modes are checked together.
An optional field either exposes a bound presence's missing arm or has an exact
typed default. Missing is never implicitly converted to null. Defaults use the
shared typed-value decoder and scalar codecs, with boundary integer tokens
rather than contract-literal integer strings. Selected decimal scale/rounding
and other explicit codecs also apply to defaults. Unknown enum/tag values,
inactive payloads, malformed field records and oversized values reject.

Application presence uses the existing cumulative source binding closure,
W12 outcome model and ordinary VIR binding validator. This preserves projected
payloads such as an application Instant inside an application Presence; there
is no separate specialization or source projection engine. The exact binding
and its pending field, arm, invariant and commutation obligations remain linked.
The actual CLR zero is a default candidate only when its retained source tag
maps to missing. A zero mapping to null is not such a candidate. Candidate
status does not discharge the public invariant or change the frozen historical
`default_arm` marker. Frozen default invariant obligations also remain pending
for T06, bound to the exact field and contract bytes.

Only `codec_id: "unix_milliseconds"` on an exact raw Int64 field classifies that
field as an internal instant carrier. An unclassified Int64 remains an integer.
The application signature and ordinary VIR parameter type remain unchanged.

These are validated attachment plans, not executable input evidence. W02 owns
canonical input capture/decoding and W03 owns output encoding/reparsing. Public
profile activation, invocation and proof discharge remain with their later
owners. The typed `check_state` precondition helper cannot create input evidence
or invoke application code.

## Retained evidence

`requests.json` retains the actual source and sidecar bytes for 56 cases.
`responses.json` retains the pinned Linux compiler responses: 55 deterministic
source captures and one artifact-free source binding rejection. Native tests
accept 15 boundary attachments and reject the other 41 cases, including that
source rejection. Accepted source captures are obtained twice and must match.

The matrix covers required/optional/non-null and missing/null/value states,
typed defaults, presence projections, null versus missing CLR defaults, raw
instant classification, fixed decimal codecs, enum carriers, UTF-16 names,
field order, limits, identity/hash/profile mismatches and stale source bytes.
Additional tests remove or change the boundary link after rebuilding VIR,
reorder embedded contracts, remove captured source facts and move source facts
between sidecar snapshots. Those attempts reject independently of Roslyn.

Reproduce locally:

```sh
cargo test -p mpk-cli --test csharp_practical_boundary
./scripts/check-fast.sh
```

Regenerate requests with `MPK_CSHARP_BOUNDARY_REQUESTS_OUT=/tmp/requests.json`
while running the matrix test. Feed them to the existing pinned Linux action:

```sh
./scripts/build-csharp-practical-frontend.sh --test-control-emission-requests < requests.json
```

The input producer remains the exact W06 control capture implementation;
`conformance.json` binds its retained input manifest and the W01 capture files.
`verification.json` records the final local gate and reviewed file hashes.
