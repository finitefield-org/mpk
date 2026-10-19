# Additional request echo representations

The original transition and rich-literal source owners fail only at their final `requests.json` byte echo. Their exact observed right operands are retained separately in [transition](transition/requests.json) and [literal](literal/requests.json). The original fixtures use ASCII Unicode escapes; the Rust serialization uses literal UTF8. Complete JSON values, including every nested sidecar UTF8 string, are identical.

The [manifest](manifest.json) pins both original and serialized representations and the exact original assertion audits. Original input bytes, source/capture values, all runtime cases, packet/selector observation loops, strict imports and component goldens remain unchanged. Each normal owner keeps its complete byte comparison after three explicit guards pin both representations and compare full JSON values. Selected repaired validation and the entire repaired T01 gate remain required.

[Selected verification](../../verification/retained-csharp-repair/input-echo-followup-repair/verification.json) records both complete repaired owners passing: transition2 contexts/999 observations and literal18 constants/1,414 observations. Build, format and target Clippy pass. The separate control-step probe and entire repaired T01 gate remain outstanding.
