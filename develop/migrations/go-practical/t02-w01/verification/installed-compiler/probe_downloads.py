"""Read-only availability diagnostics; these prefixes are not build artifacts."""
from pathlib import Path
import concurrent.futures
import datetime
import json
import ssl
import sys
import traceback
import urllib.request

ROOT = Path(__file__).resolve().parent
REPO = ROOT.parents[5]
sys.dont_write_bytecode = True
sys.path.insert(0, str(ROOT))
from run_scoped import seal, write
sys.path.insert(0, str(REPO / 'scripts'))
import rust_build_inputs as rust


def main():
    out = ROOT / (sys.argv[1] if len(sys.argv) == 2 else 'rust-download-diagnostics')
    out.mkdir(exist_ok=False)
    paths = [Path(__file__).resolve(), ROOT / 'run_scoped.py', REPO / 'scripts/rust_build_inputs.py', REPO / 'release/build-inputs/rust/build-inputs.json']
    inputs = [seal(p) for p in paths]
    urls = [rust.LLVM_URL, rust.CARGO_FUZZ_URL, f'{rust.RUST_DIST_ROOT}/rustfmt-nightly-x86_64-unknown-linux-gnu.tar.xz']
    write(out / 'inputs.json', {'inputs': inputs, 'urls': urls, 'interpreter': sys.executable, 'version': sys.version, 'TLS_verify_paths': ssl.get_default_verify_paths()._asdict(), 'scope': 'Diagnose terminal exit69 without changing any pinned URL/hash, registry, source, resource limit or native release requirement. Only HTTP status, headers and a diagnostic response prefix are observed; none is used as an original compiler/build input.'})

    def probe(item):
        index, url = item
        row = {'url': url, 'started_at_utc': datetime.datetime.now(datetime.timezone.utc).isoformat()}
        request = urllib.request.Request(url, headers={'User-Agent': 'mpk-rust-build-inputs-v0', 'Range': 'bytes=0-1023'})
        try:
            with urllib.request.urlopen(request, timeout=60) as response:
                prefix = response.read(1024)
                path = out / f'response-{index}.prefix'
                path.write_bytes(prefix)
                row.update(status=response.status, final_url=response.url, headers=dict(response.headers.items()), prefix=seal(path))
        except BaseException as error:
            row.update(error=repr(error), traceback=traceback.format_exc())
        write(out / f'request-{index}.json', row)
        return row

    with concurrent.futures.ThreadPoolExecutor(max_workers=len(urls)) as pool:
        results = list(pool.map(probe, enumerate(urls)))
    write(out / 'result.json', {'status': 'diagnostics-completed', 'requests': results, 'inputs_unchanged': inputs == [seal(p) for p in paths], 'native_Linux_release_acceptance': False})
    print(json.dumps([{k: row.get(k) for k in ['url', 'status', 'error']} for row in results]))


if __name__ == '__main__':
    main()
