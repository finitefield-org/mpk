"""Unchanged-rule scoped review of complete source preimages and actual outputs.
Run outside the sandbox: every Git invocation uses /usr/bin/git.
"""
from pathlib import Path
import base64, datetime, hashlib, json, re, runpy, subprocess, sys, types, urllib.parse

ROOT = Path.cwd()
OUT = Path(__file__).resolve().parent
BASE = '00275cd124d4951edfa88b57709718bfbc798df1'
REPORT = OUT/'input-review.json'


def git(*args):
    return subprocess.check_output(['/usr/bin/git', *args])


def seal(path, raw=None):
    raw = Path(path).read_bytes() if raw is None else raw
    return {'path': str(path), 'bytes': len(raw), 'sha256': hashlib.sha256(raw).hexdigest()}


assert git('rev-parse','HEAD').decode().strip() == BASE
assert not Path('.github/workflows').exists()
scanner = Path('scripts/check-no-active-gir.sh')
code = scanner.read_text().split("<<'PY'\n",1)[1].rsplit('\nPY',1)[0].rsplit('raise SystemExit(main())',1)[0]
module = types.ModuleType('mpk_source_scope_input_review')
module.__file__ = str(ROOT/scanner)
sys.modules[module.__name__] = module
sys.argv = [str(scanner),str(ROOT),'strict']
exec(compile(code,str(scanner),'exec'),module.__dict__)
matchers = module.load_matchers()
manifest, scanner_files, fixtures, _ = module.validate_manifest()
module.run_fixture_self_test(manifest,matchers)
exclusions = module.active_retained_record_exclusions(manifest)
assert (len(matchers),len(module.SCOPES),len(manifest['sealed_records']),len(exclusions)) == (127,4,39,51)
for name in scanner_files | fixtures:
    assert Path(name).read_bytes() == git('show',BASE+':'+name)
configured = scanner_files | fixtures | exclusions
findings, inputs, links, original_checks = [], [], [], []
cache = {}
path_rules = [m for m in matchers if m['scope']=='exact-path']


def scan(name,raw):
    key = (hashlib.sha256(raw).hexdigest(),tuple(module.path_matches(name,m['path']) for m in path_rules))
    if key not in cache:
        cache[key] = [vars(f) for f in module.detect(name,raw.decode(errors='ignore'),matchers)]
    findings.extend({**f,'path':name} for f in cache[key])


def identity(obj,domain,field):
    body = {k:v for k,v in obj.items() if k!=field}
    raw = json.dumps(body,sort_keys=True,separators=(',',':'),ensure_ascii=False).encode()
    return hashlib.sha256(domain.encode()+b'\0'+raw).hexdigest() == obj[field]


def source(obj,raw):
    return (obj['schema']=='mpk.go.source_object.v1' and len(raw)==obj['bytes']
        and hashlib.sha256(raw).hexdigest()==obj['raw_sha256']
        and identity(obj,'MPK-GO-SOURCE-OBJECT-1.0','id'))


decoded = {'whole_original_bytes':0,'whole_TypeSpec_bytes':0,'whole_Type_bytes':0,'tag_bytes_hex':0}
invalid_source_records = []


def arrays(value,name):
    if isinstance(value,list):
        for n,item in enumerate(value):
            arrays(item,name+f'[{n}]')
    if not isinstance(value,dict):
        return
    for key in decoded:
        if key not in value:
            continue
        encoded = value[key]
        if key=='tag_bytes_hex':
            raw = bytes.fromhex(encoded)
        else:
            assert isinstance(encoded,list) and all(type(x) is int and 0<=x<256 for x in encoded)
            raw = bytes(encoded)
        scan(name+'#decoded-'+key,raw)
        decoded[key] += 1
        if key=='whole_original_bytes' and 'object' in value and not source(value['object'],raw):
            # This actual negative candidate intentionally changes the raw
            # buffer while retaining the old original object, and must reject.
            assert name.endswith('modified-original-buffer.json')
            invalid_source_records.append(name)
    for key,item in value.items():
        if key not in decoded:
            arrays(item,name+'/'+key)


if not REPORT.exists():
    REPORT.write_text(json.dumps({'status':'running','basis_commit':BASE})+'\n')
