"""Unchanged-rule scoped review of every actual source epoch and original input."""
from pathlib import Path
import hashlib,json,re,subprocess,sys,tarfile,types,urllib.parse

REPO=Path.cwd();ROOT=Path(__file__).resolve().parent
BASIS='0f98236213ef379eca1ab77badf047a9cd10c79a'
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
    restored=json.loads((ROOT/'restored-corpora.json').read_bytes())
    restored_rows={}
    old_corpus_manifests=[]
    for corpus in restored['corpora']:
        old_manifest=ROOT.parent/('go-'+corpus['corpus'])/'native-original/manifest.json'
        assert old_manifest.read_bytes()==git('show',BASIS+':'+logical(old_manifest))
        record=json.loads(old_manifest.read_bytes());old_corpus_manifests.append(seal(old_manifest))
        for item in record['original_inputs']:
            name=str(Path(corpus['target'])/item['member']);restored_rows[name]={**item,'path':name}
        for row in corpus['inputs']:
            assert row==seal(Path(row['path']))
            assert Path(row['path']).read_bytes()==git('show',BASIS+':'+logical(row['path']))
    assert len(restored_rows)==26550
    findings=[];cache={};scanned=0
    path_rules=[r for r in matchers if r['scope']=='exact-path']
    def scan(name,raw):
        nonlocal scanned
        key=(hashlib.sha256(raw).hexdigest(),tuple(module.path_matches(name,r['path']) for r in path_rules))
        if key not in cache:cache[key]=[vars(r) for r in module.detect(name,raw.decode(errors='ignore'),matchers)]
        findings.extend({**r,'path':name,'excerpt_hex':r['excerpt'].encode().hex()} for r in cache[key]);scanned+=1
    epochs=[];archives=set();raw_outputs={};original_checks=0;members=0
    for path in sorted(ROOT.glob('*/inputs.json')):
        inputs=json.loads(path.read_bytes());result=json.loads(path.with_name('result.json').read_bytes())
        assert result['status'] in ['passed','failed'] and result['inputs']==seal(path)
        assert result['consumed_inputs_unchanged'] or result['expected_formatter_mutation']
        for field in ['stdout','stderr']:
            row=result[field];assert row==seal(Path(row['path']));raw_outputs[logical(row['path'])]=row
        sources={r['path']:r for r in inputs['sources']};preserved=set(inputs['preserved_original_paths'])
        assert preserved<=sources.keys()
        for name in preserved:
            row=sources[name];assert row==seal(Path(name))
            expected=storage.get(name,original.get(name,restored_rows.get(name)));assert expected is not None
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
    assert len(epochs)==13
    inspect_path=ROOT/'inspection.json';inspection=json.loads(inspect_path.read_bytes());assert inspection['status']=='passed'
    for row in inspection['inputs']:assert row==seal(Path(row['path']))
    corpus=json.loads((ROOT/'native-final/native-originals/headers/package-header-corpus.json').read_bytes())
    for row in corpus['cases']:
        for source in row['snapshot_sources']:
            raw=bytes.fromhex(source['raw_hex']);assert len(raw)==source['object']['bytes'] and hashlib.sha256(raw).hexdigest()==source['object']['raw_sha256']
            scan('go-tools/go2vir/original-package-header-cases/'+source['object']['path'],raw)
    generation=json.loads((ROOT/'graphic-generation.json').read_bytes())
    for field in ['generator','first_failure_stdout','first_failure_stderr','final_stdout','final_stderr','sdk_go_binary']:
        row=generation[field];assert all(seal(Path(row['path']))[k]==row[k] for k in ['bytes','sha256'])
    for output_name in ['graphic-original.stdout','graphic-original.stderr','graphic-final.stdout','graphic-final.stderr']:
        row=seal(ROOT/output_name);raw_outputs[logical(row['path'])]=row
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
        'manual_review':'The new private Go header view retains the entire original source object/bytes and physical offsets; only its lexer/parser EOF marks the import-header boundary. Full source parsing retains its existing owned-source and worker-stack behavior. Original snapshot namespaces determine every Go candidate; fixed-target filename/header checks, original package names, valid Unicode15 import paths, cgo-disabled exclusions, internal/external tests and active-file identities are reconstructed. Complete capture compares derived file IDs and package name with every claimed package, rejecting coherent omission of an actual unselected file while retaining both selected call contracts. All2003 directories,39 header cases, complete5853/2996 old lexer/parser cases,43-root units, zero/70 universes and32 proof bytes remain. No replacement source/stub/adapter, raised cap, new source/JSON exclusion, kernel rule, axiom or profile activation. Auxiliary/cgo-directive/embed and canonical module/compiler/export/context/launcher/program authority remain due.',
        'full_T_gate':'Deferred to final T02-W04: ./scripts/check-fast.sh','whole_repository_gate_run':False,'full_original_lexer_parser_storage_manifests':old_corpus_manifests,'complete_restored_original_lexer_parser_members':26550}
    output.write_text(json.dumps(result,indent=2)+'\n');print(json.dumps({'status':'passed','epochs':len(epochs),'archive_members':members,'original_input_rechecks':original_checks,'current_files':len(current),'new_findings':0,'links':len(links)}))

if __name__=='__main__':main()
