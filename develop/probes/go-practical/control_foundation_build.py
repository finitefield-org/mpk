"""Private additive Nat induction prototype; production/checker sources stay intact."""
from pathlib import Path
import shutil
from run_proof_pilot import HERE,ROOT,sha,json_bytes

SEED=HERE/'control-foundation'
CRATES=('mpk-core','mpk-cert','mpk-theory','mpk-kernel')
SCOPE='private proposed Sort0 Nat induction extension in mirrored checker sources; production checkers, legacy Nat.rec and public registry unchanged'
def once(s,a,b):
 if s.count(a)!=1:raise ValueError('Nat induction extension anchor: '+a[:100])
 return s.replace(a,b,1)
def sources():
 paths=[]
 for crate in CRATES:
  p=ROOT/'crates'/crate;paths.extend(x for x in p.rglob('*') if x.is_file())
 paths.extend(p for p in (ROOT/'go-tools/mpk-checker-ref').rglob('*') if p.is_file())
 paths.extend(SEED.rglob('*'));paths.extend((HERE/'control-certificates').rglob('*'))
 paths.extend((HERE/'certificates/src').rglob('*.rs'))
 paths.extend([Path(__file__),ROOT/'proofs/std/nat/std-nat.hex',ROOT/'fixtures/core-bool-cases/right-identity.hex',ROOT/'develop/probes/go-practical/support/builder.rs'])
 return [{'path':p.relative_to(ROOT).as_posix(),'sha256':sha(p.read_bytes())} for p in sorted(set(paths)) if p.is_file()]
