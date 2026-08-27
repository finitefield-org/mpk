# csharp2vir candidate project

This directory contains the inactive C# frontend project established by
`CSHARP-02-T02`. `CSHARP-02-T05` adds the frozen private `lower` argument
grammar, selection hashing, closed path preflight, immutable capture, and
strict source transport. `CSHARP-02-T06` adds the exact pinned Roslyn source
and compilation sessions, sealed reference-projection validation, phased
diagnostics, and public semantic API adapters. `CSHARP-02-T07` adds exact
declaration/type/literal/operation admission,
deterministic pure acyclic source-call closure, inert-initialization and
definite-assignment proofs, and checked syntax/operation/CFG accounting. A
clean closure still stops before contract parsing and emits no artifact;
contracts, lowering, bundle registration, and production routing belong to
later serial tasks.
`--version` remains the only successful command until those tasks are
complete.

The project is not built with an ambient `dotnet build` or package restore.
Use the repository entrypoint, which validates and extracts the exact frozen
archives, enters a network namespace, invokes the pinned SDK compiler directly,
and compares two independently clean builds:

```sh
./scripts/build-csharp-frontend.sh --check
```

The full check runs all three private executable harnesses. The T06 session and
T07 subset harnesses can also be run alone with `--test-roslyn` and
`--test-subset` against the provisioned offline closure.

The command is offline and check-only. On a new machine, an explicit,
separate provisioning step may populate the ignored raw-archive cache:

```sh
./scripts/build-csharp-frontend.sh --provision-build-inputs
```

Provisioning validates every byte count and digest before installation. The
offline check never downloads, restores, registers, or publishes anything.