selected = git('diff','--name-only','-z','HEAD','--')+git('ls-files','--others','--exclude-standard','-z')
paths = sorted(set(selected.decode().split('\0'))-{'',str(REPORT.relative_to(ROOT))})
for name in paths:
    p = ROOT/name
    assert p.is_file() and not p.is_symlink()
    raw = p.read_bytes()
    inputs.append(seal(name,raw))
    if name not in configured:
        scan(name,raw)
    if p.suffix=='.md':
        for target in re.findall(r'(?<!!)\[[^\]]*\]\(([^\n)]+)\)',raw.decode()):
            target = target.strip().strip('<>')
            if re.match(r'^[a-zA-Z][a-zA-Z0-9+.-]*:',target) or target.startswith('#'):
                continue
            path = urllib.parse.unquote(target.split('#',1)[0])
            links.append({'document':name,'target':target,'exists':(p.parent/path).resolve().exists()})

archives = [(p,json.loads(p.read_bytes())) for p in sorted(OUT.glob('*/inputs.json')) if (p.parent/'run-checks.py').is_file()]
oids = sorted({r['original_git_blob'] for _,a in archives for r in a['sources'] if 'original_git_blob' in r})
raw = subprocess.check_output(['/usr/bin/git','cat-file','--batch'],input=('\n'.join(oids)+'\n').encode())
originals, cursor = {}, 0
for oid in oids:
    end = raw.index(b'\n',cursor)
    actual,kind,size = raw[cursor:end].decode().split()
    size = int(size)
    assert (actual,kind)==(oid,'blob')
    originals[oid] = raw[end+1:end+1+size]
    cursor = end+size+2
assert cursor==len(raw)
preimages = 0
for path,a in archives:
    assert seal(path.parent/'run-checks.py')==a['actual_driver_prelaunch']
    for row in a['sources']:
        buf = originals[row['original_git_blob']] if 'original_git_blob' in row else Path(row['complete_changed_input']).read_bytes()
        assert len(buf)==row['bytes'] and hashlib.sha256(buf).hexdigest()==row['sha256']
        scan(row['path'],buf)
        preimages += 1
assert len(archives)==9 and preimages==6510
executions = []
for path in sorted(OUT.glob('*/*/result.json')):
    d = json.loads(path.read_bytes())
    assert d['status'] in {'passed','failed'}
    assert d['sources']==json.loads((path.parent.parent/'inputs.json').read_bytes())['sources']
    if not d['formatter_mutation_expected']:
        assert d['whole_source_inputs_unchanged']
    for key in ['stdout','stderr']:
        assert seal(d[key]['path'])==d[key]
    executions.append({'result':seal(path),'status':d['status'],'command':d['command']})
assert len(executions)==21 and sum(e['status']=='failed' for e in executions)==4
verification = json.loads((OUT/'verification.json').read_bytes())
for name,row in verification['current_checks'].items():
    assert seal(row['path'])==row
    d=json.loads(Path(row['path']).read_bytes());assert d['status']=='passed'
    prefix='go-tools/go2vir/' if name.startswith('Go-') else 'crates/'
    for r in d['sources']:
        if not r['path'].startswith(prefix) and '/native-components/' not in r['path']:continue
        assert seal(r['path'])=={k:r[k] for k in ['path','bytes','sha256']}
        assert Path(r['path']).stat().st_mode & 0o777==r['mode']
partial=json.loads(Path(verification['retained_unaffected_duplicate_builder_test']['path']).read_bytes())
for path,marker in [('crates/mpk-vc/tests/go_practical_source_scopes.rs',b'#[test]\nfn rejects_duplicate_and_invalid_short_bindings'),('crates/mpk-vc/src/go_practical_type_keys/source_scopes.rs',b'struct Builder')]:
    row=next(r for r in partial['sources'] if r['path']==path)
    old=Path(row['complete_changed_input']).read_bytes()
    assert old.split(marker,1)[1]==Path(path).read_bytes().split(marker,1)[1]

retention=json.loads((OUT/'rust-originals/manifest.json').read_bytes())
assert retention['status']=='passed' and retention['original_files']==463 and retention['original_bytes']==118374
original_checks=[]
for row in retention['files']:
    a,b=Path(row['original']),Path(row['retained']);buf=b.read_bytes()
    assert a.read_bytes()==buf and len(buf)==row['bytes'] and hashlib.sha256(buf).hexdigest()==row['sha256']
    assert a.stat().st_mode & 0o777==b.stat().st_mode & 0o777==row['mode']
    scan(str(b),buf)
    if b.suffix=='.json':arrays(json.loads(buf),str(b))
    original_checks.append(row)
