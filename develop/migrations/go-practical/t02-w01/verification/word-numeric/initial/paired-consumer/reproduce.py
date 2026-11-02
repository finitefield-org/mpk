"""Reproduce only the over-narrow comparison assertion on the original bytes."""
from pathlib import Path
import hashlib
import json
import subprocess

root = Path(__file__).resolve().parent.parent.parent
original = json.loads((root / 'paired-final/result.json').read_text())
assert original['status'] == 'failed'
row = original['cases'][-1]
assert row['name'] == 'SignedAddition-16'
def unchanged(record):
    raw = Path(record['path']).read_bytes()
    assert len(raw) == record['bytes'] and hashlib.sha256(raw).hexdigest() == record['sha256']
for record in [row['certificate'], row['rust_report'], original['go_binary']]:
    unchanged(record)
run = subprocess.run(row['command'], stdout=subprocess.PIPE, stderr=subprocess.PIPE, check=False)
assert run.returncode == 1 and not run.stderr
assert run.stdout == Path(row['stdout']['path']).read_bytes()
rust = json.loads(Path(row['rust_report']['path']).read_text())
go = json.loads(run.stdout)
assert rust['verdict'] == go['verdict'] == 'rejected'
detail = json.loads(rust['error_detail'])
assert detail['code'] == 'CORE_TYPE_MISMATCH'
assert go['error_kind'] == 'core_check' and go['error_detail'] == 'lambda domain mismatch'
# The original comparison consumer incorrectly assumed this one subtype.
assert detail['details']['kind'] == 'check_type_mismatch', row
