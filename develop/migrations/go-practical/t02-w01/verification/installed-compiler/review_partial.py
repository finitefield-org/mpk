"""Scoped review of completed installed-compiler work and retained originals."""
from pathlib import Path
import hashlib
import json
import re
import subprocess
import sys
import tarfile
import types
import urllib.parse

ROOT = Path(__file__).resolve().parent
REPO = ROOT.parents[5]
BASIS = '7d37e98882edf86b6f2b537ff4592eca8b589ba4'
COMPLETED_DIRS = ['alternate-registry-validation', 'case-sensitive-volume', 'go-format-final', 'go-format-final-check', 'go-format-write', 'go-vet-final', 'host-check', 'host-clippy', 'host-clippy-final', 'host-clippy-msrv', 'host-clippy-snapshot', 'host-test', 'linux-original-owner', 'linux-owner-serial', 'linux-test-bound-build', 'linux-test-bound-final', 'linux-test-build', 'linux-test-final-build', 'materialize-original', 'reproduced-go', 'reproduced-go-current', 'rust-byte-reproduction', 'rust-download-diagnostics', 'rust-download-system-python', 'rust-format-default-write', 'rust-format-final', 'rust-format-final-check', 'rust-format-msrv', 'rust-format-snapshot-check', 'rust-format-snapshot-write', 'rust-format-write', 'rust-provision', 'whole-snapshot-binding', 'whole-snapshot-default']
CODE = ['crates/mpk-cli/Cargo.toml', 'crates/mpk-cli/src/lib.rs', 'crates/mpk-cli/src/go_practical_installed_compiler.rs', 'crates/mpk-cli/tests/go_practical_installed_compiler.rs', 'go-tools/go2vir/practical_compiler_capture_test.go', 'go-tools/go2vir/practical_compiler_inputs_test.go', 'develop/docs/09_go_practical_subset_design-todo.md', 'develop/docs/go-practical-implementation-traceability-ledger.md', 'develop/migrations/go-practical/t02-w01/README.md', 'develop/migrations/go-practical/t02-w01/requirements.json']


def git(*args):
    return subprocess.check_output(['/usr/bin/git', *args], cwd=REPO)


def seal(path):
    path = Path(path)
    data = path.read_bytes()
    return {'path': str(path.resolve()), 'bytes': len(data), 'sha256': hashlib.sha256(data).hexdigest(), 'mode': path.stat().st_mode & 0o777}


