// Diagnostic only: locate the earliest failing declaration of the unchanged
// rejected stream. Prefixes are never source foundations or accepted artifacts.
fn main() {
    let path=std::env::args().nth(1).unwrap();
    let original=mpk_cert::decode_canonical_certificate(&std::fs::read(path).unwrap()).unwrap();
    let (mut low,mut high)=(0,original.declarations.len());
    while low+1<high {
        let middle=(low+high)/2;
        let mut c=original.clone();c.declarations.truncate(middle);
        match mpk_kernel::check_declarations(&c) {
            Ok(_)=>{println!("prefix {middle}: declarations checked");low=middle;},
            Err(e)=>{println!("prefix {middle}: {}",e.detail());high=middle;}
        }
    }
    println!("earliest failing declaration index {}: {}",high-1,original.name_table[original.declarations[high-1].name as usize]);
}
