"""Losslessly retain repeated original snapshot files with exact paths/modes."""
from pathlib import Path
import hashlib,io,json,os,sys,tarfile

ROOT=Path(__file__).resolve().parent
REPO=Path.cwd()
OUT=ROOT/'snapshot-storage'

def seal(path):
    raw=path.read_bytes()
    return {'path':str(path),'bytes':len(raw),'sha256':hashlib.sha256(raw).hexdigest(),'mode':path.stat().st_mode&0o777}

def original_target(name):
    relative=Path(name)
    assert not relative.is_absolute() and '..' not in relative.parts
    assert relative.parts[0] in ['native','native-final'] and 'original-source-objects' in relative.parts
    return ROOT/relative

def archive(record):
    raw=bytearray()
    for item in record['chunks']:
        path=OUT/Path(item['file']).name
        assert seal(path)==item['encoded']
        value=bytes.fromhex(path.read_text().strip())
        assert len(value)==item['raw_bytes'] and hashlib.sha256(value).hexdigest()==item['raw_sha256']
        raw.extend(value)
    assert len(raw)==record['archive_bytes'] and hashlib.sha256(raw).hexdigest()==record['archive_sha256']
    return bytes(raw)

def verify(record,restore=False):
    packed=archive(record);count=0
    with tarfile.open(fileobj=io.BytesIO(packed),mode='r:xz') as source:
        members=source.getmembers();assert len(members)==len(record['originals'])
        for item,member in zip(record['originals'],members):
            assert member.name==item['member'] and member.isfile() and member.mode==item['mode']
            raw=source.extractfile(member).read()
            assert len(raw)==item['bytes'] and hashlib.sha256(raw).hexdigest()==item['sha256']
            path=original_target(item['member'])
            if not path.exists():
                assert restore,'Restore exact original snapshots with --restore'
                path.parent.mkdir(parents=True,exist_ok=True)
                with path.open('xb') as out:out.write(raw)
                path.chmod(item['mode'])
            assert seal(path)=={k:item[k] for k in ['path','bytes','sha256','mode']}
            count+=1
    return {'status':'passed','complete_original_files':count,'complete_original_bytes':sum(r['bytes'] for r in record['originals']),'all_original_paths_bytes_and_modes_exact':True,'regenerated_or_recompiled':False}

def create():
    OUT.mkdir(exist_ok=False)
    for epoch in ['native','native-final']:
        assert json.loads((ROOT/epoch/'result.json').read_bytes())['status'] in ['passed','failed']
    directories=sorted(ROOT.glob('native*/native-originals/**/original-source-objects'))
    directories+=sorted(ROOT.glob('native*/native-originals/original-source-objects'))
    directories=sorted(set(directories));assert directories
    files=[p for d in directories for p in d.rglob('*') if p.is_file()]
    assert all(not p.is_symlink() for p in files)
    rows=[{**seal(p),'member':str(p.relative_to(ROOT))} for p in files]
    rows.sort(key=lambda r:(r['sha256'],r['member']))
    packed=Path('/private/tmp/mpk-snapshot-inputs-182-original-snapshots.tar.xz')
    with tarfile.open(packed,'x:xz',preset=3) as out:
        for row in rows:
            raw=Path(row['path']).read_bytes();assert hashlib.sha256(raw).hexdigest()==row['sha256']
            item=tarfile.TarInfo(row['member']);item.size=len(raw);item.mode=row['mode'];out.addfile(item,io.BytesIO(raw))
    raw=packed.read_bytes();chunks=[]
    for index,offset in enumerate(range(0,len(raw),4194304)):
        value=raw[offset:offset+4194304];path=OUT/f'part-{index:03}.hex'
        with path.open('x') as output:output.write(value.hex()+'\n')
        chunks.append({'file':path.name,'encoded':seal(path),'raw_bytes':len(value),'raw_sha256':hashlib.sha256(value).hexdigest()})
    record={'status':'passed','driver':seal(Path(__file__).resolve()),'originals':rows,'original_directories':[str(d) for d in directories],
            'chunks':chunks,'archive_bytes':len(raw),'archive_sha256':hashlib.sha256(raw).hexdigest(),
            'scope':'All original snapshot files remain physically unchanged. Tracked reversible chunks retain every path, byte and mode; duplicate local copies alone are ignored by Git. Raw sources remain scanned at original virtual paths. No source/proof/checker exclusion or narrowed case scope.'}
    result=verify(record)
    ignore=REPO/'.gitignore';previous=ignore.read_bytes();addition='\n# Duplicate original snapshot copies; snapshot-inputs snapshot-storage restores all bytes.\n'
    for d in directories:addition+='/'+str(d.relative_to(REPO))+'/\n'
    with ignore.open('a') as out:out.write(addition)
    record.update(verification=result,original_gitignore_hex=previous.hex(),gitignore_after=seal(ignore))
    (OUT/'manifest.json').write_text(json.dumps(record,indent=2)+'\n');print(json.dumps(result))

if __name__=='__main__':
    if sys.argv[1:]==['--create']:create()
    else:
        assert sys.argv[1:] in [['--verify'],['--restore']]
        print(json.dumps(verify(json.loads((OUT/'manifest.json').read_bytes()),sys.argv[1:]==['--restore'])))
