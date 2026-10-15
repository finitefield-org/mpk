from pathlib import Path
import json
import hashlib
import re

repo = Path('/private/tmp/mpk-w09-context-application-integration')
source = repo / 'crates/mpk-vc/src/csharp_practical_registry.rs'
text = source.read_text()
old = '230c708601f4b89feeae28af23da10ccb11eec998d990b5133cf9336369e15a2'
new = json.loads((repo / 'develop/migrations/csharp-03/foundation/foundation-descriptor.json').read_text())['content_sha256']
fields = ['ai', 'evidence', 'frontend', 'manifest', 'policy', 'release', 'source_map', 'vc', 'vir']
profiles = [
    ('CSHARP_PRACTICAL_ENTRY_SHA256', 'csharp', 'mpk.csharp.practical.v1', 'csharp_practical', 'csharp_members.v1'),
    ('SUCCESSOR_CSHARP_SCALAR_ENTRY_SHA256', 'csharp', 'mpk.csharp.scalar.v0', 'csharp_scalar', 'csharp_methods.v0'),
    ('SUCCESSOR_GO_FIXED_ENTRY_SHA256', 'go', 'mpk.go.fixed.v0', 'go_fixed', 'go_function.v0'),
    ('SUCCESSOR_JAVA_SCALAR_ENTRY_SHA256', 'java', 'mpk.java.scalar.v0', 'java_scalar', 'java_methods.v0'),
    ('SUCCESSOR_RUST_CHECKED_ENTRY_SHA256', 'rust', 'mpk.rust.checked.v0', 'rust_checked', 'rust_function.v0')]


def digest(domain, value):
    return hashlib.sha256(domain.encode() + b'\0' + json.dumps(value, sort_keys=True, separators=(',', ':'), ensure_ascii=False).encode()).hexdigest()


def registry(foundation, revision):
    entries = []
    for key, language, profile, stem, selection in profiles:
        e = dict(schema='mpk.semantic_profile.entry.v2', source_language=language, semantic_profile=profile,
                 semantic_parameters_schema='mpk.semantic_parameters.' + stem + ('.v1' if stem == 'csharp_practical' else '.v0'),
                 selection_schema='mpk.selection.' + selection,
                 foundation_descriptor=dict(schema='mpk.csharp.foundation_descriptor.v1', id='mpk.csharp.practical.foundation.v1', content_sha256=foundation),
                 contracts={f: 'mpk.profile.' + f + '.' + stem + '.v1' for f in fields})
        e['entry_sha256'] = digest('MPK-SEMANTIC-PROFILE-ENTRY-2.0', e)
        entries.append(e)
    r = dict(schema='mpk.semantic_profile.registry.v2', id='mpk.semantic_profile.registry.v2', revision=revision, profiles=entries)
    r['registry_sha256'] = digest('MPK-SEMANTIC-PROFILE-REGISTRY-2.0', r)
    return r


before = registry(old, 4)
for (key, *_), entry in zip(profiles, before['profiles']):
    assert re.search(r'pub const ' + key + r': &str =\s*"([0-9a-f]+)"', text).group(1) == entry['entry_sha256'], key
assert before['registry_sha256'] in text
after = registry(new, 5)
root = Path(__file__).parent
for label, value in [('before', before), ('after', after)]:
    (root / ('candidate-registry-' + label + '.json')).write_text(json.dumps(value, sort_keys=True, separators=(',', ':')) + '\n')
mapping = {old: new, before['registry_sha256']: after['registry_sha256']}
for previous, current in zip(before['profiles'], after['profiles']):
    mapping[previous['entry_sha256']] = current['entry_sha256']
(root / 'candidate-identity-map.json').write_text(json.dumps(mapping, indent=2, sort_keys=True) + '\n')
for previous, current in mapping.items():
    text = text.replace(previous, current)
text = text.replace('pub const SUCCESSOR_CANDIDATE_REVISION: u64 = 4;', 'pub const SUCCESSOR_CANDIDATE_REVISION: u64 = 5;')
source.write_text(text)
semantics = repo / 'develop/specs/CSHARP_PRACTICAL_FOUNDATION_V1.md'
source = repo / 'crates/mpk-vc/src/csharp_practical_vir_model.rs'
text = source.read_text().replace('d3623ade42f68ed94b9abf9b4cb2693b0c5898a6a9e9b5b650a7802a8e7926c3', hashlib.sha256(semantics.read_bytes()).hexdigest())
text = text.replace('const FOUNDATION_SEMANTICS_SIZE_BYTES: u64 = 62222;', f'const FOUNDATION_SEMANTICS_SIZE_BYTES: u64 = {len(semantics.read_bytes())};')
source.write_text(text)
print('Previous Rust frozen hashes independently reproduced; candidate revision 5 recomputed:', after['registry_sha256'])