def files():
 result={}
 for crate in CRATES:
  for p in (ROOT/'crates'/crate).rglob('*'):
   if p.is_file():result[p.relative_to(ROOT).as_posix()]=p.read_bytes()
 for p in (ROOT/'go-tools/mpk-checker-ref').rglob('*'):
  if p.is_file():result[p.relative_to(ROOT).as_posix()]=p.read_bytes()
 def edit(path,a,b):result[path]=once(result[path].decode(),a,b).encode()
 result['crates/mpk-core/src/nat_induct.rs']=(SEED/'rust/nat_induct.rs').read_bytes()
 for p in (SEED/'go').glob('*.go'):result['go-tools/mpk-checker-ref/'+p.name]=p.read_bytes()
 edit('crates/mpk-core/src/lib.rs','pub mod bool_cases;','pub mod bool_cases;\npub mod nat_induct;\npub use nat_induct::check_nat_induct_declaration;')
 edit('crates/mpk-core/src/infer.rs','crate::bool_cases::check_bool_cases_constant_levels(env, global, !levels.is_empty())?;','crate::bool_cases::check_bool_cases_constant_levels(env, global, !levels.is_empty())?;\n            crate::nat_induct::check_nat_induct_constant_levels(env, global, !levels.is_empty())?;')
 edit('crates/mpk-core/src/env.rs','if self.has_bool_cases(inductive) {','if self.has_bool_cases(inductive) || crate::nat_induct::has_nat_induct(self, inductive) {')
 edit('crates/mpk-core/src/env.rs','.with_detail("kind", "closed_bool_cases_family")','.with_detail("kind", if crate::nat_induct::has_nat_induct(self,inductive) {"closed_nat_induct_family"} else {"closed_bool_cases_family"})')
 edit('crates/mpk-kernel/src/decl_driver.rs','                    if *generated {\n                        self.env\n                            .register_generated_recursor','                    mpk_core::check_nat_induct_declaration(&mut self.terms, &self.env, &name, ty, inductive, *generated).map_err(DeclarationCheckError::core)?;\n                    if *generated {\n                        self.env\n                            .register_generated_recursor')
 r='crates/mpk-core/src/reduce_inductive.rs'
 edit(r,'let arity = if info.bool_cases {','let arity = if info.bool_cases || info.nat_induct {')
 edit(r,'if info.bool_cases && !recursor_spine.levels.is_empty() {','if (info.bool_cases || info.nat_induct) && !recursor_spine.levels.is_empty() {')
 edit(r,'if info.bool_cases && !major_spine.levels.is_empty() {','if (info.bool_cases || info.nat_induct) && !major_spine.levels.is_empty() {')
 edit(r,'    if major_spine.head == info.constructors[0] {\n        if major_spine.arguments.is_empty() {\n            return Ok(recursor_spine.arguments[0]);','    if major_spine.head == info.constructors[0] {\n        if major_spine.arguments.is_empty() {\n            return Ok(recursor_spine.arguments[usize::from(info.nat_induct)]);')
 edit(r,'''    let recursive_result = terms.app(
        recursor_const,
        [
            recursor_spine.arguments[0],
            recursor_spine.arguments[1],
            predecessor,
        ],
    );
    Ok(terms.app(recursor_spine.arguments[1], [predecessor, recursive_result]))''','''    let offset = usize::from(info.nat_induct);
    let mut arguments = recursor_spine.arguments[..2+offset].to_vec();
    arguments.push(predecessor);
    let recursive_result = terms.app(recursor_const,arguments);
    Ok(terms.app(recursor_spine.arguments[1+offset], [predecessor, recursive_result]))''')
 edit(r,'    if recursor_name == format!("{family_name}.cases") {','''    if recursor_name == format!("{family_name}.induct") {
        let signature = crate::nat_induct::nat_induct_signature(terms,env,inductive)?;
        if env.lookup(recursor).expect("recursor").ty()!=signature.ty {
            return Err(iota_error(IotaReductionErrorKind::MalformedGeneratedRecursor,term,Some(recursor),Some(inductive)));
        }
        return Ok(GeneratedRecursorInfo {shape:MvpInductiveShape::Nat,constructors:signature.constructors.to_vec(),bool_cases:false,nat_induct:true});
    }
    if recursor_name == format!("{family_name}.cases") {''')
 edit(r,'            bool_cases: true,','            bool_cases: true,\n            nat_induct: false,')
 edit(r,'                bool_cases: false,','                bool_cases: false,\n                nat_induct: false,')
 edit(r,'    bool_cases: bool,','    bool_cases: bool,\n    nat_induct: bool,')
 g='go-tools/mpk-checker-ref/core_check.go'
 edit(g,'if declaration.tag == DeclConstructor && e.hasBoolCases(declaration.inductive) {','if declaration.tag == DeclConstructor && (e.hasBoolCases(declaration.inductive) || e.hasNatInduct(declaration.inductive)) {')
 edit(g,'return 0, newCoreError(CoreCheckInvalidDeclaration, "constructor added to closed Bool cases family")','if e.hasNatInduct(declaration.inductive) {return 0,newCoreError(CoreCheckInvalidDeclaration,"constructor added to closed Nat induction family")}\n        return 0, newCoreError(CoreCheckInvalidDeclaration, "constructor added to closed Bool cases family")')
 edit(g,'\t\t\tif declaration.Tag == DeclRecursor {','\t\t\tif declaration.Tag == DeclRecursor {\n                if err := c.state.checkNatInductDeclaration(name,ty,inductive,declaration.Generated);err!=nil {return err}\n                if err := c.state.checkPriorNatInductUses(name,inductive);err!=nil {return err}')
 edit(g,'\t\tif err := s.checkBoolCasesConstantLevels(coreGlobalID(node.A), len(node.Levels) != 0); err != nil {','\t\tif err := s.checkNatInductConstantLevels(coreGlobalID(node.A),len(node.Levels)!=0);err!=nil {return 0,err}\n\t\tif err := s.checkBoolCasesConstantLevels(coreGlobalID(node.A), len(node.Levels) != 0); err != nil {')
 g='go-tools/mpk-checker-ref/defeq.go'
 edit(g,'\tisCases := recursorDecl.name == familyDecl.name+".cases"','\tisInduct := recursorDecl.name == familyDecl.name+".induct"\n\tisCases := recursorDecl.name == familyDecl.name+".cases"')
 edit(g,'recursorDecl.name != familyDecl.name+".rec" && !isCases','recursorDecl.name != familyDecl.name+".rec" && !isCases && !isInduct')
 edit(g,'\tif isCases {','''    if isInduct {
        if err:=s.checkNatInductDeclaration(recursorDecl.name,recursorDecl.ty,recursorDecl.inductive,true);err!=nil {return 0,true,err}
        if len(function.Levels)!=0 {return 0,true,newCoreError(CoreCheckInvalidDeclaration,"Nat induction is monomorphic")}
        arity=4;branchOffset=1
        if len(node.Arguments)<arity {return term,false,nil}
    }
\tif isCases {''')
 edit(g,'if isCases && len(s.terms.node(node.Arguments[arity-1]).Levels) != 0 {','if (isCases || isInduct) && len(s.terms.node(node.Arguments[arity-1]).Levels) != 0 {')
 edit(g,'\t\t\treduced = node.Arguments[0]','\t\t\treduced = node.Arguments[branchOffset]')
 edit(g,'s.reduceGeneratedNatSuccRecursor(recursor, function.Levels, node.Arguments, majorArgs[0])','s.reduceGeneratedNatSuccRecursor(recursor, function.Levels, node.Arguments, majorArgs[0],branchOffset)')
 edit(g,'predecessor coreTermID) coreTermID {','predecessor coreTermID,offset int) coreTermID {')
 edit(g,'''\trecursiveResult := s.terms.app(recursorConst, []coreTermID{
\t\trecursorArgs[0],
\t\trecursorArgs[1],
\t\tpredecessor,
\t})
\treturn s.terms.app(recursorArgs[1], []coreTermID{predecessor, recursiveResult})''','''    arguments:=append([]coreTermID{},recursorArgs[:2+offset]...)
    arguments=append(arguments,predecessor)
    recursiveResult:=s.terms.app(recursorConst,arguments)
    return s.terms.app(recursorArgs[1+offset],[]coreTermID{predecessor,recursiveResult})''')
 return result
