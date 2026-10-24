"""Restore every unchanged original native lexer/parser member, not smoke subsets."""
from pathlib import Path
import hashlib,io,json,tarfile
ROOT=Path(__file__).resolve().parent
def seal(path):
    raw=path.read_bytes();return {'path':str(path),'bytes':len(raw),'sha256':hashlib.sha256(raw).hexdigest(),'mode':path.stat().st_mode&0o777}
def main():
    results=[]
    for name in ['lex','parse']:
        source=ROOT.parent/('go-'+name)/'native-original';target=Path('/private/tmp/mpk-go-package-files-185-'+name);target.mkdir(exist_ok=False)
        manifest=json.loads((source/'manifest.json').read_bytes());parts=[];inputs=[seal(source/'manifest.json')]
        for row in manifest['chunks']:
            path=source/Path(row['encoded']['path']).name;encoded=path.read_bytes()
            assert len(encoded)==row['encoded']['bytes'] and hashlib.sha256(encoded).hexdigest()==row['encoded']['sha256']
            raw=bytes.fromhex(encoded.decode());assert len(raw)==row['raw_bytes'] and hashlib.sha256(raw).hexdigest()==row['raw_sha256'];parts.append(raw);inputs.append(seal(path))
        raw=b''.join(parts);assert len(raw)==manifest['archive_bytes'] and hashlib.sha256(raw).hexdigest()==manifest['archive_sha256']
        expected={r['member']:r for r in manifest['original_inputs']};restored=[]
        with tarfile.open(fileobj=io.BytesIO(raw),mode='r:xz') as packed:
            for member in packed:
                assert member.isfile() and member.name in expected and member.name not in restored
                relative=Path(member.name);assert not relative.is_absolute() and '..' not in relative.parts
                value=packed.extractfile(member).read();row=expected[member.name]
                assert len(value)==row['bytes'] and hashlib.sha256(value).hexdigest()==row['sha256'] and member.mode==row['mode']
                path=target/relative;path.parent.mkdir(parents=True,exist_ok=True)
                with path.open('xb') as output:output.write(value)
                path.chmod(row['mode']);restored.append(member.name)
        assert set(restored)==set(expected)
        results.append({'corpus':name,'target':str(target),'inputs':inputs,'complete_original_members':len(restored),'complete_original_bytes':sum(r['bytes'] for r in expected.values()),'case_directories':len(list(target.glob('case-*'))),'source_or_native_results_regenerated':False})
    result={'status':'passed','driver':seal(Path(__file__).resolve()),'corpora':results,'full_T_gate':'Deferred to final T02-W04: ./scripts/check-fast.sh'}
    with (ROOT/'restored-corpora.json').open('x') as out:json.dump(result,out,indent=2);out.write('\n')
    print(json.dumps({'status':'passed','corpora':[{k:r[k] for k in ['corpus','complete_original_members','case_directories']} for r in results]}))
if __name__=='__main__':main()
