# C# frontend regression seeds

These files are bounded, checked-in regression inputs for the inactive C#
frontend hardening gate. They are test inputs only: they are not project
sources, build inputs, release bundle contents, plugins, or proof evidence.

`seed-manifest.json` binds every seed by relative path, byte length, and raw
SHA-256. The T19 gate copies the closed inventory into private temporary
storage before running deterministic mutations against the source parser,
contract parser, frontend protocol boundary, compiler diagnostic normalizer,
and resource-limit implementation. Verification never provisions a fuzzer or
uses the network.
