"""Retain the complete original frontend and affected Rust inputs before edits."""
from pathlib import Path
import hashlib
import json
import sys
import tarfile

ROOT = Path(__file__).resolve().parent
REPO = ROOT.parents[5]
INSTALLED = ROOT.parent / 'installed-compiler'
REGISTERED = Path('/private/tmp/mpk-installed-compiler-205-original-go-output/frontend/bin/go2vir')
sys.dont_write_bytecode = True
sys.path.insert(0, str(INSTALLED))
from run_scoped import seal, sdk_files, write


def main():
    out = ROOT / 'prelaunch'
    out.mkdir(exist_ok=False)
    paths = {Path(__file__).resolve(), INSTALLED / 'run_scoped.py', INSTALLED / 'original-go-source.tar', INSTALLED / 'reproduced-go-current/result.json', INSTALLED / 'installed-qemu-inspection.json', REPO / 'Cargo.toml', REPO / 'Cargo.lock', REPO / 'crates/mpk-vc/Cargo.toml', REPO / 'crates/mpk-vc/src/go_practical_type_keys.rs', REPO / 'crates/mpk-cli/Cargo.toml', REPO / 'crates/mpk-cli/src/go_practical_installed_compiler.rs', REPO / 'crates/mpk-cli/tests/go_practical_installed_compiler.rs', REPO / 'release/bundles/bundle-registry.json', REPO / 'release/bundles/semantic-profile-registry.json', REPO / 'release/bundles/candidates/go.json'}
    paths.update(p for p in (REPO / 'go-tools/go2vir').rglob('*') if p.is_file())
    paths.update(p for p in (REPO / 'crates/mpk-vc/src/go_practical_type_keys').rglob('*') if p.is_file())
    paths.update((REPO / 'crates/mpk-vc/tests').glob('go_practical_compiler_*.rs'))
    before = [seal(p) for p in sorted(paths)]
    binary = seal(REGISTERED)
    sdk = sdk_files()
    archive = out / 'inputs.tar.gz'
    with tarfile.open(archive, 'x:gz', dereference=True) as target:
        for path in sorted(paths):
            target.add(path, arcname=str(path.relative_to(REPO)), recursive=False)
        target.add(REGISTERED, arcname='environment/registered-go2vir', recursive=False)
    with tarfile.open(archive) as source:
        members = source.getmembers()
        expected = {str(Path(row['path']).relative_to(REPO)): row for row in before}
        expected['environment/registered-go2vir'] = binary
        assert {member.name for member in members} == set(expected)
        for member in members:
            raw = source.extractfile(member).read()
            row = expected[member.name]
            assert member.isfile() and member.mode == row['mode'] and len(raw) == row['bytes'] and hashlib.sha256(raw).hexdigest() == row['sha256']
    unchanged = before == [seal(p) for p in sorted(paths)] and binary == seal(REGISTERED) and sdk == sdk_files()
    result = {'status': 'passed' if unchanged else 'failed', 'selection_reason': 'Preserve the whole original frontend source tree, previous registered source epoch and exact registered binary, complete affected Rust type-key sources and existing compiler tests before independently binding original analysis vendor bytes to complete pinned module archives. The full registered SDK remains individually sealed by the preceding original owner.', 'inputs': before, 'registered_frontend': binary, 'registered_sdk': sdk, 'archive': seal(archive), 'archive_members_checked': len(members), 'inputs_unchanged': unchanged, 'native_Linux_release_acceptance': False, 'full_T_gate': 'Deferred to final T02-W04: ./scripts/check-fast.sh'}
    write(out / 'result.json', result)
    print(json.dumps({'status': result['status'], 'archive_members_checked': len(members), 'inputs_unchanged': unchanged, 'registered_sdk_files': len(sdk)}))
    raise SystemExit(not unchanged)


if __name__ == '__main__':
    main()
