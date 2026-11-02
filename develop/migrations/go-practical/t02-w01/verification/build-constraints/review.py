"""Unchanged-rule scoped review of every actual source epoch and original input."""
from pathlib import Path
import hashlib,json,re,subprocess,sys,tarfile,types,urllib.parse

REPO=Path.cwd();ROOT=Path(__file__).resolve().parent
BASIS='7d2fa25c3a4ed6c4f142ea9c0bcd7195bbd14541'
sys.path=[p for p in sys.path if Path(p or '.').resolve()!=ROOT]
def git(*args):return subprocess.check_output(['/usr/bin/git',*args])
def seal(path):
    raw=path.read_bytes();return {'path':str(path),'bytes':len(raw),'sha256':hashlib.sha256(raw).hexdigest(),'mode':path.stat().st_mode&0o777}
def logical(path):
    p=Path(path);return str(p.relative_to(REPO)) if p.is_absolute() else str(p)
def main():
    assert git('rev-parse','HEAD').decode().strip()==BASIS
    output=ROOT/'review-final.json';assert not output.exists()
    scanner=REPO/'scripts/check-no-active-gir.sh'
    code=scanner.read_text().split("<<'PY'\n",1)[1].rsplit('\nPY',1)[0].rsplit('raise SystemExit(main())',1)[0]
    module=types.ModuleType('build_constraints_scoped_review');module.__file__=str(scanner)
    sys.modules[module.__name__]=module;sys.argv=[str(scanner),str(REPO),'strict']
    exec(compile(code,str(scanner),'exec'),module.__dict__)
    matchers=module.load_matchers();manifest,configured,fixtures,_=module.validate_manifest()
    module.run_fixture_self_test(manifest,matchers)
    assert (len(matchers),len(module.SCOPES),len(manifest['sealed_records']),len(module.active_retained_record_exclusions(manifest)))==(127,4,39,51)
    for name in configured|fixtures:assert (REPO/name).read_bytes()==git('show',BASIS+':'+name)
    snapshot=ROOT.parent/'snapshot-inputs';old_path=snapshot/'input-review.json'
    assert old_path.read_bytes()==git('show',BASIS+':'+logical(old_path))
    old=json.loads(old_path.read_bytes());assert old['status']=='passed' and not old['new_findings'] and not old['unchanged_basis_findings']
    assert old['complete_snapshot_members_verified_and_scanned']==77162
    store=snapshot/'snapshot-storage';storage={str(p):seal(p) for p in store.glob('*')}
    for name in storage:assert Path(name).read_bytes()==git('show',BASIS+':'+logical(name))
    original={r['path']:r for r in json.loads((store/'manifest.json').read_bytes())['originals']}
    findings=[];cache={};scanned=0
    path_rules=[r for r in matchers if r['scope']=='exact-path']
    def scan(name,raw):
        nonlocal scanned
        key=(hashlib.sha256(raw).hexdigest(),tuple(module.path_matches(name,r['path']) for r in path_rules))
        if key not in cache:cache[key]=[vars(r) for r in module.detect(name,raw.decode(errors='ignore'),matchers)]
        findings.extend({**r,'path':name,'excerpt_hex':r['excerpt'].encode().hex()} for r in cache[key]);scanned+=1
    epochs=[];archives=set();raw_outputs={};original_checks=0;members=0
    for path in sorted(ROOT.glob('*/inputs.json')):
        inputs=json.loads(path.read_bytes())
        if 'complete_archive' not in inputs:
            assert path.parent.name=='standalone';continue
        result=json.loads(path.with_name('result.json').read_bytes())
        assert result['status']=='passed' and result['inputs']==seal(path)
        assert result['consumed_inputs_unchanged'] or result['expected_formatter_mutation']
        for field in ['stdout','stderr']:
            row=result[field];assert row==seal(Path(row['path']));raw_outputs[logical(row['path'])]=row
        sources={r['path']:r for r in inputs['sources']};preserved=set(inputs['preserved_original_paths'])
        assert preserved<=sources.keys()
        for name in preserved:
            row=sources[name];assert row==seal(Path(name))
            expected=storage.get(name,original.get(name));assert expected is not None
            assert all(row[k]==expected[k] for k in ['path','bytes','sha256','mode']);original_checks+=1
        archive=Path(inputs['complete_archive']['path']);assert inputs['complete_archive']==seal(archive);archives.add(archive)
        with tarfile.open(archive,'r:gz') as packed:
            assert {m.name for m in packed.getmembers()}=={logical(n) for n in sources.keys()-preserved}
            for member in packed.getmembers():
                name=str(REPO/member.name);row=sources[name];raw=packed.extractfile(member).read()
                assert member.isfile() and member.mode==row['mode']
                assert len(raw)==row['bytes'] and hashlib.sha256(raw).hexdigest()==row['sha256']
                scan(member.name,raw);members+=1
        epochs.append({'result':seal(path.with_name('result.json')),'status':result['status'],'command':result['command'],'all_inputs':len(sources),'original_inputs_retained_by_exact_prior_storage':len(preserved)})
    assert len(epochs)==9
    inspect_path=ROOT/'inspection.json';inspection=json.loads(inspect_path.read_bytes());assert inspection['status']=='passed'
    for row in inspection['inputs']:assert row==seal(Path(row['path']))
    for row in inspection['complete_source_witnesses']:
        raw=bytes.fromhex(row['raw_hex']);assert hashlib.sha256(raw).hexdigest()==row['captured_original']['raw_sha256']
        scan('go-tools/go2vir/original-pinned-sdk/'+row['captured_original']['path'],raw)
    # Newly created primitive source bytes are decoded, not hidden inside hex.
    corpus=json.loads((ROOT/'native/native-originals/probe/build-corpus.json').read_bytes())
    for row in corpus['cases']:scan('go-tools/go2vir/original-build-cases/'+row['object']['path'],bytes.fromhex(row['raw_hex']))
    # Preserve every pre-fix review input. Changed documentation/driver bytes
    # have exact saved preimages; the old standalone reader has its exact source.
    before_path=ROOT/'review-before-standalone-fix.json';before=json.loads(before_path.read_bytes())
    assert before['status']=='passed' and not before['new_findings']
    saved={};preimages=ROOT/'post-review-preimages.tar.gz';archives.add(preimages)
    with tarfile.open(preimages,'r:gz') as packed:
        for member in packed.getmembers():
            raw=packed.extractfile(member).read();saved[member.name]=(raw,member.mode);scan(member.name,raw)
    old_test=ROOT/'old-standalone-consumer.rs';old_test_raw=old_test.read_bytes()
    for row in before['current_inputs']:
        name=logical(row['path'])
        if name=='crates/mpk-vc/tests/go_practical_source_build.rs':raw=old_test_raw;mode=old_test.stat().st_mode&0o777
        elif name in saved:raw,mode=saved[name]
        else:raw=Path(row['path']).read_bytes();mode=Path(row['path']).stat().st_mode&0o777
        assert len(raw)==row['bytes'] and hashlib.sha256(raw).hexdigest()==row['sha256'] and mode==row['mode']
    standalone=ROOT/'standalone';common=json.loads((standalone/'inputs.json').read_bytes());result=json.loads((standalone/'result.json').read_bytes())
    assert result['status']=='passed' and result['inputs']==seal(standalone/'inputs.json')
    assert result['all_original_files_restored']==77162 and result['all_original_bytes_restored']==696793808
    assert result['every_original_relative_path_byte_and_mode_exact'] and result['historical_manifest_unchanged']
    for row in common['sources']:assert row==seal(Path(row['path']))
    for row in common['exact_prior_storage_inputs']:assert row==storage[row['path']]
    archive=Path(common['archive']['path']);assert common['archive']==seal(archive);archives.add(archive)
    with tarfile.open(archive,'r:gz') as packed:
        expected={logical(r['path']):r for r in common['sources'] if r['path'] not in storage}
        assert {m.name for m in packed.getmembers()}==set(expected)
        for member in packed.getmembers():
            raw=packed.extractfile(member).read();row=expected[member.name]
            assert member.isfile() and member.mode==row['mode'] and len(raw)==row['bytes'] and hashlib.sha256(raw).hexdigest()==row['sha256']
            scan(member.name,raw);members+=1
    for phase in ['old','fixed']:
        path=standalone/phase;observed=json.loads((path/'result.json').read_bytes());pre=json.loads((path/'prelaunch.json').read_bytes())
        assert observed['prelaunch']==seal(path/'prelaunch.json') and observed['consumed_sources_unchanged']
        for field in ['stdout','stderr']:
            row=observed[field];assert row==seal(Path(row['path']));raw_outputs[logical(row['path'])]=row
        assert pre['original_source_objects_before_launch']==0
        assert pre['complete_original_inputs']==seal(standalone/'inputs.json')
        origins={str(Path(common['cwd'])/logical(r['path'])):r for r in common['sources']}
        assert set(origins)=={r['path'] for r in pre['sources']}
        for row in pre['sources']:
            original=origins[row['path']]
            if phase=='old' and row['path'].endswith('/crates/mpk-vc/tests/go_practical_source_build.rs'):original=seal(old_test)
            assert all(row[k]==original[k] for k in ['bytes','sha256','mode'])
        if phase=='old':assert observed['status']=='reproduced_original_missing_snapshot_failure' and observed['exit_code']==101 and observed['restored_source_objects_after_run']==0
        else:assert observed['status']=='passed' and observed['exit_code']==0 and observed['restored_source_objects_after_run']==77162
    for name in ['native-case-comparison.json','original-candidate-comparison.json']:
        assert (standalone/'fixed/observations'/name).read_bytes()==(ROOT/'rust/observations/build'/name).read_bytes()
    final=ROOT/'final-consumer-checks';checked=json.loads((final/'result.json').read_bytes())
    assert checked['status']=='passed' and checked['consumed_sources_unchanged'] and all(r['exit_code']==0 for r in checked['runs'])
    assert checked['prelaunch']==seal(final/'prelaunch.json')
    for field in ['stdout','stderr']:
        row=checked[field];assert row==seal(Path(row['path']));raw_outputs[logical(row['path'])]=row
    current=[];links=[]
    names=set((git('diff','--name-only','-z','HEAD')+git('ls-files','--others','--exclude-standard','-z')).decode().split('\0'))-{'',logical(output)}
    for name in sorted(names):
        path=REPO/name;assert path.is_file() and not path.is_symlink();raw=path.read_bytes();current.append(seal(path))
        if path not in archives:scan(name,raw)
        if path.suffix=='.md':
            for target in re.findall(r'\]\(([^)]+)\)',raw.decode()):
                target=target.strip('<>').split('#',1)[0]
                if not target or re.match(r'^[a-zA-Z][a-zA-Z0-9+.-]*:',target):continue
                resolved=path.parent/re.sub(r':\d+$','',urllib.parse.unquote(target));links.append({'source':name,'target':target,'exists':resolved.exists()})
    assert not findings and all(r['exists'] for r in links)
    for path in ROOT.glob('*.py'):compile(path.read_bytes(),str(path),'exec')
    result={'status':'passed','basis_commit':BASIS,'driver':seal(Path(__file__).resolve()),'matcher_count':127,'scopes':4,'seals':39,'exclusions':51,'rules_unchanged':True,
        'previous_complete_original_storage_review':seal(old_path),'exact_prior_storage_files':list(storage.values()),'reused_complete_original_source_members':77162,
        'actual_source_epochs':epochs,'complete_new_archive_members_verified_and_scanned':members,'exact_original_input_rechecks':original_checks,'logical_source_preimages_scanned':scanned,
        'independent_inspection':seal(inspect_path),'current_inputs':current,'new_findings':findings,'links':links,'raw_outputs_preserved':list(raw_outputs.values()),
        'pre_fix_review_and_all76_inputs_preserved':seal(before_path),'standalone_original_failure_and_complete_repair':seal(standalone/'result.json'),'final_affected_consumer_checks':seal(final/'result.json'),
        'manual_review':'The fixed-target filename/header guard examines all claimed active Go source buffers while retaining every excluded/unselected original in the full snapshot. Exact original MatchFile observations and primary source syslist coverage validate filename/header parsing, old-syntax ignored-complexity behavior, default target experiment/release tags and binary/test eligibility. Shared language minima retain all150 observations. Coherent rebinding controls preserve all5490 original snapshot objects and both call-contract candidates; their new target exclusions fail before capture input construction. The same full43-root units, zero/70-error/type domains and32 proof bytes remain. This is a necessary eligibility guard, not complete import/cgo membership, launcher/compiler/export/context authority or a source/program proof. No new rules, exclusions, kernel cap, axiom or public activation.',
        'full_T_gate':'Deferred to final T02-W04: ./scripts/check-fast.sh','whole_repository_gate_run':False}
    output.write_text(json.dumps(result,indent=2)+'\n');print(json.dumps({'status':'passed','epochs':len(epochs),'archive_members':members,'original_input_rechecks':original_checks,'current_files':len(current),'new_findings':0,'links':len(links)}))

if __name__=='__main__':main()
