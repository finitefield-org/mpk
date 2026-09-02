# AI Explanation Boundary

Status: the predecessor Vertex AI transport is retired from the public CLI;
the active successor exposes a sanitized request projection only

## Decision

`mpk explain` performs the same local, strict Go/Rust/C#/Java verification as
`mpk policy verify` and emits a deterministic `mpk.ai.explain.request.v2`
document. MPK does not choose or contact a model provider and does not accept a
provider response through the public CLI.

```sh
mpk explain <source-root> \
  --semantic-context <context.json> \
  --selection <selection.json> \
  [--contract <normalized-path> ...] \
  [--language <en|ja>] \
  --request-json-out <sanitized-request.json>
```

The revision-3 semantic profile supplies the AI projection contract. The
request contains a bounded summary of validated policy evidence and omits
source, contract bodies, certificate bytes, local paths, commands,
credentials, and original property identifiers.

## Trust classification

The request and all text generated from it are
`untrusted_helper_analysis`. They cannot:

- create or modify a proof property;
- change a checker verdict or axiom report;
- replace a certificate or checked theory certificate;
- alter the original `mpk.policy.evidence.v2` bytes;
- become an input to source-free proof acceptance.

Verification status must always be read from the validated local evidence and
its `trusted_evidence` references.

## External provider integration

An application may transmit the sanitized document after it leaves MPK, but
that application owns every provider concern: customer consent, project and
region selection, IAM, credentials, quotas, cost controls, retention,
abuse-monitoring terms, audit logs, response validation, and deletion.

Do not add provider, endpoint, model, API-key, token, credential-file, or
network options back to `mpk explain`. A provider-specific integration is an
external helper system, not a compatibility mode of the MPK CLI.

## Security checks

- Store local requests under an ignored, access-controlled directory.
- Never commit real customer requests or provider responses.
- Validate the v2 schema and size limits before external transmission.
- Preserve the local MPK status and opaque property references; never let
  generated prose overwrite them.
- Label all generated content as untrusted helper analysis.
- Review provider privacy and retention terms at the time of each deployment.
