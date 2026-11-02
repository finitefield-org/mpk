"""Replay a historical checkpoint from pinned Git objects, without editing it.

Invoke this tool outside the sandbox, as required for all repository Git use.
It never changes a ref or checkout. All Git invocations use /usr/bin/git.
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
RECORD = ROOT / "develop/migrations/go-practical/t01-w04/consumer-review"
COMMIT = "e19887077b7de9f512be72bfed52227ed4beebec"
TREE = "27299259ca6fd04c63f23891fcdcdbe20e574333"
GIT = "/usr/bin/git"


def sha(raw): return hashlib.sha256(raw).hexdigest()
def need(ok, message):
    if not ok: raise ValueError(message)


def git(*args, output=None):
    r = subprocess.run([GIT, *args], cwd=ROOT, check=True, stdout=output or subprocess.PIPE,
                       stderr=subprocess.PIPE)
    return r.stdout


def validate_shape(value):
    need(type(value) is dict and set(value) == {"schema", "commit", "tree", "checkpoint", "files", "scope"}, "closed source epoch")
    need(value["schema"] == "mpk.go_practical.historical_source_epoch.v0", "epoch schema")
    need(value["commit"] == COMMIT and value["tree"] == TREE, "exact historical source epoch")
    need(value["checkpoint"] == "run_release_contracts.check_retained", "fixed historical entry point")
    need(type(value["files"]) is list and value["files"], "nonempty exact source epoch")
    paths = []
    for row in value["files"]:
        need(type(row) is dict and set(row) == {"path", "sha256", "git_blob", "mode", "bytes"}, "closed epoch file")
        p = row["path"]
        need(type(p) is str and p and not p.startswith(("/", "-")) and "\\" not in p
             and all(x not in ("", ".", "..") for x in p.split("/")), "portable epoch path")
        need(row["mode"] in ("100644", "100755"), "regular tracked epoch file")
        need(type(row["bytes"]) is int and row["bytes"] >= 0, "exact epoch byte count")
        for k, n in (("sha256", 64), ("git_blob", 40)):
            need(type(row[k]) is str and len(row[k]) == n and all(c in "0123456789abcdef" for c in row[k]), "lowercase epoch identity")
        paths.append(p)
    need(paths == sorted(set(paths)), "sorted unique epoch paths")


def tree_rows(paths):
    raw = git("ls-tree", "-r", "-z", COMMIT, "--", *paths)
    rows = {}
    for entry in raw.split(b"\0"):
        if not entry: continue
        meta, path = entry.split(b"\t", 1); mode, kind, blob = meta.decode().split()
        need(kind == "blob" and mode in ("100644", "100755"), "tracked regular source bytes")
        rows[path.decode()] = (mode, blob)
    need(set(rows) == set(paths), "every epoch file exists at the pinned commit")
    return rows


def verify_git(value):
    validate_shape(value)
    need(git("rev-parse", "--verify", COMMIT + "^{tree}").decode().strip() == TREE, "historical commit/tree pair")
    rows = tree_rows([r["path"] for r in value["files"]])
    need(all(rows[r["path"]] == (r["mode"], r["git_blob"]) for r in value["files"]), "exact historical blob/mode closure")


def verify_files(value, root):
    validate_shape(value)
    for r in value["files"]:
        p = root / r["path"]; need(p.is_file() and not p.is_symlink(), "regular epoch input " + r["path"])
        raw = p.read_bytes()
        need(len(raw) == r["bytes"] and sha(raw) == r["sha256"], "historical raw bytes " + r["path"])
        mode = "100755" if p.stat().st_mode & 0o111 else "100644"
        need(mode == r["mode"], "historical executable mode " + r["path"])


def capture():
    sys.dont_write_bytecode = True
    accessed = set()
    def hook(event, args):
        if event != "open" or not isinstance(args[0], (str, bytes, os.PathLike)): return
        p = Path(os.fsdecode(args[0])).absolute()
        try: rel = p.relative_to(ROOT).as_posix()
        except ValueError: return
        if "__pycache__" not in p.parts and p.is_file(): accessed.add(rel)
    sys.addaudithook(hook)
    from run_release_contracts import check_retained
    r = check_retained()
    # A cached .pyc import need not open its .py source. Include the entire
    # tracked historical package, so replay also works with a cold import cache.
    package = {p for p in git("ls-tree", "-r", "--name-only", COMMIT, "--", "develop/probes/go-practical").decode().splitlines() if p.endswith(".py")}
    paths = sorted(accessed | package | {x["path"] for x in r["inputs"]})
    objects = tree_rows(paths); files = []
    for p in paths:
        raw = (ROOT / p).read_bytes(); mode, blob = objects[p]
        actual = hashlib.sha1(b"blob " + str(len(raw)).encode() + b"\0" + raw).hexdigest()
        need(actual == blob, "current bytes must equal pinned historical blob " + p)
        files.append({"path": p, "sha256": sha(raw), "git_blob": blob, "mode": mode, "bytes": len(raw)})
    value = {"schema": "mpk.go_practical.historical_source_epoch.v0", "commit": COMMIT, "tree": TREE,
             "checkpoint": "run_release_contracts.check_retained", "files": files,
             "scope": "actual successful historical reconstruction and complete observed read/input closure; not a production build or future-source proof"}
    verify_git(value); verify_files(value, ROOT)
    return value


def replay(value):
    verify_git(value)
    with tempfile.TemporaryDirectory(prefix="mpk-go-source-epoch-", dir="/private/tmp") as td:
        dest = Path(td); archive = dest / "epoch.tar"
        with archive.open("wb") as out:
            git("archive", "--format=tar", COMMIT, "--", *[r["path"] for r in value["files"]], output=out)
        with tarfile.open(archive) as tf:
            allowed = {r["path"] for r in value["files"]}
            for m in tf.getmembers():
                need(not m.name.startswith("/") and ".." not in Path(m.name).parts, "safe historical archive path")
                need(m.isdir() or m.isfile() and m.name in allowed, "only exact historical regular inputs")
            tf.extractall(dest)
        archive.unlink(); verify_files(value, dest)
        code = "from run_release_contracts import check_retained; import json; r=check_retained(); print(json.dumps({k:r[k] for k in ('cases','hash_vectors','retained_tuples','Go_ordinary_elaborated','profile_activated')},sort_keys=True))"
        env = {**os.environ, "PYTHONDONTWRITEBYTECODE": "1", "PYTHONPATH": str(dest / "develop/probes/go-practical")}
        proc = subprocess.run([sys.executable, "-c", code], cwd=dest, env=env,
                              capture_output=True, timeout=120)
        need(proc.returncode == 0 and not proc.stderr, "historical checkpoint replay: " + proc.stderr.decode(errors="replace")[-3000:])
        verify_files(value, dest)
        return {"commit": COMMIT, "tree": TREE, "files": len(value["files"]),
                "input_bytes": sum(r["bytes"] for r in value["files"]), "exit_code": proc.returncode,
                "stdout_sha256": sha(proc.stdout), "result": json.loads(proc.stdout),
                "historical_sources_changed": False, "production_refs_changed": False}


def main():
    p = argparse.ArgumentParser(description=__doc__); p.add_argument("mode", choices=("capture", "replay")); args = p.parse_args()
    RECORD.mkdir(parents=True, exist_ok=True); path = RECORD / "historical-source-epoch.json"
    if args.mode == "capture":
        value = capture(); path.write_text(json.dumps(value, indent=2, sort_keys=True) + "\n")
        print(json.dumps({"files": len(value["files"]), "bytes": sum(r["bytes"] for r in value["files"]), "commit": COMMIT}))
    else:
        value = json.loads(path.read_text()); result = replay(value)
        (RECORD / "historical-replay.json").write_text(json.dumps(result, indent=2, sort_keys=True) + "\n")
        print(json.dumps(result, sort_keys=True))


if __name__ == "__main__": main()
