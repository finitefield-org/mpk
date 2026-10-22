"""Read every byte and original mode, including retained mode0600 trees.

Archives are transport only. Their contents retain original logical paths for
unchanged scoped review; no source text is hidden by the encoding.
"""
from pathlib import Path
import functools,json,runpy
ROOT=Path(__file__).resolve().parent
REPO=ROOT.parents[5]
@functools.lru_cache(None)
def index():
    ledger=json.loads((ROOT/'native-original-relocations.json').read_bytes())
    assert ledger['status']=='passed' and len(ledger['groups'])==3
    read=runpy.run_path(str(ROOT/'read-originals.py'))['read']
    result={}
    for group in ledger['groups']:
        manifest,files=read(group['archive'])
        assert len(files)==group['files'] and sum(map(len,files.values()))==group['bytes']
        for row in manifest['original_inputs']:
            path=Path(row['path']).resolve();assert path not in result
            result[path]=(files[row['member']],row['mode'],group['archive'],row['member'])
    return result

def kept(path):
    path=Path(path)
    if path.is_file():return path.read_bytes(),path.stat().st_mode&0o777
    raw,mode,_,_=index()[path.resolve()]
    return raw,mode
