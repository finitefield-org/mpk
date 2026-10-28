"""Audit full original analysis sources and the module/checksum controls."""
from pathlib import Path
import hashlib
import importlib.util
import json
import sys

ROOT = Path(__file__).resolve().parent
sys.dont_write_bytecode = True
spec = importlib.util.spec_from_file_location('prior_snapshot_inspection', ROOT.parent / 'analysis-namespace/inspect_snapshot.py')
prior = importlib.util.module_from_spec(spec)
spec.loader.exec_module(prior)


def seal(path):
    raw = path.read_bytes()
    return {'path': str(path.resolve()), 'bytes': len(raw), 'sha256': hashlib.sha256(raw).hexdigest()}


def main():
    native = ROOT / 'native'
    packet = json.loads((native / 'native.json').read_bytes())
    previous = json.loads((ROOT.parent / 'analysis-namespace/native/native.json').read_bytes())
    original_sources = {row['object']['id']: row for row in previous['original_sources']}
    assert len(original_sources) == packet['original_sources'] == 213 and packet['vendor_files'] == 133
    assert packet['info_or_root_authority_claimed'] is False
    expected = {'original': True, 'frontend-comment': True, 'wrong-module': False, 'wrong-go-version': False, 'wrong-dependency-version': False, 'wrong-indirect': False, 'duplicate-requirement': False, 'replacement': False, 'exclusion': False, 'retraction': False, 'toolchain': False, 'wrong-archive-sum': False, 'wrong-mod-sum': False, 'duplicate-identical-sum': True, 'conflicting-sum': False, 'unread-sums': False, 'changed-cached-mod': False, 'unread-cached-mod': False, 'wrong-ziphash': False, 'ziphash-trim-space': True}
    cases = {}
    for case in packet['cases']:
        raw = bytes.fromhex(case['snapshot_hex']); wire = json.loads(raw)
        assert prior.canonical(wire) == raw and prior.native_digest(wire) == wire['snapshot_sha256']
        body = {key: value for key, value in wire.items() if key != 'content_sha256'}
        assert hashlib.sha256(b'MPK-PRIVATE-GO-SOURCE-SNAPSHOT-INPUTS-1.0\0' + prior.canonical(body)).hexdigest() == wire['content_sha256']
        assert [obj['id'] for obj in wire['objects']] == case['source_ids']
        assert case['name'] not in cases
        cases[case['name']] = (case, wire)
    assert {name:case['accepted'] for name,(case,_) in cases.items()} == expected
    baseline = cases['original'][1]; original_ids = {obj['id'] for obj in baseline['objects']}
    assert original_ids == set(original_sources)
    original_by_path = {(row['object']['capability'], row['object']['path']): row for row in original_sources.values()}
    changed_files = set()
    for name,(case,wire) in cases.items():
        assert wire['capabilities'] == baseline['capabilities'] and wire['directories'] == baseline['directories']
        for obj in wire['objects']:
            if obj['id'] in original_sources:
                assert obj == original_sources[obj['id']]['object']
        ids = {obj['id'] for obj in wire['objects']}; changed = case['changed']
        if changed is None:
            if name == 'original':
                assert ids == original_ids
                continue
            key = ('analysis-frontend', 'go.sum') if name == 'unread-sums' else ('analysis-archives', 'golang.org/x/mod/@v/v0.27.0.mod')
            assert name in ['unread-sums','unread-cached-mod'] and ids == original_ids - {original_by_path[key]['object']['id']}
            continue
        obj = changed['object']; data = (native / 'changed-bytes' / changed['raw_file']).read_bytes()
        assert changed['raw_file'] == obj['raw_sha256'] + '.raw' and len(data) == obj['bytes'] and hashlib.sha256(data).hexdigest() == obj['raw_sha256']
        identity = {key: value for key, value in obj.items() if key != 'id'}
        assert hashlib.sha256(b'MPK-GO-SOURCE-OBJECT-1.0\0' + prior.canonical(identity)).hexdigest() == obj['id']
        old = original_by_path[(obj['capability'], obj['path'])]['object']
        assert obj['kind'] == old['kind'] and ids == (original_ids - {old['id']}) | {obj['id']}
        assert next(item for item in wire['objects'] if item['id'] == obj['id']) == obj
        changed_files.add(changed['raw_file'])
    assert changed_files == {p.name for p in (native / 'changed-bytes').glob('*.raw')}
    for name in ['prelaunch-restored', 'sdk-restore', 'native', 'go-format-write', 'go-checks', 'module-check', 'module-tests', 'manifest-tests', 'sums-tests', 'module-clippy', 'rust-format']:
        result = json.loads((ROOT / name / 'result.json').read_bytes())
        assert result['status'] == 'passed' and result.get('consumed_inputs_unchanged', True)
    output = (ROOT / 'module-tests/stdout.log').read_text()
    assert output.count('test result: ok. 2 passed;') == 2 and 'test result: ok. 1 passed;' in output
    for name in ['manifest-tests','sums-tests']:
        assert 'test result: ok. 1 passed;' in (ROOT / name / 'stdout.log').read_text()
    summary = {'status':'passed','driver':seal(Path(__file__).resolve()),'native':seal(native/'native.json'),'go_result':seal(native/'result.json'),'rust_result':seal(ROOT/'module-tests/result.json'),'original_sources':213,'vendor_files':133,'archive_count':3,'controls':expected,'accepted':sum(expected.values()),'rejected':sum(not value for value in expected.values()),'snapshot_and_changed_source_digests_independently_recomputed':True,'all_other_original_sources_and_namespace_entries_preserved':True,'cached_info_or_original_root_authenticated':False,'source_to_binary_proof_claimed':False,'native_Linux_release_acceptance':False,'whole_repository_gate_run':False,'full_T_gate':'Deferred to final T02-W04: ./scripts/check-fast.sh'}
    out=ROOT/'inspection.json'; assert not out.exists(); out.write_text(json.dumps(summary,indent=2)+'\n')
    print(json.dumps({'status':summary['status'],'original_sources':213,'controls':len(expected),'accepted':summary['accepted'],'rejected':summary['rejected']}))


if __name__ == '__main__':
    main()
