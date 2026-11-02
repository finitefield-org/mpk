"""Retain the exact separate emulator and runtime inputs used by the probes."""
from pathlib import Path
import hashlib
import json
import sys
import tarfile

ROOT = Path(__file__).resolve().parent
VOLUME = Path('/private/tmp/mpk-go-installed-205-fs')
sys.dont_write_bytecode = True
sys.path.insert(0, str(ROOT))
from run_scoped import seal, write


def main():
    out = ROOT / 'qemu-retained-environment'
    out.mkdir(exist_ok=False)
    names = ['qemu-user/qemu-x86_64-static', 'qemu-backports/qemu-x86_64', 'qemu-probe/probe', 'qemu-amd64-runtime/lib64/ld-linux-x86-64.so.2', 'qemu-amd64-runtime/lib/x86_64-linux-gnu/libc.so.6', 'qemu-amd64-runtime/lib/x86_64-linux-gnu/libgcc_s.so.1']
    before = {name: seal(VOLUME / name) for name in names}
    archive = out / 'inputs.tar.gz'
    with tarfile.open(archive, 'x:gz', dereference=True) as target:
        for name in names:
            target.add(VOLUME / name, arcname=name, recursive=False)
    with tarfile.open(archive) as target:
        members = target.getmembers()
        assert {member.name for member in members} == set(names)
        for member in members:
            raw = target.extractfile(member).read()
            expected = before[member.name]
            assert member.isfile() and member.mode == expected['mode'] and len(raw) == expected['bytes'] and hashlib.sha256(raw).hexdigest() == expected['sha256']
    unchanged = before == {name: seal(VOLUME / name) for name in names}
    report = {'status': 'passed' if unchanged else 'failed', 'scope': 'Retain exact emulator, probe and pinned-image runtime bytes in addition to the original program and bundle evidence. These are local test environment inputs, not product binaries or native Linux release acceptance.', 'inputs': before, 'archive': seal(archive), 'driver': seal(Path(__file__).resolve()), 'inputs_unchanged': unchanged, 'members_checked': len(names), 'native_Linux_release_acceptance': False, 'full_T_gate': 'Deferred to final T02-W04: ./scripts/check-fast.sh'}
    write(out / 'result.json', report)
    print(json.dumps({'status': report['status'], 'members_checked': len(names), 'inputs_unchanged': unchanged, 'archive_bytes': report['archive']['bytes']}))
    raise SystemExit(not unchanged)


if __name__ == '__main__':
    main()
