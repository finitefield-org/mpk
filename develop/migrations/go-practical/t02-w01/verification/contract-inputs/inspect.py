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
    out=ROOT/'inspection';out.mkdir(exist_ok=False)
    inputs={Path(__file__).resolve(),ROOT/'native-final/result.json',ROOT/'rust/result.json'}
    for name in ['native/native-originals','native-final/native-originals','rust/observations']:
        inputs.update(p for p in (ROOT/name).rglob('*') if p.is_file())
    rows=[seal(p) for p in sorted(inputs)]
    (out/'driver.py').write_bytes(Path(__file__).read_bytes())
    (out/'prelaunch.json').write_text(json.dumps({'sources':rows,'driver':seal(out/'driver.py'),'command':['python3','-B',str(Path(__file__).resolve())]},indent=2)+'\n')
    start=time.monotonic()
    for name in ['native-final','rust']:
        result=json.loads((ROOT/name/'result.json').read_bytes());assert result['status']=='passed' and result['exit_code']==0
    owned=ROOT/'native-final/native-originals/owned';empty=ROOT/'native-final/native-originals/empty'
    packets=[packet(owned,'contract-inputs.json'),packet(empty,'contract-inputs-empty.json')]
    assert [r['complete_captured_overlays'] for r in packets]==[3,0]
    first=json.loads((ROOT/'native/native-originals/contract-inputs.json').read_bytes())
    final=json.loads((owned/'contract-inputs.json').read_bytes())
    assert first['components_hex']==final['components_hex'] and first['sources']==final['sources'] and first['overlays']==final['overlays']
    damaged=json.loads((ROOT/'native-final/native-originals/damaged/damaged-contract-input.json').read_bytes())
    assert {r['control'] for r in damaged['controls']}=={'raw-bytes','missing-list-member','missing-buffer','missing-path'} and all(r['sticky'] for r in damaged['controls'])
    events=[json.loads(l) for l in (ROOT/'native-final/stdout.log').read_text().splitlines()]
    passed=[e['Test'] for e in events if e['Action']=='pass' and e.get('Test')]
    assert len(passed)==6 and not any(e['Action']=='fail' for e in events)
    rust=json.loads((ROOT/'rust/observations/actual-original-contracts.json').read_bytes())
    assert rust['status']=='passed' and len(rust['functions'])==2 and rust['unselected_overlays']==1
    unchanged=rows==[seal(p) for p in sorted(inputs)];assert unchanged
    report={'status':'passed','prelaunch':seal(out/'prelaunch.json'),'driver':seal(out/'driver.py'),'seconds':time.monotonic()-start,'consumed_inputs_unchanged':unchanged,'packets':packets,'native_passed_roots':passed,'initial_and_final_complete_source_and_overlay_bytes_equal':True,'damaged_inventory_controls':4,'scope':'Independent original byte/object/hash checks and complete actual Go package/overlay membership. Private input linkage only; not launcher, capture-context or source/program authority.'}
    (out/'result.json').write_text(json.dumps(report,indent=2)+'\n')
    print(json.dumps({'status':'passed','native_roots':len(passed),'source_and_contract_packets':len(packets),'complete_original_objects':sum(p['complete_original_objects'] for p in packets)}))

if __name__=='__main__':main()
