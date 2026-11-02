"""Preserve terminal mode0600 originals in the existing lossless archive format.

Git cannot retain mode0600. Original physical trees are moved intact to local
temporary retention only after byte/mode/archive comparison. Every original
logical path remains indexed, read and scanned from its complete archive.
No source finding or source epoch is omitted by the archival representation.
"""
from pathlib import Path
import hashlib,json,runpy,shutil,subprocess,sys
ROOT=Path(__file__).resolve().parent
for epoch in ['native']:
    result=json.loads((ROOT/epoch/'result.json').read_bytes())
    assert result['status']=='passed' and result['whole_source_inputs_unchanged']
jobs=[('native-output-original',ROOT/'native/native-originals')]
ledger=ROOT/'native-original-relocations.json';assert not ledger.exists()
records=[]
for name,source in jobs:
    subprocess.run([sys.executable,str(ROOT/'archive-originals.py'),name,str(source)],check=True)
    read=runpy.run_path(str(ROOT/'read-originals.py'))['read']
    manifest,files=read(name)
    for row in manifest['original_inputs']:
        p=Path(row['path']);assert p.read_bytes()==files[row['member']]
        assert p.stat().st_mode&0o777==row['mode']
    retained=Path('/private/tmp')/('mpk-go-source-string-index-'+name+'-retained')
    assert not retained.exists();shutil.move(str(source),str(retained))
    for row in manifest['original_inputs']:
        p=retained/row['member'];assert p.read_bytes()==files[row['member']]
        assert p.stat().st_mode&0o777==row['mode']
    records.append({'archive':name,'original_logical_directory':str(source),'retained_physical_directory':str(retained),'files':len(files),'bytes':sum(map(len,files.values())),'all_original_bytes_and_modes_exact_before_and_after_relocation':True})
    ledger.write_text(json.dumps({'status':'preserving','reason':'Git records only regular/executable modes; original mode0600 remains in the complete archive and intact local retention. Every logical original remains part of scoped content review.','groups':records},indent=2)+'\n')
    print(json.dumps(records[-1]),flush=True)
ledger.write_text(json.dumps({'status':'passed','actual_driver_sha256':hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),'reason':'Git cannot represent original mode0600. No metadata/byte/mode normalization, source pruning, scanner exclusion or test restart. Complete virtual originals remain explicitly read and scanned with unchanged rules.','groups':records},indent=2)+'\n')
