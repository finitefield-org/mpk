"""Freeze an isolated source copy for the unary-product fix during live original checking."""
from pathlib import Path
import hashlib,io,json,os,subprocess,tarfile
ROOT=Path(__file__).resolve().parent
BASE='7ead1ea45e602338b52a0d9ddf6d30db92782432'
assert subprocess.check_output(['/usr/bin/git','rev-parse','HEAD']).decode().strip()==BASE
copy=Path('/private/tmp/mpk-source-value-relations-unary-copy');copy.mkdir(exist_ok=False)
raw=subprocess.check_output(['/usr/bin/git','archive','--format=tar',BASE,'Cargo.toml','Cargo.lock','crates','proofs/std'])
with tarfile.open(fileobj=io.BytesIO(raw)) as archive:archive.extractall(copy,filter='data')
changed=['crates/mpk-vc/src/go_practical_ordinary.rs','crates/mpk-vc/src/go_practical_ordinary/source_value_relations.rs','crates/mpk-vc/tests/go_practical_source_value_relations.rs','crates/mpk-vc/tests/go_practical_source_value_relations_controls.rs']
for name in changed:
    source=Path(name);target=copy/name;target.parent.mkdir(parents=True,exist_ok=True);target.write_bytes(source.read_bytes());target.chmod(source.stat().st_mode&0o777)
corpora=['error-dispatch/corpus.json','source-domains/corpus.json','source-error-representation/corpus-heterogeneous.json']
for suffix in corpora:
    source=ROOT.parent/suffix;target=copy/source.relative_to(Path.cwd());target.parent.mkdir(parents=True,exist_ok=True);target.write_bytes(source.read_bytes());target.chmod(source.stat().st_mode&0o777)
p=copy/changed[1];text=p.read_text();pos=text.index('    fn source(module:')
text=text[:pos]+'''    fn needs_function_adapter(&self) -> bool {
        match self {
            Self::Vector(..) => true,
            Self::Product(parts) if parts.len() == 1 => parts[0].needs_function_adapter(),
            _ => false,
        }
    }
'''+text[pos:]
text=text.replace('if matches!(view, View::Vector(..)) {','if view.needs_function_adapter() {');p.write_text(text)
p=copy/changed[3];p.write_text(p.read_text().replace('source-value-relations/final-small/certificates','source-value-relations/small-fixed/certificates'))
# This exact source copy owns its Cargo target; none of the live original
# executable, sources, output directories or source receipts is mutated.
manifest={'status':'isolated_unary_product_fix_prepared','basis_commit':BASE,'copy':str(copy),'archive_sha256':hashlib.sha256(raw).hexdigest(),'archive_bytes':len(raw),'overlaid_sources':changed,'copied_original_corpora':corpora,'source_fix':'Root vectors and unary products containing a vector expose their function carrier. All other source recipes remain unchanged.','original_live_inputs_mutated':False}
(ROOT/'isolated-copy.json').write_text(json.dumps(manifest,indent=2)+'\n');print(json.dumps(manifest))
