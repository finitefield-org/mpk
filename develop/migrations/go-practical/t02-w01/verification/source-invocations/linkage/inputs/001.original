// Use the exact frozen integration-test source adapter; perform generation only.
include!("/Users/kazuyoshitoshiya/ff/mpk/crates/mpk-vc/tests/go_practical_source_invocation.rs");
fn main() {
    for name in ["pure", "wide"] {
        let s = selection(name);
        let c = generate_go_practical_source_invocation_catalogue(&s).unwrap();
        write(&format!("{name}-inventory.json"), &serde_json::to_vec_pretty(&json!({"sites": c.sites().iter().map(|site_ref| site(&s,site_ref)).collect::<Vec<_>>(), "cases": s.domain().error_cases().iter().map(|e|e.value_type_id()).collect::<Vec<_>>()})).unwrap());
        for (i, site_ref) in c.sites().iter().enumerate() {
            let key = format!("{name}-{i:03}");
            let result = c.generate_row(site_ref.caller(), site_ref.node());
            match result {
                Ok(row) => {
                    let original=mpk_cert::decode_canonical_certificate(row.prior_certificate_bytes()).unwrap();
                    let candidate=mpk_cert::decode_canonical_certificate(row.certificate_bytes()).unwrap();
                    let after=dependencies::declarations(&candidate);
                    for (name,value) in dependencies::declarations(&original){assert_eq!(after.get(&name),Some(&value),"complete original declaration {name}");}
                    assert_eq!(c.import_row(row.certificate_bytes(),site_ref.caller(),site_ref.node()).unwrap(),row);
                    write(&format!("{key}.mpcert"), row.certificate_bytes());
                    write(&format!("{key}.sources.json"), &serde_json::to_vec_pretty(&metadata(name, &s, &row)).unwrap());
                    println!("{}", json!({"name":key,"generated":true,"bytes":row.certificate_bytes().len(),"kernel_checked":false}));
                }
                Err(error) => {
                    write(&format!("{key}.generation-error.json"), &serde_json::to_vec_pretty(&json!({"name":key,"error":format!("{error:?}"),"site":site(&s,site_ref),"kernel_checked":false})).unwrap());
                    panic!("complete original {key}: {error:?}");
                }
            }
        }
    }
}
