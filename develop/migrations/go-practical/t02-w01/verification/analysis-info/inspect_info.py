"""Audit native module-info source bytes, snapshot controls and test receipts."""
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
    return {'path':str(path.resolve()),'bytes':len(raw),'sha256':hashlib.sha256(raw).hexdigest()}


def main():
    native=ROOT/'native'; packet=json.loads((native/'native.json').read_bytes())
    previous=json.loads((ROOT.parent/'analysis-namespace/native/native.json').read_bytes())
    originals={r['object']['id']:r for r in previous['original_sources']}
    assert len(originals)==packet['original_sources']==213 and packet['vendor_files']==133
    assert packet['root_or_vcs_authority_claimed'] is False
    rows=packet['observations']; assert len(rows)==36 and len({r['name'] for r in rows})==36
    raw_files=set()
    for row in rows:
        rec=row['source']; obj=rec['object']; data=(native/'observed-bytes'/rec['raw_file']).read_bytes()
        assert obj['kind']=='metadata' and rec['raw_file']==obj['raw_sha256']+'.raw'
        assert len(data)==obj['bytes'] and hashlib.sha256(data).hexdigest()==obj['raw_sha256']
        identity={k:v for k,v in obj.items() if k!='id'}
        assert hashlib.sha256(b'MPK-GO-SOURCE-OBJECT-1.0\0'+prior.canonical(identity)).hexdigest()==obj['id']
        if row['name'].startswith('original-'):
            assert obj['id'] in originals and originals[obj['id']]['object']==obj
            assert data==(ROOT.parent/'analysis-namespace/native/original-bytes'/rec['raw_file']).read_bytes()
        else:
            assert obj['capability']=='analysis-archives' and obj['path']=='golang.org/x/mod/@v/v0.27.0.info'
        raw_files.add(rec['raw_file'])
    assert raw_files=={p.name for p in (native/'observed-bytes').glob('*.raw')}
    observed={r['name']:r['accepted'] for r in rows}
    assert observed['string-at-limit'] and not observed['string-one-over'] and observed['depth-at-limit'] and not observed['depth-one-over']
    assert observed['time-single-hour'] and observed['time-comma-fraction'] and observed['time-zone-24-60'] and not observed['time-escaped-z']
    assert observed['unicode-fold-version'] and not observed['unicode-fold-origin-field']
    snapshots={}
    for case in packet['snapshot_controls']:
        raw=bytes.fromhex(case['snapshot_hex']); wire=json.loads(raw)
        assert prior.canonical(wire)==raw and prior.native_digest(wire)==wire['snapshot_sha256']
        body={k:v for k,v in wire.items() if k!='content_sha256'}
        assert hashlib.sha256(b'MPK-PRIVATE-GO-SOURCE-SNAPSHOT-INPUTS-1.0\0'+prior.canonical(body)).hexdigest()==wire['content_sha256']
        assert [obj['id'] for obj in wire['objects']]==case['source_ids']
        snapshots[case['name']]=(case,wire)
    assert {n:r['accepted'] for n,(r,_) in snapshots.items()}=={'original-owner':True,'valid-rewrite':True,'wrong-version':False,'escaped-time':False,'unread-info':False}
    base=snapshots['original-owner'][1]; ids={o['id'] for o in base['objects']}; assert ids==set(originals)
    old=next(o for o in base['objects'] if o['capability']=='analysis-archives' and o['path']=='golang.org/x/mod/@v/v0.27.0.info')
    for name,(case,wire) in snapshots.items():
        assert wire['capabilities']==base['capabilities'] and wire['directories']==base['directories']
        for obj in wire['objects']:
            if obj['id'] in originals: assert obj==originals[obj['id']]['object']
        current={obj['id'] for obj in wire['objects']}; changed=case['changed']
        if name=='original-owner': assert current==ids and changed is None
        elif name=='unread-info': assert current==ids-{old['id']} and changed is None
        else:
            assert changed['object']['path']==old['path'] and changed['object']['capability']==old['capability']
            assert current==(ids-{old['id']})|{changed['object']['id']}
            assert changed['raw_file'] in raw_files
    for name in ['prelaunch','native','go-format-write','go-checks','info-check','info-tests','fold-tests','info-clippy','rust-format']:
        result=json.loads((ROOT/name/'result.json').read_bytes()); assert result['status']=='passed' and result.get('consumed_inputs_unchanged',True)
    output=(ROOT/'info-tests/stdout.log').read_text()
    assert output.count('test result: ok. 2 passed;')==3 and 'test result: ok. 1 passed;' in output
    assert 'test result: ok. 1 passed;' in (ROOT/'fold-tests/stdout.log').read_text()
    result={'status':'passed','driver':seal(Path(__file__).resolve()),'native':seal(native/'native.json'),'go_result':seal(native/'result.json'),'rust_result':seal(ROOT/'info-tests/result.json'),'original_sources':213,'vendor_files':133,'observations':observed,'observations_count':len(rows),'accepted':sum(observed.values()),'rejected':sum(not v for v in observed.values()),'snapshot_controls':{n:r['accepted'] for n,(r,_) in snapshots.items()},'all_snapshot_and_metadata_identities_recomputed':True,'original_info_bytes_match_previous_capture':True,'unchanged_snapshot_members_preserved':True,'original_root_or_vcs_authentication':False,'source_to_binary_proof_claimed':False,'native_Linux_release_acceptance':False,'whole_repository_gate_run':False,'full_T_gate':'Deferred to final T02-W04: ./scripts/check-fast.sh'}
    out=ROOT/'inspection.json'; assert not out.exists(); out.write_text(json.dumps(result,indent=2)+'\n')
    print(json.dumps({k:result[k] for k in ['status','original_sources','observations_count','accepted','rejected']}))


if __name__=='__main__':
    main()
