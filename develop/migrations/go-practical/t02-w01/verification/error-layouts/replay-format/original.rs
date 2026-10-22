//! Reuse completed checker results only for literally unchanged complete bytes.
use mpk_vc::go_practical_ordinary::generate_go_practical_error_layout_foundation;
use sha2::{Digest, Sha256};

#[test]
fn current_producer_keeps_every_previously_checked_complete_layout_stream() {
    let root=std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(
        "../../develop/migrations/go-practical/t02-w01/verification/error-layouts/final/certificates");
    let mut observations=vec![];
    for n in 0..=70 {
        let original=root.join(format!("matrix-N{n}.mpcert"));
        let report_path=original.with_extension("rust.json");
        let report:serde_json::Value=serde_json::from_slice(&std::fs::read(&report_path).unwrap()).unwrap();
        assert_eq!(report["verdict"],"accepted");
        assert_eq!(report["axiom_count"],0);
        assert_eq!(report["module"],format!("Mpk.Go.Practical.ErrorLayout.N{n}"));
        let prior=std::fs::read(&original).unwrap();
        let current=generate_go_practical_error_layout_foundation(n).unwrap();
        assert_eq!(current.certificate_bytes(),prior,"complete original bytes N{n}");
        let digest=format!("{:x}",Sha256::digest(&prior));
        assert_eq!(report["hashes"]["certificate"],digest);
        observations.push(serde_json::json!({"case_count":n,"bytes":prior.len(),"sha256":digest,
            "complete_original_and_current_bytes_literal_equal":true}));
    }
    if let Some(out)=std::env::var_os("MPK_GO_ERROR_LAYOUT_REPLAY_OUTPUT") {
        use std::io::Write;
        std::fs::OpenOptions::new().create_new(true).write(true).open(out).unwrap()
            .write_all(&serde_json::to_vec_pretty(&observations).unwrap()).unwrap();
    }
    println!("all71 complete current-produced streams literally equal the original checked bytes; no kernel recheck or broader gate");
}
