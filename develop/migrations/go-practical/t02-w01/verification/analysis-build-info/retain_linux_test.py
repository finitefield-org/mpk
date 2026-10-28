"""Retain the exact test executable produced by the local Linux build."""
from pathlib import Path
import json
import os
import shutil
import sys

ROOT = Path(__file__).resolve().parent
sys.dont_write_bytecode = True
sys.path.insert(0, str(ROOT.parent / 'installed-compiler'))
from run_scoped import seal, write


def main():
    build = ROOT.parent / 'installed-compiler/linux-test-analysis-info/result.json'
    receipt = json.loads(build.read_bytes())
    assert receipt['status'] == 'passed' and receipt['consumed_inputs_unchanged']
    source = Path(receipt['executable']['path'])
    assert seal(source) == receipt['executable'] and source.read_bytes().startswith(b'\x7fELF')
    out = ROOT / 'linux-test-retained'
    out.mkdir(exist_ok=False)
    retained = out / 'test-executable.elf'
    shutil.copyfile(source, retained)
    os.chmod(retained, receipt['executable']['mode'])
    before, copied, after = seal(source), seal(retained), seal(source)
    assert before == after and all(before[key] == copied[key] for key in ['bytes', 'sha256', 'mode'])
    result = {'status': 'passed', 'source': before, 'retained': copied, 'build_receipt': seal(build), 'driver': seal(Path(__file__).resolve()), 'source_unchanged': True, 'native_Linux_release_acceptance': False, 'full_T_gate': 'Deferred to final T02-W04: ./scripts/check-fast.sh'}
    write(out / 'result.json', result)
    print(json.dumps({'status': result['status'], 'bytes': copied['bytes'], 'sha256': copied['sha256']}))


if __name__ == '__main__':
    main()
