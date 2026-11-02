"""Check actual archived changed prelaunch originals, preserving the initial record.

Git-blob originals are independently read/mode-checked by review-inputs.py;
only physically captured changed originals are counted by this audit.
"""
from pathlib import Path
import hashlib,json,runpy
root=Path(__file__).resolve().parent
old=root/'virtual-original-byte-mode-audit.json'
prior=root/'virtual-original-byte-mode-audit-initial-count.json'
if old.exists() and not prior.exists():prior.write_bytes(old.read_bytes())
driver=root/'virtual-original-audit-driver.py'
assert not driver.exists();driver.write_bytes(Path(__file__).read_bytes())
v=runpy.run_path(str(root/'virtual-originals.py'));index=v['index']()
assert len(index)==32933 and sum(len(row[0]) for row in index.values())==261804869
controls,mode=v['kept'](root/'native-corrected/native-originals/dispatch-controls.json')
assert mode==0o600 and json.loads(controls)['complete_error_payload_types']==6
receipt=json.loads((root/'derived/inputs.json').read_bytes());changed=0;git_rows=0
for row in receipt['sources']:
    if 'complete_changed_input' in row:
        raw,mode=v['kept'](row['complete_changed_input'])
        assert len(raw)==row['bytes'] and hashlib.sha256(raw).hexdigest()==row['sha256'] and mode==row['mode']
        changed+=1
    else:
        assert 'original_git_blob' in row;git_rows+=1
out={'status':'passed','actual_driver_prelaunch_sha256':hashlib.sha256(driver.read_bytes()).hexdigest(),'virtual_files':len(index),'virtual_bytes':sum(len(row[0]) for row in index.values()),'complete_changed_derived_prelaunch_rows_checked':changed,'original_Git_blob_rows_deferred_to_scoped_input_review':git_rows,'native_fixture_original_mode':0o600,'all_captured_changed_derived_original_bytes_modes_match':True,'initial_record_count_was_total_rows_not_only_checked_changed_rows':'virtual-original-byte-mode-audit-initial-count.json','initial_count_record_preserved_without_relabeling_as_native_or_source_failure':True,'contents_still_subject_to_unchanged_scoped_review':True}
assert changed+git_rows==len(receipt['sources'])
(root/'virtual-original-byte-mode-audit.json').write_text(json.dumps(out,indent=2)+'\n');print(json.dumps(out))
