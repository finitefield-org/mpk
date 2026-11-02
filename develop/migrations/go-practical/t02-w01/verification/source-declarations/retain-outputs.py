"""Retain actual emitted bytes and modes; compare each original literally."""
from pathlib import Path
import datetime, hashlib, json

OUT = Path(__file__).resolve().parent
original = Path('/private/tmp/mpk-go-source-declarations-formatted-rust-outputs')
destination = OUT / 'rust-originals'
destination.mkdir(exist_ok=False)
rows = []
for p in sorted(original.rglob('*')):
    assert not p.is_symlink()
    if not p.is_file():
        continue
    raw = p.read_bytes()
    mode = p.stat().st_mode & 0o777
    target = destination / p.relative_to(original)
    target.parent.mkdir(parents=True, exist_ok=True)
    target.write_bytes(raw)
    target.chmod(mode)
    assert target.read_bytes() == raw and target.stat().st_mode & 0o777 == mode
    rows.append({'original': str(p), 'retained': str(target), 'mode': mode,
                 'bytes': len(raw), 'sha256': hashlib.sha256(raw).hexdigest()})
record = {'status': 'passed', 'observed_at_utc': datetime.datetime.now(datetime.timezone.utc).isoformat(),
          'actual_driver_prelaunch': str(OUT / 'original-retention' / 'driver.py'),
          'original_files': len(rows), 'original_bytes': sum(r['bytes'] for r in rows),
          'all_full_original_bytes_and_modes_compared_literally': True, 'files': rows}
(destination / 'manifest.json').write_text(json.dumps(record, indent=2) + '\n')
print(json.dumps({k:record[k] for k in ['status','original_files','original_bytes']}))
