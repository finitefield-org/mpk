"""Observe the existing installed loader's exact failure in a separate copy."""
from pathlib import Path
import json
import os
import shutil
import subprocess
import sys
import time

ROOT=Path(__file__).resolve().parent
REPO=ROOT.parents[5]
VOLUME=Path('/private/tmp/mpk-go-installed-205-fs')
OLD=VOLUME/'installed-test'
NEW=VOLUME/'installed-diagnostic'
IMAGE='sha256:d617f1b5c2a18778ba357ba0528e973bd24992b0dd18000dfbca059f4a15ebda'
sys.dont_write_bytecode=True
sys.path.insert(0,str(ROOT))
from run_scoped import seal,write
sys.path.insert(0,str(REPO/'scripts'))
import successor_release_bundles as release


def main():
    out=ROOT/'installed-root-diagnostic'
    out.mkdir(exist_ok=False)
    build=json.loads((ROOT/'linux-test-diagnostic/result.json').read_bytes())
    assert build['status']=='passed' and build['consumed_inputs_unchanged']
    binary=Path(build['executable']['path'])
    assert seal(binary)==build['executable']
    registry,_=release.validate_release_models()
    bundles={v['bundle_id']:v for key in ['frontend_bundles','toolchain_bundles'] for v in registry[key]}
    for key,bundle in bundles.items():release.validate_root(OLD/'libexec/mpk/bundles'/key,bundle)
    paths={Path(__file__).resolve(),ROOT/'run_scoped.py',ROOT/'installed-root-binding/result.json',ROOT/'installed-root-binding/commands.json',ROOT/'linux-test-diagnostic/result.json',binary,REPO/'scripts/successor_release_bundles.py'}
    paths.update(p for p in OLD.rglob('*') if p.is_file())
    before=[seal(p) for p in sorted(paths)]
    write(out/'inputs.json',{'inputs':before,'scope':'Preserve the failed full original root, copy every actual registered bundle byte and both registry transports, replace only the copied test executable, and observe the existing descriptor-relative loader error. No path override is added to production.'})
    report={'status':'running','inputs':seal(out/'inputs.json'),'native_Linux_release_acceptance':False}
    write(out/'result.json',report)
    start=time.monotonic()
    try:
        assert not NEW.exists()
        shutil.copytree(OLD,NEW,copy_function=shutil.copy2)
        (NEW/'bin').chmod(0o755)
        (NEW/'bin/mpk').chmod(0o755)
        shutil.copyfile(binary,NEW/'bin/mpk')
        (NEW/'bin/mpk').chmod(0o555)
        release.normalize_directories(NEW)
        command=['/usr/local/bin/docker','run','--pull=never','--rm','--platform','linux/amd64','--network','none','--read-only','--cap-drop','ALL','--security-opt','no-new-privileges','--user',f'{os.getuid()}:{os.getgid()}','--mount',f'type=bind,src={NEW},dst=/installed,readonly',IMAGE,'/installed/bin/mpk','--diagnose-installed']
        child=subprocess.run(command,cwd=REPO,capture_output=True,check=False)
        (out/'stdout.log').write_bytes(child.stdout);(out/'stderr.log').write_bytes(child.stderr)
        assert child.returncode==0 and not child.stderr
        observation=json.loads(child.stdout)
        report.update(status='diagnosed',command=command,observation=observation,stdout=seal(out/'stdout.log'),stderr=seal(out/'stderr.log'))
    except BaseException as error:
        report.update(status='failed',error=repr(error))
        raise
    finally:
        report.update(seconds=time.monotonic()-start,original_inputs_unchanged=before==[seal(p) for p in sorted(paths)])
        write(out/'result.json',report)
    print(json.dumps({key:report[key] for key in ['status','seconds','original_inputs_unchanged','observation']}))


if __name__=='__main__':
    main()
