"""Focused rejection controls around the real complete snapshot storage."""
from pathlib import Path
import hashlib,importlib.util,json,shutil,tempfile

ROOT=Path(__file__).resolve().parent
SOURCE=ROOT.parent/'snapshot-inputs'

def seal(path):
    raw=path.read_bytes()
    return {'path':str(path),'bytes':len(raw),'sha256':hashlib.sha256(raw).hexdigest(),'mode':path.stat().st_mode&0o777}

def main():
    path=ROOT/'controls.json';assert not path.exists()
    spec=importlib.util.spec_from_file_location('original_snapshot_restore',SOURCE/'preserve-originals.py')
    module=importlib.util.module_from_spec(spec);spec.loader.exec_module(module)
    record=json.loads((SOURCE/'snapshot-storage/manifest.json').read_bytes())
    inputs=[seal(SOURCE/'preserve-originals.py'),seal(SOURCE/'snapshot-storage/manifest.json')]
    inputs.extend(seal(SOURCE/'snapshot-storage'/r['file']) for r in record['chunks'])
    checked=[]
    def reject(name,fn):
        try:fn()
        except (AssertionError,IndexError):checked.append({'name':name,'rejected':True})
        else:raise AssertionError(name)
    with tempfile.TemporaryDirectory(prefix='mpk-snapshot-183-controls-',dir='/private/tmp') as temp:
        base=Path(temp);module.ROOT=base
        member=record['originals'][0]['member']
        assert module.original_target(member)==base/member
        checked.append({'name':'complete_original_relative_member_accepted','accepted':True})
        for name in ['', '/'+member,'../'+member,member.replace('native-originals/','native-originals/../',1),member.replace('native-originals/','native-originals//',1),member.replace('native-originals/','native-originals/./',1),'native/original-source-objects/'+'a'*64+'.raw',member[:-4]+'txt',member.replace('native-originals','unrelated')]:
            reject('unsafe_member:'+name,lambda name=name:module.original_target(name))
        outside=base/'outside';outside.mkdir();sentinel=outside/'sentinel';sentinel.write_bytes(b'unchanged external bytes\x00')
        (base/'native-final').symlink_to(outside,target_is_directory=True)
        reject('restore_target_symlink',lambda:module.original_target(member))
        assert sentinel.read_bytes()==b'unchanged external bytes\x00' and sorted(p.name for p in outside.iterdir())==['sentinel']
        corrupt=base/'existing.raw';row=next(r for r in record['originals'] if r['member'].endswith('.raw'))
        corrupt.write_bytes(b'corrupt existing original\x00');before=seal(corrupt)
        reject('existing_corrupt_raw_not_overwritten',lambda:module.verify_saved_file(corrupt,row));assert seal(corrupt)==before
        linked=base/'linked.raw';linked.symlink_to(corrupt)
        reject('existing_leaf_symlink',lambda:module.verify_saved_file(linked,row))
        module.OUT=base/'storage';module.OUT.mkdir()
        first=record['chunks'][0];chunk=module.OUT/first['file'];shutil.copy2(SOURCE/'snapshot-storage'/first['file'],chunk)
        with chunk.open('r+b') as out:out.seek(0);out.write(b'Z')
        before=seal(chunk);reject('encoded_chunk_corruption',lambda:module.archive(record));assert seal(chunk)==before
        module.OUT=base/'linked-storage';module.OUT.symlink_to(SOURCE/'snapshot-storage',target_is_directory=True)
        reject('storage_ancestor_symlink',lambda:module.archive(record))
        assert sentinel.read_bytes()==b'unchanged external bytes\x00'
    assert inputs==[seal(Path(r['path'])) for r in inputs]
    result={'status':'passed','driver':seal(Path(__file__).resolve()),'inputs':inputs,'controls':checked,'complete_original_manifest_members':len(record['originals']),'all_original_storage_inputs_unchanged':True,'controls_scope':'Focused restorer helper rejection controls supplement the complete original relocated restoration and the actual recompiled relocated Rust consumers. No reduced snapshot replaces either complete integration run.','full_T_gate':'Deferred to final T02-W04: ./scripts/check-fast.sh'}
    path.write_text(json.dumps(result,indent=2)+'\n');print(json.dumps({'status':'passed','controls':len(checked)}))

if __name__=='__main__':main()
