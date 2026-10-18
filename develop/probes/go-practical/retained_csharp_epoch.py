"""Replay retained C# metadata at its exact source epoch, outside the sandbox.

This does not run present-source proofs, finish the pending C# refreeze, or
grant runtime/profile authority. Git objects and both historical helpers stay
unchanged. Every exported input is verified before and after replay.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys
import tarfile
import tempfile

ROOT = Path(__file__).resolve().parents[3]
RECORDS = ROOT / "develop/migrations/go-practical/t01-w05/retained-csharp"
EPOCHS = {
    "published": ("16946c6ae2a82004be03f49ba52726001dd3317f", 1645),
    "private": ("40469d298f1d69558713d02ea0b553a1dd583bf5", 115),
}


def need(condition, message):
    if not condition:
        raise ValueError(message)


def git(*args, output=None):
    return subprocess.run(
        ["/usr/bin/git", *args], cwd=ROOT, check=True,
        stdout=output or subprocess.PIPE, stderr=subprocess.PIPE,
    ).stdout


def load(epoch):
    value = json.loads((RECORDS / (epoch + "-epoch.json")).read_bytes())
    commit, count = EPOCHS[epoch]
    need(set(value) == {"schema", "epoch", "commit", "tree", "files", "results", "scope"}, "closed source epoch")
    need(value["schema"] == "mpk.go_practical.retained_csharp_source_epoch.v0", "epoch schema")
    need(value["epoch"] == epoch and value["commit"] == commit, "fixed epoch")
    need(len(value["files"]) == count, "complete retained input count")
    paths = []
    for row in value["files"]:
        need(set(row) == {"path", "git_blob", "mode", "sha256", "bytes"}, "closed file record")
        path = row["path"]
        need(isinstance(path, str) and path and not path.startswith(("/", "-"))
             and "\\" not in path and ":" not in path.split("/")[0]
             and not any(ord(c) < 32 or ord(c) == 127 for c in path)
             and all(p not in ("", ".", "..") for p in path.split("/")), "safe source path")
        need(row["mode"] in ("100644", "100755"), "regular source input")
        need(type(row["bytes"]) is int and row["bytes"] >= 0, "exact byte count")
        for field, length in (("git_blob", 40), ("sha256", 64)):
            digest = row[field]
            need(type(digest) is str and len(digest) == length
                 and all(c in "0123456789abcdef" for c in digest), "source identity")
        paths.append(path)
    need(paths == sorted(set(paths)), "sorted unique inputs")
    need(git("rev-parse", commit + "^{tree}").decode().strip() == value["tree"], "commit/tree binding")
    arguments = ["ls-tree", "-r", "-z", commit]
    if epoch == "private":
        arguments += ["--", *paths]
    observed = {}
    for entry in git(*arguments).split(b"\0"):
        if not entry:
            continue
        meta, path = entry.split(b"\t", 1)
        mode, kind, blob = meta.decode().split()
        need(kind == "blob" and mode in ("100644", "100755"), "regular Git source")
        observed[path.decode()] = (mode, blob)
    need(observed == {r["path"]: (r["mode"], r["git_blob"]) for r in value["files"]}, "exact Git input closure")
    return value


def verify_files(value, directory):
    for row in value["files"]:
        path = directory / row["path"]
        need(path.is_file() and not path.is_symlink(), "regular input " + row["path"])
        data = path.read_bytes()
        need(len(data) == row["bytes"] and hashlib.sha256(data).hexdigest() == row["sha256"], "raw input " + row["path"])
        need(("100755" if path.stat().st_mode & 0o111 else "100644") == row["mode"], "mode " + row["path"])


def replay(epoch):
    value = load(epoch)
    with tempfile.TemporaryDirectory(prefix="mpk-retained-csharp-") as temporary:
        directory = Path(temporary)
        archive = directory / "epoch.tar"
        paths = [row["path"] for row in value["files"]]
        with archive.open("wb") as output:
            git("archive", "--format=tar", value["commit"], "--", *paths, output=output)
        with tarfile.open(archive) as stream:
            for member in stream.getmembers():
                need(not member.name.startswith("/") and ".." not in Path(member.name).parts, "safe archive path")
                need(member.isdir() or member.isfile() and member.name in paths, "closed regular archive")
            stream.extractall(directory)
        archive.unlink()
        verify_files(value, directory)
        scripts = ["profile_freeze.py"]
        if epoch == "published":
            scripts.append("profile_package.py")
        observations = []
        for script in scripts:
            command = [sys.executable, "develop/probes/csharp-03/" + script, "--check"]
            result = subprocess.run(command, cwd=directory, capture_output=True,
                                    env={**os.environ, "PYTHONDONTWRITEBYTECODE": "1"}, timeout=120)
            need(result.returncode == 0 and not result.stderr, script + ": " + result.stderr.decode(errors="replace")[-2000:])
            observations.append({"helper": script, "exit_code": result.returncode,
                                 "stdout_sha256": hashlib.sha256(result.stdout).hexdigest()})
        verify_files(value, directory)
    return {"epoch": epoch, "commit": value["commit"], "tree": value["tree"],
            "files": len(paths), "observations": observations,
            "historical_inputs_changed": False, "current_application_proofs": False,
            "profile_activated": False}


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("epoch", choices=EPOCHS)
    args = parser.parse_args()
    print(json.dumps(replay(args.epoch), sort_keys=True))
