"""Check the live Go/Rust scope routing and retain exact reviewed document bytes."""
from pathlib import Path
import hashlib,json,re,urllib.parse
ROOT=Path(__file__).resolve().parent;REPO=Path.cwd()
paths=[Path(r['path']) for r in json.loads((ROOT/'scope-document-originals.json').read_bytes())['sources']]
out=ROOT/'scope-document-review';out.mkdir(exist_ok=False)
inputs=[];links=[]
for p in paths:
    raw=p.read_bytes();saved=out/'inputs'/p;saved.parent.mkdir(parents=True,exist_ok=True);saved.write_bytes(raw);saved.chmod(p.stat().st_mode&0o777)
    inputs.append({'path':str(p),'bytes':len(raw),'sha256':hashlib.sha256(raw).hexdigest(),'mode':p.stat().st_mode&0o777,'complete_reviewed_document':str(saved)})
    for target in re.findall(r'\[[^\]]*\]\(([^)]+)\)',raw.decode()):
        target=target.strip('<>');url=urllib.parse.urlsplit(target)
        if url.scheme or not url.path:continue
        local=p.parent/urllib.parse.unquote(url.path)
        links.append({'source':str(p),'target':target,'exists':local.exists()})
for p in paths[:2]:
    text=p.read_text();assert text.startswith('# Retired multi-language') and 'Other-language implementation, resumption and release tasks have been removed' in text
assert 'resume it after Go' not in Path('develop/roadmap/ROADMAP.md').read_text()
registry=json.loads(Path('release/bundles/semantic-profile-registry.json').read_bytes())
bundles=json.loads(Path('release/bundles/bundle-registry.json').read_bytes())
assert registry['revision']==4 and {p['source_language'] for p in registry['profiles']}=={'go','rust'}
assert len(bundles['frontend_bundles'])==len(bundles['toolchain_bundles'])==2 and len(bundles['tuples'])==3
assert {t['semantic_context']['source_language'] for t in bundles['tuples']}=={'go','rust'}
assert all(row['exists'] for row in links)
report={'status':'Go_Rust_scope_document_links_and_registry_inventory_passed','inputs':inputs,'links':links,'installed_registry_revision':4,'program_languages':['go','rust'],'frontend_bundles':2,'toolchain_bundles':2,'release_tuples':3,'scope':'Affected documentation routing and unchanged registry inventory only. Existing Go/Rust support tests remain in their actual earlier epochs. This is not a fresh native frontend or release gate.'}
(out/'result.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps({'status':report['status'],'reviewed_documents':len(inputs),'local_links':len(links)}))
