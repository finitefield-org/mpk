"""Additional original Int dependency identities; never checker authority."""
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
code = reader.read_text().split('sources=[]', 1)[0]
exec(compile(code, str(reader), 'exec'), namespace)
Tables = namespace['Tables']
current = Tables(seal(root / 'explicit-flag-transport/rust-construction/certificates/word-signed-refinement.mpcert'))
prerequisite_record = json.loads((root.parent / 'word-signed-division/verification.json').read_text())['fresh_component']['certificate']
word_record = json.loads((root.parent / 'word-foundation/verification.json').read_text())['fresh_component']['certificate']
assert prerequisite_record['sha256'] == '2d23c58db425fc11d653c8c7966db69a0f71495d614712cd7b8abc0f2246e11d'
assert word_record['sha256'] == '23c88fb260bf9d27c2defb64a892f275653cd1741ed030f00a911fed1b2aa11c'
prerequisite, word = Tables(prerequisite_record), Tables(word_record)
candidate_hashes = current.fingerprints()
for name, value in prerequisite.fingerprints().items():
    assert candidate_hashes[name] == value, name
additional = sorted(name for name in current.index if name not in prerequisite.index)
new = [name for name in additional if name.startswith('Mpk.Go.Practical.Word.SignedDivision.')]
old = [name for name in additional if name not in new]
assert len(prerequisite.index) == 3483 and len(current.index) == 3897
assert len(old) == 166 and len(new) == 248
reference = word.fingerprints()
for name in old:
    assert name.startswith('Mpk.Go.Practical.Int.') and candidate_hashes[name] == reference[name], name
record = {
    'schema': 'mpk.go_practical.signed_refinement_retained_identity_observation.v1',
    'status': 'matched_structural_identities_not_checker_authority',
    'observed_at_utc': datetime.datetime.now(datetime.timezone.utc).isoformat(),
    'certificate': current.record, 'original_prerequisite_certificate': prerequisite_record,
    'original_full_word_certificate': word_record,
    'reader_source': seal(reader), 'reader_extraction': 'Only definitions before the first sources=[] marker; no original planning main is executed or modified.',
    'parser_sources': [seal('crates/mpk-cert/src/decode.rs'), seal('crates/mpk-cert/src/binary_tags.rs')],
    'all_original_prerequisite_named_identities': 3483,
    'additional_original_Int_named_identities': 166, 'new_signed_refinement_declarations': 248,
    'additional_original_Int_names': old, 'new_signed_refinement_names': new,
    'scope': 'Normalized named-DAG type/body/proof/level/family metadata matches over exact sealed full tables. The new source wrapper selects166 additional genuine original Int equivalence/congruence dependencies from the complete original word input. This diagnostic metadata comparison supplies no proof verdict, import/context/source authority or completion claim. Both unchanged production cores still check the entire new certificate, including every actual proof term.',
}
out = root / 'retained-identities.json'
assert not out.exists()
out.write_text(json.dumps(record, indent=2) + '\n')
print(json.dumps({key: record[key] for key in ['status', 'all_original_prerequisite_named_identities',
                                             'additional_original_Int_named_identities', 'new_signed_refinement_declarations']}))
