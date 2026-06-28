# Core inductive positivity fixtures

These fixtures enumerate MVP inductive positivity verdicts that must remain
deterministic. The `mpk-core` unit tests load every `*.fixture` file in this
directory and require each listed case to match its expected verdict.

The current IND-002 fixture set covers:

- accepted documented MVP shapes: Bool, Nat, and Eq;
- rejection of undocumented constructor shape patterns;
- rejection of negative recursive occurrences;
- rejection of recursive occurrences under unknown functors;
- rejection of constructor results that do not return the inductive family.
