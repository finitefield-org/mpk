from pathlib import Path
import hashlib,json,shutil
OUT=Path(__file__).resolve().parent;target=Path('/private/tmp/mpk-go-lex-input-20261021');target.mkdir(exist_ok=False);(target/'sources').mkdir()
original=Path('/private/tmp/mpk-go-type-keys-formatted-go-selected-native/original-source-objects');sdk=Path('/private/tmp/mpk-go-practical-original-packages-sdk-20261021/go');inputs=[];ids=[]
def seal(p):
 raw=Path(p).read_bytes();return {'path':str(p),'bytes':len(raw),'sha256':hashlib.sha256(raw).hexdigest()}
for path in sorted(original.glob('*.json')):
 object=json.loads(path.read_bytes())
 if object['kind']!='go_file':continue
 raw=path.with_suffix('.raw').read_bytes();assert len(raw)==object['bytes']and hashlib.sha256(raw).hexdigest()==object['raw_sha256'];id=object['id'];ids.append(id)
 for src,suffix in [(path,'.json'),(path.with_suffix('.raw'),'.raw')]:
  dst=target/'sources'/(id+suffix);dst.write_bytes(src.read_bytes());inputs.append({'original':seal(src),'actual':seal(dst)})
source=sdk/'src/go/scanner/scanner_test.go';(target/'scanner_test.go').write_bytes(source.read_bytes());inputs.append({'original':seal(source),'actual':seal(target/'scanner_test.go')})
manifest={'sources':ids,'scanner_test':'scanner_test.go'};(target/'manifest.json').write_text(json.dumps(manifest,indent=2)+'\n')
record={'status':'prepared_original_bytes','input_directory':str(target),'parent_original_archive':'develop/migrations/go-practical/t02-w01/verification/type-keys/native/manifest.json','pinned_whole_sdk_inventory':'develop/migrations/go-practical/t02-w01/verification/array-equality/native-go/sdk-inputs.json','complete_inputs':inputs,'manifest':seal(target/'manifest.json'),'source_objects':len(ids)}
(OUT/'corpus-inputs.json').write_text(json.dumps(record,indent=2)+'\n');print(json.dumps({'status':record['status'],'source_objects':len(ids)}))
