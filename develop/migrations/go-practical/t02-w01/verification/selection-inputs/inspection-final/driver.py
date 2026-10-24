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

def selection(directory,name,observed=None):
    result=packet(directory,name)
    p=json.loads((directory/name).read_bytes())
    raw=bytes.fromhex(p['selection_hex']);v=json.loads(raw)
    assert raw==canonical(v) and set(v)=={'schema','source_contract_inputs_sha256','roots','content_sha256'}
    assert v['schema']=='mpk.go.practical.source_selection_inputs.v1'
    assert identity('MPK-PRIVATE-GO-SOURCE-SELECTION-INPUTS-1.0',v,'content_sha256')==v['content_sha256']
    base=json.loads(bytes.fromhex(p['envelope_hex']))
    assert base['content_sha256']==v['source_contract_inputs_sha256']
    component=json.loads(bytes.fromhex(p['components_hex']))
    packages={p['id']:p for p in component['packages']}
    keys=[];matched=[];by_span={}
    if observed is not None:
        for name,r in observed.items():by_span.setdefault((r['source'],r['start'],r['end']),[]).append(name)
    for root in v['roots']:
        assert set(root)=={'package_id','source_object_id','start_byte','end_byte'}
        package=packages[root['package_id']]
        assert root['source_object_id'] in package['source_objects']
        key=(root['package_id'],root['source_object_id'],root['start_byte'],root['end_byte'])
        keys.append(key)
        if observed is not None:
            possible=by_span.get((root['source_object_id'],root['start_byte'],root['end_byte']),[])
            assert len(possible)==1 and possible[0].startswith(package['import_path']+'.')
            matched.append(possible[0])
    assert 0<len(keys)<=8192 and keys==sorted(set(keys))
    return {**result,'physical_roots':len(keys),'matched_native_functions':matched,'all_original_root_fields_hashed':True}

