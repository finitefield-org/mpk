"""Compare every preserved execution original literally, including partial output."""
from pathlib import Path
import base64,datetime,hashlib,json,runpy
OUT=Path(__file__).resolve().parent
RUN=OUT/'original-byte-audit';RUN.mkdir(exist_ok=False);(RUN/'driver.py').write_bytes(Path(__file__).read_bytes())
read=runpy.run_path(str(OUT/'read-originals.py'))['read'];count=size=0;rows=[];originals={}
for name in ['native-original','importer-build-failure','importer-original','rust-partial','rust-original']:
 manifest,files=read(name);originals[name]=files
 for row in manifest['original_inputs']:
  p=Path(row['path']);raw=p.read_bytes();assert raw==files[row['member']]and p.stat().st_mode&0o777==row['mode'];count+=1;size+=len(raw)
 rows.append({'name':name,'files':manifest['complete_original_files'],'bytes':manifest['complete_original_bytes']})
assert count==11339 and size==88718325 and not originals['importer-build-failure']
rust=originals['rust-original'];native=originals['native-original']
for group,number in [('previous',5),('current',7)]:
 base=OUT/'native-components'if group=='current'else OUT.parent/'type-keys/native-components'
 for n in range(number):
  case=base/f'case-{n:02}';before=json.loads(rust[f'{group}-{n:02}-before.json'])
  wire=(case/'components.json').read_bytes();assert before['components']==json.loads(wire)
  for source in before['sources']:
   obj=source['object'];raw=bytes(source['whole_original_bytes']);assert raw==(case/'sources'/(obj['id']+'.raw')).read_bytes()
  actual=json.loads(rust[f'{group}-{n:02}-actual.json']);assert actual['source_typing_or_context_or_program_authority']is False
  if group=='current':
   observed=json.loads(native[f'source-types-{n:02}.json']);assert base64.b64decode(observed['canonical_components'],validate=True)==wire
   for source in observed['sources']:
    assert base64.b64decode(source['raw'],validate=True)==(case/'sources'/(source['object']['id']+'.raw')).read_bytes()
for label in ['array-length','array-child','field-order','field-tag','embedded-marker','alias-target','nominal-underlying']:
 assert json.loads(rust[label+'-actual.json'])=={'error':'Mismatch'}
assert json.loads(rust['context-mismatch-actual.json'])=={'source_error':'Locator','package_error':'Scope'}
assert 'previous-01-before.json'in originals['rust-partial']and'previous-01-actual.json'not in originals['rust-partial']
assert 'embedded-marker-before.json'in originals['rust-partial']and'embedded-marker-actual.json'not in originals['rust-partial']
record={'status':'passed','observed_at_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'actual_driver_prelaunch_sha256':hashlib.sha256((RUN/'driver.py').read_bytes()).hexdigest(),'archives':rows,'actual_original_files_compared_literally':count,'actual_original_bytes_compared_literally':size,'all_original_modes_compared':True,'actual_native_streams':7,'actual_paired_streams':12,'all_observed_components_and_source_bytes_equal_execution_originals':True,'fully_rehashed_source_body_controls':7,'context_mismatch_controls':2,'partial_or_failed_original_output_namespaces_preserved_without_missing_output_reconstruction':True,'not_recreated_or_recompiled':True}
(RUN/'result.json').write_text(json.dumps(record,indent=2)+'\n');print(json.dumps(record))
