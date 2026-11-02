"""Retire remaining active roadmap routing, retaining complete original Git blobs."""
from pathlib import Path
import hashlib,json,subprocess
ROOT=Path(__file__).resolve().parent
BASE='7ead1ea45e602338b52a0d9ddf6d30db92782432'
def git(*args):return subprocess.check_output(['/usr/bin/git',*args])
assert git('rev-parse','HEAD').decode().strip()==BASE
paths=['develop/docs/06_multilanguage_frontend_design.md','develop/docs/06_multilanguage_frontend_design-todo.md','develop/roadmap/ROADMAP.md','develop/README.md','develop/migrations/go-practical/t02-w01/README.md']
rows=[]
for name in paths:
    p=Path(name);raw=p.read_bytes();assert raw==git('show',BASE+':'+name)
    rows.append({'path':name,'bytes':len(raw),'sha256':hashlib.sha256(raw).hexdigest(),'mode':p.stat().st_mode&0o777,'original_commit':BASE,'original_git_blob':git('rev-parse',BASE+':'+name).decode().strip(),'complete_original_bytes_retained_in_git':True})
(ROOT/'scope-document-originals.json').write_text(json.dumps({'basis_commit':BASE,'sources':rows},indent=2)+'\n')
for name in paths[:2]:
    p=Path(name);text=p.read_text();heading,body=text.split('\n',1)
    notice='''
Retired by the [Go/Rust-only language scope](../specs/LANGUAGE_SUPPORT_SCOPE_V1.md).
This document preserves historical decisions and receipts; none of its
other-language implementation, resumption or release tasks are current work.
The active plans are the [Go work plan](09_go_practical_subset_design-todo.md)
and the [Rust design](05_rust_frontend_design.md).

'''
    p.write_text(heading+'\n'+notice+body)
p=Path(paths[2]);text=p.read_text();start=text.index('Post-Rust source-language expansion is governed by');end=text.index('The Phase 0 through Phase 12 entries below',start)
text=text[:start]+'''The current product supports Go and Rust only, under
[the normative language scope](../specs/LANGUAGE_SUPPORT_SCOPE_V1.md).
Practical Go v1/v2 continues in the accepted feature priority and serial T/W
order in [the Go work plan](../docs/09_go_practical_subset_design-todo.md).
T02-W01 remains in progress; completing the source/program foundation and
local/native release gates is required before activation. No removed-language
release or resumption is part of this roadmap. Earlier multi-language designs
and migration receipts retain only their historical meaning.

'''+text[end:];p.write_text(text)
p=Path(paths[3]);text=p.read_text().replace('`docs/`, `tasks/` and `roadmap/` contain supporting design and tracking files.','''`docs/`, `tasks/` and `roadmap/` contain supporting design and tracking files.
Older multi-language plans are retired historical records. They do not
authorize another frontend, future language phase or release prerequisite.''');p.write_text(text)
p=Path(paths[4]);text=p.read_text();heading,body=text.split('\n',1)
p.write_text(heading+'''

Current scope: [Go and Rust only](../../../specs/LANGUAGE_SUPPORT_SCOPE_V1.md).
Earlier checkpoint text below records its original epoch. Removed-language
work and historical gate flags do not apply to current execution. The current
cumulative T02 gate is final W04: `./scripts/check-fast.sh`, with no
language-specific skip option. Release gates are the root README's local
Go/Rust Linux gates.
'''+body)
print(json.dumps({'status':'remaining_active_multilanguage_roadmap_retired','original_complete_Git_blobs':len(rows)}))
