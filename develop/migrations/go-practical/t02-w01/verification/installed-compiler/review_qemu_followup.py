"""Review only the added installed-root evidence and changed documentation."""
from pathlib import Path
import hashlib
import json
import sys
import tarfile
import types

ROOT = Path(__file__).resolve().parent
REPO = ROOT.parents[5]


def seal(path):
    data = path.read_bytes()
    return {'path': str(path.resolve()), 'bytes': len(data), 'sha256': hashlib.sha256(data).hexdigest(), 'mode': path.stat().st_mode & 0o777}


def main():
    scanner = REPO / 'scripts/check-no-active-gir.sh'
    code = scanner.read_text().split("<<'PY'\n", 1)[1].rsplit('\nPY', 1)[0].rsplit('raise SystemExit(main())', 1)[0]
    module = types.ModuleType('installed_go_qemu_followup_review')
    module.__file__ = str(scanner)
    sys.modules[module.__name__] = module
    sys.argv = [str(scanner), str(REPO), 'strict']
    exec(compile(code, str(scanner), 'exec'), module.__dict__)
    matchers = module.load_matchers()
    manifest, _, _, _ = module.validate_manifest()
    module.run_fixture_self_test(manifest, matchers)
    assert (len(matchers), len(module.SCOPES), len(manifest['sealed_records']), len(module.active_retained_record_exclusions(manifest))) == (127, 4, 39, 51)
    names = ['build_qemu_probe.py', 'check_qemu_probe.py', 'diagnose_installed.py', 'diagnose_qemu_installed.py', 'extract_qemu_runtime.py', 'finalize_rust.py', 'inspect_installed.py', 'inspect_installed_qemu.py', 'openat2_probe.py', 'qemu_openat2.go', 'run_installed.py', 'run_installed_qemu.py', 'run_openat2_probe.py', 'run_original_binding_diagnostic.py', 'run_original_binding_root_check.py', 'run_qemu_probe.py', 'review_qemu_followup.py', 'README.md']
    directories = ['host-clippy-diagnostic', 'host-clippy-root-check', 'installed-qemu-binding', 'installed-qemu-diagnostic', 'installed-root-binding', 'installed-root-diagnostic', 'installed-seccomp-diagnostic', 'linux-test-bound-root-check', 'linux-test-diagnostic', 'llvm-ranged-retrieval', 'openat2-environment-diagnostic', 'qemu-amd64-runtime', 'qemu-backports-openat2', 'qemu-backports-setup', 'qemu-bookworm-openat2', 'qemu-probe-build', 'qemu-probe-check', 'qemu-user-setup', 'rust-finalized-bundles', 'rust-format-diagnostic-check', 'rust-format-diagnostic-write', 'rust-format-root-check-write', 'rust-retry-reproduction', 'whole-snapshot-diagnostic', 'whole-snapshot-root-check']
    paths = {ROOT / name for name in names}
    paths.update(p for name in directories for p in (ROOT / name).rglob('*') if p.is_file())
    paths.add(ROOT / 'installed-qemu-inspection.json')
    paths.update(REPO / name for name in ['crates/mpk-cli/tests/go_practical_installed_compiler.rs', 'develop/docs/09_go_practical_subset_design-todo.md', 'develop/docs/go-practical-implementation-traceability-ledger.md', 'develop/migrations/go-practical/t02-w01/README.md', 'develop/migrations/go-practical/t02-w01/requirements.json', 'develop/migrations/go-practical/t02-w01/verification.json'])
    rows, findings, members = [], [], 0
    for path in sorted(paths):
        assert path.is_file()
        data = path.read_bytes()
        relative = str(path.relative_to(REPO))
        rows.append(seal(path))
        if path.name.endswith('.tar.gz'):
            with tarfile.open(path) as archive:
                for member in archive:
                    if member.isfile():
                        raw = archive.extractfile(member).read()
                        assert len(raw) == member.size
                        findings.extend(vars(item) for item in module.detect(member.name, raw.decode(errors='ignore'), matchers))
                        members += 1
        else:
            findings.extend(vars(item) for item in module.detect(relative, data.decode(errors='ignore'), matchers))
        if path.suffix == '.py':
            compile(data, relative, 'exec')
    inspection = json.loads((ROOT / 'installed-qemu-inspection.json').read_bytes())
    assert inspection['status'] == 'passed' and inspection['actual_installed_root_exercised'] and not inspection['native_Linux_release_acceptance']
    for name in ['whole-snapshot-diagnostic', 'host-clippy-diagnostic', 'rust-format-diagnostic-check', 'qemu-probe-check', 'rust-finalized-bundles']:
        assert json.loads((ROOT / name / 'result.json').read_bytes())['status'] == 'passed'
    for name in ['installed-root-binding', 'rust-retry-reproduction']:
        assert json.loads((ROOT / name / 'result.json').read_bytes())['status'] == 'failed'
    for name in ['develop/migrations/go-practical/t02-w01/requirements.json', 'develop/migrations/go-practical/t02-w01/verification.json']:
        json.loads((REPO / name).read_bytes())
    for name in ['develop/docs/09_go_practical_subset_design-todo.md', 'develop/docs/go-practical-implementation-traceability-ledger.md', 'develop/migrations/go-practical/t02-w01/README.md', str((ROOT / 'README.md').relative_to(REPO))]:
        assert sum(line.lstrip().startswith('```') for line in (REPO / name).read_text().splitlines()) % 2 == 0
    result = {'status': 'passed' if not findings else 'failed', 'selection_reason': 'Review changed Rust test, the installed-root recovery and QEMU test drivers, their original inputs and outputs, and updated Go/Rust-only status records. Preserve existing scanner matchers, seals and exclusions; do not run the whole T02 gate.', 'inputs': rows, 'archive_members': members, 'findings': findings, 'matcher_count': len(matchers), 'scopes': len(module.SCOPES), 'seals': len(manifest['sealed_records']), 'exclusions': len(module.active_retained_record_exclusions(manifest)), 'actual_installed_root_exercised': True, 'native_Linux_release_acceptance': False, 'whole_repository_gate_run': False, 'full_T_gate': 'Deferred to final T02-W04: ./scripts/check-fast.sh'}
    (ROOT / 'review-qemu-followup.json').write_text(json.dumps(result, indent=2) + '\n')
    print(json.dumps({'status': result['status'], 'inputs': len(rows), 'archive_members': members, 'findings': len(findings)}))
    raise SystemExit(bool(findings))


if __name__ == '__main__':
    main()
