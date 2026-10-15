from pathlib import Path
import json,shutil
repo=Path('/private/tmp/mpk-w09-packed-pattern-proofs');root=Path('/private/tmp/mpk-w09-foundation-proofs')
parent=repo/'develop/migrations/csharp-03/ordinary-foundation';base=parent/'verification-logs/foundation-proofs';source=root/'server-final-1e5338e9'
audit=json.loads((source/'final-audit.json').read_bytes())
assert audit['status']=='passed' and audit['source_commit']=='1e5338e94f792a84b8ebe22ed2452bef896791c8'
assert audit['git_blob_files_verified']==924 and audit['exported_files_verified']==91
assert audit['exported_files_match_local'] and audit['legacy_certificate_bytes_unchanged']
dest=base/'server-linux/final';assert not dest.exists();dest.mkdir()
for path in source.iterdir():
 if path.is_dir():shutil.copytree(path,dest/path.name)
 elif path.name.endswith('.log'):shutil.copyfile(path,dest/(path.name+'.txt'))
 else:shutil.copyfile(path,dest/path.name)
for name in ('final-linux-fetch.py','preserve-final-linux.py'):shutil.copyfile(root/name,base/'tool-sources'/name)
p=base/'verification.json';v=json.loads(p.read_bytes());v.update(status='passed_scoped_local_dual_checker_linux_verification',linux_verification='passed independent terminal audit of exact public source 1e5338e9; all three targeted tests, Clippy and format; 924 Git blobs, all logs/test binary, 91 exact new outputs and both 92-file unchanged legacy sets verified',linux_audit='server-linux/final/final-audit.json',linux_receipt='server-linux/final/status.json');p.write_text(json.dumps(v,indent=2,sort_keys=True)+'\n')
p=parent/'unit-7-foundation-proofs-review.md';s=p.read_text();old='''Exact published-source Linux verification is running at 1e5338e9 after verifying
924 source/fixture Git blobs. A setup path typo was repaired on the same clean
checkout before any test had started; both setup attempts are retained.
Its terminal audit is pending. See verification-logs/foundation-proofs/.''';new='''Exact published-source Linux verification at 1e5338e9 passes all three targeted
tests, Clippy and format. Its independent terminal audit verifies 924 source/
fixture Git blobs, all test logs and actual test-binary hashes, 91 exact new
outputs and both 92-file unchanged legacy sets. A setup path typo was repaired
on the same clean checkout before any test had started; both attempts remain.
These scoped checks do not complete application proof assembly or W09.
See verification-logs/foundation-proofs/.''';assert old in s;p.write_text(s.replace(old,new))
p=repo/'develop/docs/08_csharp_practical_subset_design-todo.md';s=p.read_text();old='''Exact
public-source Linux verification is running at 1e5338e9; its terminal receipt
remains pending.''';new='''Exact
public-source Linux verification at 1e5338e9 passes all three targeted tests,
Clippy, format and its independent terminal archive audit.''';assert old in s;p.write_text(s.replace(old,new))
p=parent/'w09-completion-audit.json';v=json.loads(p.read_bytes());u=next(x for x in v['units'] if x['unit']==7);c=u['integrated_original_foundation_checkpoint'];c['status']='passed_scoped_local_same_byte_dual_checker_and_linux_checks';c['scope']='A single ordinary context preserves the full checked operation prefix and exact original universal type equations. All 182 fresh dual-checker stages and the exact public-source Linux tests/lint/format pass independent terminal audits. All 25 generic pending operations, 458 binding sequents without supplied component proofs, native/application scopes and every original application proof ID remain open; W09 is not complete.';p.write_text(json.dumps(v,indent=2)+'\n')
print('Preserved complete 1e5338e9 Linux receipt; W09 and application proof assembly remain incomplete.')
