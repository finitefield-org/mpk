"""Review the separately retained local test-environment inputs."""
from pathlib import Path
import hashlib
import json
import sys
import tarfile
import types

ROOT = Path(__file__).resolve().parent
REPO = ROOT.parents[5]


def main():
    scanner = REPO / 'scripts/check-no-active-gir.sh'
    code = scanner.read_text().split("<<'PY'\n", 1)[1].rsplit('\nPY', 1)[0].rsplit('raise SystemExit(main())', 1)[0]
    module = types.ModuleType('installed_go_retained_qemu_review')
    module.__file__ = str(scanner)
    sys.modules[module.__name__] = module
    sys.argv = [str(scanner), str(REPO), 'strict']
    exec(compile(code, str(scanner), 'exec'), module.__dict__)
    matchers = module.load_matchers()
    manifest, _, _, _ = module.validate_manifest()
    module.run_fixture_self_test(manifest, matchers)
    assert (len(matchers), len(module.SCOPES), len(manifest['sealed_records']), len(module.active_retained_record_exclusions(manifest))) == (127, 4, 39, 51)
    paths = [Path(__file__).resolve(), ROOT / 'retain_qemu_inputs.py', ROOT / 'qemu-retained-environment/result.json']
    rows, findings = [], []
    for path in paths:
        data = path.read_bytes()
        rows.append({'path': str(path), 'bytes': len(data), 'sha256': hashlib.sha256(data).hexdigest()})
        findings.extend(vars(item) for item in module.detect(str(path.relative_to(REPO)), data.decode(errors='ignore'), matchers))
        if path.suffix == '.py':
            compile(data, str(path), 'exec')
    receipt = json.loads(paths[-1].read_bytes())
    assert receipt['status'] == 'passed' and receipt['inputs_unchanged'] and receipt['members_checked'] == 6
    archive = ROOT / 'qemu-retained-environment/inputs.tar.gz'
    raw_archive = archive.read_bytes()
    assert len(raw_archive) == receipt['archive']['bytes'] and hashlib.sha256(raw_archive).hexdigest() == receipt['archive']['sha256']
    with tarfile.open(archive) as data:
        members = data.getmembers()
        assert {item.name for item in members} == set(receipt['inputs'])
        for item in members:
            raw = data.extractfile(item).read()
            expected = receipt['inputs'][item.name]
            assert item.isfile() and item.mode == expected['mode'] and len(raw) == expected['bytes'] and hashlib.sha256(raw).hexdigest() == expected['sha256']
            findings.extend(vars(finding) for finding in module.detect(item.name, raw.decode(errors='ignore'), matchers))
    result = {'status': 'passed' if not findings else 'failed', 'selection_reason': 'Review the exact retained emulator, diagnostic and pinned-image runtime bytes added after the main scoped review; preserve all original scanner matchers and archive members.', 'inputs': rows, 'archive': receipt['archive'], 'archive_members': len(members), 'findings': findings, 'matcher_count': len(matchers), 'scopes': len(module.SCOPES), 'seals': len(manifest['sealed_records']), 'exclusions': len(module.active_retained_record_exclusions(manifest)), 'native_Linux_release_acceptance': False, 'whole_repository_gate_run': False, 'full_T_gate': 'Deferred to final T02-W04: ./scripts/check-fast.sh'}
    (ROOT / 'review-retained-qemu.json').write_text(json.dumps(result, indent=2) + '\n')
    print(json.dumps({'status': result['status'], 'archive_members': len(members), 'findings': len(findings)}))
    raise SystemExit(bool(findings))


if __name__ == '__main__':
    main()
