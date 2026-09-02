#!/usr/bin/env python3
"""Small hostile-input tests for the T02 owner; no JDK or Docker required."""

import copy
import importlib.util
import io
import json
import os
from pathlib import Path
import shutil
import stat
import sys
import tarfile
import tempfile
import unittest
import warnings
import zipfile


ROOT = Path(__file__).resolve().parent.parent
SPEC = importlib.util.spec_from_file_location("java_build_owner", ROOT / "scripts/java_build_inputs.py")
BUILD = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(BUILD)
CLASS = b"\xca\xfe\xba\xbe\x00\x00\x00\x45test-class-shape"


def fixture_inputs():
    payload = b"pinned compiler fixture\n"
    return {
        "archive": {"root": "jdk"},
        "archive_policy": {"max_entries": 32, "max_archive_bytes": 65536,
                           "max_extracted_bytes": 65536},
        "jdk_inventory": [
            {"path": ".", "kind": "directory", "mode": "0755"},
            {"path": "bin", "kind": "directory", "mode": "0755"},
            {"path": "bin/java", "kind": "regular", "mode": "0755",
             "bytes": len(payload), "sha256": BUILD.sha256(payload)},
            {"path": "legal", "kind": "directory", "mode": "0755"},
            {"path": "legal/LICENSE", "kind": "regular", "mode": "0444",
             "bytes": 7, "sha256": BUILD.sha256(b"license")},
            {"path": "legal/alias", "kind": "symlink", "mode": "0777", "target": "LICENSE"},
        ],
    }


def fixture_members(inputs):
    result = []
    for item in inputs["jdk_inventory"]:
        name = "jdk" if item["path"] == "." else "jdk/" + item["path"]
        header = tarfile.TarInfo(name)
        header.mode = int(item["mode"], 8)
        payload = b""
        if item["kind"] == "directory":
            header.type = tarfile.DIRTYPE
        elif item["kind"] == "symlink":
            header.type = tarfile.SYMTYPE
            header.linkname = item["target"]
        else:
            payload = b"license" if item["path"].startswith("legal/") else b"pinned compiler fixture\n"
            header.size = len(payload)
        result.append((header, payload))
    return result


def write_tar(path, members):
    with tarfile.open(path, "w:gz", format=tarfile.USTAR_FORMAT) as archive:
        for header, payload in members:
            archive.addfile(header, io.BytesIO(payload) if header.isreg() else None)


