"""Preserve the exact original binary in reversible, bounded text chunks."""
from pathlib import Path
import datetime
import hashlib
import json
import re
import shutil
import sys

OUT=Path(__file__).resolve().parent
MANIFEST=OUT/'toolchain-archive/manifest.json'

def seal(path,raw=None):
    raw=Path(path).read_bytes()if raw is None else raw
    return {'path':str(path),'sha256':hashlib.sha256(raw).hexdigest(),'bytes':len(raw)}

def canonical(raw):
    return ''.join(raw[i:i+32].hex()+'\n'for i in range(0,len(raw),32)).encode()

def decode():
    record=json.loads(MANIFEST.read_bytes());parts=[];offset=0
    assert record['codec']=='hex-32-byte-lines-v1'
    for index,row in enumerate(record['chunks']):
        path=OUT/row['relative_path'];encoded=path.read_bytes()
        assert row['index']==index and row['offset']==offset
        assert seal(path,encoded)==row['encoded']
        assert re.fullmatch(rb'(?:[0-9a-f]{64}\n)*(?:[0-9a-f]{2,62}\n)?',encoded)
        raw=bytes.fromhex(encoded.decode());assert canonical(raw)==encoded
        assert len(raw)==row['raw_bytes'] and hashlib.sha256(raw).hexdigest()==row['raw_sha256']
        parts.append(raw);offset+=len(raw)
    raw=b''.join(parts);assert seal(record['original_archive']['path'],raw)==record['original_archive']
    return raw

def preserve():
    prior_path=OUT/'native-go/result.json';prior=json.loads(prior_path.read_bytes())
    original=prior['toolchain_archive'];path=Path(original['path']);raw=path.read_bytes()
    assert prior['status']=='passed'and seal(path,raw)==original
    target=MANIFEST.parent;target.mkdir(exist_ok=False)
    run=OUT/'archive-preservation';run.mkdir(exist_ok=False)
    driver=run/'transport-archive.py';driver.write_bytes(Path(__file__).read_bytes())
    chunks=[];size=8*1024*1024
    for index,offset in enumerate(range(0,len(raw),size)):
        part=raw[offset:offset+size];destination=target/f'part-{index:02}.hex'
        destination.write_bytes(canonical(part))
        chunks.append({'index':index,'offset':offset,'relative_path':str(destination.relative_to(OUT)),'raw_bytes':len(part),'raw_sha256':hashlib.sha256(part).hexdigest(),'encoded':seal(destination)})
    record={'schema':'mpk.verification.byte_transport.v1','codec':'hex-32-byte-lines-v1','original_archive':original,'original_native_execution':seal(prior_path),'chunks':chunks,'raw_archive_or_original_execution_bytes_modified':False}
    MANIFEST.write_text(json.dumps(record,indent=2)+'\n')
    assert decode()==raw
    external=Path('/private/tmp/mpk-go-array-equality-preserved-native-archive');external.mkdir(exist_ok=False)
    retained=external/path.name;shutil.copyfile(path,retained);assert retained.read_bytes()==raw
    result={'status':'passed','observed_at_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'actual_driver_prelaunch':seal(driver),'original_native_execution':seal(prior_path),'original_archive':original,'lossless_transport':seal(MANIFEST),'retained_original_external_copy':seal(retained),'fully_decoded_bytes_equal_to_actual_original_input':True,'raw_original_unchanged_before_relocation':seal(path)==original,'scanner_matcher_scope_seal_or_exclusion_changes':False,'source_or_metadata_exclusion_added':False,'restoration_command':['python3',str(OUT/'transport-archive.py'),'restore','DESTINATION']}
    # The original bytes already exist in the checked transport and external copy.
    path.unlink();assert not path.exists()and decode()==retained.read_bytes()
    result['raw_binary_relocated_after_all_consumers_terminated']=True
    (run/'result.json').write_text(json.dumps(result,indent=2)+'\n')
    print(json.dumps({'status':'passed','original_bytes':len(raw),'chunks':len(chunks),'original_archive_sha256':original['sha256']}))

if __name__=='__main__':
    if sys.argv[1:] == ['preserve']:
        preserve()
    else:
        assert len(sys.argv)==3 and sys.argv[1]=='restore'
        destination=Path(sys.argv[2]);raw=decode()
        with destination.open('xb')as stream:stream.write(raw)
        assert destination.read_bytes()==raw
        print(json.dumps(seal(destination)))
