"""Independently compare the retained prepared inventory with original SDK bytes."""
from pathlib import Path
import datetime, hashlib, json, os, stat
root=Path(__file__).resolve().parent.relative_to(Path.cwd().resolve())
out=root/'original-observation';out.mkdir(exist_ok=False)
source=Path('/private/tmp/mpk-go-practical-toolchains/go1.25.0-darwin-arm64/go')
def seal(p):
    data=p.read_bytes();return {'path':str(p),'sha256':hashlib.sha256(data).hexdigest(),'bytes':len(data)}
def typed(value):
    raw=json.dumps(value,sort_keys=True,separators=(',',':'),ensure_ascii=False).encode()
    return hashlib.sha256(b'MPK-BUNDLE-CONTENT-0.1\0'+raw).hexdigest()
record={'status':'running','started_at_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'original_sdk_root':str(source),'CSharp_tests_executed':False,'T02_W01_complete':False,'native_Linux_acceptance_claimed':False,'public_profile_activated':False,'full_T02_gate':'deferred to W04: check-fast.sh --skip-csharp'}
try:
    original=root/'captured/selected/prepared-image.json'
    data=json.loads(original.read_text());record['input']=seal(original)
    inventory=data['inventory'];files=inventory['files'];identity=data['toolchain'];host=data['host']
    assert host=='darwin_arm64' and inventory['schema']=='mpk.release.bundle_inventory.v0'
    assert inventory['scope']=={'kind':'toolchain_bundle','bundle_id':identity['bundle_id']}
    assert typed(inventory)==identity['distribution_sha256']
    expected={}
    for directory,dirs,names in os.walk(source,followlinks=False):
        parent=Path(directory);relative=parent.relative_to(source)
        dirs[:]=[d for d in dirs if d!='testdata' and not(relative==Path('.') and d=='test')]
        assert all(not (parent/d).is_symlink() for d in dirs)
        for name in names:
            if name.endswith('_test.go'):continue
            p=parent/name;assert stat.S_ISREG(p.lstat().st_mode)
            relative=p.relative_to(source).as_posix()
            expected['go/'+relative]={'path':'go/'+relative,'size_bytes':p.stat().st_size,'sha256':hashlib.sha256(p.read_bytes()).hexdigest(),'executable':relative in ['bin/go','bin/gofmt'] or relative.startswith('pkg/tool/'+host+'/')}
    assert files==[expected[p] for p in sorted(expected)]
    components=data['components'];seen={};projected=[]
    for component in components:
        identity_component=component['Identity'];members=component['Inventory'];name=identity_component['name']
        assert identity_component['release']=='go1.25.0'
        assert members['schema']==inventory['schema']
        assert members['scope']=={'kind':'component','bundle_id':identity['bundle_id'],'component_name':name}
        for member in members['files']:
            assert member==expected[member['path']] and member['path'] not in seen
            seen[member['path']]=name
        if identity_component['kind']=='content':
            assert typed(members)==identity_component['content_sha256'] and all(not m['executable'] for m in members['files'])
        else:
            assert identity_component['kind']=='executable' and len(members['files'])==1 and members['files'][0]['executable'] and members['files'][0]['sha256']==identity_component['binary_sha256']
        projected.append(identity_component)
    assert projected==identity['components'] and set(seen)==set(expected)
    audit=Path('develop/migrations/go-practical/t01-w01/entry-audit.json')
    for witness in json.loads(audit.read_text())['compiler_candidate']['source_witnesses']:
        assert expected[witness['path']]==witness
    record.update(status='passed',files=len(files),bytes=sum(f['size_bytes'] for f in files),components=len(components),executables=sum(f['executable'] for f in files),standard_source_directories=len({str(Path(f['path']).parent).removeprefix('go/src/') for f in files if f['path'].startswith('go/src/') and f['path'].endswith('.go') and str(Path(f['path']).parent)!='go/src'}),distribution_sha256=identity['distribution_sha256'],source_witnesses=seal(audit),original_archive=seal(source.parent.parent/'go1.25.0.darwin-arm64.tar.gz'),selection_reason='Independently enumerate original SDK files using the existing release test/testdata pruning and executable-mode rule, compare every original file length/hash and recompute root/component domain-separated canonical identities and exact partition. Prepared image only, not a registered practical context or source/program/native release proof.')
except Exception as error:
    record.update(status='failed',error=repr(error))
record['finished_at_utc']=datetime.datetime.now(datetime.timezone.utc).isoformat()
(out/'result.json').write_text(json.dumps(record,indent=2)+'\n')
print(json.dumps({k:record[k] for k in ['status','files','bytes','components','executables'] if k in record}))
if record['status']!='passed':raise SystemExit(1)