assert not invalid_source_records
read=runpy.run_path(str(OUT/'read-native.py'))['read']
native_files=native_bytes=source_objects=decoded_native=0

def native_values(value,name):
    global decoded_native
    if isinstance(value,list):
        for n,item in enumerate(value):native_values(item,name+f'[{n}]')
    if not isinstance(value,dict):return
    if 'whole_original_bytes' in value:
        buf=base64.b64decode(value['whole_original_bytes'],validate=True)
        assert source(value['object'],buf);scan(name+'#whole-original-source',buf);decoded_native+=1
    if 'whole_original_sources' in value:
        for path,encoded in value['whole_original_sources'].items():
            buf=base64.b64decode(encoded,validate=True);scan(name+'#original-'+path,buf);decoded_native+=1
    if 'canonical_components' in value:
        buf=base64.b64decode(value['canonical_components'],validate=True);scan(name+'#canonical-components',buf);decoded_native+=1
    for key in ['stdout','stderr']:
        if key in value and value[key] is not None:
            buf=base64.b64decode(value[key],validate=True);scan(name+'#'+key,buf);decoded_native+=1
    for key,item in value.items():
        if key not in {'whole_original_bytes','whole_original_sources','canonical_components','stdout','stderr'}:native_values(item,name+'/'+key)

for name in ['native-count-failure','native-dot-conflict','native-original']:
    meta,files=read(name)
    native_files+=meta['complete_original_files'];native_bytes+=meta['complete_original_bytes']
    for member,buf in files.items():
        proxy=str(OUT/name/member);scan(proxy,buf)
        if member.endswith('.json'):
            obj=json.loads(buf);native_values(obj,proxy)
            if member.startswith('original-source-objects/'):
                assert source(obj,files[member[:-4]+'raw']);source_objects+=1
    for row in meta['original_inputs']:
        p=Path(row['path']);assert p.read_bytes()==files[row['member']] and p.stat().st_mode & 0o777==row['mode']
    if name=='native-original':
        observed=json.loads(files['scope-components.json'])
        assert base64.b64decode(observed['canonical_components'],validate=True)==(OUT/'native-components/components.json').read_bytes()
        assert files['scope-uses.json']==(OUT/'native-components/uses.json').read_bytes()
assert source_objects==10979
uses=json.loads((OUT/'native-components/uses.json').read_bytes())['uses'];assert len(uses)==117
for n,row in enumerate(uses):
    expected={k:v for k,v in row.items() if k not in {'source_object_id','start_byte','end_byte'}}
    assert json.loads((OUT/'rust-originals/exact-paired'/f'query-{n:04}.json').read_bytes())==row
    assert json.loads((OUT/'rust-originals/exact-paired'/f'actual-{n:04}.json').read_bytes())==expected
    if n<90:assert (OUT/'rust-originals/original-partial'/f'actual-{n:04}.json').read_bytes()==(OUT/'rust-originals/exact-paired'/f'actual-{n:04}.json').read_bytes()
assert json.loads((OUT/'rust-originals/original-partial/actual-0090.json').read_bytes()) is None
assert not (OUT/'rust-originals/original-partial/query-0091.json').exists()
audit=json.loads((OUT/'original-byte-audit/result.json').read_bytes());assert audit['status']=='passed'
assert native_files+retention['original_files']==audit['actual_original_files_compared_literally']==22433
assert native_bytes+retention['original_bytes']==audit['actual_original_bytes_compared_literally']==174595501
assert hashlib.sha256((OUT/'original-byte-audit/driver.py').read_bytes()).hexdigest()==audit['actual_driver_prelaunch_sha256']

frozen = json.loads(Path('develop/migrations/go-practical/t02-w01/verification/native-observation/frozen-design-preservation.json').read_bytes())['inputs']
assert len(frozen)==255 and all(seal(r['path'])==r for r in frozen)
frontend = json.loads(Path(verification['unchanged_frontend_basis']['path']).read_bytes())['build_inputs']
assert len(frontend)==189
for obj in frontend:
    assert source(obj,(Path('go-tools/go2vir')/obj['path']).read_bytes())
for key in ['previous_source_declaration_basis','unchanged_parser_basis','unchanged_lexer_basis',
            'unchanged_frontend_basis','unchanged_ordinary_basis','unchanged_ordinary_paired_basis']:
    row = verification[key]
    assert seal(row['path'])==row
