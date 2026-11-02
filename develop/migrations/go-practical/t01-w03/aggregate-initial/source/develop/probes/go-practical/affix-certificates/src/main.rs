// The predecessor remains unchanged. The build recipe supplies an untrusted
// candidate producer with a separate CLI; both kernels still check every claim.
include!(concat!(env!("OUT_DIR"), "/predecessor/main.rs"));
