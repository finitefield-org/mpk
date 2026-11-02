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

def main():
    out=ROOT/'inspection-header-final';out.mkdir(exist_ok=False)
    inputs={Path(__file__).resolve(),ROOT/'native/result.json',ROOT/'native-final/result.json',ROOT/'rust-header-final/result.json',ROOT.parent/'source-inputs/corpus.json'}
    for name in ['native/native-originals','native-final/native-originals','rust-header-final/observations','spec-reference']:
        inputs.update(p for p in (ROOT/name).rglob('*') if p.is_file())
    rows=[seal(p) for p in sorted(inputs)]
    (out/'driver.py').write_bytes(Path(__file__).read_bytes())
    (out/'prelaunch.json').write_text(json.dumps({'sources':rows,'driver':seal(out/'driver.py'),'command':['python3','-B',str(Path(__file__).resolve())]},indent=2)+'\n')
    start=time.monotonic()
    for name in ['native','native-final','rust-header-final']:
        result=json.loads((ROOT/name/'result.json').read_bytes());assert result['status']=='passed' and result['exit_code']==0
    owned=ROOT/'native/native-originals/owned'
    checked=packet(owned,'language-owned-inputs.json');assert checked['source_objects']==4 and checked['complete_captured_overlays']==0
    corpus=json.loads((ROOT/'native-final/native-originals/probe/language-corpus.json').read_bytes())
    observations=json.loads((ROOT/'rust-header-final/observations/original-language-comparison.json').read_bytes())
    assert corpus['original_compiler']=='go1.25.0' and not corpus['native_type_facts_enter_reader'] and not corpus['application_bodies_executed']
    assert len(corpus['cases'])==len(observations['rows'])==150
    native_accepted=0
    for original,observed in zip(corpus['cases'],observations['rows']):
        assert original['name']==observed['name']
        component=json.loads(bytes.fromhex(original['components_hex']))
        assert identity('MPK-GO-TYPE-COMPONENTS-2.0',component,'content_sha256')==component['content_sha256']
        assert component['source_objects']==[r['object'] for r in original['sources']]
        assert component['declarations']==component['aliases']==component['underlying']==[] and len(component['types'])==1
        for r in original['sources']:source(r['object'],bytes.fromhex(r['raw_hex']))
        if original['metadata_accepted']:
            native_accepted+=1
            for a,b in [('module_requirement','module'),('build_requirement','build'),('effective_version','effective'),('language_version','language')]:
                assert original[a]==observed[b]
        else:assert observed['error'] and observed['source'] and observed['offset']>=0
    assert native_accepted==138
    predecessor=json.loads((ROOT/'native/native-originals/probe/language-corpus.json').read_bytes())
    by_name={row['name']:row for row in corpus['cases']}
    assert len(predecessor['cases'])==132 and all(row==by_name[row['name']] for row in predecessor['cases'])
    events_final=[json.loads(l) for l in (ROOT/'native-final/stdout.log').read_text().splitlines()]
    assert sum(e['Action']=='pass' and bool(e.get('Test')) for e in events_final)==1 and not any(e['Action']=='fail' for e in events_final)
    versions=json.loads((owned/'language-owned-versions.json').read_bytes())
    original=json.loads((owned/'language-owned-inputs.json').read_bytes())
    files={r['object']['path']:r for r in original['sources']}
    assert set(files)=={'go.mod','policy.go','modern.go','legacy.go'}
    expected={'policy.go':'go1.25.0','modern.go':'go1.22','legacy.go':'go1.21'}
    assert {p:versions['files'][r['object']['id']] for p,r in files.items() if p!='go.mod'}==expected
    assert b'//go:build go1.22' in bytes.fromhex(files['modern.go']['raw_hex'])
    assert b'//go:build go1.21' in bytes.fromhex(files['legacy.go']['raw_hex'])
    assert b'func Unused(c chan int,f func(int)int)' in bytes.fromhex(files['policy.go']['raw_hex'])
    assert b'func init(){panic(' in bytes.fromhex(files['policy.go']['raw_hex'])
    selection=json.loads((ROOT/'rust-header-final/observations/owned-language-selection.json').read_bytes())
    assert selection['status']=='passed' and selection['checked_root_and_transitive_bodies']==2 and selection['unselected_old_file_retained']
    assert {r['selected'] for r in selection['old_root_and_callee_rejected']}=={'Legacy','CallsLegacy'}
    for r in selection['old_root_and_callee_rejected']:
        assert r['origin']==files['legacy.go']['object']['id']
        raw=bytes.fromhex(files['legacy.go']['raw_hex']);assert raw[r['offset']:].startswith(b'func Legacy(')
    reference=json.loads((ROOT/'spec-reference/inputs.json').read_bytes());assert reference['pinned_version']=='go1.25.0'
    for r in reference['sources']:
        a=Path(r['original_path']).read_bytes();b=Path(r['reference_file']).read_bytes()
        assert a==b and len(a)==r['bytes'] and hashlib.sha256(a).hexdigest()==r['sha256']
    units=json.loads((ROOT.parent/'source-inputs/corpus.json').read_bytes())
    for unit in units['units']:
        for r in unit['input']['sources']:source(r['object'],bytes.fromhex(r['raw_hex']))
    events=[json.loads(l) for l in (ROOT/'native/stdout.log').read_text().splitlines()]
    passed=[e['Test'] for e in events if e['Action']=='pass' and e.get('Test')]
    assert len(passed)==2 and not any(e['Action']=='fail' for e in events)
    stdout=(ROOT/'rust-header-final/stdout.log').read_text()
    assert stdout.count('test result: ok. 4 passed')==2 and stdout.count('test result: ok. 3 passed')==1
    unchanged=rows==[seal(p) for p in sorted(inputs)];assert unchanged
    report={'status':'passed','prelaunch':seal(out/'prelaunch.json'),'driver':seal(out/'driver.py'),'seconds':time.monotonic()-start,'consumed_inputs_unchanged':unchanged,'packet':checked,'native_passed_roots':passed,'pinned_parser_cases':150,'native_metadata_accepted':138,'native_metadata_rejected':12,'rust_passed_roots':11,'all_132_predecessor_cases_byte_exact':True,'corrected_native_probe_roots':1,'original_source_units':len(units['units']),'exact_pinned_go_references':len(reference['sources']),'all_original_scopes_preserved':True,'scope':'Independent original byte/hash/native-membership and effective file-version comparison. Selected/transitive bodies checked; no build selection, full module, installed context, source/IR or program proof authority.'}
    (out/'result.json').write_text(json.dumps(report,indent=2)+'\n')
    print(json.dumps({'status':'passed','cases':150,'original_objects':checked['complete_original_objects'],'native_roots':2,'rust_roots':11}))

if __name__=='__main__':main()
