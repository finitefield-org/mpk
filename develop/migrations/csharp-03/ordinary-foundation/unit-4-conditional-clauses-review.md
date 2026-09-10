# Conditional clause component: direct review

Checked the frozen expression importer and retained encoding: arguments are
condition, when_true, when_false even though the descriptive field-type map is
printed in sorted key order. The new ordinary body respects that order and
checks each nominal type rather than accepting equal-depth carriers as the same
type. It applies identical selector variables to both branches and performs Bool
elimination only at the result leaves. Existing recipes retain their bodies.

The helper corpus checks branch direction, low/high selected leaves and nominal
mismatch rejection. A fresh original source/contract context exercises both
branches at signed boundaries. The previous context cannot be reused across the
changed sidecar; the importer rejected it and the exact request was recaptured.
The failed helper test import was corrected by qualifying the existing validator;
no checker or acceptance rule changed.

All three current candidates, including structural/public integration, passed
unchanged Rust/Go checking with zero axioms and
actual hash-corruption rejection. Existing seven-source clause byte replay and
thirty boundary observations also passed. Partial operations remain rejected;
the new conditional support does not infer definedness of either branch or
claim native/application proofs. No further component finding was identified;
remaining W09 requirements are unchanged.

The integrated conditional source retains the exact standalone clause dependency
closure and its previously checked bytes. Only the new integrated candidate was
added to checker execution; all three retained/current PASS names and hashes were
reconciled. The integrated-source import/runtime check and affected lint passed.
