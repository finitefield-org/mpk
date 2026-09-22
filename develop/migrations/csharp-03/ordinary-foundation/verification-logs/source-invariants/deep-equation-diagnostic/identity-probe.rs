#[path = "/Users/kazuyoshitoshiya/mpk/crates/mpk-vc/src/csharp_practical_ordinary_test_eval.rs"]
#[allow(dead_code)]
mod core_eval;
fn main() {
    use core_eval::{bit, run, V};
    let root = "/Users/kazuyoshitoshiya/mpk/develop/migrations/csharp-03/ordinary-foundation/source-invariants/";
    let manifest: serde_json::Value = serde_json::from_slice(&std::fs::read(format!("{root}certificates.json")).unwrap()).unwrap();
    let row = manifest["sources"].as_array().unwrap().iter().find(|r| r["id"] == "boundary-map-compound").unwrap();
    let metadata = &row["metadata"];
    let id = "mpk.csharp.source.b6b592255570c29551edd02432ac059654faaeb1a0fb2816c9d37351f25ee41c";
    let def = metadata["definitions"].as_array().unwrap().iter().find(|r| r["invariant"]["type_id"] == id).unwrap();
    let domain = metadata["public_domains"].as_array().unwrap().iter().find(|r| r["carrier"]["type_id"] == id).unwrap();
    let hex = std::fs::read_to_string(format!("{root}boundary-map-compound.hex")).unwrap();
    let bytes = hex.trim().as_bytes().chunks_exact(2).map(|c| u8::from_str_radix(std::str::from_utf8(c).unwrap(), 16).unwrap()).collect::<Vec<_>>();
    let cert = mpk_cert::decode_canonical_certificate(&bytes).unwrap();
    // This source has one stored member at exactly the source cube depth.
    // Its generated field getter is eta-expanded identity. Compare a test-only
    // beta/eta-equivalent direct identity; no generated corpus file is changed.
    assert_eq!(def["member_reads"].as_array().unwrap().len(), 1);
    assert_eq!(def["member_reads"][0]["depth"], 34);
    let getter = def["member_reads"][0]["definition"].as_str().unwrap();
    let mut direct = cert.clone();
    let index = direct.declarations.iter().position(|d| direct.name_table[d.name as usize] == getter).unwrap();
    let mpk_cert::encode::DeclarationKind::Def { value, .. } = direct.declarations[index].kind else { panic!() };
    let mpk_cert::encode::TermNode::Lam { ty, body: mut original } = direct.term_table[value as usize] else { panic!() };
    for _ in 0..34 {
        let mpk_cert::encode::TermNode::Lam { body, .. } = direct.term_table[original as usize] else { panic!("expected selector binder") };
        original = body;
    }
    let mpk_cert::encode::TermNode::App { function, ref arguments } = direct.term_table[original as usize] else { panic!("expected exact receiver application") };
    assert_eq!(direct.term_table[function as usize], mpk_cert::encode::TermNode::Var(34));
    assert_eq!(arguments.len(), 34);
    for (i, arg) in arguments.iter().enumerate() {
        assert_eq!(direct.term_table[*arg as usize], mpk_cert::encode::TermNode::Var(33-i as u32));
    }
    let receiver = direct.term_table.len() as u32;
    direct.term_table.push(mpk_cert::encode::TermNode::Var(0));
    let replacement = direct.term_table.len() as u32;
    direct.term_table.push(mpk_cert::encode::TermNode::Lam { ty, body: receiver });
    let mpk_cert::encode::DeclarationKind::Def { value, .. } = &mut direct.declarations[index].kind else { panic!() };
    *value = replacement;
    let started = std::time::Instant::now();
    eprintln!("begin body with exact direct identity member getter (diagnostic AST copy)");
    let result = bit(run(&direct, def["body_definition"].as_str().unwrap(), vec![V::UniformCube(false,34)]));
    eprintln!("end direct body: {result}, {:?}", started.elapsed());
    for (kind, name) in [("domain", domain["valid_definition"].as_str().unwrap()), ("body",def["body_definition"].as_str().unwrap())] {
        let started = std::time::Instant::now();
        eprintln!("begin {kind}, depth 34, independently constructed all-zero cube");
        let result = bit(run(&cert, name, vec![V::UniformCube(false,34)]));
        eprintln!("end {kind}: {result}, {:?}",started.elapsed());
    }
}
