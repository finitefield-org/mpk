"""Copy only exact frozen dependency archives from the existing local cache."""
from pathlib import Path
import hashlib
import json
import os
import shutil
import sys

ROOT = Path(__file__).resolve().parent
REPO = ROOT.parents[5]
DEST = Path('/private/tmp/mpk-go-installed-205-fs/originals/downloads')
sys.dont_write_bytecode = True
sys.path.insert(0, str(ROOT))
from run_scoped import seal, write


def main():
    out = ROOT/'rust-crate-cache-seed'
    out.mkdir(exist_ok=False)
    vector_path = REPO/'develop/specs/vectors/rust-build-inputs-v0.json'
    vector = json.loads(vector_path.read_bytes())
    packages = {}
    for graph in vector['dependency_graphs'].values():
        for package in graph:
            if package.get('checksum'):
                key = (package['name'], package['version'])
                prior = packages.setdefault(key, package['checksum'])
                assert prior == package['checksum']
    source_root = Path.home()/'.cargo/registry/cache'
    rows = []
    for (name, version), digest in sorted(packages.items()):
        filename = f'{name}-{version}.crate'
        candidates = [p for p in source_root.glob(f'*/{filename}') if seal(p)['sha256'] == digest]
        assert candidates, f'no exact local cached archive: {filename}'
        source = sorted(candidates)[0]
        rows.append({'name':name, 'version':version, 'expected_sha256':digest, 'source':seal(source), 'destination':str(DEST/filename)})
    inputs = [seal(Path(__file__).resolve()), seal(ROOT/'run_scoped.py'), seal(vector_path)]
    write(out/'inputs.json', {'inputs':inputs, 'archives':rows, 'scope':'Use all exact existing local frozen crate archives; verify each full SHA-256 before and after copying. No dependency, version, checksum, source inventory or build profile changes.'})
    result = {'status':'running', 'inputs':seal(out/'inputs.json')}
    write(out/'result.json', result)
    for row in rows:
        source = Path(row['source']['path'])
        destination = Path(row['destination'])
        assert not destination.exists()
        temporary = destination.with_name(destination.name+'.local-copy')
        with source.open('rb') as input_stream, temporary.open('xb') as output_stream:
            shutil.copyfileobj(input_stream, output_stream, 1024*1024)
        assert seal(temporary)['sha256'] == row['expected_sha256'] and seal(source) == row['source']
        os.replace(temporary, destination)
        row['copied'] = seal(destination)
    result.update(status='passed', archives=rows, inputs_unchanged=inputs == [seal(p['path']) for p in inputs], full_T_gate='Deferred to final T02-W04: ./scripts/check-fast.sh')
    write(out/'result.json', result)
    print(json.dumps({'status':result['status'], 'archives':len(rows), 'inputs_unchanged':result['inputs_unchanged']}))


if __name__ == '__main__':
    main()
