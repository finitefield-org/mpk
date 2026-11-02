"""Reconstruct each exact live driver from its sealed original identity."""
from pathlib import Path
import hashlib
import json

ROOT = Path(__file__).resolve().parent
current = (ROOT / 'run_live.py').read_text()
assert current.count('preflight-retirement') == 4
retirement_check = " and owner.get('export_cache_retirement_checked_before_binding') is True"
assert current.count(retirement_check) == 1
volume = current.replace('preflight-retirement', 'preflight-control').replace(retirement_check, '')
host = volume.replace('preflight-control', 'preflight-final').replace('import uuid\n', '')
start = host.index('    benchmark = json.loads(')
end = host.index("    source = inputs['inputs']", start)
host = host[:start] + host[end:]
host = host.replace("['native-originals', 'source-coord', 'export-coord', 'combined-coord']}", "['tmp', 'native-originals', 'source-coord', 'export-coord', 'combined-coord']}", 1)
start = host.index("    volume = 'mpk-go-combined-live-'")
end = host.index("    for name in ['source-coord', 'export-coord', 'combined-coord']:", start)
common = "    common = ['/usr/local/bin/docker', 'run', '--pull=never', '--rm', '--network', 'none', '--read-only', '--cap-drop', 'ALL', '--security-opt', 'no-new-privileges', '--user', f'{os.getuid()}:{os.getgid()}', '--mount', f'type=bind,src={REPO},dst=/workspace,readonly', '--mount', f'type=bind,src={folders[\"tmp\"]},dst=/tmp', '--mount', f'type=bind,src={folders[\"native-originals\"]},dst=/evidence']\n"
host = host[:start] + common + host[end:]
start = host.index("    write(out / 'inputs.json'")
end = host.index('\n', start)
line = "    write(out / 'inputs.json', {'selection_reason': 'Open each actual source and generated export root before its capture, retain both matched root sets and bind them to the complete original export owner without replacing unread cache entries.', 'driver': seal(Path(__file__)), 'preflight_inputs': seal(ROOT / 'preflight-final/inputs.json'), 'preflight_result': seal(ROOT / 'preflight-final/result.json'), 'preflight_archive': inputs['archive'], 'go_command': go, 'rust_command': rust, 'go_environment': genv, 'rust_environment': renv, 'temporary_root': str(work)})"
host = host[:start] + line + host[end:]
out = ROOT / 'runner-history'
assert not out.exists()
out.mkdir(mode=0o700)
rows = []
for epoch, text in [('live-original', host), ('live-volume', volume), ('live-retirement', current)]:
    expected = json.loads((ROOT / epoch / 'inputs.json').read_bytes())['driver']
    raw = text.encode()
    assert len(raw) == expected['bytes'] and hashlib.sha256(raw).hexdigest() == expected['sha256']
    path = out / (epoch + '.py')
    path.write_bytes(raw)
    path.chmod(expected['mode'])
    assert path.read_bytes() == raw and path.stat().st_mode & 0o777 == expected['mode']
    rows.append({'epoch': epoch, 'stored_file': path.name, 'original': expected})
result = {'status': 'passed', 'selection_reason': 'Recover and verify the exact original driver bytes for each failed and successful live epoch, including the original path and mode.', 'drivers': rows, 'original_inputs_modified': False}
(out / 'receipt.json').write_text(json.dumps(result, indent=2) + '\n')
print(json.dumps({'status': result['status'], 'exact_drivers': len(rows)}), flush=True)