def main():
    out=ROOT/'inspection-final';out.mkdir(exist_ok=False)
    inputs={Path(__file__).resolve(),ROOT/'native/result.json',ROOT/'native-final/result.json',ROOT/'rust/result.json',ROOT/'method-init-go-validity/result.json',ROOT.parent/'source-inputs/corpus.json',ROOT.parent/'source-invocations/corpus.json'}
    for name in ['native/native-originals','native-final/native-originals','method-init-go-validity','rust/observations']:
        inputs.update(p for p in (ROOT/name).rglob('*') if p.is_file())
    rows=[seal(p) for p in sorted(inputs)]
    (out/'driver.py').write_bytes(Path(__file__).read_bytes())
    (out/'prelaunch.json').write_text(json.dumps({'sources':rows,'driver':seal(out/'driver.py'),'command':['python3','-B',str(Path(__file__).resolve())]},indent=2)+'\n')
    start=time.monotonic()
    for name in ['native','native-final','rust']:
        result=json.loads((ROOT/name/'result.json').read_bytes());assert result['status']=='passed' and result['exit_code']==0
    directory=ROOT/'native/native-originals/selection-owned'
    native=json.loads((directory/'physical-root-observations.json').read_bytes())
    checked=[selection(directory,'selection-inputs.json',native['functions']),selection(directory,'selection-boundary-inputs.json',native['functions'])]
    assert checked[0]['physical_roots']==3 and checked[1]['physical_roots']==8192
    assert set(checked[0]['matched_native_functions'])=={'example.com/policy.Root','example.com/policy.Count.Work','example.com/policy.Other.Work'}
    assert set(checked[1]['matched_native_functions'])=={f'example.com/policy.Bound{i:04}' for i in range(8192)}
    ownership=json.loads((directory/'selection-ownership.json').read_bytes())
    assert ownership['passed'] and ownership['exact_root_count']==8192 and len(ownership['controls'])==9 and all(c['rejected'] for c in ownership['controls'])
    assert ownership['root_permutation_byte_identical'] and ownership['returned_bytes_independent'] and ownership['owner_retirement_keeps_inputs']
    originals=json.loads((ROOT.parent/'source-inputs/corpus.json').read_bytes())
    previous=json.loads((ROOT.parent/'source-invocations/corpus.json').read_bytes())
    for name in ['pure','wide']:
        directory=ROOT/'native/native-originals'/name
        checked.append(selection(directory,'physical-selection-inputs.json'))
        p=json.loads((directory/'physical-selection-inputs.json').read_bytes())
        old=next(u for u in originals['units'] if u['name']==name)
        assert p['components_hex']==old['input']['components_hex'] and p['sources']==old['input']['sources']
        unit=next(u for u in previous['units'] if u['name']==name)
        expected={(r['source'],r['start'],r['end']) for r in unit['roots']}
        v=json.loads(bytes.fromhex(p['selection_hex']))
        assert {(r['source_object_id'],r['start_byte'],r['end_byte']) for r in v['roots']}==expected
    method=ROOT/'native-final/native-originals/method-init'
    checked.append(selection(method,'selection-method-init.json'))
    actual=json.loads((method/'selection-method-init.json').read_bytes())
    original={r['object']['path']:bytes.fromhex(r['raw_hex']) for r in actual['sources']}
    for name in ['go.mod','policy.go']:
        assert original[name]==(ROOT/'method-init-go-validity/originals'/name).read_bytes()
    assert json.loads((ROOT/'method-init-go-validity/result.json').read_bytes())['status']=='passed'
    method_result=json.loads((ROOT/'rust/observations/ordinary-init-method.json').read_bytes())
    assert method_result['status']=='passed' and method_result['physical_and_named_contract_derivations_equal'] and method_result['package_initializer_rejected']
    for name in ['owned-physical-selection.json','physical-selection-controls.json','physical-selection-boundary.json','all-original-root-inputs.json']:
        assert json.loads((ROOT/'rust/observations'/name).read_bytes())['status']=='passed'
    limits=json.loads((ROOT/'rust/observations/physical-selection-boundary.json').read_bytes())
    assert limits['derived_functions']==limits['original_roots']==limits['expression_nodes']==8192 and not limits['complete_program_proof']
    controls=json.loads((ROOT/'rust/observations/physical-selection-controls.json').read_bytes())
    assert len(controls['rejections'])==15 and controls['coherent_other_root_is_new_untrusted_candidate'] and not controls['hash_is_launcher_authentication']
    events=[json.loads(l) for l in (ROOT/'native/stdout.log').read_text().splitlines()]
    passed=[e['Test'] for e in events if e['Action']=='pass' and e.get('Test')]
    assert len([t for t in passed if '/' not in t])==7 and len([t for t in passed if '/' in t])==2 and not any(e['Action']=='fail' for e in events)
    rust=(ROOT/'rust/stdout.log').read_text()
    final_events=[json.loads(l) for l in (ROOT/'native-final/stdout.log').read_text().splitlines()]
    assert sum(e['Action']=='pass' and bool(e.get('Test')) for e in final_events)==1 and not any(e['Action']=='fail' for e in final_events)
    assert rust.count('test result: ok. 4 passed')==2 and rust.count('test result: ok. 5 passed')==1 and rust.count('test result: ok. 3 passed')==1 and rust.count('test result: ok. 6 passed')==1
    unchanged=rows==[seal(p) for p in sorted(inputs)];assert unchanged
    report={'status':'passed','prelaunch':seal(out/'prelaunch.json'),'driver':seal(out/'driver.py'),'seconds':time.monotonic()-start,'consumed_inputs_unchanged':unchanged,'packets':checked,'native_passed_roots':passed,'rust_passed_roots':22,'all_original_zero_70_case_source_bytes_and_roots_unchanged':True,'exact_root_boundary':8192,'scope':'Independent complete original object/packet hashes and native source/package/function membership. Complete original scope retained. Physical-root input binding and contract candidates only; self-hashes do not authenticate capture or intent, and no installed context or application proof is claimed.'}
    (out/'result.json').write_text(json.dumps(report,indent=2)+'\n')
    print(json.dumps({'status':'passed','complete_packets':len(checked),'rust_roots':22,'native_roots':8,'native_subtests':2,'root_boundary':8192}))

if __name__=='__main__':main()
