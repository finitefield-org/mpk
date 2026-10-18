use super::*;
use foundation_proof_tests::{false_type, unchanged_prefix};
use identity_proof_tests::app;
use mpk_cert::encode::{DeclarationKind, TermNode};

fn hex(path: std::path::PathBuf) -> Vec<u8> {
    let value = fs::read_to_string(path)
        .unwrap()
        .split_whitespace()
        .collect::<String>();
    (0..value.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&value[i..i + 2], 16).unwrap())
        .collect()
}
#[test]
fn csharp_03_t06_w09_actual_default_proofs_original_source() {
    std::thread::Builder::new().stack_size(64*1024*1024).spawn(|| {
        let bundle=b();let out=std::env::var_os("MPK_W09_ACTUAL_DEFAULT_PROOFS_OUT").map(std::path::PathBuf::from);if let Some(p)=&out{fs::create_dir_all(p).unwrap();}
        let base=Path::new(env!("CARGO_MANIFEST_DIR")).join("../../develop/migrations/csharp-03/ordinary-foundation/verification-logs");
        let filter=std::env::var("MPK_W09_ACTUAL_DEFAULT_SOURCE_FILTER").ok();
        let(mut contexts,mut proofs,mut supplied,mut remaining,mut pending,mut pending_defaults,mut unchanged)=(0,0,0,0,0,0,0);
        for(id,row,facts)in sources(){
            if filter.as_ref().is_some_and(|f|f!=&id){continue}
            let(context,captures)=support::replay_context(&bundle,&row);
            let source=ValidatedDataSource::import_captured_facts(&bundle,&context,&captures,&serde_json::to_vec(&facts).unwrap()).unwrap();
            let emitted=emit_data_phase(&bundle,&context,&captures,&source).unwrap();let vir=emitted.vir();
            eprintln!("{id}: preserve original complete actual-default sequents");
            let p=generate_csharp_practical_ordinary_default_proofs(vir).unwrap_or_else(|e|panic!("{id}: {e:?}"));
            assert_eq!(p.identity().canonical_bytes(),fs::read(base.join(format!("identity-proofs/local/certificates/{id}.json"))).unwrap());
            let old=hex(if p.identity().proofs().is_empty(){base.join(format!("foundation-proofs/local/certificates/{id}.hex"))}else{base.join(format!("identity-proofs/local/certificates/{id}.hex"))});
            assert_eq!(p.identity().certificate_bytes(),old);
            let full=generate_csharp_practical_vc(PracticalVcSource{artifact_context:&context,captured_inputs:&captures,vir}).unwrap();let vc=full.binding_vcs();
            let original_defaults=vc.sequents().iter().filter(|s|s.kind=="actual_default").collect::<Vec<_>>();
            let definition=generate_csharp_practical_ordinary_binding_defaults(vir).unwrap();
            assert_eq!(p.proofs().iter().map(|p|&p.sequent).collect::<Vec<_>>(),definition.conditions().iter().map(|c|&c.sequent).collect::<Vec<_>>());
            assert_eq!(p.pending_defaults(),definition.pending_defaults());
            let default_ids=p.proofs().iter().map(|p|&p.sequent.id).chain(p.pending_defaults().iter().map(|p|&p.sequent.id)).collect::<BTreeSet<_>>();
            assert_eq!(original_defaults.iter().map(|s|&s.id).collect::<BTreeSet<_>>(),default_ids);
            assert!(p.pending_defaults().iter().all(|p|p.reason==OrdinaryBindingDefaultPendingReason::SourceUseProofRequired));
            let all=p.identity().supplied_binding_sequent_ids().iter().chain(p.proofs().iter().map(|p|&p.sequent.id)).collect::<BTreeSet<_>>();
            assert_eq!(p.supplied_binding_sequent_ids(),vc.sequents().iter().filter(|s|all.contains(&s.id)).map(|s|s.id.clone()).collect::<Vec<_>>());
            assert_eq!(p.remaining_binding_sequent_ids(),vc.sequents().iter().filter(|s|!all.contains(&s.id)).map(|s|s.id.clone()).collect::<Vec<_>>());
            assert_eq!(p.pending_proof_ids(),vc.sequents().iter().map(|s|s.id.clone()).collect::<Vec<_>>());
            let c=mpk_cert::decode_canonical_certificate(p.certificate_bytes()).unwrap();let before=mpk_cert::decode_canonical_certificate(p.definition_certificate_bytes()).unwrap();
            unchanged_prefix(&mpk_cert::decode_canonical_certificate(&old).unwrap(),&before);unchanged_prefix(&before,&c);
            let metadata:Value=serde_json::from_slice(&p.canonical_bytes()).unwrap();assert_eq!(metadata["application_scope_pending"],true);assert_eq!(metadata["proof_check_pending"],true);assert!(metadata["static_transformers"].as_u64().unwrap()<=mpk_vc::csharp_practical_vc_model::STATIC_TRANSFORMERS_MAX);
            if p.proofs().is_empty(){assert_eq!(p.certificate_bytes(),old);unchanged+=1;}else{
                validate_csharp_practical_certificate_structure(&c).unwrap();assert!(c.proof_node_table.is_empty()&&c.theory_certificates.is_empty());assert_eq!(mpk_kernel::verify_certificate_bytes(p.certificate_bytes()).unwrap().axiom_count,0);
                assert!(!metadata["compatible_shared_declarations"].as_array().unwrap().is_empty());
                for proof in p.proofs(){
                    assert!(original_defaults.contains(&&proof.sequent));assert_eq!(proof.goals.iter().map(|g|&g.goal).collect::<Vec<_>>(),proof.sequent.goals.iter().collect::<Vec<_>>());
                    assert!(proof.sequent.subjects.is_empty()&&proof.sequent.assumptions.is_empty());
                    let declaration=c.declarations.iter().find(|d|c.name_table[d.name as usize]==proof.proposition_definition).unwrap();let DeclarationKind::Def{value,..}=declaration.kind else{panic!("missing closed theorem type")};
                    let conjunction=app(&c,value,"Std.Logic.And");assert_eq!(conjunction.len(),2);
                    for(&ty,goal)in conjunction.iter().zip(&proof.goals){
                        let terms=app(&c,ty,"Std.Eq");assert_eq!(terms.len(),3);let TermNode::Const{global,..}=c.term_table[terms[1]as usize]else{panic!("missing exact original goal")};assert_eq!(c.name_table[c.declarations[global as usize].name as usize],goal.definition);
                        let TermNode::Const{global,..}=c.term_table[terms[2]as usize]else{panic!("missing original truth")};assert_eq!(c.name_table[c.declarations[global as usize].name as usize],"Std.Bool.true");
                    }
                }
                let goal=&p.proofs()[0].goals[0].definition;assert_eq!(mpk_kernel::verify_certificate_bytes(&false_type(before,goal)).unwrap().axiom_count,0);let wrong=false_type(c,goal);assert_eq!(mpk_kernel::verify_certificate_bytes(&wrong).unwrap_err().kind(),mpk_kernel::VerificationErrorKind::CoreCheck);
                assert_eq!(import_csharp_practical_ordinary_default_proofs(&p.canonical_bytes(),p.certificate_bytes(),vir).unwrap(),p);
                assert!(import_csharp_practical_ordinary_default_proofs(&p.canonical_bytes(),&old,vir).is_err());assert!(import_csharp_practical_ordinary_default_proofs(&p.canonical_bytes(),&wrong,vir).is_err());
                for field in ["schema","identity","original_default_program_sha256","original_default_certificate_sha256","proofs","pending_defaults","supplied_binding_sequent_ids","remaining_binding_sequent_ids"]{let mut bad=metadata.clone();bad[field]=json!(null);assert!(import_csharp_practical_ordinary_default_proofs(&serde_json::to_vec(&bad).unwrap(),p.certificate_bytes(),vir).is_err(),"{field}");}
                if let Some(dir)=&out{for(suffix,bytes)in[("",p.certificate_bytes()),("-wrong",wrong.as_slice())]{fs::write(dir.join(format!("{id}{suffix}.hex")),bytes.iter().map(|b|format!("{b:02x}")).collect::<String>()+"\n").unwrap();}}
            }
            if let Some(dir)=&out{fs::write(dir.join(format!("{id}.json")),p.canonical_bytes()).unwrap();}
            contexts+=1;proofs+=p.proofs().len();supplied+=p.supplied_binding_sequent_ids().len();remaining+=p.remaining_binding_sequent_ids().len();pending+=p.pending_proof_ids().len();pending_defaults+=p.pending_defaults().len();
        }
        eprintln!("actual-default proof assembly: {contexts} contexts, {proofs} proofs, {supplied} supplied, {remaining} remaining, {pending} application IDs pending, {pending_defaults} source-use defaults pending, {unchanged} unchanged certificates");
        if filter.is_none(){assert_eq!((contexts,proofs,supplied,remaining,pending,pending_defaults,unchanged),(45,3,534,453,987,32,42));}
    }).unwrap().join().unwrap();
}
