"""Independently compare complete original captures and actual build outputs."""
from pathlib import Path
import hashlib
import json

ROOT = Path(__file__).resolve().parent
REPO = ROOT.parents[5]


def sha(raw):
    return hashlib.sha256(raw).hexdigest()


def load(name):
    root = ROOT/name
    packet = json.loads((root/'native.json').read_bytes())
    rows = {}
    for row in packet['original_sources']:
        obj = row['object']
        raw = (root/'original-bytes'/row['raw_file']).read_bytes()
        assert len(raw) == obj['bytes'] and sha(raw) == obj['raw_sha256']
        assert obj['id'] not in rows
        rows[obj['id']] = (obj, raw)
    assert set(rows) == set(packet['source_ids'])
    snapshot = json.loads(bytes.fromhex(packet['snapshot_hex']))
    entries = next(d['entries'] for d in snapshot['directories'] if d['capability'] == 'analysis-frontend' and d['path'] == '')
    profile = [e['type_bits'] for e in entries if bytes.fromhex(e['name_hex']) == b'default.pgo']
    return packet, rows, profile


def main():
    registered = ROOT.parent/'registered-analysis/native-fixed'
    prior = json.loads((registered/'native.json').read_bytes())
    expected = {r['object']['id']: (r['object'], (registered/'original-bytes'/r['raw_file']).read_bytes()) for r in prior['original_sources']}
    assert len(expected) == 196 and len(prior['build_objects']) == 184
    before, before_rows, before_profile = load('native-before-fix')
    auto, auto_rows, auto_profile = load('native-mode-aware')
    assert before_rows == expected and before_profile == [0] and auto_profile == [0]
    extra = set(auto_rows) - set(expected)
    assert len(extra) == 1 and set(expected).issubset(auto_rows)
    assert all(auto_rows[k] == v for k, v in expected.items())
    obj, actual_profile = auto_rows[next(iter(extra))]
    sdk_profile = (Path('/private/tmp/mpk-installed-compiler-205-original-go-output/toolchain/go/src/cmd/compile/default.pgo')).read_bytes()
    assert obj['path'] == 'default.pgo' and obj['kind'] == 'auxiliary'
    assert actual_profile == sdk_profile == (ROOT/'native-mode-aware/original-profile.raw').read_bytes() == (ROOT/'native-before-fix/original-profile.raw').read_bytes()
    assert len(actual_profile) == 293923 and sha(actual_profile) == 'd7355ec9e3c8bf05e214b202174ede4205092f852124e6f7f7287e22f6b5d980'
    old_build = {o['id']: o for o in before['build_objects']}
    new_build = {o['id']: o for o in auto['build_objects']}
    assert len(old_build) == 184 and len(new_build) == 185 and set(new_build) == set(old_build) | extra
    assert all(new_build[k] == v for k, v in old_build.items())
    controls = []
    for name, typ in [('regular', 0), ('directory', 2147483648), ('symlink', 134217728), ('oversized', 0), ('absent', None)]:
        packet, rows, profile = load('native-mode-aware/controls/'+name)
        assert rows == expected and len(packet['build_objects']) == 184
        assert {o['id']: o for o in packet['build_objects']} == old_build
        assert profile == ([] if typ is None else [typ])
        controls.append({'name': name, 'root_entry_type_bits': typ, 'source_objects': len(rows), 'build_objects': len(packet['build_objects']), 'all_original_bytes_equal': True})
    oversized = (ROOT/'native-mode-aware/controls/oversized/original-ignored-profile.raw').read_bytes()
    assert len(oversized) == 16777217 and not any(oversized)
    receipt = json.loads((ROOT/'actual-build-original/result.json').read_bytes())
    assert receipt['status'] == 'passed'
    bins = {}
    for item in receipt['builds']:
        raw = bytes.fromhex((ROOT/'actual-build-original'/f"{item['mode']}.elf.hex").read_text())
        assert len(raw) == item['binary']['bytes'] and sha(raw) == item['binary']['sha256']
        bins[item['mode']] = {'bytes': len(raw), 'sha256': sha(raw), 'pgo_lines': item['build_info_pgo_lines'], 'byte_equal_registered': item['byte_equal_registered']}
    assert bins['auto']['pgo_lines'] == ['build\t-pgo=default.pgo'] and bins['off']['pgo_lines'] == []
    assert not bins['auto']['byte_equal_registered'] and bins['off']['byte_equal_registered']
    assert 'default.pgo' in (ROOT/'native-mode-aware/plan-auto.log').read_text()
    assert 'default.pgo' not in (ROOT/'native-mode-aware/plan-off.log').read_text()
    result = {'status': 'passed', 'selection_reason': 'Independently compare every original captured object and build input across the original, PGO modes and all ignored-entry controls; verify exact registered SDK profile and both real binary bytes.', 'registered_original_objects': len(expected), 'registered_original_build_inputs': len(old_build), 'auto_objects': len(auto_rows), 'auto_build_inputs': len(new_build), 'every_original_object_and_byte_equal': True, 'every_original_build_input_equal': True, 'profile_bytes': len(actual_profile), 'profile_sha256': sha(actual_profile), 'controls': controls, 'oversized_supplemental_bytes': len(oversized), 'oversized_supplemental_sha256': sha(oversized), 'actual_binaries': bins, 'native_Linux_release_acceptance': False, 'full_T_gate': 'Deferred to final T02-W04: ./scripts/check-fast.sh'}
    output = ROOT/'inspection.json'
    assert not output.exists()
    output.write_text(json.dumps(result, indent=2)+'\n')
    print(json.dumps({'status': result['status'], 'original_objects': len(expected), 'controls': len(controls), 'actual_binaries': 2}), flush=True)


if __name__ == '__main__':
    main()