def producer_files():
 result={p.relative_to(HERE/'certificates/src').as_posix():p.read_text() for p in (HERE/'certificates/src').rglob('*.rs')}
 s=result['main.rs'];s=''.join(('//'+x[3:] if x.startswith('//!') else x)+'\n' for x in s.splitlines())
 builder=ROOT/'develop/probes/go-practical/support/builder.rs'
 s=once(s,'"../../support/builder.rs"',__import__('json').dumps(str(builder)))
 s=once(s,'let args = std::env::args().collect::<Vec<_>>();','let args = std::env::args().collect::<Vec<_>>();\n    if args.get(1).map(String::as_str)==Some("control-generate") { assert_eq!(args.len(),4); word::generate_control(Path::new(&args[2]),Path::new(&args[3])); return; }')
 result['main.rs']=s;result['word.rs']+='\nmod control;\npub(super) fn generate_control(root:&Path,output:&Path) { control::generate(root,output); }\n'
 result['word/control.rs']=(HERE/'control-certificates/src/control.rs').read_text()
 return {p:s.encode() for p,s in result.items()}
def prepare(destination):
 destination=Path(destination)
 if destination.exists():raise ValueError('private prototype output already exists')
 data=files()
 for name,raw in data.items():
  p=destination/name;p.parent.mkdir(parents=True,exist_ok=True);p.write_bytes(raw)
 manifest='''[workspace]
members=["crates/mpk-core","crates/mpk-cert","crates/mpk-theory","crates/mpk-kernel","probe"]
resolver="2"
[workspace.package]
version="0.1.0"
edition="2021"
license="MIT OR Apache-2.0"
repository="https://github.com/"
homepage="https://github.com/"
[workspace.lints.rust]
unsafe_code="forbid"
'''
 (destination/'Cargo.toml').write_text(manifest)
 shutil.copytree(HERE/'control-certificates',destination/'probe')
 p=destination/'probe/Cargo.toml';text=p.read_text().replace('[workspace]','').replace('../../../../crates/','../crates/');p.write_text(text)
 p=destination/'probe/build.rs';text=p.read_text().replace('let root = manifest.ancestors().nth(4).unwrap();',f'let root = Path::new({__import__("json").dumps(str(ROOT))});').replace('let source = manifest.parent().unwrap().join("certificates/src");','let source = root.join("develop/probes/go-practical/certificates/src");');p.write_text(text)
 (destination/'Cargo.lock').write_bytes((SEED/'Cargo.lock').read_bytes())
 receipt={'scope':SCOPE,'inputs':sources(),'generated':[{ 'path':p,'sha256':sha(raw)} for p,raw in sorted(data.items())]}
 (destination/'private-build-inputs.json').write_bytes(json_bytes(receipt));return receipt
if __name__=='__main__':
 import sys
 result=prepare(sys.argv[1]);print(f"Prepared {len(result['generated'])} private mirrored inputs; production sources unchanged")