class BuildInputsTests(unittest.TestCase):
    def test_checked_in_recipe_inventory_and_inactive_release_are_bound(self):
        inputs = BUILD.load_toolchain()
        self.assertEqual(inputs["toolchain_inputs_sha256"], BUILD.TOOLCHAIN_HASH)
        descriptor = BUILD.load_descriptor()
        inventory = BUILD.load_inventory(descriptor)
        self.assertEqual(inventory["build_recipe_sha256"], BUILD.sha256(BUILD.canonical(BUILD.RECIPE)))
        BUILD.validate_active_boundary()

    def test_toolchain_mutations_reject_even_after_self_hash_repair(self):
        original = BUILD.load_toolchain()
        for field, value in (
            ("native_image", "docker.io/library/python:latest"),
            ("compiler_profile_id", "host-javac"),
            ("extra", True),
        ):
            with self.subTest(field=field):
                mutated = copy.deepcopy(original)
                mutated[field] = value
                payload = {key: item for key, item in mutated.items() if key != "toolchain_inputs_sha256"}
                mutated["toolchain_inputs_sha256"] = BUILD.sha256(BUILD.TOOLCHAIN_DOMAIN + BUILD.canonical(payload))
                with self.assertRaises(BUILD.BuildFailure):
                    BUILD.validate_toolchain(mutated)

    def test_json_refuses_duplicates_numbers_encoding_and_noncanonical_transport(self):
        for data in (
            b'{"x":1,"x":2}', b'{"a":{"x":1,"x":2}}', b'{"n":1.0}',
            b'{"n":NaN}', b'{"x":"\\ud800"}', b'\xef\xbb\xbf{}', b'{"x":"\xff"}',
        ):
            with self.subTest(data=data):
                with self.assertRaises(BUILD.BuildFailure):
                    BUILD.strict_json(data)
        for data in (b"{}", b"{ }\n", b'{"b":0,"a":0}\n'):
            with self.assertRaises(BUILD.BuildFailure):
                BUILD.strict_json(data, canonical_transport=True)
        self.assertEqual(BUILD.strict_json(b'{"a":0}\n', canonical_transport=True), {"a": 0})

    def test_source_inventory_captures_all_files_and_refuses_aliases(self):
        with tempfile.TemporaryDirectory(prefix="java-project-test-") as directory:
            root = Path(directory) / "project"
            shutil.copytree(ROOT / BUILD.PROJECT, root)
            self.assertEqual(BUILD.project_records(root), BUILD.project_records(ROOT / BUILD.PROJECT))
            extra = root / "src/mpk/java2vir/Injected.java"
            extra.write_text("class Injected {}\n")
            with self.assertRaises(BUILD.BuildFailure):
                BUILD.project_records(root)
            extra.unlink()
            empty = root / "unused"
            empty.mkdir()
            with self.assertRaises(BUILD.BuildFailure):
                BUILD.project_records(root)
            empty.rmdir()
            source = root / "NOTICE.txt"
            outside = Path(directory) / "notice"
            source.rename(outside)
            source.symlink_to(outside)
            with self.assertRaises((BUILD.BuildFailure, OSError)):
                BUILD.project_records(root)

    def test_regular_capture_rejects_hardlinks_symlinks_special_files_and_oversize(self):
        with tempfile.TemporaryDirectory(prefix="java-capture-test-") as directory:
            root = Path(directory)
            source = root / "source"
            source.write_bytes(b"source")
            expected = BUILD.record_bytes("source", b"source")
            BUILD.copy_verified(source, root / "copy", expected, 64)
            self.assertEqual((root / "copy").read_bytes(), b"source")
            os.link(source, root / "hard-link")
            with self.assertRaises(BUILD.BuildFailure):
                BUILD.read_bytes(source, 64)
            (root / "hard-link").unlink()
            (root / "symlink").symlink_to(source)
            with self.assertRaises((BUILD.BuildFailure, OSError)):
                BUILD.read_bytes(root / "symlink", 64)
            os.mkfifo(root / "fifo")
            with self.assertRaises(BUILD.BuildFailure):
                BUILD.read_bytes(root / "fifo", 64)
            with source.open("wb") as output:
                output.truncate(1024 * 1024)
            with self.assertRaises(BUILD.BuildFailure):
                BUILD.copy_verified(source, root / "rejected", expected, 64)
            self.assertFalse((root / "rejected").exists())

    def test_file_change_during_capture_rejects(self):
        with tempfile.TemporaryDirectory(prefix="java-race-test-") as directory:
            path = Path(directory) / "source"
            path.write_bytes(b"original")
            with self.assertRaises(BUILD.BuildFailure):
                with BUILD.opened_regular(path, 64) as (source, _):
                    self.assertEqual(source.read(), b"original")
                    path.write_bytes(b"modified-longer")

    def test_archive_materialization_preserves_readonly_modes_and_relative_links(self):
        inputs = fixture_inputs()
        with tempfile.TemporaryDirectory(prefix="java-archive-test-") as directory:
            root = Path(directory)
            archive = root / "jdk.tar.gz"
            write_tar(archive, fixture_members(inputs))
            BUILD.extract_jdk(archive, root / "jdk", inputs)
            BUILD.validate_jdk(root / "jdk", inputs)
            self.assertEqual(stat.S_IMODE((root / "jdk/legal/LICENSE").stat().st_mode), 0o444)
            self.assertEqual(os.readlink(root / "jdk/legal/alias"), "LICENSE")

    def test_archive_traversal_duplicates_missing_extra_links_and_modes_reject(self):
        inputs = fixture_inputs()
        for mutation in ("traversal", "absolute", "duplicate", "missing", "extra",
                         "hardlink", "link_escape", "mode", "bytes", "budget"):
            with self.subTest(mutation=mutation), tempfile.TemporaryDirectory(prefix="java-bad-tar-") as directory:
                root = Path(directory)
                members = fixture_members(inputs)
                expected = copy.deepcopy(inputs)
                if mutation == "traversal":
                    members[2][0].name = "jdk/../escape"
                elif mutation == "absolute":
                    members[2][0].name = "/jdk/bin/java"
                elif mutation == "duplicate":
                    members.append(copy.deepcopy(members[2]))
                elif mutation == "missing":
                    members.pop(2)
                elif mutation == "extra":
                    extra = copy.deepcopy(members[2])
                    extra[0].name = "jdk/extra"
                    members.append(extra)
                elif mutation == "hardlink":
                    members[2][0].type = tarfile.LNKTYPE
                    members[2][0].linkname = "jdk/legal/LICENSE"
                elif mutation == "link_escape":
                    members[5][0].linkname = "../../escape"
                elif mutation == "mode":
                    members[2][0].mode = 0o4755
                elif mutation == "bytes":
                    members[2] = (members[2][0], b"wrong compiler fixture\n\n")
                    members[2][0].size = len(members[2][1])
                elif mutation == "budget":
                    expected["archive_policy"]["max_extracted_bytes"] = 1
                write_tar(root / "bad.tar.gz", members)
                with self.assertRaises(BUILD.BuildFailure):
                    BUILD.extract_jdk(root / "bad.tar.gz", root / "jdk", expected)
                self.assertFalse((root / "escape").exists())

    def test_extracted_jdk_mutations_reject(self):
        inputs = fixture_inputs()
        for mutation in ("bytes", "mode", "extra", "link", "hardlink", "directory_link"):
            with self.subTest(mutation=mutation), tempfile.TemporaryDirectory(prefix="java-jdk-test-") as directory:
                root = Path(directory)
                write_tar(root / "jdk.tar.gz", fixture_members(inputs))
                jdk = root / "jdk"
                BUILD.extract_jdk(root / "jdk.tar.gz", jdk, inputs)
                if mutation == "bytes":
                    (jdk / "bin/java").write_bytes(b"modified compiler")
                elif mutation == "mode":
                    (jdk / "bin/java").chmod(0o777)
                elif mutation == "extra":
                    (jdk / "extra").write_bytes(b"extra")
                elif mutation == "link":
                    (jdk / "legal/alias").unlink()
                    (jdk / "legal/alias").symlink_to("../../escape")
                elif mutation == "hardlink":
                    os.link(jdk / "bin/java", root / "hard-link")
                else:
                    (jdk / "bin").rename(root / "outside")
                    (jdk / "bin").symlink_to(root / "outside")
                with self.assertRaises(BUILD.BuildFailure):
                    BUILD.validate_jdk(jdk, inputs)

    def test_jar_is_byte_deterministic_and_closed(self):
        files = {"mpk/java2vir/Main.class": CLASS, "mpk/java2vir/Other.class": CLASS + b"other"}
        jar = BUILD.make_jar(files)
        self.assertEqual(jar, BUILD.make_jar(dict(reversed(list(files.items())))))
        self.assertEqual(BUILD.inspect_jar(jar), files)
        for suffix in (b"\x00", b"unaccounted data"):
            with self.assertRaises(BUILD.BuildFailure):
                BUILD.inspect_jar(jar + suffix)

    def test_jar_manifest_service_class_metadata_and_bytecode_mutations_reject(self):
        baseline = BUILD.make_jar({"mpk/java2vir/Main.class": CLASS})
        for mutation in ("classpath", "service", "duplicate", "timestamp", "compressed",
                         "comment", "extra", "mode", "version", "preview", "package"):
            with self.subTest(mutation=mutation):
                output = io.BytesIO()
                with zipfile.ZipFile(io.BytesIO(baseline)) as source, zipfile.ZipFile(output, "w") as target:
                    for entry in source.infolist():
                        content = source.read(entry)
                        if mutation == "classpath" and entry.filename == "META-INF/MANIFEST.MF":
                            content = b"Manifest-Version: 1.0\nClass-Path: /external.jar\n\n"
                        elif entry.filename.endswith(".class"):
                            if mutation == "timestamp":
                                entry.date_time = (2026, 8, 31, 0, 0, 0)
                            elif mutation == "compressed":
                                entry.compress_type = zipfile.ZIP_DEFLATED
                            elif mutation == "comment":
                                entry.comment = b"host-dependent"
                            elif mutation == "extra":
                                entry.extra = b"\xfe\xca\x00\x00"
                            elif mutation == "mode":
                                entry.external_attr = (stat.S_IFREG | 0o755) << 16
                            elif mutation == "version":
                                content = content[:6] + b"\x00\x44" + content[8:]
                            elif mutation == "preview":
                                content = content[:4] + b"\xff\xff" + content[6:]
                            elif mutation == "package":
                                entry.filename = "unlisted/Injected.class"
                        target.writestr(entry, content)
                    if mutation == "service":
                        target.writestr("META-INF/services/javax.annotation.processing.Processor", b"Injected\n")
                    if mutation == "duplicate":
                        with warnings.catch_warnings():
                            warnings.simplefilter("ignore", UserWarning)
                            target.writestr("mpk/java2vir/Main.class", CLASS)
                with self.assertRaises(BUILD.BuildFailure):
                    BUILD.inspect_jar(output.getvalue())

    def test_subprocess_input_environment_output_and_time_are_bounded(self):
        environment = {"LANG": "C.UTF-8", "LC_ALL": "C.UTF-8"}
        code, stdout, stderr = BUILD.execute(
            [sys.executable, "-I", "-S", "-c",
             "import sys,os; assert not sys.stdin.buffer.read(); assert 'JAVA_HOME' not in os.environ; print('ok')"],
            environment=environment, limit=16, timeout=5)
        self.assertEqual((code, stdout, stderr), (0, b"ok\n", b""))
        with self.assertRaises(BUILD.BuildFailure) as failure:
            BUILD.execute([sys.executable, "-I", "-S", "-c", "print('x'*65536)"],
                          environment=environment, limit=16, timeout=5)
        self.assertEqual(failure.exception.code, "JAVA_BUILD_OUTPUT_LIMIT")
        with self.assertRaises(BUILD.BuildFailure) as failure:
            BUILD.execute([sys.executable, "-I", "-S", "-c", "import time; time.sleep(60)"],
                          environment=environment, timeout=0.05)
        self.assertEqual(failure.exception.code, "JAVA_BUILD_TIMEOUT")
        with self.assertRaises(BUILD.BuildFailure) as failure:
            BUILD.execute(
                [sys.executable, "-I", "-S", "-c", "import os,time; os.close(1); os.close(2); time.sleep(60)"],
                environment=environment, timeout=0.05)
        self.assertEqual(failure.exception.code, "JAVA_BUILD_TIMEOUT")

    def test_candidate_export_never_replaces_existing_output(self):
        with tempfile.TemporaryDirectory(prefix="java-output-test-") as directory:
            existing = Path(directory).resolve() / "existing"
            existing.mkdir()
            sentinel = existing / "user-file"
            sentinel.write_bytes(b"keep")
            with self.assertRaises(BUILD.BuildFailure):
                BUILD.export_candidate(str(existing), b"", {}, {})
            self.assertEqual(sentinel.read_bytes(), b"keep")
            alias = existing.parent / "alias"
            alias.symlink_to(existing)
            with self.assertRaises(BUILD.BuildFailure):
                BUILD.export_candidate(str(alias), b"", {}, {})


if __name__ == "__main__":
    result = unittest.TestResult()
    unittest.defaultTestLoader.loadTestsFromTestCase(BuildInputsTests).run(result)
    if not result.wasSuccessful():
        for _, failure in result.errors + result.failures:
            print(failure, file=sys.stderr)
        raise SystemExit(1)
