"""Inspect every actual original pattern/resource input and retained proof byte."""
from pathlib import Path
import base64,hashlib,json,re,tarfile
ROOT=Path(__file__).resolve().parent
INPUTS={}
def seal(p):
    p=Path(p);raw=p.read_bytes();return {'path':str(p),'bytes':len(raw),'sha256':hashlib.sha256(raw).hexdigest(),'mode':p.stat().st_mode&0o777}
def read(p):
    INPUTS[str(p)]=seal(p);return json.loads(p.read_bytes())
def raw(p):
    INPUTS[str(p)]=seal(p);return p.read_bytes()
def verify_source(row,data):
    obj=row['object'];assert len(data)==obj['bytes'] and hashlib.sha256(data).hexdigest()==obj['raw_sha256']
def main():
    output=ROOT/'inspection.json';assert not output.exists()
    results={p.parent.name:read(p) for p in sorted(ROOT.glob('*/result.json'))};assert len(results)==22
    assert all(r['status']=='passed' and (r['consumed_inputs_unchanged'] or r['expected_formatter_mutation']) for r in results.values())
    assert (ROOT/'go-format-check/stdout.log').stat().st_size==0 and (ROOT/'go-format-check-groups/stdout.log').stat().st_size==0
    native=ROOT/'native/native-originals';observed=ROOT/'rust-original/observations/embedded'
    corpus=read(native/'matrix/complete-embedded-candidates.json');old_path=ROOT.parent/'embedded-inputs/native/native-originals/matrix/embedded-source-corpus.json';old_raw=raw(old_path);old=json.loads(old_raw)
    assert bytes.fromhex(corpus['original_corpus_raw_hex'])==old_raw
    cases=corpus['cases'];assert len(cases)==54 and sum(r['accepted'] for r in cases)==27
    comparison=read(observed/'original-matrix-comparison.json')['cases'];assert len(comparison)==54
    whole_bytes=0;objects=0;resources=0
    for index,(case,previous,checked) in enumerate(zip(cases,old['cases'],comparison)):
        assert case['case']==previous['case']==checked['case'] and case['accepted']==previous['accepted']==checked['accepted']
        assert case['original_files']==previous['whole_original_files'] and case['links']==previous['links']
        original={name:base64.b64decode(value,validate=True) for name,value in case['original_files'].items()};whole_bytes+=sum(map(len,original.values()))
        sources=case['snapshot']['snapshot_sources'];assert {s['object']['path'] for s in sources}==set(original)
        for row in sources:verify_source(row,bytes.fromhex(row['raw_hex']));assert bytes.fromhex(row['raw_hex'])==original[row['object']['path']]
        objects+=len(sources);assert checked['complete_original_objects']==len(sources)
        assert checked['source_pattern_hex']==case['native_patterns_hex']
        before_assertion=read(native/f'matrix/original-native-case-{index:02d}.json')
        assert before_assertion=={k:v for k,v in case.items() if k!='native_selected_files'}
        stdout=base64.b64decode(case['native_stdout'],validate=True);stderr=base64.b64decode(case['native_stderr'],validate=True)
        if case['accepted']:
            command=json.loads(stdout);assert command['EmbedFiles']==case['native_selected_files']==checked['result']['files'];resources+=len(checked['result']['objects'])
            assert checked['result']['objects']==previous['objects']
            for pattern,source_list in checked['result']['patterns'].items():assert [o['path'] for o in source_list]==previous['pattern_files'][pattern]
        else:assert stderr and 'rejected' in checked['result']
    assert corpus['original_roots_removed'] and not corpus['caller_source_adapter_or_fallback']
    pragmas=read(native/'headers/original-embedded-directives.json')['cases'];pragma_results=read(observed/'original-pragma-comparison.json')['cases'];assert len(pragmas)==len(pragma_results)==54
    for case,checked in zip(pragmas,pragma_results):
        assert case['case']==checked['case'];source=bytes.fromhex(case['original_source_hex']);assert len(case['snapshot_sources'])==1;verify_source(case['snapshot_sources'][0],source)
        assert bytes.fromhex(case['snapshot_sources'][0]['raw_hex'])==source
        if case['native_observation']['diagnostic']:assert 'rejected' in checked
        else:assert case['native_patterns_hex']==checked['pattern_hex']
    headers_by_name={c['case']:c for c in pragmas}
    # The three original body cases distinguish arbitrary escaped bytes from
    # literal invalid UTF-8, including incomplete sequences, with no source edit.
    assert headers_by_name['source-11']['native_patterns_hex']==['6f726967696e616cefbfbd62797465']
    assert headers_by_name['source-12']['native_patterns_hex']==['6c69746572616cefbfbdefbfbd']
    assert headers_by_name['source-13']['native_patterns_hex']==['ff2d6c69746572616cefbfbd']
    glob=read(native/'glob/original-pinned-globs.json');matched=read(ROOT/'glob-original/observations/embedded/native-glob-comparison.json');assert glob['cases']==matched['cases']
    assert glob['all_original_pinned_cases']==56 and glob['additional_actual_raw_byte_cases']==17 and len(glob['cases'])==73
    sdk_saved=ROOT/'native/pinned-sdk-match-test.original';assert bytes.fromhex(glob['complete_original_test_source_hex'])==raw(sdk_saved)
    external=read(ROOT/'native/inputs.json')['complete_external_originals'];assert len(external)==1 and external[0]['saved']==seal(sdk_saved)
    assert external[0]['original']==seal(Path(external[0]['original']['path'])) and results['native']['external_originals_unchanged']
    rows={bytes.fromhex(r['pattern_hex']):r for r in glob['cases'] if isinstance(r['case'],str)}
    assert not rows[b'*?*?']['matched'] and not rows['*?*[\ufffd]'.encode()]['matched']
    whole=read(native/'originals/whole-original-embedded-patterns.json')['units'];replayed=read(observed/'whole-original-pattern-comparison.json')['units'];assert len(whole)==len(replayed)==3
    directory_counts=[];pattern_counts=[0,0,0]
    for unit,actual in zip(whole,replayed):
        assert unit['unit']==actual['unit'] and unit['complete_original_objects']==actual['original_objects'] and len(unit['packages'])==len(actual['packages'])
        directory_counts.append(len(actual['packages']))
        for wanted,checked in zip(unit['packages'],actual['packages']):
            assert (wanted['capability'],wanted['directory'])==(checked['capability'],checked['directory'])
            if wanted['diagnostic']:assert 'rejected' in checked;continue
            for i,(native_key,key) in enumerate([('native_patterns_hex','patterns_hex'),('native_test_patterns_hex','test_patterns_hex'),('native_external_test_patterns_hex','external_test_patterns_hex')]):
                assert wanted[native_key]==checked[key];pattern_counts[i]+=len(checked[key])
    assert directory_counts==[668,668,667]
    directory=ROOT.parent/'embedded-inputs/native/native-originals/owned';packet=read(directory/'complete-embedded-capture.json');old_resources=read(directory/'original-embedded-resources.json');owner=read(observed/'whole-original-owned-resources.json')
    assert owner['original_objects']==len(packet['snapshot_sources'])==5535 and owner['original_selected_contract_functions']==1
    assert owner['resources']==old_resources['selected'] and owner['files']==old_resources['files'] and owner['independent_original_source_pattern_and_resource_selection'] and not owner['source_or_context_program_proof']
    total=0;kinds={}
    for row in packet['snapshot_sources']:
        data=raw(directory/'original-source-objects'/row['raw_file']);verify_source(row,data);total+=len(data);kind=row['object']['kind'];kinds[kind]=kinds.get(kind,0)+1
    assert total==107062681
    controls=read(observed/'coherent-original-resource-omissions.json');assert controls['original_objects']==5535 and controls['same_original_selected_contract_functions']==1 and len(controls['cases'])==2
    assert {r['kind'] for r in controls['cases']}=={'EmbedSource','EmbedNamespace'}
    original_wire=json.loads(bytes.fromhex(packet['snapshot_hex']));original_capture=json.loads(bytes.fromhex(packet['capture_hex']))
    for row in controls['cases']:
        wire=json.loads(bytes.fromhex(row['changed_snapshot_hex']));capture=json.loads(bytes.fromhex(row['changed_capture_hex']));assert row['same_original_source_selection_and_contract']
        assert capture['selection_inputs_sha256']==original_capture['selection_inputs_sha256'] and capture['snapshot_inputs_sha256']==wire['content_sha256']
        omitted=row['omitted_original_object'];assert wire['objects']==[o for o in original_wire['objects'] if o['id']!=omitted['id']]
        assert wire['capabilities']==original_wire['capabilities']
        if row['case']=='buffer':assert wire['directories']==original_wire['directories']
        else:assert wire['directories']==[d for d in original_wire['directories'] if not (d['capability']==omitted['capability'] and d['path']=='assets/empty')]
    groups=read(ROOT/'native-groups/native-originals/groups/original-test-pattern-groups.json')
    group_comparison=read(ROOT/'rust-groups/observations/embedded/original-test-pattern-groups.json')
    assert len(groups['cases'])==len(group_comparison['cases'])==3 and groups['original_roots_removed'] and group_comparison['original_test_patterns_retained_without_production_resource_reads']
    group_counts=[0,0]
    for case,checked in zip(groups['cases'],group_comparison['cases']):
        assert case['case']==checked['case'] and case['accepted']
        files={name:base64.b64decode(value,validate=True) for name,value in case['original_files'].items()}
        sources=case['snapshot']['snapshot_sources'];assert len(sources)==len(files)==checked['original_objects']
        for row in sources:verify_source(row,bytes.fromhex(row['raw_hex']));assert bytes.fromhex(row['raw_hex'])==files[row['object']['path']]
        assert json.loads(base64.b64decode(case['native_stdout'],validate=True))['EmbedFiles']==case['native_selected_files']==checked['production_files']==['assets/prod.txt']
        for native_key,key in [('native_patterns_hex','production_pattern_hex'),('native_test_patterns_hex','internal_test_pattern_hex'),('native_external_test_patterns_hex','external_test_pattern_hex')]:assert case[native_key]==checked[key]
        group_counts[0]+=len(checked['internal_test_pattern_hex']);group_counts[1]+=len(checked['external_test_pattern_hex'])
        retained=read(ROOT/f"native-groups/native-originals/groups/original-group-{case['case']:02d}.json")
        assert retained=={k:v for k,v in case.items() if k!='native_selected_files'}
    assert group_counts==[2,4]
    source_correspondence=[]
    for epoch,name in [('rust-original','crates/mpk-vc/tests/go_practical_embedded_selection.rs'),('native','go-tools/go2vir/practical_embedded_selection_test.go')]:
        receipt=read(ROOT/epoch/'inputs.json');archive=Path(receipt['complete_archive']['path']);INPUTS[str(archive)]=seal(archive)
        with tarfile.open(archive,'r:gz') as packed:before=packed.extractfile(name).read()
        current=Path.cwd()/name;after=raw(current);assert after.startswith(before)
        source_correspondence.append({'epoch':epoch,'source':str(current),'complete_previous_source_prefix_bytes':len(before),'previous_source_sha256':hashlib.sha256(before).hexdigest(),'only_new_test_function_appended':True})
    receipt=read(ROOT/'rust-original/inputs.json');production=[]
    for row in receipt['sources']:
        name=Path(row['path'])
        if '/crates/mpk-vc/src/go_practical_type_keys' in str(name) and (name.name=='go_practical_type_keys.rs' or 'source_package' in name.parts):
            assert row==seal(name);production.append(row)
    assert len(production)>=6
    witnesses=[];sdk=Path('/private/tmp/mpk-go125-20261025-download/go')
    for name in ['src/go/build/read.go','src/path/match.go','src/strconv/quote.go','src/cmd/vendor/golang.org/x/mod/module/module.go']:
        source=next(s for s in packet['snapshot_sources'] if s['object']['path']==name);data=raw(directory/'original-source-objects'/source['raw_file']);assert data==(sdk/name).read_bytes()
        witnesses.append({'path':name,'object':source['object'],'raw_hex':data.hex()})
    proofs=read(ROOT/'rust-original/observations/sources/original-proof-replay.json')['complete_original_call_proofs'];assert len(proofs)==32
    for row in proofs:
        data=raw(Path(row['original_certificate']));assert len(data)==row['bytes'] and hashlib.sha256(data).hexdigest()==row['raw_sha256']
    units=read(ROOT/'rust-original/observations/snapshots/all-original-capture-units.json')['units'];assert len(units)==2 and all(r['original_roots']==43 and r['all_source_components_and_bytes_equal'] for r in units)
    stdout=raw(ROOT/'rust-original/stdout.log').decode();counts=[int(n) for n in re.findall(r'test result: ok\. (\d+) passed; 0 failed; 0 ignored;',stdout)];assert counts==[3,2,5,2,9,4]
    events=[json.loads(l) for l in raw(ROOT/'native/stdout.log').decode().splitlines()];roots=[r['Test'] for r in events if r.get('Action')=='pass' and r.get('Test') and '/' not in r['Test']];assert len(roots)==4 and not any(r['Action']=='fail' for r in events)
    group_stdout=raw(ROOT/'rust-groups/stdout.log').decode();assert re.findall(r'test result: ok\. (\d+) passed; 0 failed; 0 ignored;',group_stdout)==['1']
    group_events=[json.loads(l) for l in raw(ROOT/'native-groups/stdout.log').decode().splitlines()];group_roots=[r['Test'] for r in group_events if r.get('Action')=='pass' and r.get('Test')]
    assert len(group_roots)==1 and not any(r['Action']=='fail' for r in group_events)
    result={'status':'passed','driver':seal(Path(__file__).resolve()),'inputs':list(INPUTS.values()),'source_epochs':22,'original_native_resource_cases':54,'native_acceptances':27,'native_rejections':27,'all_case_original_source_bytes':whole_bytes,'all_case_original_objects':objects,'original_selected_resource_identities':resources,'original_pragma_cases':54,'original_pragma_native_rejections':sum(bool(c['native_observation']['diagnostic']) for c in pragmas),'pinned_native_glob_cases':56,'additional_native_byte_glob_cases':17,'all_native_glob_cases':73,'complete_original_package_directories':directory_counts,'original_pattern_counts_production_test_external':pattern_counts,'complete_original_owner_objects':5535,'complete_original_owner_bytes':total,'complete_original_owner_source_kinds':kinds,'original_application_resources':4,'coherent_original_resource_controls':controls,'complete_pinned_primary_source_witnesses':witnesses,'complete_pinned_original_match_tests':seal(sdk_saved),'affected_rust_integration_roots':sum(counts)+1,'nonempty_internal_external_test_pattern_cases':3,'original_group_test_pattern_counts':group_counts,'complete_earlier_test_source_prefixes':source_correspondence,'production_source_correspondence':production,'affected_rust_glob_unit_roots':1,'actual_native_roots':roots+group_roots,'original_roots_per_unit':43,'original_proof_byte_streams':32,'original_error_case_universes':sorted({r['error_cases'] for r in proofs}),'remaining':'Canonical module/compiler/export/capture/context/descriptor/registry and launcher/root authority, required module-link target/stat observations, compiler pragma placement/variable validation, full source/IR/state/storage/frame/callee/program proofs, all56 foundation families/all169 demands, native Linux gates and remaining T02-T11 remain due.','full_T_gate':'Deferred to final T02-W04: ./scripts/check-fast.sh'}
    output.write_text(json.dumps(result,indent=2)+'\n');print(json.dumps({k:result[k] for k in ['status','source_epochs','original_native_resource_cases','original_pragma_cases','all_native_glob_cases','complete_original_package_directories','affected_rust_integration_roots','affected_rust_glob_unit_roots']}))
if __name__=='__main__':main()
