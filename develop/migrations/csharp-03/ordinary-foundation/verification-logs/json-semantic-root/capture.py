import sys,json,hashlib
from pathlib import Path
sys.path.insert(0,'/repo/scripts')
import csharp_practical_build_inputs as practical
request=Path('/capture/requests.json').read_bytes()
first=practical.test_data_phase(requests=request)
second=practical.test_data_phase(requests=request)
assert first==second,'nondeterministic capture'
rows=json.loads(first)
assert len(rows)==5
for row in rows:
 assert 'facts' in row and 'reject' not in row,row
Path('/capture/responses.json').write_bytes(first)
Path('/capture/receipt.json').write_text(json.dumps({'request_sha256':hashlib.sha256(request).hexdigest(),'responses_sha256':hashlib.sha256(first).hexdigest(),'cases':len(rows),'identical_double_capture':True},indent=2)+'\n')
print('Five semantic-root source contexts accepted by identical offline double capture')
