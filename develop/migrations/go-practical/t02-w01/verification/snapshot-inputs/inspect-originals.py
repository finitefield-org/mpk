"""Independently check full original object/packet and native package membership."""
from pathlib import Path
import base64,datetime,hashlib,json,subprocess,time

ROOT=Path(__file__).resolve().parent

def seal(path):
    raw=path.read_bytes()
    return {'path':str(path),'bytes':len(raw),'sha256':hashlib.sha256(raw).hexdigest(),'mode':path.stat().st_mode&0o777}

def canonical(value):return json.dumps(value,sort_keys=True,separators=(',',':'),ensure_ascii=False).encode()
def identity(domain,value,field):return hashlib.sha256(domain.encode()+b'\0'+canonical({k:v for k,v in value.items() if k!=field})).hexdigest()

def source(object,raw):
    assert set(object)=={'schema','id','capability','path','kind','raw_sha256','bytes'}
    assert object['schema']=='mpk.go.source_object.v1'
    assert len(raw)==object['bytes'] and hashlib.sha256(raw).hexdigest()==object['raw_sha256']
    assert identity('MPK-GO-SOURCE-OBJECT-1.0',object,'id')==object['id']

def objects(raw):
    decoder=json.JSONDecoder();text=raw.decode();offset=0;rows=[]
    while offset<len(text):
        while offset<len(text) and text[offset].isspace():offset+=1
        if offset==len(text):break
        row,offset=decoder.raw_decode(text,offset);rows.append(row)
    return rows

def packet(directory,name):
    record=json.loads((directory/name).read_bytes())
    envelope_raw=bytes.fromhex(record['envelope_hex']);envelope=json.loads(envelope_raw)
    assert set(envelope)=={'schema','image_sha256','compiler_sha256','main_source_object_id','requested_package','source_components_sha256','overlays','content_sha256'}
    assert envelope_raw==canonical(envelope) and envelope['schema']=='mpk.go.practical.source_contract_inputs.v1'
    assert identity('MPK-PRIVATE-GO-SOURCE-CONTRACT-INPUTS-1.0',envelope,'content_sha256')==envelope['content_sha256']
    raw=bytes.fromhex(record['components_hex']);component=json.loads(raw)
    assert raw==canonical(component) and identity('MPK-GO-TYPE-COMPONENTS-2.0',component,'content_sha256')==envelope['source_components_sha256']==component['content_sha256']
    assert component['declarations']==component['underlying']==component['aliases']==[] and len(component['types'])==1
    assert component['types'][0]['kind']=='builtin' and component['types'][0]['name']=='bool'
    captured={};original_count=original_bytes=0
    for metadata in sorted((directory/'original-source-objects').glob('*.json')):
        object=json.loads(metadata.read_bytes());data=metadata.with_suffix('.raw').read_bytes();source(object,data)
        assert metadata.stem==object['id'];captured[object['id']]=object
        original_count+=1;original_bytes+=len(data)
    assert envelope['main_source_object_id'] in captured
    originals={}
    for field in ['sources','overlays']:
        for row in record[field]:
            object=row['object'];data=bytes.fromhex(row['raw_hex']);source(object,data)
            assert captured[object['id']]==object
            assert data==(directory/'original-source-objects'/(object['id']+'.raw')).read_bytes()
            assert object['id'] not in originals;originals[object['id']]=object
    overlay_objects=[row['object'] for row in record['overlays']]
    assert overlay_objects==envelope['overlays']==sorted((o for o in captured.values() if o['kind']=='overlay'),key=lambda o:o['id'])
    assert component['source_objects']==[row['object'] for row in record['sources']]
    assert component['package_roots']==[p['id'] for p in component['packages']]
    used=set()
    native=objects(base64.b64decode(json.loads((directory/'storage-original-exports.json').read_bytes())['stdout']))
    locations=json.loads((directory/'storage-original-inputs.json').read_bytes())['roots']
    physical={str(Path(r['Directory'])/o['path']):o for r in locations for o in captured.values() if r['Capability']==o['capability']}
    native_packages={p['ImportPath']:p for p in native if p['ImportPath']!='unsafe'}
    assert {p['import_path'] for p in component['packages']}==set(native_packages)
    for package in component['packages']:
        assert identity('MPK-GO-TYPE-PACKAGE-2.0',package,'id')==package['id']
        native_package=native_packages[package['import_path']]
        assert package['package_name']==native_package['Name']
        active=sorted(physical[str(Path(native_package['Dir'])/name)]['id'] for name in native_package['GoFiles'])
        assert active==package['source_objects']
        used.update(active);used.add(package['module_source_object_id'])
    assert used=={row['object']['id'] for row in record['sources']}
    return {'packet':seal(directory/name),'complete_original_objects':original_count,'complete_original_bytes':original_bytes,'source_objects':len(record['sources']),'packages':len(component['packages']),'complete_captured_overlays':len(overlay_objects),'native_package_membership_exact':True,'raw_source_and_contract_bytes_exact':True}

