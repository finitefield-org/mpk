fn main() {
 let args: Vec<String> = std::env::args().collect();
 assert!(cfg!(debug_assertions));
 if args[1] == "debug" { debug_assert!(std::hint::black_box(false), "profile-debug-probe"); }
 else { let value: u8 = args[2].parse().unwrap(); println!("{}", value + std::hint::black_box(1u8)); }
}
