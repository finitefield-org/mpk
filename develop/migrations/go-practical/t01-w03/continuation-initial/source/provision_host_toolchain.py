#!/usr/bin/env python3
"""Explicit preparation only: fetch/check the pinned host analysis toolchain."""

import argparse
import json
from pathlib import Path
import shutil
import subprocess
import tarfile
import tempfile

from run_module_probes import HERE, DEFAULT_GOROOT, digest, verify_host_toolchain


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--cache-root", type=Path, default=DEFAULT_GOROOT.parents[1])
    args = parser.parse_args()
    root = args.cache_root.resolve()
    root.mkdir(parents=True, exist_ok=True)
    pin = json.loads((HERE / "host-toolchain.json").read_text())
    archive = root / pin["archive"]
    if not archive.exists():
        temporary = root / (pin["archive"] + ".partial")
        subprocess.run([
            "/usr/bin/curl", "--fail", "--location", "--silent", "--show-error",
            "--retry", "2", "--max-time", "180", "--output", str(temporary), pin["download_url"],
        ], check=True)
        if temporary.stat().st_size != pin["archive_bytes"] or digest(temporary.read_bytes()) != pin["archive_sha256"]:
            raise SystemExit("downloaded compiler archive identity differs")
        temporary.rename(archive)
    if archive.stat().st_size != pin["archive_bytes"] or digest(archive.read_bytes()) != pin["archive_sha256"]:
        raise SystemExit("cached compiler archive identity differs")
    installation = root / "go1.25.0-darwin-arm64"
    if not installation.exists():
        temporary = Path(tempfile.mkdtemp(prefix="go1.25.0-extract-", dir=root))
        try:
            with tarfile.open(archive) as bundle:
                bundle.extractall(temporary, filter="data")
            temporary.rename(installation)
        finally:
            if temporary.exists():
                shutil.rmtree(temporary)
    goroot = installation / "go"
    inventory = verify_host_toolchain(goroot, pin)
    print(f"PASS: official archive and {inventory['files']} extracted files match")
    print(goroot)


if __name__ == "__main__":
    main()
