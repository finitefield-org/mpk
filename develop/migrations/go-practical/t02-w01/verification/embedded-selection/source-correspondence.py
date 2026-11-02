"""Verify complete production sources against the passing original epochs."""
from pathlib import Path
import hashlib,json
ROOT=Path(__file__).resolve().parent;REPO=Path.cwd()
def seal(p):
    raw=p.read_bytes();return {'path':str(p),'bytes':len(raw),'sha256':hashlib.sha256(raw).hexdigest(),'mode':p.stat().st_mode&0o777}
def main():
    output=ROOT/'source-correspondence.json';assert not output.exists();inputs=[];groups={}
    for epoch,base,suffix,exclude_tests in [('rust-original',REPO/'crates/mpk-vc/src','.rs',False),('native',REPO/'go-tools/go2vir','.go',True)]:
        path=ROOT/epoch/'inputs.json';inputs.append(seal(path));receipt=json.loads(path.read_bytes());checked=[]
        for row in receipt['sources']:
            source=Path(row['path'])
            if source.is_relative_to(base) and source.suffix==suffix and not (exclude_tests and source.name.endswith('_test.go')):
                assert row==seal(source);checked.append(row)
        assert checked;groups[epoch]=checked
    parent=REPO/'crates/mpk-vc/src/go_practical_type_keys/source_package.rs'
    assert str(parent) in {r['path'] for r in groups['rust-original']}
    path=ROOT/'inspection.json';inputs.append(seal(path));inspection=json.loads(path.read_bytes());prefixes=inspection['complete_earlier_test_source_prefixes']
    for row in prefixes:
        source=Path(row['source']);data=source.read_bytes();assert hashlib.sha256(data[:row['complete_previous_source_prefix_bytes']]).hexdigest()==row['previous_source_sha256']
    result={'status':'passed','driver':seal(Path(__file__).resolve()),'inputs':inputs,'complete_unchanged_original_production_sources':groups,'complete_prior_test_source_prefixes':prefixes,'only_new_group_test_functions_added_after_original_passing_epochs':True,'new_rust_group_result':seal(ROOT/'rust-groups/result.json'),'new_native_group_result':seal(ROOT/'native-groups/result.json'),'source_or_context_program_proof':False}
    output.write_text(json.dumps(result,indent=2)+'\n');print(json.dumps({'status':'passed','unchanged_rust_production_sources':len(groups['rust-original']),'unchanged_go_production_sources':len(groups['native']),'unchanged_complete_prior_test_prefixes':len(prefixes)}))
if __name__=='__main__':main()
