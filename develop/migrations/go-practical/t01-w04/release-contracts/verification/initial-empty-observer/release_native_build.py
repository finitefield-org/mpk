"""Build compact release observer against unchanged native validators."""
import tomllib
from policy_native_build import prefix, command, linked_inputs, TARGET
from run_proof_pilot import ROOT, HERE, sha
from check_scope_report import require


def sources():
    return linked_inputs()+[{'path':p.relative_to(ROOT).as_posix(),'sha256':sha(p.read_bytes())} for p in sorted((HERE/'release-native').rglob('*')) if p.is_file()]


def build(bootstrap=False):
    folder=HERE/'release-native';p,audit=prefix();lock=folder/'Cargo.lock'
    if bootstrap:
        stage=TARGET/'release-bootstrap';stage.mkdir(exist_ok=True)
        (stage/'src').mkdir(exist_ok=True);(stage/'src/main.rs').write_text('fn main() {}\n')
        (stage/'Cargo.toml').write_text((folder/'Cargo.toml').read_text().replace('../../../../crates/','/workspace/crates/'))
        (stage/'Cargo.lock').write_bytes((ROOT/'Cargo.lock').read_bytes())
        command(p+['cargo','build','--offline','--manifest-path','/target/release-bootstrap/Cargo.toml'])
        lock.write_bytes((stage/'Cargo.lock').read_bytes())
    require(lock.is_file(),'bootstrap lock before authoritative build')
    allowed={(r['name'],r['version'],r.get('checksum')) for r in tomllib.loads((ROOT/'Cargo.lock').read_text())['package']}
    for r in tomllib.loads(lock.read_text())['package']:
        if 'source'in r:require((r['name'],r['version'],r.get('checksum'))in allowed,'exact existing dependency version/checksum')
    before=sources();result=command(p+['cargo','build','--offline','--locked','--manifest-path','/workspace/develop/probes/go-practical/release-native/Cargo.toml'])
    require(b'warning:'not in result.stderr,'new release observer compiles without warnings')
    require(before==sources(),'all release observer/consumer sources fixed during build')
    binary=TARGET/'debug/go-practical-release-native';audit.update(binary_sha256=sha(binary.read_bytes()),build_source_inputs=before)
    return p+['/target/debug/go-practical-release-native'],audit
