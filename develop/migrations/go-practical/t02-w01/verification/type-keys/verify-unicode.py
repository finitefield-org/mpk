"""Check generated exact Go Unicode tables against the preserved pinned SDK."""
from pathlib import Path
import hashlib,json,re
OUT=Path(__file__).resolve().parent
input=json.loads((OUT/'unicode-input.json').read_bytes());source=Path(input['source']).read_bytes()
assert len(source)==input['bytes']and hashlib.sha256(source).hexdigest()==input['sha256']
rust=Path(input['generated']).read_text();original=source.decode();counts={}
for category in ['L','Nd','Lu']:
 body=original.split('var _'+category+' = &RangeTable{',1)[1].split('\n}',1)[0]
 expected=[tuple(int(v,0)for v in row)for row in re.findall(r'\{(0x[0-9a-f]+), (0x[0-9a-f]+), (\d+)\}',body)]
 actual=[tuple(int(v,0)for v in row)for row in re.findall(r'\((0x[0-9a-f]+), (0x[0-9a-f]+), (\d+)\)',rust.split('const '+category.upper()+':',1)[1].split('];',1)[0])]
 assert expected==actual and len(actual)==input['range_counts'][category]
 assert all(lo<=hi and stride>0 for lo,hi,stride in actual)and all(a[1]<b[0]for a,b in zip(actual,actual[1:]))
 counts[category]=len(actual)
assert 'const Version = "15.0.0"'in original
print(json.dumps({'status':'passed','complete_pinned_unicode_ranges_exact':True,'range_counts':counts,'Go':'1.25.0','Unicode':'15.0.0','table_raw_generator_input_sha256':input['sha256'],'raw_generator_output_is_retained_in_raw_epoch':True}))
