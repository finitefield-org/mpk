"""Replace retired active plans with Go/Rust routing; retain full original bytes."""
from pathlib import Path
import hashlib,json,subprocess
ROOT=Path(__file__).resolve().parent
basis=json.loads((ROOT/'scope-document-originals.json').read_bytes())
archive=Path('develop/migrations/go-rust-only/historical-plans');archive.mkdir(exist_ok=False)
rows=[]
for row in basis['sources'][:2]:
    p=Path(row['path']);previous=p.read_bytes();mode=p.stat().st_mode&0o777
    original=subprocess.check_output(['/usr/bin/git','show',basis['basis_commit']+':'+row['path']])
    assert hashlib.sha256(original).hexdigest()==row['sha256'] and len(original)==row['bytes']
    for suffix,raw in [('.original',original),('.retirement-notice.original',previous)]:
        saved=archive/(p.name+suffix);saved.write_bytes(raw);saved.chmod(mode)
        rows.append({'original_path':str(p),'retained_path':str(saved),'bytes':len(raw),'sha256':hashlib.sha256(raw).hexdigest(),'mode':mode,'original_Git_epoch':suffix=='.original'})
    title='Retired multi-language implementation plan' if 'todo' in p.name else 'Retired multi-language frontend design'
    p.write_text('# '+title+'\n\n'+'''MPK supports [Go and Rust only](../specs/LANGUAGE_SUPPORT_SCOPE_V1.md).
Other-language implementation, resumption and release tasks have been removed
from the current plan. Continue the [Go work plan](09_go_practical_subset_design-todo.md)
and the [Rust design](05_rust_frontend_design.md).

'''+f'The [complete original record](../migrations/go-rust-only/historical-plans/{p.name}.original)\n'+'''is retained as historical evidence. It provides no current frontend support,
release acceptance, future language task or prerequisite for Go/Rust work.
''')
(ROOT/'retired-plan-preservation.json').write_text(json.dumps({'status':'complete_original_and_intermediate_plan_bytes_retained','basis_commit':basis['basis_commit'],'sources':rows,'no_scanner_rule_or_exclusion_changed':True},indent=2)+'\n')
print(json.dumps({'status':'retired_plan_routes_to_Go_Rust','complete_original_buffers':len(rows)}))