def snapshot(packet,directory=None):
    import struct
    raw=bytes.fromhex(packet['snapshot_hex']);wire=json.loads(raw)
    assert raw==canonical(wire) and set(wire)=={'schema','capabilities','directories','objects','snapshot_sha256','content_sha256'}
    assert wire['schema']=='mpk.go.practical.source_snapshot_inputs.v1'
    assert identity('MPK-PRIVATE-GO-SOURCE-SNAPSHOT-INPUTS-1.0',wire,'content_sha256')==wire['content_sha256']
    h=hashlib.sha256(b'MPK-GO-PRIVATE-SOURCE-SNAPSHOT-1.0\0')
    def number(n):h.update(struct.pack('<Q',n))
    def text(raw):number(len(raw));h.update(raw)
    number(len(wire['capabilities']))
    for c in wire['capabilities']:text(c.encode())
    number(len(wire['directories']));entries=0
    for d in wire['directories']:
        text(d['capability'].encode());text(d['path'].encode());number(len(d['entries']))
        for e in d['entries']:
            text(bytes.fromhex(e['name_hex']));number(e['type_bits']);entries+=1
    number(len(wire['objects']))
    for o in wire['objects']:text(o['id'].encode())
    assert h.hexdigest()==wire['snapshot_sha256']
    assert wire['objects']==[r['object'] for r in packet['snapshot_sources']]
    total=0
    for r in packet['snapshot_sources']:
        o=r['object']
        if 'raw_hex' in r:data=bytes.fromhex(r['raw_hex'])
        else:
            assert directory is not None and r['raw_file']==o['id']+'.raw'
            data=(directory/'original-source-objects'/r['raw_file']).read_bytes()
            assert json.loads((directory/'original-source-objects'/(o['id']+'.json')).read_bytes())==o
        source(o,data);total+=len(data)
    return {'objects':len(wire['objects']),'source_bytes':total,'capabilities':len(wire['capabilities']),'directories':len(wire['directories']),'directory_entries':entries,'native_snapshot_sha256':wire['snapshot_sha256']}

