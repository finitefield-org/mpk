"""Private fixed Nat.recType extension on the retained Nat.induct prototype."""
import json
import shutil
from pathlib import Path
import control_foundation_build as prior
from run_proof_pilot import HERE, ROOT, json_bytes, sha

SEED = HERE/"termination-foundation"
PRODUCER = HERE/"termination-certificates"
SCOPE = "private canonical monomorphic Nat.recType returning Sort0 types plus retained Sort0 Nat.induct; production/legacy checkers and Certificate v0 byte format unchanged"


def files():
    data = prior.files()
    for name, raw in (
        ("crates/mpk-core/src/nat_rec_type.rs", (SEED/"rust/nat_rec_type.rs").read_bytes()),
        ("go-tools/mpk-checker-ref/nat_rec_type.go", (SEED/"go/nat_rec_type.go").read_bytes()),
    ):
        data[name] = raw
    def edit(path, old, new):
        data[path] = prior.once(data[path].decode(), old, new).encode()
    edit("crates/mpk-core/src/lib.rs", "pub mod nat_induct;",
         "pub mod nat_induct;\npub mod nat_rec_type;\npub use nat_rec_type::check_nat_rec_type_declaration;")
    edit("crates/mpk-core/src/infer.rs",
         "crate::nat_induct::check_nat_induct_constant_levels(env, global, !levels.is_empty())?;",
         "crate::nat_induct::check_nat_induct_constant_levels(env, global, !levels.is_empty())?;\n            crate::nat_rec_type::check_nat_rec_type_constant_levels(env, global, !levels.is_empty())?;")
    edit("crates/mpk-core/src/env.rs",
         "self.has_bool_cases(inductive) || crate::nat_induct::has_nat_induct(self, inductive)",
         "self.has_bool_cases(inductive) || crate::nat_induct::has_nat_induct(self, inductive) || crate::nat_rec_type::has_nat_rec_type(self,inductive)")
    edit("crates/mpk-kernel/src/decl_driver.rs",
         "mpk_core::check_nat_induct_declaration(&mut self.terms, &self.env, &name, ty, inductive, *generated).map_err(DeclarationCheckError::core)?;",
         "mpk_core::check_nat_induct_declaration(&mut self.terms, &self.env, &name, ty, inductive, *generated).map_err(DeclarationCheckError::core)?;\n                    mpk_core::check_nat_rec_type_declaration(&mut self.terms,&self.env,&name,ty,inductive,*generated).map_err(DeclarationCheckError::core)?;")
    path = "crates/mpk-core/src/reduce_inductive.rs"
    edit(path, "let info = generated_recursor_info(env, terms, recursor_spine.head, inductive, term)?;",
         'let info = generated_recursor_info(env, terms, recursor_spine.head, inductive, term)?;\n    let type_recursor = recursor_decl.name().as_str().ends_with(".recType");')
    raw = data[path].decode()
    old = "(info.bool_cases || info.nat_induct) &&"
    if raw.count(old) != 2:
        raise ValueError("type-recursion monomorphic reduction guards")
    data[path] = raw.replace(old, "(info.bool_cases || info.nat_induct || type_recursor) &&").encode()
    edit(path, '    if recursor_name == format!("{family_name}.induct") {',
         '''    if recursor_name == format!("{family_name}.recType") {
        let signature=crate::nat_rec_type::nat_rec_type_signature(terms,env,inductive)?;
        if env.lookup(recursor).expect("recursor").ty()!=signature.ty {
            return Err(iota_error(IotaReductionErrorKind::MalformedGeneratedRecursor,term,Some(recursor),Some(inductive)));
        }
        return Ok(GeneratedRecursorInfo {shape:MvpInductiveShape::Nat,constructors:signature.constructors.to_vec(),bool_cases:false,nat_induct:false});
    }
    if recursor_name == format!("{family_name}.induct") {''')
    path = "go-tools/mpk-checker-ref/core_check.go"
    edit(path, "e.hasBoolCases(declaration.inductive) || e.hasNatInduct(declaration.inductive)",
         "e.hasBoolCases(declaration.inductive) || e.hasNatInduct(declaration.inductive) || e.hasNatRecType(declaration.inductive)")
    edit(path, 'if e.hasNatInduct(declaration.inductive) {return 0,newCoreError(CoreCheckInvalidDeclaration,"constructor added to closed Nat induction family")}',
         'if e.hasNatInduct(declaration.inductive) || e.hasNatRecType(declaration.inductive) {return 0,newCoreError(CoreCheckInvalidDeclaration,"constructor added to closed Nat recursion family")}')
    edit(path, "if err := c.state.checkPriorNatInductUses(name,inductive);err!=nil {return err}",
         "if err := c.state.checkPriorNatInductUses(name,inductive);err!=nil {return err}\n                if err:=c.state.checkNatRecTypeDeclaration(name,ty,inductive,declaration.Generated);err!=nil {return err}\n                if err:=c.state.checkPriorNatRecTypeUses(name,inductive);err!=nil {return err}")
    edit(path, "if err := s.checkNatInductConstantLevels(coreGlobalID(node.A),len(node.Levels)!=0);err!=nil {return 0,err}",
         "if err := s.checkNatInductConstantLevels(coreGlobalID(node.A),len(node.Levels)!=0);err!=nil {return 0,err}\n        if err:=s.checkNatRecTypeConstantLevels(coreGlobalID(node.A),len(node.Levels)!=0);err!=nil {return 0,err}")
    path = "go-tools/mpk-checker-ref/defeq.go"
    edit(path, 'isInduct := recursorDecl.name == familyDecl.name+".induct"',
         'isInduct := recursorDecl.name == familyDecl.name+".induct"\n    isTypeRec := recursorDecl.name==familyDecl.name+".recType"')
    edit(path, 'recursorDecl.name != familyDecl.name+".rec" && !isCases && !isInduct',
         'recursorDecl.name != familyDecl.name+".rec" && !isCases && !isInduct && !isTypeRec')
    edit(path, '    if isInduct {',
         '''    if isTypeRec {
        if err:=s.checkNatRecTypeDeclaration(recursorDecl.name,recursorDecl.ty,recursorDecl.inductive,true);err!=nil {return 0,true,err}
        if len(function.Levels)!=0 {return 0,true,newCoreError(CoreCheckInvalidDeclaration,"Nat type recursion is monomorphic")}
    }
    if isInduct {''')
    edit(path, "(isCases || isInduct) && len(s.terms.node(node.Arguments[arity-1]).Levels) != 0",
         "(isCases || isInduct || isTypeRec) && len(s.terms.node(node.Arguments[arity-1]).Levels) != 0")
    return data


