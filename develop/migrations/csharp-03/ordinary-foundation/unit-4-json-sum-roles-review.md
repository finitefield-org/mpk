# W09 unit 4 tagged JSON roles: source and runtime review

The new fixture captures the actual original C# source, boundary/method
contracts and six semantic bindings. It reaches all five frozen sum templates:
Option<int>, Lookup<int>, Result<int,bool>, Validation<int,int>,
BoundaryField<int>, and Result<Option<int>,bool>. The last instance requires
recursive sum parsing; the two Result instances have genuinely distinct child
types on their arms. The source also reaches a 4096-capacity integer sequence,
two enums and seven source products.

The first draft used Option<Option<int>>. The unchanged original-input path
rejected it with CSHARP_PRACTICAL_TYPE/nested_option, consistently with the
closed-type validator. Both rejected request and response are retained under
verification-logs/json-sums. The accepted fixture instead uses the already
permitted Result<Option<int>,bool>; no frontend or closed-type rule changed.
The initial request-test compilation used a private VIR getter; the test now
inspects the public emitted closure. A subsequently started source test had no
accepted fixture and failed; it is not an acceptance receipt.

The source test checks all six layouts against the original carrier program,
exact closed-template linkage, the Validation arm's 256-element role bound,
reconstruction/import, four sum metadata mutations, and complete dependency
closures of primitive parsers and standalone syntax. The new candidate remains
ordinary Certificate v0: 145424 terms,2295 declarations,7882 counted static
transformers. The standard same-byte Go/Rust checker harness has a dedicated
one-vector test. Existing certificates and checker rules were not modified by
this coverage extension.

Twenty small cases observe complete C8 packets for Lookup, Result and
BoundaryField. Six more observe complete C8 packets for nested Result/Option,
including depth30/31 child limits. All 26 complete packets passed in184.41s. The three BoundaryField arms remain distinct;
JSON null is neither a missing field nor a value payload. Result boolean payloads
cannot be parsed as integers or vice versa. Strict payload presence and tag
selection are exercised on failure as well as success.

Validation runs on original complete JSON documents with 0,1,256,257 errors.
It checks all header bits, all bits of every active integer error, the sequence
length and tag, neighboring/high padding and selected inactive capacity slots.
This is explicitly selected observation of a wide C20 packet, not exhaustive
observation of every inactive cell. Additional cases exercise the valid arm,
wrong payload type, missing payload and nested depth boundary. The generic
sequence capacity run remains separate; its binary/candidate were not restarted.

Option<string> uses the previously checked original document candidate. Four
new runtime cases cover absent payload, empty string, A plus a supplementary
Unicode character, and invalid null payload. Probes include full UTF-16 units,
length/tag/header and inactive endpoints. They passed with 2390-2630 selected
bits per wide packet; this is not an exhaustive C21 storage check.

Terminal results and still-live jobs are recorded separately in the progress
receipt. Do not infer completion from the source test or from individual cases
printed by an ongoing runtime test. Map/set/Transition JSON, complete boundary
relations, and W09 units3-8 remain outstanding. There is no component-only
commit/push or W09 completion. The T-wide gate remains at T06-W12.
