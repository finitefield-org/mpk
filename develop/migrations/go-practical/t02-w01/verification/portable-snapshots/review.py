"""Review the changed restoration path and every complete relocated build input."""
from pathlib import Path
import hashlib,json,re,subprocess,sys,tarfile,types,urllib.parse

REPO=Path.cwd()
ROOT=Path(__file__).resolve().parent
BASIS='2e1330a6dc4e155399f836002491fd1e2dc13200'
sys.path=[p for p in sys.path if Path(p or '.').resolve()!=ROOT]

def git(*args):return subprocess.check_output(['/usr/bin/git',*args])
def seal(path):
    raw=path.read_bytes()
    return {'path':str(path),'bytes':len(raw),'sha256':hashlib.sha256(raw).hexdigest(),'mode':path.stat().st_mode&0o777}
def logical(path):
    path=Path(path);return str(path.relative_to(REPO)) if path.is_absolute() else str(path)

def main():
    assert git('rev-parse','HEAD').decode().strip()==BASIS
    output=ROOT/'review.json';assert not output.exists()
    scanner=REPO/'scripts/check-no-active-gir.sh'
    code=scanner.read_text().split("<<'PY'\n",1)[1].rsplit('\nPY',1)[0].rsplit('raise SystemExit(main())',1)[0]
    module=types.ModuleType('portable_snapshot_scoped_review');module.__file__=str(scanner)
    sys.modules[module.__name__]=module;sys.argv=[str(scanner),str(REPO),'strict']
    exec(compile(code,str(scanner),'exec'),module.__dict__)
    matchers=module.load_matchers();manifest,configured,fixtures,_=module.validate_manifest()
    module.run_fixture_self_test(manifest,matchers)
    assert (len(matchers),len(module.SCOPES),len(manifest['sealed_records']),len(module.active_retained_record_exclusions(manifest)))==(127,4,39,51)
    for name in configured|fixtures:assert (REPO/name).read_bytes()==git('show',BASIS+':'+name)
    findings=[];scanned=[]
    def scan(name,raw):
        findings.extend({**vars(row),'excerpt_hex':row.excerpt.encode().hex()} for row in module.detect(name,raw.decode(errors='ignore'),matchers))
        scanned.append({'path':name,'bytes':len(raw),'sha256':hashlib.sha256(raw).hexdigest()})
    raw_outputs={}
    def receipt(row):
        assert row==seal(Path(row['path']));return row
    original=json.loads((ROOT/'original-reproduction.json').read_bytes())
    assert original['status']=='original_relocation_rejected' and original['exit_code']==1
    assert (ROOT/'original-driver.py').read_bytes()==git('show',BASIS+':develop/migrations/go-practical/t02-w01/verification/snapshot-inputs/preserve-originals.py')
    saved=json.loads((ROOT/'relocated-source-preservation.json').read_bytes())
    receipt(saved['original_prelaunch']);receipt(saved['exact_preserved_preimage'])
    assert all(saved['original_consumed_source'][k]==saved['exact_preserved_preimage'][k] for k in ['bytes','sha256','mode'])
    for result in [original,*[json.loads(p.read_bytes()) for p in ROOT.glob('*/result.json')]]:
        for field in ['stdout','stderr']:
            row=receipt(result[field]);raw_outputs[logical(row['path'])]=row
    rust=json.loads((ROOT/'relocated-rust/result.json').read_bytes())
    assert rust['status']=='passed' and rust['original_and_relocated_consumed_inputs_unchanged']
    assert rust['original_snapshot_members_restored']==77162 and rust['original_snapshot_bytes_restored']==696793808
    assert rust['historical_manifest_byte_identical'] and rust['all_original_relative_paths_bytes_and_modes_exact']
    pre_path=ROOT/'relocated-rust/prelaunch.json';assert rust['prelaunch']==seal(pre_path)
    pre=json.loads(pre_path.read_bytes());receipt(pre['workspace_archive'])
    sources={logical(row['path']):row for row in pre['sources']}
    data_prefix='develop/migrations/go-practical/t02-w01/verification/snapshot-inputs/snapshot-storage/'
    storage={name for name in sources if name.startswith(data_prefix)}
    for name in storage:
        receipt(sources[name]);assert (REPO/name).read_bytes()==git('show',BASIS+':'+name)
    # Reuse the exact previously reviewed complete original source storage.
    # Every immutable chunk and the full manifest still match the basis tree.
    previous=ROOT.parent/'snapshot-inputs/input-review.json'
    assert previous.read_bytes()==git('show',BASIS+':'+logical(previous))
    old=json.loads(previous.read_bytes())
    assert old['status']=='passed' and not old['new_findings'] and not old['unchanged_basis_findings']
    assert old['complete_snapshot_members_verified_and_scanned']==77162
    archive=Path(pre['workspace_archive']['path']);members=0
    with tarfile.open(archive,'r:gz') as packed:
        assert {m.name for m in packed.getmembers()}==set(sources)-storage
        for member in packed.getmembers():
            row=sources[member.name];raw=packed.extractfile(member).read()
            assert member.isfile() and member.mode==row['mode']
            assert len(raw)==row['bytes'] and hashlib.sha256(raw).hexdigest()==row['sha256']
            receipt(row);scan(member.name,raw);members+=1
    current=[];links=[]
    names=set((git('diff','--name-only','-z','HEAD')+git('ls-files','--others','--exclude-standard','-z')).decode().split('\0'))-{'',logical(output)}
    for name in sorted(names):
        path=REPO/name;assert path.is_file() and not path.is_symlink()
        raw=path.read_bytes();current.append(seal(path))
        if path!=archive:scan(name,raw)
        if path.suffix=='.md':
            for target in re.findall(r'\]\(([^)]+)\)',raw.decode()):
                target=target.strip('<>').split('#',1)[0]
                if not target or re.match(r'^[a-zA-Z][a-zA-Z0-9+.-]*:',target):continue
                resolved=path.parent/re.sub(r':\d+$','',urllib.parse.unquote(target))
                links.append({'source':name,'target':target,'exists':resolved.exists()})
    assert not findings and all(row['exists'] for row in links)
    controls=json.loads((ROOT/'controls.json').read_bytes());assert controls['status']=='passed' and len(controls['controls'])==15
    for row in controls['inputs']:receipt(row)
    for path in [ROOT/'run-rust.py',ROOT/'run-relocated.py',ROOT/'check-controls.py',Path(__file__).resolve(),ROOT.parent/'snapshot-inputs/preserve-originals.py']:compile(path.read_bytes(),str(path),'exec')
    result={'status':'passed','basis_commit':BASIS,'driver':seal(Path(__file__).resolve()),'matcher_count':127,'scopes':4,'seals':39,'exclusions':51,'rules_unchanged':True,'previous_complete_original_storage_review':seal(previous),'all_original_storage_files_exact_basis_bytes':len(storage),'reused_complete_original_source_members':77162,'complete_new_workspace_archive_members_verified_and_scanned':members,'scanned_inputs':scanned,'current_inputs':current,'new_findings':findings,'links':links,'raw_outputs_preserved':list(raw_outputs.values()),'manual_review':'Absolute origin paths remain immutable provenance; original member identities and every byte/mode remain checked. Restore destinations are resolved under the current root with canonical relative spelling and symlink guards; existing corrupt data is rejected without replacement. Full original relocation failure and all later input epochs remain exact. Only Python code changed; the actual relocated Rust binaries exercise automatic restoration without any pre-restored snapshot members. No source-language support, proof rule, context authority or resource limit changes.','full_T_gate':'Deferred to final T02-W04: ./scripts/check-fast.sh','whole_repository_gate_run':False}
    output.write_text(json.dumps(result,indent=2)+'\n')
    print(json.dumps({'status':'passed','archive_members':members,'current_files':len(current),'reused_original_members':77162,'new_findings':0,'links':len(links)}))

if __name__=='__main__':main()
