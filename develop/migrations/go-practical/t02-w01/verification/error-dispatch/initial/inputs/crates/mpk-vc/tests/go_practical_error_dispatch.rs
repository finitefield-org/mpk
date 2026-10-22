use mpk_vc::go_practical_type_keys::*;
use serde_json::{json,Value};
use std::path::PathBuf;
fn directory()->PathBuf{PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../develop/migrations/go-practical/t02-w01/verification/error-dispatch")}
fn corpus()->Value{serde_json::from_slice(&std::fs::read(directory().join("corpus.json")).unwrap()).unwrap()}
fn unhex(v:&Value)->Vec<u8>{v.as_str().unwrap().as_bytes().chunks_exact(2).map(|p|u8::from_str_radix(std::str::from_utf8(p).unwrap(),16).unwrap()).collect()}
fn keys(c:&Value)->GoTypeKeyComponents{let p=&c["input"];let sources=p["sources"].as_array().unwrap().iter().map(|s|GoTypeSourceBuffer::import(&serde_json::to_vec(&s["object"]).unwrap(),&unhex(&s["raw_hex"])).unwrap()).collect::<Vec<_>>();let keys=import_go_type_key_components(&unhex(&p["components_hex"]),&sources).unwrap();assert_eq!(keys.types().len(),1);keys}
fn retain(name:&str,value:&Value){if let Some(output)=std::env::var_os("MPK_GO_ERROR_DISPATCH_RUST_OUTPUT"){use std::io::Write;let p=PathBuf::from(output);std::fs::create_dir_all(&p).unwrap();let mut raw=serde_json::to_vec(value).unwrap();raw.push(b'\n');std::fs::OpenOptions::new().write(true).create_new(true).open(p.join(name)).unwrap().write_all(&raw).unwrap();}}