for name in ['crates/mpk-vc/src/go_practical_type_keys/lex.rs',
             'crates/mpk-vc/src/go_practical_type_keys/lex',
             'crates/mpk-vc/src/go_practical_type_keys/parse.rs',
             'crates/mpk-vc/src/go_practical_type_keys/parse',
             'crates/mpk-vc/src/go_practical_type_keys/validate',
             'crates/mpk-vc/tests/go_practical_parse.rs','crates/mpk-vc/tests/go_practical_lex.rs',
             'crates/mpk-vc/src/go_practical_type_keys/source_declarations.rs',
             'crates/mpk-vc/tests/go_practical_type_keys.rs',
             'crates/mpk-vc/src/go_practical_ordinary',
             'crates/mpk-vc/src/go_practical_ordinary.rs','crates/mpk-core','crates/mpk-kernel',
             'go-tools/mpk-checker-ref','proofs/std','Cargo.toml','Cargo.lock']:
    assert not git('diff','--name-only','HEAD','--',name)
old_validate=git('show',BASE+':crates/mpk-vc/src/go_practical_type_keys/validate.rs')
helper=b'pub(super) fn exported(c: char) -> bool {\n    unicode::upper(c)\n}\n'
assert Path('crates/mpk-vc/src/go_practical_type_keys/validate.rs').read_bytes().replace(helper,b'',1)==old_validate
old_root=git('show',BASE+':crates/mpk-vc/src/go_practical_type_keys.rs')
assert old_root.split(b'pub fn import_go_type_key_components',1)[1]==Path('crates/mpk-vc/src/go_practical_type_keys.rs').read_bytes().split(b'pub fn import_go_type_key_components',1)[1]
production={p.name for p in Path('go-tools/go2vir').glob('*.go') if not p.name.endswith('_test.go')}
assert production=={r['path'] for r in frontend if '/'not in r['path'] and r['path'].endswith('.go')}
for name in production:assert Path('go-tools/go2vir',name).read_bytes()==git('show',BASE+':go-tools/go2vir/'+name)
for name in ['develop/docs/go-practical-implementation-traceability-ledger.md',
             'develop/migrations/go-practical/t02-w01/README.md']:
    assert Path(name).read_bytes().startswith(git('show',BASE+':'+name))
old = json.loads(git('show',BASE+':develop/migrations/go-practical/t02-w01/requirements.json'))
latest = json.loads(Path('develop/migrations/go-practical/t02-w01/requirements.json').read_bytes())
for row in latest['requirements']:
    if row['id'] in {'T02-W01-R01','T02-W01-R06'}:
        assert row.pop('preparatory_source_scope_membership_checkpoint')['status']=='scoped_checks_passed'
assert latest==old
missing = [r for r in links if not r['exists']]
unchanged = all(seal(r['path'])==r for r in inputs)
record = {'status':'passed' if not findings and not missing and unchanged else 'failed',
    'basis_commit':BASE,'observed_at_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),
    'inputs':inputs,'source_inputs_unchanged':unchanged,'findings':findings,'missing_links':missing,
    'links_checked':len(links),'source_epochs':len(archives),'source_preimages':preimages,
    'actual_check_executions':executions,'original_files_compared_literally':native_files+len(original_checks),
    'original_bytes_compared_literally':native_bytes+sum(r['bytes'] for r in original_checks),
    'decoded_original_arrays':decoded,'decoded_native_values':decoded_native,'verified_native_source_objects':source_objects,'intentional_bad_source_records':invalid_source_records,
    'frozen_inputs_unchanged':255,'frontend_inputs_unchanged':189,
    'matcher_scope_seal_exclusion_counts':[127,4,39,51],'new_exclusions':False,
    'source_type_or_context_or_program_authority':False,'public_profile_activated':False,
    'full_T_gate_passed':False}
scan(str(REPORT.relative_to(ROOT)),json.dumps(record,indent=2).encode())
record['findings'] = findings
record['status'] = 'passed' if not findings and not missing and unchanged else 'failed'
REPORT.write_text(json.dumps(record,indent=2)+'\n')
print(json.dumps({k:record[k] for k in ['status','source_inputs_unchanged','source_preimages',
    'original_files_compared_literally','decoded_original_arrays']}))
print(json.dumps({'findings':findings[:10],'missing_links':missing[:10]}))
raise SystemExit(record['status']!='passed')
