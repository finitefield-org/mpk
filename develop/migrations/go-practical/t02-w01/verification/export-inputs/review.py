"""Unchanged-rule scoped review of every actual source epoch and original input."""
from pathlib import Path
import base64,hashlib,json,re,subprocess,sys,tarfile,types,urllib.parse

REPO=Path.cwd();ROOT=Path(__file__).resolve().parent
BASIS='9cfbb4a524b799f982b1e478eaef41ed8c56f126'
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
    for name in ['package-files/review.json','cgo-directives/review.json','embedded-inputs/review.json','embedded-selection/review.json']:
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
    decoded_export_transports=0
    def scan_export_transport(name,raw):
        nonlocal decoded_export_transports
        if not name.endswith('.json'):return
        basename=Path(name).name
        if basename not in ['complete-original-export-inputs.json','coherent-native-export-controls.json','coherent-original-archive-controls.json']:return
        packet=json.loads(raw)
        def metadata(encoded):
            nonlocal decoded_export_transports
            doc=json.loads(bytes.fromhex(encoded));decoded_export_transports+=1
            for field in ['metadata_hex','diagnostics_hex']:
                if field in doc:scan('develop/migrations/go-practical/t02-w01/verification/export-inputs/original-native-output/'+field+'.raw',bytes.fromhex(doc[field]))
        if 'export_inputs_hex' in packet:metadata(packet['export_inputs_hex'])
        for row in packet.get('cases',[]):
            for field in ['complete_changed_export_inputs_hex','changed_export_hex']:
                if field in row:metadata(row[field])
            if 'complete_changed_archive_hex' in row:
                data=bytes.fromhex(row['complete_changed_archive_hex']);obj=row['changed_object'];assert len(data)==obj['bytes'] and hashlib.sha256(data).hexdigest()==obj['raw_sha256']
                scan('go-tools/go2vir/original-go-archive-controls/'+row['case']+'.raw',data)
    epochs=[];archives=set();raw_outputs={};original_checks=0;members=0
    for path in sorted(ROOT.glob('*/inputs.json')):
        inputs=json.loads(path.read_bytes());result=json.loads(path.with_name('result.json').read_bytes())
        assert result['status'] in ['passed','failed'] and result['inputs']==seal(path)
        if result['status']=='failed':assert path.parent.name=='native' and result['exit_code']==1
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
                if member.name.endswith('/embedded-source-corpus.json'):scan_embed_corpus(raw)
                scan_export_transport(member.name,raw)
        epochs.append({'result':seal(path.with_name('result.json')),'status':result['status'],'command':result['command'],'all_inputs':len(sources),'original_inputs_retained_by_exact_prior_storage':len(preserved)})
    assert len(epochs)==10
    inspect_path=ROOT/'inspection.json';inspection=json.loads(inspect_path.read_bytes());assert inspection['status']=='passed'
    for row in inspection['inputs']:assert row==seal(Path(row['path']))
    assert inspection['original_failed_harness_epoch_retained'] and len(inspection['actual_native_passed_roots'])==2
    for witness in inspection['complete_original_source_witnesses']:
        raw=bytes.fromhex(witness['raw_hex']);obj=witness['object']
        assert len(raw)==obj['bytes'] and hashlib.sha256(raw).hexdigest()==obj['raw_sha256']
        scan('go-tools/go2vir/original-pinned-sdk/'+witness['path'],raw)
    current=[];links=[]
    names=set((git('diff','--name-only','-z','HEAD')+git('ls-files','--others','--exclude-standard','-z')).decode().split('\0'))-{'',logical(output)}
    for name in sorted(names):
        path=REPO/name;assert path.is_file() and not path.is_symlink();raw=path.read_bytes();current.append(seal(path))
        if path not in archives:
            scan(name,raw);scan_export_transport(name,raw)
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
        'decoded_original_embed_cases_including_archived_preimages':decoded_cases,'decoded_export_transports_including_archived_preimages':decoded_export_transports,
        'manual_review':'The same original owner mutex supplies complete source capture and original command/export bytes; no caller metadata, typed/source package or export callback enters the producer. Rust reconstructs whole original source/auxiliary/resource/import/vendor membership and selected-root/path/export correspondence, retains all native metadata and every archive byte, checks duplicate native fields before typed projection, validates archive bounds/target/build IDs/padding and closes the entire non-intrinsic export set. All180 packages/179 archives/5663 objects/164865655 bytes remain intact, including unused HTTP body and unexecuted initialization. All22 affected Rust roots and21 metadata/four fully resealed archive controls pass with original selection and one Root contract. The new native root passes; a later evidence-directory collision prevents the existing ownership root from running. The whole initial failed epoch remains exact; the unchanged ownership root passes under a separate directory without repeating the successful export root. Strict affected Clippy/vet/formats pass. Both43-root/zero-or70-error/type domains and32 proof streams remain complete. All ten consumed source epochs and exact outputs, original/prepared/formatter states and decoded native/changed archive bytes remain preserved. Full compiler-image/root/module/context/registry/launcher/source/export/program/foundation authority remains due. No original source rewrite/stub, new language/source kind/rule/seal/exclusion, axiom or raised kernel budget.',
        'full_T_gate':'Deferred to final T02-W04: ./scripts/check-fast.sh','whole_repository_gate_run':False,'complete_prior_owned_source_reviews':prior_owner_reviews}
    output.write_text(json.dumps(result,indent=2)+'\n');print(json.dumps({'status':'passed','epochs':len(epochs),'archive_members':members,'original_input_rechecks':original_checks,'current_files':len(current),'new_findings':0,'links':len(links)}))

if __name__=='__main__':main()