def sources():
    rows = {r["path"]:r["sha256"] for r in prior.sources()}
    for p in list(SEED.rglob("*"))+list(PRODUCER.rglob("*"))+[Path(__file__)]:
        if p.is_file():
            rows[p.relative_to(ROOT).as_posix()] = sha(p.read_bytes())
    return [{"path":p,"sha256":h} for p,h in sorted(rows.items())]


def prepare(destination):
    destination = Path(destination)
    prior.prepare(destination)
    data = files()
    for name, raw in data.items():
        p = destination/name
        p.parent.mkdir(parents=True,exist_ok=True)
        p.write_bytes(raw)
    shutil.rmtree(destination/"probe")
    shutil.copytree(PRODUCER,destination/"probe")
    p = destination/"probe/Cargo.toml"
    p.write_text(p.read_text().replace("[workspace]","").replace("../../../../crates/","../crates/"))
    p = destination/"probe/build.rs"
    p.write_text(p.read_text().replace("let root = manifest.ancestors().nth(4).unwrap();",
                                     "let root = Path::new("+json.dumps(str(ROOT))+");")
                 .replace('let source = manifest.parent().unwrap().join("certificates/src");',
                          'let source = root.join("develop/probes/go-practical/certificates/src");'))
    (destination/"Cargo.lock").write_bytes((PRODUCER/"Cargo.lock").read_bytes())
    receipt = {"scope":SCOPE,"inputs":sources(),
               "generated":[{"path":p,"sha256":sha(raw)} for p,raw in sorted(data.items())]}
    (destination/"private-build-inputs.json").write_bytes(json_bytes(receipt))
    return receipt


if __name__ == "__main__":
    import sys
    receipt = prepare(sys.argv[1])
    print(f"Prepared {len(receipt['generated'])} private mirrored inputs; production unchanged")
