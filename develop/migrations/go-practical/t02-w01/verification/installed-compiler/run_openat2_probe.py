"""Record unchanged openat2 observations in both local Docker profiles."""
from pathlib import Path
import json
import os
import subprocess
import sys

ROOT=Path(__file__).resolve().parent
REPO=ROOT.parents[5]
IMAGE='sha256:d617f1b5c2a18778ba357ba0528e973bd24992b0dd18000dfbca059f4a15ebda'
INSTALLED=Path('/private/tmp/mpk-go-installed-205-fs/installed-diagnostic')
sys.dont_write_bytecode=True
sys.path.insert(0,str(ROOT))
from run_scoped import seal,write


def main():
    out=ROOT/'openat2-environment-diagnostic'
    out.mkdir(exist_ok=False)
    paths=[Path(__file__).resolve(),ROOT/'openat2_probe.py',ROOT/'installed-root-diagnostic/result.json']
    inputs=[seal(p) for p in paths]
    write(out/'inputs.json',{'inputs':inputs,'scope':'Check the exact read-only openat2 flags used by the existing installed-root loader on the complete copied release root and an ordinary /tmp directory, with both default and unconfined local Docker seccomp. This is environment diagnosis, not a native Linux release gate.'})
    results=[]
    for mode in ['default','unconfined']:
        command=['/usr/local/bin/docker','run','--pull=never','--rm','--platform','linux/amd64','--network','none','--read-only','--cap-drop','ALL','--security-opt','no-new-privileges']
        if mode=='unconfined':command.extend(['--security-opt','seccomp=unconfined'])
        command.extend(['--user',f'{os.getuid()}:{os.getgid()}','--mount',f'type=bind,src={INSTALLED},dst=/installed,readonly','--mount',f'type=bind,src={ROOT/"openat2_probe.py"},dst=/probe.py,readonly',IMAGE,'/usr/bin/python3','/probe.py'])
        process=subprocess.run(command,capture_output=True,check=False)
        (out/f'{mode}.stdout').write_bytes(process.stdout)
        (out/f'{mode}.stderr').write_bytes(process.stderr)
        assert process.returncode==0 and not process.stderr
        results.append({'mode':mode,'command':command,'observation':json.loads(process.stdout),'stdout':seal(out/f'{mode}.stdout'),'stderr':seal(out/f'{mode}.stderr')})
    result={'status':'diagnosed','inputs_unchanged':inputs==[seal(p) for p in paths],'results':results,'native_Linux_release_acceptance':False}
    write(out/'result.json',result)
    print(json.dumps({'status':result['status'],'results':[{'mode':r['mode'],'observation':r['observation']} for r in results]}))


if __name__=='__main__':
    main()
