# csharp2vir candidate project

This directory contains the inactive C# frontend project established by
`CSHARP-02-T02`. It intentionally supports only `--version`; source capture,
Roslyn compilation sessions, lowering, frontend envelopes, bundle registration,
and production routing belong to later serial tasks.

The project is not built with an ambient `dotnet build` or package restore.
Use the repository entrypoint, which validates and extracts the exact frozen
archives, enters a network namespace, invokes the pinned SDK compiler directly,
and compares two independently clean builds:

```sh
./scripts/build-csharp-frontend.sh --check
```

The command is offline and check-only. On a new machine, an explicit,
separate provisioning step may populate the ignored raw-archive cache:

```sh
./scripts/build-csharp-frontend.sh --provision-build-inputs
```

Provisioning validates every byte count and digest before installation. The
offline check never downloads, restores, registers, or publishes anything.
