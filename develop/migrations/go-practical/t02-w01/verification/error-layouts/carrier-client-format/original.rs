//! Exercise the exported carrier name through an actual typed client.
use mpk_cert::encode::{Certificate,Declaration,DeclarationKind,LevelNode,TermNode};
use mpk_vc::go_practical_ordinary::{generate_go_practical_error_layout_foundation,
    import_go_practical_error_layout_foundation,GoOrdinaryError};

fn term(c:&mut Certificate,t:TermNode)->u32 {let i=c.term_table.len() as u32;c.term_table.push(t);i}
fn global(c:&Certificate,n:&str)->usize {c.declarations.iter().position(|d|c.name_table[d.name as usize]==n).unwrap()}
fn constant(c:&mut Certificate,n:&str)->u32 {let global=global(c,n) as u32;term(c,TermNode::Const{global,levels:vec![]})}
fn call(c:&mut Certificate,n:&str,arguments:Vec<u32>)->u32 {let function=constant(c,n);term(c,TermNode::App{function,arguments})}
fn candidate(wrong:bool,client:bool)->Vec<u8> {
    let f=generate_go_practical_error_layout_foundation(0).unwrap();
    let mut c=mpk_cert::decode_canonical_certificate(f.certificate_bytes()).unwrap();
    if client {
        let carrier=constant(&mut c,"Mpk.Go.Practical.ErrorLayout.N0.Carrier");
        let nil=constant(&mut c,"Mpk.Go.Practical.ErrorLayout.N0.Nil");
        let ty=call(&mut c,"Std.Eq",vec![carrier,nil,nil]);
        let proof=call(&mut c,"Std.Eq.refl",vec![carrier,nil]);
        let name=c.name_table.len() as u32;
        c.name_table.push("Control.ErrorLayout.NamedCarrierClient".into());
        c.declarations.push(Declaration{name,kind:DeclarationKind::Theorem{ty,proof}});
    }
    if wrong {
        let i=global(&c,"Mpk.Go.Practical.ErrorLayout.N0.Carrier");
        let boolean=constant(&mut c,"Std.Bool");
        let DeclarationKind::Def{ty,reducibility,..}=c.declarations[i].kind else{panic!()};
        c.declarations[i].kind=DeclarationKind::Def{ty,value:boolean,reducibility};
    }
    let names=c.name_table.clone();c.name_table.sort();
    for d in &mut c.declarations {d.name=c.name_table.binary_search(&names[d.name as usize]).unwrap() as u32;}
    for l in &mut c.level_table {if let LevelNode::Param(n)=l {*n=c.name_table.binary_search(&names[*n as usize]).unwrap() as u32;}}
    c.export_block=mpk_cert::build_export_block(&c).unwrap();
    c.axiom_report=mpk_cert::build_axiom_report(&c).unwrap();
    c.hashes.export_hash=mpk_cert::export_block_hash(&c.export_block);
    c.hashes.axiom_report_hash=mpk_cert::axiom_report_hash_for_report(&c.axiom_report);
    mpk_cert::encode_certificate_bounded(&c,2097152).unwrap()
}
#[test]
fn named_carrier_client_checks_and_wrong_named_carrier_genuinely_type_rejects() {
    use std::io::Write;
    for (wrong,client,name) in [(false,true,"honest-named-carrier-client"),
        (true,false,"unreferenced-carrier-is-only-math"),(true,true,"empty-carrier-extra-slot-client")] {
        let bytes=candidate(wrong,client);
        let accepted=!wrong || !client;
        let out=std::env::var_os("MPK_GO_ERROR_LAYOUT_CARRIER_CLIENT_OUTPUT").map(std::path::PathBuf::from);
        if let Some(out)=&out {
            std::fs::create_dir_all(out).unwrap();
            std::fs::OpenOptions::new().create_new(true).write(true).open(out.join(format!("{name}.mpcert"))).unwrap().write_all(&bytes).unwrap();
        }
        let result=mpk_kernel::verify_certificate_bytes_json_output(&bytes);
        if let Some(out)=out {
            std::fs::OpenOptions::new().create_new(true).write(true).open(out.join(format!("{name}.rust.json"))).unwrap().write_all(format!("{}\n",result.json).as_bytes()).unwrap();
        }
        assert_eq!(result.accepted,accepted,"{name}: {}",result.json);
        let report:serde_json::Value=serde_json::from_str(&result.json).unwrap();
        if !accepted {
            assert_eq!(report["error_code"],"KERNEL_CORE_CHECK");
            let detail:serde_json::Value=serde_json::from_str(report["error_detail"].as_str().unwrap()).unwrap();
            assert_eq!(detail["code"],"CORE_TYPE_MISMATCH");
        }else{
            assert_eq!(report["axiom_count"],0);
            assert_eq!(import_go_practical_error_layout_foundation(&bytes,0),Err(GoOrdinaryError::Linkage));
        }
    }
}
