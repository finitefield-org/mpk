// Observe the unchanged full failed stream after only removing redundant
// nonvector identity-view applications. This is diagnostic, not source import.
fn main() {
    use mpk_cert::encode::TermNode;
    let args=std::env::args().collect::<Vec<_>>();
    let mut c=mpk_cert::decode_canonical_certificate(&std::fs::read(&args[1]).unwrap()).unwrap();
    let names=std::fs::read_to_string(&args[2]).unwrap();
    let mut changed=0;
    for i in 0..c.term_table.len() {
        if let TermNode::App{function,arguments}=c.term_table[i].clone() {
            if let TermNode::Const{global,..}=c.term_table[function as usize] {
                let name=&c.name_table[c.declarations[global as usize].name as usize];
                if names.lines().any(|n|n==name) {
                    assert_eq!(arguments.len(),1);
                    c.term_table[i]=c.term_table[arguments[0] as usize].clone();changed+=1;
                }
            }
        }
    }
    assert!(changed>0);
    c.export_block=mpk_cert::build_export_block(&c).unwrap();
    c.axiom_report=mpk_cert::build_axiom_report(&c).unwrap();
    c.hashes.export_hash=mpk_cert::export_block_hash(&c.export_block);
    c.hashes.axiom_report_hash=mpk_cert::axiom_report_hash_for_report(&c.axiom_report);
    let raw=mpk_cert::encode_certificate_bounded(&c,2097152).unwrap();
    std::fs::write(&args[3],&raw).unwrap();
    let r=mpk_kernel::verify_certificate_bytes_json_output(&raw);
    println!("{}",r.json);assert!(r.accepted);
}
