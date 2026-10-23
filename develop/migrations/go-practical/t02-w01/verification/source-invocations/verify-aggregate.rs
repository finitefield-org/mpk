fn main() {
    let path=std::env::args_os().nth(1).expect("complete original certificate path");
    let bytes=std::fs::read(path).expect("complete original bytes");
    let result=mpk_kernel::verify_certificate_bytes_json_output(&bytes);
    println!("{}",result.json);
    if !result.accepted {std::process::exit(1);}
}
