"""Complete the real Rust bundle using the original runtime-image search rules."""
from pathlib import Path
import json
import os
import shutil
import subprocess
import sys
import time
import traceback

ROOT = Path(__file__).resolve().parent
REPO = ROOT.parents[5]
VOLUME = Path('/private/tmp/mpk-go-installed-205-fs')
RETRY = VOLUME/'retry'
WORK = VOLUME/'finalize-rust'
OUTPUT = VOLUME/'originals/registered'
sys.dont_write_bytecode = True
sys.path.insert(0, str(ROOT))
from run_scoped import seal, write
sys.path.insert(0, str(REPO/'scripts'))
import rust_build_inputs as rust
import successor_release_bundles as release


def main():
    out = ROOT/'rust-finalized-bundles'
    out.mkdir(exist_ok=False)
    WORK.mkdir(exist_ok=False)
    assert not OUTPUT.exists()
    prior = json.loads((ROOT/'rust-retry-reproduction/result.json').read_bytes())
    phases = json.loads((ROOT/'rust-retry-reproduction/phases.json').read_bytes())
    assert prior['status'] == 'failed' and prior['consumed_inputs_unchanged']
    assert next(row for row in phases if row['name']=='original-frozen-rust-frontend-build')['status'] == 'passed'
    registry, candidates = release.validate_release_models()
    candidate = candidates['rust']
    source_toolchain = RETRY/'original-build-inputs/toolchain'
    source_release = RETRY/'writable/target/x86_64-unknown-linux-gnu/release'
    inputs = {Path(__file__).resolve(), ROOT/'run_scoped.py', REPO/'scripts/rust_build_inputs.py', REPO/'scripts/successor_release_bundles.py', REPO/'scripts/release_bundles.py', REPO/'develop/specs/vectors/rust-build-inputs-v0.json', ROOT/'rust-retry-reproduction/result.json', ROOT/'rust-retry-reproduction/phases.json', ROOT/'rust-retry-reproduction/commands.json'}
    inputs.update(p for p in source_toolchain.rglob('*') if p.is_file())
    inputs.update(source_release/name for name in ['rust2vir','rust2vir-driver'])
    inputs.update(p for p in (REPO/'release/bundles').rglob('*.json'))
    before = [seal(p) for p in sorted(inputs)]
    write(out/'inputs.json', {'inputs':before, 'selection_reason':'Keep the successful whole frozen Rust build and all original toolchain files. Reapply the existing runtime projection search over /lib and /usr/lib in declared order, with equal duplicate bytes, then the unchanged portable C++ normalization. Validate every file in both registered Rust bundles. This repairs only the prior observer path assumption and is not native Linux release acceptance.'})
    report = {'status':'running', 'inputs':seal(out/'inputs.json'), 'native_Linux_release_acceptance':False, 'full_T_gate':'Deferred to final T02-W04: ./scripts/check-fast.sh'}
    write(out/'result.json', report)
    calls=[]
    original_run=subprocess.run

    def observed(*args, **kwargs):
        result=original_run(*args, **kwargs)
        index=len(calls);row={'command':list(map(str,args[0])), 'exit_code':result.returncode}
        for name in ['stdout','stderr']:
            path=out/f'command-{index:02d}-{name}.log'
            path.write_bytes(getattr(result,name) or b'')
            row[name]=seal(path)
        calls.append(row);write(out/'commands.json',calls)
        return result

    subprocess.run=observed
    start=time.monotonic()
    try:
        vector=rust.load_vector()
        view=rust.docker_image_view(rust.RUNTIME_IMAGE, WORK/'runtime-image')
        cache=WORK/'cache';cache.mkdir()
        shutil.copytree(source_toolchain, cache/'toolchain', copy_function=shutil.copy2)
        target=WORK/'target/x86_64-unknown-linux-gnu/release';target.mkdir(parents=True)
        for name in ['rust2vir','rust2vir-driver']:
            shutil.copy2(source_release/name,target/name)
        projection=vector['production_projection']['native_runtime_projection']
        for item in candidate['toolchain_bundles'][0]['inventory']['files']:
            if not item['path'].startswith('native-runtime/'):
                continue
            relative=item['path'].removeprefix('native-runtime/').replace('libstdcxx.so.6','libstdc++.so.6')
            if relative == 'lib64/ld-linux-x86-64.so.2':
                source=projection['interpreter_source'].lstrip('/')
            else:
                soname=Path(relative).name
                matches=[]
                for directory in projection['soname_source_directories']:
                    name=f"{directory.lstrip('/')}/{soname}"
                    try:
                        matches.append((name,view.resolve(name)))
                    except rust.RustBuildFailure:
                        continue
                assert matches and all(rust.same_file_bytes(matches[0][1][1], other[1][1]) for other in matches[1:])
                source=matches[0][0]
            view.copy_file(source,cache/'native-runtime'/relative,executable=item['executable'])
        rust.normalize_portable_cpp_runtime(cache,WORK/'target')
        for item in candidate['frontend_bundles'][0]['inventory']['files']:
            release.copy_described(target/Path(item['path']).name,OUTPUT/'frontend'/item['path'],item)
        for item in candidate['toolchain_bundles'][0]['inventory']['files']:
            relative=item['path']
            source=cache/relative if relative.startswith('native-runtime/') else cache/'toolchain'/relative
            release.copy_described(source,OUTPUT/'toolchain'/relative,item)
        for name,bundle in [('frontend',candidate['frontend_bundles'][0]),('toolchain',candidate['toolchain_bundles'][0])]:
            release.normalize_directories(OUTPUT/name)
            release.validate_root(OUTPUT/name,bundle)
        report.update(status='passed', frontend=str(OUTPUT/'frontend'), toolchain=str(OUTPUT/'toolchain'), complete_frontend=candidate['frontend_bundles'][0]['inventory'], complete_toolchain=candidate['toolchain_bundles'][0]['inventory'])
    except BaseException as error:
        report.update(status='failed',error=repr(error),traceback=traceback.format_exc())
        raise
    finally:
        subprocess.run=original_run
        report.update(seconds=time.monotonic()-start, consumed_inputs_unchanged=before==[seal(p) for p in sorted(inputs)])
        write(out/'result.json',report)
    print(json.dumps({key:report[key] for key in ['status','seconds','consumed_inputs_unchanged','frontend','toolchain']}))


if __name__ == '__main__':
    main()