def main():
    assert git('rev-parse', 'HEAD').decode().strip() == BASIS
    scanner = REPO / 'scripts/check-no-active-gir.sh'
    code = scanner.read_text().split("<<'PY'\n", 1)[1].rsplit('\nPY', 1)[0].rsplit('raise SystemExit(main())', 1)[0]
    module = types.ModuleType('installed_go_scoped_review')
    module.__file__ = str(scanner)
    sys.modules[module.__name__] = module
    sys.argv = [str(scanner), str(REPO), 'strict']
    exec(compile(code, str(scanner), 'exec'), module.__dict__)
    matchers = module.load_matchers()
    manifest, configured, fixtures, _ = module.validate_manifest()
    module.run_fixture_self_test(manifest, matchers)
    assert (len(matchers), len(module.SCOPES), len(manifest['sealed_records']), len(module.active_retained_record_exclusions(manifest))) == (127, 4, 39, 51)
    for name in configured | fixtures:
        assert (REPO / name).read_bytes() == git('show', BASIS + ':' + name)
    assert scanner.read_bytes() == git('show', BASIS + ':' + str(scanner.relative_to(REPO)))
    previous = ROOT.parent / 'compiler-elf/review-final.json'
    assert previous.read_bytes() == git('show', BASIS + ':' + str(previous.relative_to(REPO)))
    assert json.loads(previous.read_bytes())['status'] == 'passed'

    paths = {REPO / name for name in CODE}
    paths.update(p for p in ROOT.iterdir() if p.is_file() and p.name != 'review-partial.json')
    paths.update(p for name in COMPLETED_DIRS for p in (ROOT / name).rglob('*') if p.is_file())
    exact_rules = [row for row in matchers if row['scope'] == 'exact-path']
    cache, findings, current, links = {}, [], [], []
    scanned = archive_members = 0

    def scan(name, data):
        nonlocal scanned
        key = (hashlib.sha256(data).hexdigest(), tuple(module.path_matches(name, row['path']) for row in exact_rules))
        if key not in cache:
            cache[key] = [vars(row) for row in module.detect(name, data.decode(errors='ignore'), matchers)]
        findings.extend({**row, 'path': name} for row in cache[key])
        scanned += 1

    for path in sorted(paths):
        assert path.is_file() and not path.is_symlink()
        row = seal(path)
        current.append(row)
        data = path.read_bytes()
        name = str(path.relative_to(REPO))
        if path.name.endswith(('.tar.gz', '.tar')):
            with tarfile.open(path, 'r:*') as archive:
                for member in archive:
                    if member.isfile():
                        body = archive.extractfile(member).read()
                        assert len(body) == member.size
                        scan(member.name, body)
                        archive_members += 1
        else:
            scan(name, data)
            if 'original-source-objects' in path.parts:
                scan('go-tools/go2vir/original-graph-objects/' + path.name, data)
        if path.suffix == '.py':
            compile(data, name, 'exec')
        if path.suffix == '.md':
            for target in re.findall(r'\]\(([^)]+)\)', data.decode()):
                target = target.strip('<>').split('#', 1)[0]
                if not target or re.match(r'^[a-zA-Z][a-zA-Z0-9+.-]*:', target):
                    continue
                resolved = path.parent / re.sub(r':\d+$', '', urllib.parse.unquote(target))
                links.append({'source': name, 'target': target, 'exists': resolved.exists()})

    original = ROOT / 'linux-owner-serial/native-originals/versioned-owned'
    packet = json.loads((original / 'complete-original-module-graph-inputs.json').read_bytes())
    wire = json.loads(bytes.fromhex(packet['compiler_inputs_hex']))
    assert len(wire['inventory']['files']) == len(packet['compiler_files']) == 5577
    for entry, item in zip(wire['inventory']['files'], packet['compiler_files']):
        data = (original / 'original-compiler-files' / item['raw_file']).read_bytes()
        assert item['entry'] == entry and len(data) == entry['size_bytes'] and hashlib.sha256(data).hexdigest() == entry['sha256'] and item['raw_file'] == entry['sha256'] + '.raw'
    assert len(packet['snapshot_sources']) == 5697 and len(packet['overlays']) == 0
    for item in packet['snapshot_sources']:
        data = (original / 'original-source-objects' / item['raw_file']).read_bytes()
        assert len(data) == item['object']['bytes'] and hashlib.sha256(data).hexdigest() == item['object']['raw_sha256']
    for item in packet['sources']:
        data = bytes.fromhex(item['raw_hex'])
        assert len(data) == item['object']['bytes'] and hashlib.sha256(data).hexdigest() == item['object']['raw_sha256']
        scan('go-tools/go2vir/original-selected-sources/' + item['object']['path'], data)
    for field in ['capture_hex', 'compiler_inputs_hex', 'components_hex', 'envelope_hex', 'export_inputs_hex', 'module_graph_inputs_hex', 'module_observations_hex', 'selection_hex', 'snapshot_hex']:
        scan('go-tools/go2vir/original-owner/' + field + '.json', bytes.fromhex(packet[field]))
    assert len(json.loads(bytes.fromhex(packet['selection_hex']))['roots']) == 1
    assert len(json.loads(bytes.fromhex(packet['export_inputs_hex']))['packages']) == 182
    checks = {}
    for name in ['go-vet-final', 'go-format-final-check', 'host-clippy-final', 'rust-format-final-check', 'whole-snapshot-default', 'linux-owner-serial', 'linux-test-bound-final', 'alternate-registry-validation']:
        record = json.loads((ROOT / name / 'result.json').read_bytes())
        assert record['status'] == 'passed'
        checks[name] = seal(ROOT / name / 'result.json')
    assert not findings and all(row['exists'] for row in links)
    result = {'status': 'passed', 'basis_commit': BASIS, 'driver': seal(__file__), 'matcher_count': len(matchers), 'scopes': len(module.SCOPES), 'seals': len(manifest['sealed_records']), 'exclusions': len(module.active_retained_record_exclusions(manifest)), 'previous_review': seal(previous), 'current_inputs': current, 'scanned_items': scanned, 'archive_members': archive_members, 'complete_linux_sdk_files': 5577, 'complete_original_sources': 5697, 'complete_original_packages': 182, 'root_selections': 1, 'contract_overlays': 0, 'checks': checks, 'new_findings': findings, 'links': links, 'installed_root_verified': False, 'native_Linux_release_acceptance': False, 'whole_repository_gate_run': False, 'full_T_gate': 'Deferred to final T02-W04: ./scripts/check-fast.sh'}
    (ROOT / 'review-partial.json').write_text(json.dumps(result, indent=2) + '\n')
    print(json.dumps({key:result[key] for key in ['status','scanned_items','archive_members','complete_linux_sdk_files','complete_original_sources']}))


if __name__ == '__main__':
    main()
