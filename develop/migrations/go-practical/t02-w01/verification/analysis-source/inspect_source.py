"""Audit original analysis source bytes and exact source/namespace controls."""
from pathlib import Path
import hashlib
import importlib.util
import json
import sys

ROOT=Path(__file__).resolve().parent
sys.dont_write_bytecode=True
spec=importlib.util.spec_from_file_location('prior_snapshot_inspection',ROOT.parent/'analysis-namespace/inspect_snapshot.py');prior=importlib.util.module_from_spec(spec);spec.loader.exec_module(prior)


def seal(path):
    data=path.read_bytes();return {'path':str(path.resolve()),'bytes':len(data),'sha256':hashlib.sha256(data).hexdigest()}


def main():
    native=ROOT/'native';packet=json.loads((native/'native.json').read_bytes())
    old=json.loads((ROOT.parent/'analysis-namespace/native/native.json').read_bytes())
    originals={r['object']['id']:r for r in old['original_sources']}
    assert len(originals)==packet['original_sources']==213 and packet['frontend_go_files']==65 and packet['vendor_files']==133
    assert packet['root_or_binary_authority_claimed'] is False
    frontend=[r for r in originals.values() if r['object']['capability']=='analysis-frontend' and r['object']['kind']=='go_file' and '/' not in r['object']['path']]
    vendor=[r for r in originals.values() if r['object']['capability']=='analysis-frontend' and r['object']['kind']=='go_file' and r['object']['path'].startswith('vendor/')]
    assert len(frontend)==65 and len(vendor)==126
    repository=ROOT.parents[5]
    for row in frontend+vendor:
        original=(ROOT.parent/'analysis-namespace/native/original-bytes'/row['raw_file']).read_bytes()
        assert (repository/'go-tools/go2vir'/row['object']['path']).read_bytes()==original
    source=next(r for r in frontend if r['object']['path']=='practical_analysis.go');original_raw=(ROOT.parent/'analysis-namespace/native/original-bytes'/source['raw_file']).read_bytes()
    additions={'embed-argument':'\n//go:embed asset.txt\n','embed-without-argument':'\n//go:embed\n','embed-prefix':'\n//go:embedding\n','embed-stripped-carriage-return':'\n//go:\rembed asset.txt\n','embed-after-code':'\nvar mpkEmbedProbe = 1 //go:embed x\n','embed-in-quoted-string':'\nconst mpkEmbedProbe = "//go:embed x"\n','embed-in-raw-string':'\nconst mpkEmbedProbe = `//go:embed x`\n','embed-in-block-comment':'\n/* first\n//go:embed x\n*/\n','embed-after-other-comment-text':'\n// text //go:embed x\n','spaced-comment':'\n// go:embed x\n','invalid-syntax':'\nfunc ( }\n','unclosed-comment':'\n/* unterminated'}
    entries={'unread-test-go':(b'missing_test.go',0),'unknown-asset':(b'unread.data',0),'unmodeled-assembly':(b'unread.S',0),'symlink-frontend':(b'ghost.go',1<<27),'nonutf8-unrelated':(b'\xff.data',0),'nonutf8-go':(b'\xff.go',0)}
    cases={}
    for row in packet['cases']:
        raw=bytes.fromhex(row['snapshot_hex']);wire=json.loads(raw)
        assert prior.canonical(wire)==raw and prior.native_digest(wire)==wire['snapshot_sha256']
        body={k:v for k,v in wire.items() if k!='content_sha256'}
        assert hashlib.sha256(b'MPK-PRIVATE-GO-SOURCE-SNAPSHOT-INPUTS-1.0\0'+prior.canonical(body)).hexdigest()==wire['content_sha256']
        assert [o['id'] for o in wire['objects']]==row['source_ids'] and row['name'] not in cases
        cases[row['name']]=(row,wire)
    assert len(cases)==20
    baseline=cases['original'][1];base_ids={o['id'] for o in baseline['objects']};assert base_ids==set(originals)
    def dirs(wire):return {(d['capability'],d['path']):[(e['name_hex'],e['type_bits']) for e in d['entries']] for d in wire['directories']}
    base_dirs=dirs(baseline);root=('analysis-frontend','');raw_files=set()
    for name,(row,wire) in cases.items():
        assert wire['capabilities']==baseline['capabilities']
        current_dirs=dirs(wire)
        if name in entries:
            assert {k:v for k,v in current_dirs.items() if k!=root}=={k:v for k,v in base_dirs.items() if k!=root}
            entry,mode=entries[name];assert set(current_dirs[root])-set(base_dirs[root])=={(entry.hex(),mode)} and not set(base_dirs[root])-set(current_dirs[root])
        else:assert current_dirs==base_dirs
        ids={o['id'] for o in wire['objects']}
        for obj in wire['objects']:
            if obj['id'] in originals:assert obj==originals[obj['id']]['object']
        if name=='unread-frontend':assert ids==base_ids-{source['object']['id']} and row['changed'] is None
        elif name in additions:
            change=row['changed'];obj=change['object'];raw=(native/'changed-bytes'/change['raw_file']).read_bytes();assert raw==original_raw+additions[name].encode()
            assert len(raw)==obj['bytes'] and hashlib.sha256(raw).hexdigest()==obj['raw_sha256'] and change['raw_file']==obj['raw_sha256']+'.raw'
            identity={k:v for k,v in obj.items() if k!='id'};assert hashlib.sha256(b'MPK-GO-SOURCE-OBJECT-1.0\0'+prior.canonical(identity)).hexdigest()==obj['id']
            assert ids==(base_ids-{source['object']['id']})|{obj['id']};raw_files.add(change['raw_file'])
        else:assert ids==base_ids and row['changed'] is None
        if row['accepted']:assert row['sources']==191
    assert raw_files=={p.name for p in (native/'changed-bytes').glob('*.raw')}
    for name in ['prelaunch','native','go-format-write','go-checks','source-check','source-tests','parse-tests','lex-tests','source-clippy','rust-format']:
        result=json.loads((ROOT/name/'result.json').read_bytes());assert result['status']=='passed' and result.get('consumed_inputs_unchanged',True)
    for name in ['source-tests','parse-tests','lex-tests']:
        assert 'test result: ok. 1 passed;' in (ROOT/name/'stdout.log').read_text()
    status={n:r['accepted'] for n,(r,_) in cases.items()}
    result={'status':'passed','driver':seal(Path(__file__).resolve()),'native':seal(native/'native.json'),'go_result':seal(native/'result.json'),'rust_result':seal(ROOT/'source-tests/result.json'),'original_sources':213,'frontend_go_files':65,'vendor_go_files':126,'observations':status,'observations_count':len(status),'accepted':sum(status.values()),'rejected':sum(not v for v in status.values()),'all_snapshots_and_changed_source_identities_recomputed':True,'all_other_original_sources_and_namespace_entries_preserved':True,'original_root_or_binary_authenticated':False,'exhaustive_go_parser_equivalence_proved':False,'native_Linux_release_acceptance':False,'whole_repository_gate_run':False,'full_T_gate':'Deferred to final T02-W04: ./scripts/check-fast.sh'}
    out=ROOT/'inspection.json';assert not out.exists();out.write_text(json.dumps(result,indent=2)+'\n')
    print(json.dumps({k:result[k] for k in ['status','original_sources','frontend_go_files','vendor_go_files','observations_count','accepted','rejected']}))


if __name__=='__main__':main()
