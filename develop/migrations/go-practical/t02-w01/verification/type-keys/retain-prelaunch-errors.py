"""Copy original tool-return records, never reconstruct separate process streams."""
from pathlib import Path
import hashlib,json
source=Path('/Users/kazuyoshitoshiya/.codex/sessions/2026/10/16/rollout-2026-10-16T00-43-21-01a1403b-77e8-7741-95e4-e05ed6659ed6.jsonl');out=Path(__file__).resolve().parent/'prelaunch-errors';out.mkdir(exist_ok=True)
need={b'251921',b'982ab0',b'5fecb4',b'3810a2'};matches=[];offset=0
with source.open('rb')as f:
 for number,line in enumerate(f,1):
  obj=json.loads(line);p=obj.get('payload',{})
  if obj.get('type')=='response_item'and 'output'in p and all(token in line for token in need)and b'AssertionError'in line:
   path=out/f'original-tool-output-{len(matches):02}.jsonl';path.write_bytes(line);matches.append({'path':str(path),'bytes':len(line),'sha256':hashlib.sha256(line).hexdigest(),'source_line':number,'source_byte_offset':offset,'original_record_type':p.get('type'),'original_call_id':p.get('call_id')})
  offset+=len(line)
assert len(matches)==1
record={'status':'original_records_preserved','original_transcript':str(source),'original_records':matches,'observed_prelaunch_driver_failures':4,'cause':'Independent commands raced while first creating shared whole-source preimages. Existing-but-incomplete file read failed the equality assertion before the four product consumers launched. Complete preimages were then reused and all four commands passed; the current driver serializes preimage creation with a lock.','product_tests_launched_by_failed_attempts':False,'process_streams':'The tool originally returned merged command output; the original JSONL tool return is retained byte-for-byte. No missing separate process stdout/stderr reconstructed.','retention_probe_initial_failure':'The first in-memory search required function_call_output, but this session uses custom tool-return records. The original transcript remained intact; the actual original output record was subsequently found and copied.'}
(out/'manifest.json').write_text(json.dumps(record,indent=2)+'\n');print(json.dumps({'status':record['status'],'original_records':len(matches),'prelaunch_failures':4}))
