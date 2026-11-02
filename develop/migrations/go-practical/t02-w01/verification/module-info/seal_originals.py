"""Seal every original versioned and local application dependency input."""
from pathlib import Path
import hashlib
import json
import sys

ROOT=Path(__file__).resolve().parent
REPO=ROOT.parents[5]
sys.dont_write_bytecode=True
sys.path.insert(0,str(ROOT.parent/'installed-compiler'))
from run_scoped import seal


def main():
    dirs=[ROOT.parent/'module-observations/native/native-originals'/name for name in ['versioned-owned','modules-owned']]
    paths=[p for directory in dirs for p in directory.rglob('*') if p.is_file()]
    paths=sorted(paths); assert len(paths)>20000
    before=[seal(p) for p in paths]
    assert before==[seal(p) for p in paths]
    result={'status':'passed','selection_reason':'Independently seal complete original versioned and local application/module captures, including all SDK/source objects, before applying the shared info validator. Their original archived byte storage and prior receipts remain in the repository.','file_count':len(before),'bytes':sum(r['bytes'] for r in before),'inputs_unchanged':True,'files':before,'native_Linux_release_acceptance':False,'full_T_gate':'Deferred to final T02-W04: ./scripts/check-fast.sh'}
    (ROOT/'original-inputs.json').write_text(json.dumps(result,indent=2)+'\n')
    print(json.dumps({k:result[k] for k in ['status','file_count','bytes','inputs_unchanged']}))


if __name__=='__main__':
    main()
