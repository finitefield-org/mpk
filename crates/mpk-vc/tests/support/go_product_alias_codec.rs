use mpk_cert::encode::{Certificate, LevelNode};
use mpk_vc::go_practical_ordinary::generate_go_practical_product_foundation;

pub(super) fn baseline() -> Certificate {
    static ORIGINAL: std::sync::OnceLock<Certificate> = std::sync::OnceLock::new();
    ORIGINAL
        .get_or_init(|| {
            mpk_cert::decode_canonical_certificate(
                generate_go_practical_product_foundation()
                    .unwrap()
                    .certificate_bytes(),
            )
            .unwrap()
        })
        .clone()
}
pub(super) fn global(c: &Certificate, name: &str) -> usize {
    c.declarations
        .iter()
        .position(|d| c.name_table[d.name as usize] == name)
        .unwrap()
}
pub(super) fn encode(mut c: Certificate) -> Vec<u8> {
    let old = c.name_table.clone();
    c.name_table.sort();
    for d in &mut c.declarations {
        d.name = c.name_table.binary_search(&old[d.name as usize]).unwrap() as u32;
    }
    for level in &mut c.level_table {
        if let LevelNode::Param(name) = level {
            *name = c.name_table.binary_search(&old[*name as usize]).unwrap() as u32;
        }
    }
    c.export_block = mpk_cert::build_export_block(&c).unwrap();
    c.axiom_report = mpk_cert::build_axiom_report(&c).unwrap();
    assert_eq!(c.axiom_report.summary.total_axiom_count, 0);
    c.hashes.export_hash = mpk_cert::export_block_hash(&c.export_block);
    c.hashes.axiom_report_hash = mpk_cert::axiom_report_hash_for_report(&c.axiom_report);
    mpk_cert::encode_certificate_bounded(&c, 2_097_152).unwrap()
}
