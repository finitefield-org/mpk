# Certificate v0 internal projection fixtures

These four JSON files exercise the source-neutral Certificate v0 assembler's
internal checked Bool/BV and declaration projection. They are not active
frontend, VC, policy, API, release, or installed-image inputs. Production
source artifacts use the sole successor schemas; the active successor policy
path validates VC v2 and skeleton v2 before passing only their checked
function/declaration projection to the unchanged Certificate v0 assembler.
