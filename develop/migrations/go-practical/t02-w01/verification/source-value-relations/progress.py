"""Read live observations without treating a partial report as acceptance."""
from pathlib import Path
import datetime,json,time
ROOT=Path(__file__).resolve().parent
def read(path):
    for _ in range(5):
        try:return json.loads(path.read_bytes())
        except json.JSONDecodeError:time.sleep(.05)
    return None
rust=[v for p in (ROOT/'final-large/certificates').glob('*.rust.json') if (v:=read(p)) is not None]
go=read(ROOT/'paired-large/result.json')
print(json.dumps({'at_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'Rust':{'accepted':sum(v['verdict']=='accepted' for v in rust),'rejected':sum(v['verdict']=='rejected' for v in rust),'run_status':read(ROOT/'final-large/result.json')['status']},'Go':{'accepted':sum(v['go_observation']['verdict']=='accepted' for v in go['cases']),'rejected':sum(v['go_observation']['verdict']=='rejected' for v in go['cases']),'run_status':go['status']},'total':71}))
