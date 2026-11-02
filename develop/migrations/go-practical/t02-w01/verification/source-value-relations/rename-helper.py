"""Rename one private Rust helper without changing any generated expression."""
from pathlib import Path
import hashlib,json,re
ROOT=Path(__file__).resolve().parent;REPO=Path.cwd()
relative=Path('crates/mpk-vc/src/go_practical_ordinary/source_value_relations.rs')
copy=Path(json.loads((ROOT/'isolated-copy.json').read_bytes())['copy'])
assert json.loads((ROOT/'lint/result.json').read_bytes())['status']=='failed'
assert json.loads((ROOT/'producer-replay/result.json').read_bytes())['status']=='passed'
out=ROOT/'helper-name-adjustment';out.mkdir(exist_ok=False)
source=REPO/relative;other=copy/relative
before=source.read_bytes();assert before==other.read_bytes()
assert before.count(b'from_equality')==3 and b'equality_lift' not in before
assert all(b'from_equality' not in s for s in re.findall(rb'"(?:[^"\\]|\\.)*"',before))
after=before.replace(b'from_equality',b'equality_lift');assert after.replace(b'equality_lift',b'from_equality')==before
(out/'driver.py').write_bytes(Path(__file__).read_bytes())
(out/'before.rs').write_bytes(before);(out/'after.rs').write_bytes(after)
mode=source.stat().st_mode&0o777
for p in [source,other]:p.write_bytes(after);p.chmod(mode)
rows=[]
for name in json.loads((ROOT/'isolated-copy.json').read_bytes())['overlaid_sources']:
    p=Path(name);raw=p.read_bytes();assert raw==(copy/name).read_bytes()
    rows.append({'path':name,'bytes':len(raw),'sha256':hashlib.sha256(raw).hexdigest(),'mode':p.stat().st_mode&0o777,'exact_current_worktree_and_isolated_copy_match':True})
report={'status':'private_helper_identifier_only_and_final_source_binding','old_name':'from_equality','new_name':'equality_lift','identifier_occurrences':3,'generated_expression_strings_unchanged':True,'whole_source_round_trip_exact':True,'native_proof_bytes_or_rules_changed':False,'source_bytes_before_sha256':hashlib.sha256(before).hexdigest(),'source_bytes_after_sha256':hashlib.sha256(after).hexdigest(),'mode':mode,'sources':rows,'scope':'Clippy wrong-self-convention fix only; generated Expr constructors, string names, theorem statements and proofs are byte-for-byte unchanged after reversing this identifier substitution. Full kernel matrices and four byte replays remain valid for those identical recipes. This is a new source epoch, not a relabeling of prior inputs.'}
(out/'result.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps({'status':report['status'],'occurrences':3}))
