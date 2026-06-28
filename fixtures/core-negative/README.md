# Core negative definitional-equality fixtures

These fixtures enumerate core conversions that must stay rejected by MPK v0
definitional equality. The `mpk-core` unit tests load every `*.fixture` file in
this directory and require each listed case to evaluate to `reject`.

The current DEFEQ-004 fixture set covers:

- eta conversion;
- proof irrelevance as conversion;
- theorem proof unfolding.
