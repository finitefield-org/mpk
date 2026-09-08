# W03 local task review

Scope: CSHARP-03-T05-W03; complete output implementation, shared encoding helper,
cumulative source linkage, persisted replay, tests and retained evidence.
Review is local, without delegation or hosted CI.

## Findings addressed

1. The earlier T02 JSON receipt compared JSON trees without validating the
   original returned source type. The sealed W03 output run validates the full
   typed value, independently decodes encoded bytes and checks every observable
   field before retaining any output artifacts.
2. Semantic projection alone could discard source storage. Output uses the
   original source type and all stored members; binding projection and source
   reconstruction remain ordinary VIR operations with pending obligations.
3. Reparse equality must follow frozen value semantics. Arrays/sequence
   snapshots share their existing representation and decimals their existing
   scale-independent value equality. All other storage and float bits remain
   exact. Configured formatting that changes a decimal value rejects.
4. The output manifest must extend the exact byte-validated input run. W03
   revalidates that run against current source/context and links both captures.
   Exact-byte/full-artifact replay rejects repaired hashes, stale manifests and
   input provenance splices.
5. Output limits must not accidentally be tested through oversized input.
   Actual-source product tests use small input, then independently construct
   returned values at the cell and document byte boundaries. Source-map
   assertions require a present reference instead of comparing absent keys.
6. Initial compiler specimens contained unreachable constructors. The retained
   specimens call those constructors and include their complete source closure;
   all three retained captures pass the unchanged pinned source gate.

## Final review

No open findings. The task does not implement transition semantics, public
activation, an application runtime or proof discharge. Error returns contain
closed codes and no output run. The frozen CLI protocol file and historical
probe sources remain unchanged. `verification.json` records final verification
and reviewed file hashes.
