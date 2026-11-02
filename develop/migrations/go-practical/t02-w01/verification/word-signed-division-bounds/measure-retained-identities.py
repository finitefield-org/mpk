"""Compare sealed original named metadata; never a proof/import authority."""
from pathlib import Path
import datetime
import hashlib
import json

root = Path(__file__).resolve().parent.relative_to(Path.cwd().resolve())

def seal(path):
    path = Path(path)
    raw = path.read_bytes()
    return {'path': str(path), 'sha256': hashlib.sha256(raw).hexdigest(), 'bytes': len(raw)}

reader = root.parent / 'word-signed-division/prerequisite-analysis/measure.py'
namespace = {'__file__': str(reader.resolve())}
exec(compile(reader.read_text().split('sources=[]', 1)[0], str(reader), 'exec'), namespace)
Tables = namespace['Tables']
current = Tables(seal(root / 'actual-minimum-classification/rust-construction/certificates/word-division-bounds.mpcert'))
old_record = json.loads((root.parent / 'word-signed-refinement/verification.json').read_text())['fresh_component']['certificate']
half_record = json.loads((root.parent / 'word-signed-bounds/verification.json').read_text())['fresh_component']['certificate']
assert old_record['sha256'] == 'a38c1903fb7add8bf7156b484d747612b52a985899227453a772f3a3b6498487'
assert half_record['sha256'] == '8895b359ea61393aaf241c82e26366498f06cd12a0645e5ba37e3cbe14c41a31'
old, half = Tables(old_record), Tables(half_record)
actual = current.fingerprints()
for name, value in old.fingerprints().items():
    assert actual[name] == value, name

def mapped(name):
    for width in [8, 16, 32, 64]:
        for prefix, lemma in [('Word.Order', 'IsZeroNatural'), ('Word.Order', 'DefinedDivisorPositive'),
                              ('Word.SignedMagnitude', 'ZeroSign')]:
            if name == f'Mpk.Go.Practical.{prefix}.W{width}.{lemma}':
                name = f'Mpk.Go.Practical.Word.Zero.W{width}.{lemma}'
    for prefix, role in [('Word.Order', 'Order'), ('Word.Division', 'Unsigned'),
                         ('Word.SignedMagnitude', 'Magnitude'), ('Word.SignedBounds', 'Bounds'),
                         ('Word.SignedProduct', 'Arithmetic'), ('Word.Zero', 'Zero')]:
        start = 'Mpk.Go.Practical.' + prefix + '.'
        if name.startswith(start):
            return 'Mpk.Go.Practical.Word.SignedDivision.Prerequisite.' + role + '.' + name[len(start):]
    return name

half.declarations = [(mapped(name), kind, values) for name, kind, values in half.declarations]
reference = half.fingerprints()
additional = sorted(name for name in current.index if name not in old.index)
new = [name for name in additional if name.startswith('Mpk.Go.Practical.Word.SignedDivision.Bounds.')]
original = [name for name in additional if name not in new]
assert len(current.index) == 4093 and len(old.index) == 3897
assert len(original) == 8 and len(new) == 188
for name in original:
    assert actual[name] == reference[name], name
expected = {f'Mpk.Go.Practical.Word.SignedDivision.Prerequisite.Bounds.HalfPositive{depth}' for depth in [3, 4, 5, 6]}
expected |= {f'Mpk.Go.Practical.Word.SignedDivision.Prerequisite.Bounds.W{width}.MagnitudeHalfBalance' for width in [8, 16, 32, 64]}
assert set(original) == expected
record = {
    'schema': 'mpk.go_practical.signed_division_bounds_retained_identity_observation.v1',
    'status': 'matched_structural_identities_not_checker_authority',
    'observed_at_utc': datetime.datetime.now(datetime.timezone.utc).isoformat(),
    'certificate': current.record, 'original_refinement_certificate': old_record,
    'original_full_signed_bounds_certificate': half_record,
    'reader_source': seal(reader), 'reader_extraction': 'Only definitions before sources=[]; no original planning main is executed or modified.',
    'parser_sources': [seal('crates/mpk-cert/src/decode.rs'), seal('crates/mpk-cert/src/binary_tags.rs')],
    'all_original_refinement_named_identities': 3897, 'additional_original_half_named_identities': 8,
    'new_division_bounds_declarations': 188, 'additional_original_names': original, 'new_division_bounds_names': new,
    'scope': 'Normalized named-DAG type/body/proof/level/family metadata matches every prior declaration and eight extra original half positivity/balance declarations after the same fixed prerequisite renaming/substitution. This small reader supplies diagnostic identity evidence only. Both unchanged production cores check the complete new ordinary certificate including all proof terms; no fingerprint or receipt confers proof/import/source authority.',
}
path = root / 'retained-identities.json'
assert not path.exists()
path.write_text(json.dumps(record, indent=2) + '\n')
print(json.dumps({key: record[key] for key in ['status', 'all_original_refinement_named_identities',
                                             'additional_original_half_named_identities', 'new_division_bounds_declarations']}))
