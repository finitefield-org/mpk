"""Retain the actual inline audit command from its original pre-execution record."""
from pathlib import Path
import hashlib,json
OUT=Path(__file__).resolve().parent;source=Path('/Users/kazuyoshitoshiya/.codex/sessions/2026/10/16/rollout-2026-10-16T00-43-21-01a1403b-77e8-7741-95e4-e05ed6659ed6.jsonl');matches=[];offset=0
with source.open('rb')as f:
 for number,line in enumerate(f,1):
  obj=json.loads(line);p=obj.get('payload',{});command=p.get('input','')
  if obj.get('type')=='response_item'and isinstance(command,str)and 'complete_original_bytes_unchanged'in command and 'for row in rows:'in command and "run=OUT/'sdk-check'"in command:
   matches.append((number,offset,line,obj.get('timestamp')))
  offset+=len(line)
assert matches
number,offset,raw,timestamp=matches[0];path=OUT/'sdk-check/actual-original-tool-call.jsonl';path.write_bytes(raw);p=OUT/'sdk-check/result.json';result=json.loads(p.read_bytes());result['actual_inline_driver_original_pre_execution_record']={'path':str(path),'bytes':len(raw),'sha256':hashlib.sha256(raw).hexdigest(),'original_task_transcript':str(source),'source_line':number,'source_byte_offset':offset,'recorded_at_utc':timestamp};result['driver_source_not_reconstructed']=True;p.write_text(json.dumps(result,indent=2)+'\n');print(json.dumps({'status':'actual_original_inline_command_preserved','bytes':len(raw)}))
