#!/usr/bin/env python3
"""Compare a probe JDK tree with the complete, frozen archive projection."""

import hashlib
import json
from pathlib import Path
import stat
import sys


def check(root, vector_path):
    if root.is_symlink() or not root.is_dir():
        raise ValueError("JDK root must be a directory, not a link")
    value = json.loads(vector_path.read_text())
    inventory = value["toolchain_inputs"]["jdk_inventory"]
    expected = {item["path"]: item for item in inventory}
    actual = {".": root}
    actual.update({str(path.relative_to(root)): path for path in root.rglob("*")})
    if actual.keys() != expected.keys():
        raise ValueError("JDK inventory membership differs")
    for name, path in actual.items():
        metadata = path.lstat()
        item = expected[name]
        # POSIX symlink permissions do not control access; macOS extraction
        # reports 0755 while Linux reports 0777. Still require the exact link
        # type and target below. File/directory modes remain exact.
        if item["kind"] != "symlink" and f"{stat.S_IMODE(metadata.st_mode):04o}" != item["mode"]:
            raise ValueError(f"JDK mode differs: {name}")
        if item["kind"] == "directory":
            if not stat.S_ISDIR(metadata.st_mode):
                raise ValueError(f"JDK directory differs: {name}")
        elif item["kind"] == "symlink":
            if not stat.S_ISLNK(metadata.st_mode) or str(path.readlink()) != item["target"]:
                raise ValueError(f"JDK link differs: {name}")
        elif item["kind"] == "regular":
            if not stat.S_ISREG(metadata.st_mode) or metadata.st_nlink != 1:
                raise ValueError(f"JDK file type or hard-link alias differs: {name}")
            with path.open("rb") as source:
                digest = hashlib.file_digest(source, "sha256").hexdigest()
            if metadata.st_size != item["bytes"] or digest != item["sha256"]:
                raise ValueError(f"JDK bytes differ: {name}")
        else:
            raise ValueError("unknown inventory kind")


if __name__ == "__main__":
    if len(sys.argv) != 2:
        raise SystemExit("usage: check-jdk.py /absolute/path/to/pinned/jdk")
    vector = Path(__file__).resolve().parents[2] / "specs/vectors/java-profile-v0.json"
    try:
        check(Path(sys.argv[1]), vector)
    except (ValueError, OSError, KeyError) as error:
        raise SystemExit(f"Java probe JDK rejected: {error}")
