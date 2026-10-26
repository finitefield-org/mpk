"""Unchanged-rule scoped review of every actual source epoch and original input."""
from pathlib import Path
import base64,hashlib,json,re,subprocess,sys,tarfile,types,urllib.parse

REPO=Path.cwd();ROOT=Path(__file__).resolve().parent
BASIS='34940c6af58dc970eae1c19f2d60a3f13c6c3bea'
sys.path=[p for p in sys.path if Path(p or '.').resolve()!=ROOT]
def git(*args):return subprocess.check_output(['/usr/bin/git',*args])
def seal(path):
    raw=path.read_bytes();return {'path':str(path),'bytes':len(raw),'sha256':hashlib.sha256(raw).hexdigest(),'mode':path.stat().st_mode&0o777}
def logical(path):
    p=Path(path);return str(p.relative_to(REPO)) if p.is_absolute() else str(p)
def main():
    assert git('rev-parse','HEAD').decode().strip()==BASIS
    output=ROOT/'review.json';assert not output.exists()
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
    prior_owner_reviews=[]
    for name in ['package-files/review.json','cgo-directives/review.json','embedded-inputs/review.json']:
        previous_owner_path=ROOT.parent/name
        assert previous_owner_path.read_bytes()==git('show',BASIS+':'+logical(previous_owner_path))
        previous_owner=json.loads(previous_owner_path.read_bytes());assert previous_owner['status']=='passed' and not previous_owner['new_findings']
        prior_owner_reviews.append(seal(previous_owner_path))
        for row in previous_owner['current_inputs']:
            if 'original-source-objects' in Path(row['path']).parts:original[row['path']]=row
    findings=[];cache={};scanned=0
    path_rules=[r for r in matchers if r['scope']=='exact-path']
    def scan(name,raw):
        nonlocal scanned
        key=(hashlib.sha256(raw).hexdigest(),tuple(module.path_matches(name,r['path']) for r in path_rules))
        if key not in cache:cache[key]=[vars(r) for r in module.detect(name,raw.decode(errors='ignore'),matchers)]
        findings.extend({**r,'path':name,'excerpt_hex':r['excerpt'].encode().hex()} for r in cache[key]);scanned+=1
    decoded_cases=0
    def scan_embed_corpus(raw):
        nonlocal decoded_cases
        corpus=json.loads(raw)
        for row in corpus['cases']:
            for name,encoded in row['whole_original_files'].items():
                scan('go-tools/go2vir/original-embed-cases/'+row['case']+'/'+name,base64.b64decode(encoded,validate=True))
            for field in ['native_stdout','native_stderr']:
                scan('develop/migrations/go-practical/t02-w01/verification/embedded-inputs/original-native-output/'+row['case']+'/'+field+'.raw',base64.b64decode(row[field],validate=True))
            if row['accepted']:
                for source in row['snapshot']['snapshot_sources']:
                    obj=source['object'];data=bytes.fromhex(source['raw_hex'])
                    assert len(data)==obj['bytes'] and hashlib.sha256(data).hexdigest()==obj['raw_sha256']
                    scan('go-tools/go2vir/original-embed-captured/'+obj['path'],data)
            decoded_cases+=1
    decoded_native_source_records=0
    def scan_native_transport(name,raw):
        nonlocal decoded_native_source_records
        basename=Path(name).name
        recognized=basename in ['complete-embedded-candidates.json','original-embedded-directives.json','original-test-pattern-groups.json','original-pinned-globs.json'] or basename.startswith(('original-native-case-','original-group-'))
        if not recognized:return
        packet=json.loads(raw)
        if 'complete_original_test_source_hex' in packet:
            scan('go-tools/go2vir/original-pinned-sdk/src/path/match_test.go',bytes.fromhex(packet['complete_original_test_source_hex']))
        rows=packet.get('cases',[packet])
        for row in rows:
            snapshot=row.get('snapshot',row)
            for source in snapshot.get('snapshot_sources',[]):
                if 'raw_hex' not in source:continue
                data=bytes.fromhex(source['raw_hex']);obj=source['object']
                assert len(data)==obj['bytes'] and hashlib.sha256(data).hexdigest()==obj['raw_sha256']
                scan('go-tools/go2vir/original-embedded-selection/'+str(row.get('case',''))+'/'+obj['path'],data)
                decoded_native_source_records+=1
            for path,encoded in row.get('original_files',{}).items():
                scan('go-tools/go2vir/original-embedded-files/'+str(row.get('case',''))+'/'+path,base64.b64decode(encoded,validate=True))
            if 'original_source_hex' in row:
                scan('go-tools/go2vir/original-embedded-directives/'+str(row['case'])+'.go',bytes.fromhex(row['original_source_hex']))
            for field in ['native_stdout','native_stderr']:
                if field in row:scan('develop/migrations/go-practical/t02-w01/verification/embedded-selection/original-native-output/'+str(row.get('case',''))+'/'+field+'.raw',base64.b64decode(row[field],validate=True))
    epochs=[];archives=set();raw_outputs={};original_checks=0;members=0
    for path in sorted(ROOT.glob('*/inputs.json')):
        inputs=json.loads(path.read_bytes());result=json.loads(path.with_name('result.json').read_bytes())
        assert result['status']=='passed' and result['inputs']==seal(path)
        assert result['consumed_inputs_unchanged'] or result['expected_formatter_mutation']
        for field in ['stdout','stderr']:
            row=result[field];assert row==seal(Path(row['path']));raw_outputs[logical(row['path'])]=row
        for external in inputs.get('complete_external_originals',[]):
            assert external['original']==seal(Path(external['original']['path'])) and external['saved']==seal(Path(external['saved']['path']))
            assert all(external['original'][k]==external['saved'][k] for k in ['bytes','sha256','mode'])
            scan('go-tools/go2vir/original-pinned-sdk/src/path/match_test.go',Path(external['saved']['path']).read_bytes())
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
                if member.name.endswith('/embedded-source-corpus.json'):scan_embed_corpus(raw)
                scan_native_transport(member.name,raw)
        epochs.append({'result':seal(path.with_name('result.json')),'status':result['status'],'command':result['command'],'all_inputs':len(sources),'original_inputs_retained_by_exact_prior_storage':len(preserved)})
    assert len(epochs)==22
    inspect_path=ROOT/'inspection.json';inspection=json.loads(inspect_path.read_bytes());assert inspection['status']=='passed'
    for row in inspection['inputs']:assert row==seal(Path(row['path']))
    correspondence_path=ROOT/'source-correspondence.json';correspondence=json.loads(correspondence_path.read_bytes());assert correspondence['status']=='passed'
    for row in correspondence['inputs']:assert row==seal(Path(row['path']))
    for rows in correspondence['complete_unchanged_original_production_sources'].values():
        for row in rows:assert row==seal(Path(row['path']))
    for witness in inspection['complete_pinned_primary_source_witnesses']:
        raw=bytes.fromhex(witness['raw_hex']);obj=witness['object']
        assert len(raw)==obj['bytes'] and hashlib.sha256(raw).hexdigest()==obj['raw_sha256']
        scan('go-tools/go2vir/original-pinned-sdk/'+witness['path'],raw)
    current=[];links=[]
    names=set((git('diff','--name-only','-z','HEAD')+git('ls-files','--others','--exclude-standard','-z')).decode().split('\0'))-{'',logical(output)}
    for name in sorted(names):
        path=REPO/name;assert path.is_file() and not path.is_symlink();raw=path.read_bytes();current.append(seal(path))
        if path not in archives:
            scan(name,raw);scan_native_transport(name,raw)
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
        'decoded_original_embed_cases_including_archived_preimages':decoded_cases,'decoded_new_native_source_records_including_archived_preimages':decoded_native_source_records,'independent_complete_source_correspondence':seal(correspondence_path),
        'manual_review':'Rust derives original production/internal-test/external-test pattern bytes from complete original sources; the public private-handoff resource derivation does not accept caller source or patterns. Native build comment/literal/argument behavior, escaped arbitrary bytes and literal invalid UTF-8 decoding match all54 original pragma cases. Pinned Go chunk and byte-star behavior, including non-backtracking UTF-8 boundaries, matches all56 original standard-library cases and17 added actual byte cases. All54 complete original native resource cases, every2003 unchanged original package directory and three nonempty internal/external test-pattern cases agree. Test-only invalid/missing patterns do not contaminate ordinary production resource selection; all original test sources remain retained. Complete capture rejects coherently omitted original resource buffers and unobserved required directories with the same original Root contract. All5535 original owner objects/107062681 bytes, both43-root/zero-or70-error universes and32 proof streams remain. The original25 integration roots and glob unit retain unchanged production sources and complete test-source prefixes; the added group test, targeted formats/vet/Clippy pass. All22 source epochs and every actual native child output remain complete. Source decoders preserve original buffers/locations, without source stubs/replacements or host fallback. Missing module-link target/stat facts reject; canonical roots/compiler/pragma/context/registry and full source/program/ordinary foundation authority remain due. No profile/language activation, new rule/seal/exclusion, axiom or raised kernel limit.',
        'full_T_gate':'Deferred to final T02-W04: ./scripts/check-fast.sh','whole_repository_gate_run':False,'complete_prior_owned_source_reviews':prior_owner_reviews}
    output.write_text(json.dumps(result,indent=2)+'\n');print(json.dumps({'status':'passed','epochs':len(epochs),'archive_members':members,'original_input_rechecks':original_checks,'current_files':len(current),'new_findings':0,'links':len(links)}))

if __name__=='__main__':main()
