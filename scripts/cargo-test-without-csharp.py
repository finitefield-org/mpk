#!/usr/bin/env python3
"""Run the development workspace gate with dedicated C# tests deferred.

Cargo has no exclusion selector for integration-test targets. Select every
other target explicitly, and filter C# unit/doc test names. Shared tests stay
selected. This does not certify the deferred C# release.
"""

import argparse
import json
import shlex
import subprocess


def test_plan(metadata):
    commands = []
    omitted = []
    members = set(metadata["workspace_members"])
    for package in metadata["packages"]:
        if package["id"] not in members:
            continue
        selected = []
        docs = False
        for target in package["targets"]:
            kind = target["kind"]
            if "custom-build" in kind:
                continue
            if "csharp" in target["name"].lower():
                omitted.append({"package": package["name"], "target": target["name"]})
                continue
            docs |= target["doctest"]
            if not target["test"]:
                continue
            if "lib" in kind:
                selected.append("--lib")
            elif kind == ["bin"]:
                selected += ["--bin", target["name"]]
            elif kind == ["test"]:
                selected += ["--test", target["name"]]
            elif kind == ["example"]:
                selected += ["--example", target["name"]]
            elif kind == ["bench"]:
                selected += ["--bench", target["name"]]
            else:
                raise ValueError(f"unclassified test target: {package['name']} {target}")
        base = ["cargo", "test", "--package", package["name"], "--no-fail-fast"]
        filters = ["--", "--skip", "csharp", "--skip", "CSharp", "--skip", "CSHARP"]
        if selected:
            commands.append(base + selected + filters)
        if docs:
            commands.append(base + ["--doc"] + filters)
    return {"commands": commands, "omitted_integration_targets": omitted,
            "test_name_filters": ["csharp", "CSharp", "CSHARP"],
            "scope": "Go development; C# validation deferred by user instruction"}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--plan-only", action="store_true")
    args = parser.parse_args()
    metadata = json.loads(subprocess.check_output(
        ["cargo", "metadata", "--offline", "--no-deps", "--format-version=1"], text=True))
    plan = test_plan(metadata)
    print(json.dumps(plan, indent=2), flush=True)
    if args.plan_only:
        return 0
    failed = False
    for command in plan["commands"]:
        print("\n==> " + shlex.join(command), flush=True)
        failed |= subprocess.run(command, check=False).returncode != 0
    return int(failed)


if __name__ == "__main__":
    raise SystemExit(main())