def main():
    out=ROOT/'inspection-final';out.mkdir(exist_ok=False)
    inputs={Path(__file__).resolve(),ROOT/'native/result.json',ROOT/'native-final/result.json',ROOT/'rust/result.json',ROOT.parent/'source-inputs/corpus.json',ROOT.parent/'source-invocations/corpus.json'}
    for name in ['native/native-originals','native-final/native-originals','rust/observations']:
        inputs.update(p for p in (ROOT/name).rglob('*') if p.is_file())
    rows=[seal(p) for p in sorted(inputs)]
    (out/'driver.py').write_bytes(Path(__file__).read_bytes())
    (out/'prelaunch.json').write_text(json.dumps({'sources':rows,'driver':seal(out/'driver.py'),'command':['python3','-B',str(Path(__file__).resolve())]},indent=2)+'\n')
    start=time.monotonic()
    for name in ['native-final','rust']:
        result=json.loads((ROOT/name/'result.json').read_bytes());assert result['status']=='passed' and result['exit_code']==0
    namespace=ROOT/'native/native-originals/namespace'
    small=json.loads((namespace/'snapshot-inputs.json').read_bytes());checked=[snapshot(small)]
    assert checked[0]['objects']==9
    assert {r['object']['kind'] for r in small['snapshot_sources']}=={'go_file','go_mod','go_sum','overlay','metadata','export_data','excluded_source','auxiliary'}
    raw_names=json.loads((namespace/'raw-native-name-unit.json').read_bytes());checked.append(snapshot(raw_names))
    native_queries=json.loads((namespace/'namespace-lookups.json').read_bytes())['queries']
    observed=json.loads((ROOT/'rust/observations/native-lookup-comparison.json').read_bytes())['original_native_queries']
    assert len(native_queries)==len(observed)==30
    assert [(q['name'],q['outcome']) for q in native_queries]==[(q['name'],q['outcome']) for q in observed]
    original=json.loads((ROOT.parent/'source-inputs/corpus.json').read_bytes())
    originals_by_name={u['name']:u for u in original['units']}
    roots={}
    for name,filename in [('owned','capture-inputs.json'),('pure','complete-capture-inputs.json'),('wide','complete-capture-inputs.json')]:
        directory=ROOT/('native-final/native-originals' if name=='owned' else 'native/native-originals')/name
        p=json.loads((directory/filename).read_bytes());checked.append(snapshot(p,directory));packet(directory,filename)
        capture_raw=bytes.fromhex(p['capture_hex']);capture=json.loads(capture_raw)
        assert capture_raw==canonical(capture) and set(capture)=={'schema','selection_inputs_sha256','snapshot_inputs_sha256','content_sha256'}
        assert capture['schema']=='mpk.go.practical.source_capture_inputs.v1'
        assert identity('MPK-PRIVATE-GO-SOURCE-CAPTURE-INPUTS-1.0',capture,'content_sha256')==capture['content_sha256']
        selection=json.loads(bytes.fromhex(p['selection_hex']));complete=json.loads(bytes.fromhex(p['snapshot_hex']))
        assert capture['selection_inputs_sha256']==selection['content_sha256'] and capture['snapshot_inputs_sha256']==complete['content_sha256']
        assert identity('MPK-PRIVATE-GO-SOURCE-SELECTION-INPUTS-1.0',selection,'content_sha256')==selection['content_sha256']
        roots[name]=len(selection['roots'])
        if name in originals_by_name:
            prior=originals_by_name[name]['input']
            assert p['components_hex']==prior['components_hex'] and p['sources']==prior['sources']
            assert roots[name]==43
    assert roots['owned']==1
    for name in ['native-lookup-comparison.json','native-name-bytes.json','snapshot-controls.json','actual-owned-capture.json','all-original-capture-units.json']:
        assert json.loads((ROOT/'rust/observations'/name).read_bytes())['status']=='passed'
    events=[json.loads(l) for l in (ROOT/'native/stdout.log').read_text().splitlines()]
    passed=[e['Test'] for e in events if e['Action']=='pass' and e.get('Test')]
    assert {e.get('Test') for e in events if e['Action']=='fail'}=={'TestPracticalCaptureInputsKeepOriginalOwnedUniverse',None}
    final_events=[json.loads(l) for l in (ROOT/'native-final/stdout.log').read_text().splitlines()]
    assert not any(e['Action']=='fail' for e in final_events)
    expected=set()
    import re
    for path in (Path.cwd()/'go-tools/go2vir').glob('*test.go'):
        expected.update(re.findall(r'^func (TestPractical(?:Snapshot|CaptureInputs|SelectionInputs|SourceInvocationOriginalSignatures)[A-Za-z0-9_]*)\(',path.read_text(),re.MULTILINE))
    assert set(t for t in passed if '/' not in t)==expected-{'TestPracticalCaptureInputsKeepOriginalOwnedUniverse'}
    assert {'TestPracticalCaptureInputsKeepOriginalOwnedUniverse','TestPracticalStandardLibraryOriginalSourcesTypecheckWithNoFilesystemFallback'}<=set(e.get('Test') for e in final_events if e['Action']=='pass')
    rust=(ROOT/'rust/stdout.log').read_text()
    assert rust.count('test result: ok. 4 passed')==1 and rust.count('test result: ok. 5 passed')==1
    unchanged=rows==[seal(p) for p in sorted(inputs)];assert unchanged
    report={'status':'passed','prelaunch':seal(out/'prelaunch.json'),'driver':seal(out/'driver.py'),'seconds':time.monotonic()-start,'consumed_inputs_unchanged':unchanged,'snapshots':checked,'native_passed_tests':passed,'rust_passed_roots':9,'original_roots':roots,'whole_original_byte_and_native_digest_inspection':True,'scope':'Independently reconstruct exact framed native snapshot digests from every raw byte/type/object, complete source buffers and selection/capture linkage. Native namespace lookups agree. Capture authorization, installed context and application/source/IR proofs remain separate.'}
    (out/'result.json').write_text(json.dumps(report,indent=2)+'\n')
    print(json.dumps({'status':'passed','snapshots':len(checked),'rust_roots':9,'native_roots':len(expected),'native_tests_with_subtests':len(passed)}))

if __name__=='__main__':main()
