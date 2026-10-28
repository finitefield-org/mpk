"""Compare the existing full original Go application while its roots are live."""
from pathlib import Path
import json
import os
import shutil
import subprocess
import sys
import time

ROOT=Path(__file__).resolve().parent
REPO=ROOT.parents[5]
SDK=Path('/private/tmp/mpk-installed-compiler-205-original-go-output/toolchain')
CARGO=Path('/private/tmp/mpk-go-source-root-matching/cargo')
TARGET=Path('/private/tmp/mpk-go-source-root-matching/target')
REGISTRY=Path('/Users/kazuyoshitoshiya/.cargo/registry')
GOCACHE=Path('/private/tmp/mpk-installed-compiler-205-go-cache')
RUST_IMAGE='sha256:7cf1e580ef5539f03b58560753e8ab84c8c360960d99dff714004aa98f203977'
GO_IMAGE='docker.io/library/golang@sha256:ebd54034f076819b3054f155db53660ded951612bb4dfd277f933e62059e5d5a'
sys.dont_write_bytecode=True
sys.path.insert(0,str(ROOT.parent/'installed-compiler'))
from run_scoped import seal,sdk_files,write


def main():
    epoch=sys.argv[1];assert epoch.replace('-','').isalnum() and REPO==Path.cwd()
    previous=json.loads((ROOT/'preflight-go-retry/result.json').read_bytes());old=json.loads((ROOT/'preflight/result.json').read_bytes());inputs=json.loads((ROOT/'preflight-go-retry/inputs.json').read_bytes())
    assert previous['status']=='passed' and previous['original_inputs_unchanged'] and old['checks']['rust-check']['status']=='passed' and old['checks']['rust-format']['status']=='passed'
    sources=inputs['sources'];sdk=inputs['registered_sdk'];assert sources==[seal(Path(r['path'])) for r in sources] and sdk==sdk_files()
    assert seal(Path(inputs['archive']['path']))==inputs['archive']
    out=ROOT/epoch;out.mkdir(exist_ok=False)
    work=Path('/private/tmp')/f'mpk-go-live-original-roots-{epoch}';assert not work.exists();work.mkdir(mode=0o700)
    shared=work/'tmp';shared.mkdir(mode=0o700);evidence=work/'native-originals';evidence.mkdir(mode=0o700);coord=work/'coord';coord.mkdir(mode=0o700)
    common=['/usr/local/bin/docker','run','--pull=never','--rm','--network','none','--read-only','--cap-drop','ALL','--security-opt','no-new-privileges','--user',f'{os.getuid()}:{os.getgid()}','--mount',f'type=bind,src={REPO},dst=/workspace,readonly','--mount',f'type=bind,src={shared},dst=/tmp','--mount',f'type=bind,src={evidence},dst=/evidence','--mount',f'type=bind,src={coord},dst=/coord']
    genv={'GOROOT':'/sdk/go','GOTOOLCHAIN':'local','GOENV':'off','GOWORK':'off','GOFLAGS':'-mod=vendor','GOPROXY':'off','GOSUMDB':'off','CGO_ENABLED':'0','GOOS':'linux','GOARCH':'amd64','GOAMD64':'v1','GOMAXPROCS':'1','GOCACHE':'/cache/build','GOMODCACHE':'/cache/mod','MPK_GO_PRACTICAL_SYNTAX_EVIDENCE':'/evidence','MPK_GO_LIVE_ROOT_COORD':'/coord'}
    go=common+['--platform','linux/amd64','--mount',f'type=bind,src={SDK},dst=/sdk,readonly','--mount',f'type=bind,src={GOCACHE},dst=/cache','--workdir','/workspace/go-tools/go2vir']
    for k,v in genv.items():go.extend(['--env',f'{k}={v}'])
    go.extend([GO_IMAGE,'/sdk/go/bin/go','test','-json','-count=1','-timeout=40m','-run','^TestPracticalInstalledCompilerInputsKeepWholeRegisteredSDKAndPackages$'])
    renv={'CARGO_HOME':'/cargo','CARGO_TARGET_DIR':'/target','CARGO_NET_OFFLINE':'true','CARGO_INCREMENTAL':'0','CARGO_PROFILE_DEV_DEBUG':'0','HOME':'/tmp'}
    rust=common+['--platform','linux/arm64','--mount',f'type=bind,src={CARGO},dst=/cargo','--mount',f'type=bind,src={REGISTRY},dst=/cargo/registry,readonly','--mount',f'type=bind,src={TARGET},dst=/target','--workdir','/workspace']
    for k,v in renv.items():rust.extend(['--env',f'{k}={v}'])
    rust.extend([RUST_IMAGE,'cargo','run','--offline','--locked','-p','mpk-cli','--example','go_practical_live_roots','--','/coord'])
    write(out/'inputs.json',{'selection_reason':'Open three actual original application, dependency and registered-SDK root handles before the unchanged Go capture; compare every resulting source and observed namespace entry while the original roots remain live. Preserve the complete program, exports, compiler and original inputs. This is not launcher authority, application proof or native Linux release acceptance.','driver':seal(Path(__file__)),'preflight_inputs':seal(ROOT/'preflight-go-retry/inputs.json'),'preflight_archive':inputs['archive'],'preflight_go':seal(ROOT/'preflight-go-retry/result.json'),'preflight_rust':seal(ROOT/'preflight/result.json'),'registered_sdk_files':len(sdk),'go_command':go,'rust_command':rust,'go_environment':genv,'rust_environment':renv,'temporary_root':str(work)})
    report={'status':'running','native_Linux_release_acceptance':False,'full_T_gate':'Deferred to final T02-W04: ./scripts/check-fast.sh'};write(out/'result.json',report)
    start=time.monotonic();rust_code=None;go_code=None
    with (out/'go.stdout').open('xb') as gout,(out/'go.stderr').open('xb') as gerr,(out/'rust.stdout').open('xb') as rout,(out/'rust.stderr').open('xb') as rerr:
        gp=subprocess.Popen(go,cwd=REPO,stdout=gout,stderr=gerr)
        print(json.dumps({'phase':'waiting_for_original_roots','go_pid':gp.pid}),flush=True);last=start
        while not (coord/'roots-ready.json').exists() and gp.poll() is None:
            time.sleep(1)
            if time.monotonic()-last>=30: print(json.dumps({'phase':'waiting_for_original_roots','seconds':round(time.monotonic()-start)}),flush=True);last=time.monotonic()
        if (coord/'roots-ready.json').exists():
            rp=subprocess.Popen(rust,cwd=REPO,stdout=rout,stderr=rerr)
            print(json.dumps({'phase':'observer_opening_roots','rust_pid':rp.pid,'seconds':round(time.monotonic()-start)}),flush=True);last=time.monotonic()
            while gp.poll() is None or rp.poll() is None:
                if rp.poll() not in (None,0):
                    signal=coord/('matched.json' if (coord/'opened.json').exists() else 'opened.json')
                    if not signal.exists():write(signal,{'status':'failed','diagnostic':'observer process ended before emitting its signal'})
                if gp.poll() is not None and rp.poll() is None and not (coord/'snapshot-ready.json').exists():
                    write(coord/'snapshot-ready.json',{'status':'failed','diagnostic':'Go process ended before emitting its snapshot'})
                time.sleep(1)
                if time.monotonic()-last>=30:
                    phase='matching_original_snapshot' if (coord/'snapshot-ready.json').exists() else 'capturing_original_application' if (coord/'opened.json').exists() else 'observer_opening_roots'
                    print(json.dumps({'phase':phase,'seconds':round(time.monotonic()-start)}),flush=True);last=time.monotonic()
            rust_code=rp.wait()
        go_code=gp.wait()
    captured=out/'coord';captured.mkdir()
    for p in sorted(coord.iterdir()):
        if p.is_file():shutil.copyfile(p,captured/p.name)
    files=[seal(p) for p in sorted(evidence.rglob('*')) if p.is_file()]
    write(out/'original-evidence-manifest.json',{'root':str(evidence),'files':files,'file_count':len(files),'total_bytes':sum(x['bytes'] for x in files)})
    matching=json.loads((captured/'matched.json').read_bytes()) if (captured/'matched.json').exists() else {}
    opened=json.loads((captured/'opened.json').read_bytes()) if (captured/'opened.json').exists() else {}
    source_unchanged=sources==[seal(Path(r['path'])) for r in sources];sdk_unchanged=sdk==sdk_files()
    okay=go_code==0 and rust_code==0 and matching.get('status')=='passed' and matching.get('opened_before_capture') is True and opened.get('capabilities')==3 and matching.get('original_source_objects',0)>=5480 and source_unchanged and sdk_unchanged
    report.update(status='passed' if okay else 'failed',go_exit_code=go_code,rust_exit_code=rust_code,seconds=time.monotonic()-start,original_inputs_unchanged=source_unchanged,registered_sdk_unchanged=sdk_unchanged,matched=matching,original_evidence_manifest=seal(out/'original-evidence-manifest.json'),go_stdout=seal(out/'go.stdout'),go_stderr=seal(out/'go.stderr'),rust_stdout=seal(out/'rust.stdout'),rust_stderr=seal(out/'rust.stderr'))
    write(out/'result.json',report);print(json.dumps({k:report[k] for k in ['status','go_exit_code','rust_exit_code','seconds','original_inputs_unchanged','registered_sdk_unchanged']}),flush=True);raise SystemExit(not okay)


if __name__=='__main__':main()
