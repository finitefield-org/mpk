//! Actual ordinary value views compared to complete original Go observations.
use super::actual::Interpreter;
use mpk_cert::encode::Certificate;
use serde::{Deserialize, Serialize};
use serde_json::json;
type Retain = fn(&str, &str, &[u8]);
#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Case {
    id: String, kind: String, length: usize, index: usize, depth: usize,
    bytes: Vec<u8>, replacement: Vec<u8>, tag: u8, replacement_tag: u8, value1: u8, value2: u8,
}
#[derive(Debug, Deserialize, Serialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
struct Snapshot { bytes: Vec<u8>, tag: u8 }
#[derive(Debug, Deserialize, Serialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
struct Native { id: String, snapshots: Vec<Snapshot>, alias_before: u8, alias_after: u8 }
fn save(retain: Retain, name: &str, ext: &str, value: serde_json::Value) {
    retain(name,ext,format!("{}\n",serde_json::to_string_pretty(&value).unwrap()).as_bytes());
}
fn lc(name: &str) -> String { format!("Mpk.Go.Practical.Storage.Lens.{name}") }
fn pc(name: &str) -> String { format!("Mpk.Go.Practical.Product.{name}") }
fn word(e: &mut Interpreter<'_>, n: u64) -> u32 {
    let raw=e.raw(n,[0;256]); assert_eq!(e.length(raw),n);
    e.invoke("Mpk.Go.Practical.String.Length",&[raw])
}
fn byte_type(e: &mut Interpreter<'_>) -> u32 {
    let b=e.invoke("Std.Bool",&[]);
    e.invoke("Mpk.Go.Practical.Array.D3.Carrier",&[b])
}
fn observe_byte(e: &mut Interpreter<'_>, ty: u32, value: u32) -> u8 {
    let pair=e.invoke(&pc("Make"),&[ty,ty,value,value]);
    let mut result=0;
    for k in 0..8 {
        let mut args=vec![ty,ty,pair];args.extend((0..3).rev().map(|j|u32::from(k&(1<<j)!=0)));
        let v=e.invoke(&pc("Left"),&args); if e.boolean(v) {result|=1<<k;}
    }
    result
}
fn raw(e: &mut Interpreter<'_>, n: usize, bytes: &[u8]) -> u32 {
    assert_eq!(bytes.len(),256);
    let r=e.raw(n as u64,bytes.try_into().unwrap());
    assert_eq!(e.length(r),n as u64);assert_eq!(e.bytes(r).to_vec(),bytes);r
}
fn depth(n: usize) -> usize { n.next_power_of_two().trailing_zeros() as usize }
fn make_root(e: &mut Interpreter<'_>, row: &Case, bytes: &[u8], tag: u8, ty: u32) -> u32 {
    let r=raw(e,row.length,bytes);
    if row.kind=="chain" {
        let mut root=e.byte_input(bytes[0]);
        for &mark in &bytes[1..=row.depth] {
            let mark=e.byte_input(mark);root=e.invoke(&pc("Make"),&[ty,ty,root,mark]);
        }
        root
    } else {
        let d=if row.kind=="nested"{8}else{depth(row.length)};
        let mut args=vec![r];args.extend(vec![0;8-d]);
        let mut data=e.invoke("Mpk.Go.Practical.String.Data",&args);
        if row.length<1<<d {
            let n=e.invoke("Mpk.Go.Practical.String.Size",&[r]);let z=e.byte_input(0);
            data=e.invoke(&format!("Mpk.Go.Practical.Array.D{d}.Normalize"),&[ty,z,n,data]);
        }
        let tag=e.byte_input(tag);e.invoke(&pc("Make"),&[ty,ty,data,tag])
    }
}
fn compose(e: &mut Interpreter<'_>, ty: u32, l: u32, m: u32) -> u32 {
    e.invoke(&lc("Compose"),&[ty,ty,ty,l,m])
}
fn view(e: &mut Interpreter<'_>, row: &Case, ty: u32, alternate: bool) -> u32 {
    if row.kind=="chain" {
        let mut result=e.invoke(&lc("Identity"),&[ty]);
        for _ in 0..row.depth {
            let field=e.invoke(&lc("LeftField"),&[ty,ty]);result=compose(e,ty,field,result);
        }
        result
    } else {
        let field=e.invoke(&lc("LeftField"),&[ty,ty]);
        if row.kind=="array" {
            let i=word(e,row.index as u64);
            let element=e.invoke(&lc(&format!("ArrayD{}Element",depth(row.length))),&[ty,i]);
            compose(e,ty,field,element)
        } else {
            let d=row.depth;let inner=1<<(8-d);
            let i=word(e,(row.index/inner)as u64);let j=word(e,(row.index%inner)as u64);
            let outer=e.invoke(&lc(&format!("ArrayD{d}Element")),&[ty,i]);
            let inner=e.invoke(&lc(&format!("ArrayD{}Element",8-d)),&[ty,j]);
            if alternate {let first=compose(e,ty,field,outer);compose(e,ty,first,inner)}
            else {let last=compose(e,ty,outer,inner);compose(e,ty,field,last)}
        }
    }
}
fn snapshot(e: &mut Interpreter<'_>, row: &Case, ty: u32, mut root: u32) -> Snapshot {
    let mut bytes=vec![0;256];
    if row.kind=="chain" {
        for k in (1..=row.depth).rev() {
            let mark=e.invoke(&pc("Right"),&[ty,ty,root]);bytes[k]=observe_byte(e,ty,mark);
            root=e.invoke(&pc("Left"),&[ty,ty,root]);
        }
        bytes[0]=observe_byte(e,ty,root);Snapshot{bytes,tag:0}
    } else {
        let d=if row.kind=="nested"{8}else{depth(row.length)};
        for (i,byte) in bytes[..1<<d].iter_mut().enumerate() {
            for k in 0..8 {
                let mut args=vec![ty,ty,root];
                args.extend((0..d).rev().map(|j|u32::from(i&(1<<j)!=0)));
                args.extend((0..3).rev().map(|j|u32::from(k&(1<<j)!=0)));
                let v=e.invoke(&pc("Left"),&args);if e.boolean(v){*byte|=1<<k;}
            }
        }
        assert!(bytes[row.length..].iter().all(|b|*b==0));
        let tag=e.invoke(&pc("Right"),&[ty,ty,root]);let tag=observe_byte(e,ty,tag);Snapshot{bytes,tag}
    }
}
pub(super) fn run(c: &Certificate, retain: Retain) {
    let path=std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../develop/migrations/go-practical/t02-w01/verification/storage-lens/native-go-current");
    let cases:Vec<Case>=serde_json::from_slice(&std::fs::read(path.join("cases.json")).unwrap()).unwrap();
    let native:Vec<Native>=serde_json::from_slice(&std::fs::read(path.join("views.stdout.log")).unwrap()).unwrap();
    assert_eq!(cases.len(),203);assert_eq!(cases.len(),native.len());
    let(mut total,mut outputs)=(0usize,0usize);
    for (row,go) in cases.iter().zip(native.iter()) {
        save(retain,&row.id,"input.json",json!({"whole_original_case":row,"all64_index_bits":(0..64).map(|j|row.index as u64&(1<<j)!=0).collect::<Vec<_>>(),"source_type_path_live_store_or_permission_proof":false}));
        let mut e=Interpreter::new(c);let ty=byte_type(&mut e);
        let original=make_root(&mut e,row,&row.bytes,row.tag,ty);
        let replacement=make_root(&mut e,row,&row.replacement,row.replacement_tag,ty);
        let lens=view(&mut e,row,ty,false);let x=e.byte_input(row.value1);let y=e.byte_input(row.value2);
        let changed=e.invoke(&lc("Put"),&[ty,ty,lens,original,x]);
        let before=e.invoke(&lc("Get"),&[ty,ty,lens,changed]);let before=observe_byte(&mut e,ty,before);
        let after=e.invoke(&lc("Get"),&[ty,ty,lens,replacement]);let after=observe_byte(&mut e,ty,after);
        let last=e.invoke(&lc("Put"),&[ty,ty,lens,replacement,y]);
        let snapshots=[original,changed,replacement,last,original].into_iter().map(|r|snapshot(&mut e,row,ty,r)).collect::<Vec<_>>();
        let result=Native{id:row.id.clone(),snapshots,alias_before:before,alias_after:after};
        save(retain,&row.id,"observation.json",json!({"actual_ordinary":result,"original_native_Go":go,"actual_AST_steps":e.steps(),"definition_callbacks":false,"theorem_bodies":false}));
        assert_eq!(&result,go,"{}",row.id);outputs+=5;
        if row.kind=="nested" {
            let alternate=view(&mut e,row,ty,true);let actual=e.invoke(&lc("Put"),&[ty,ty,alternate,replacement,y]);
            let observed=snapshot(&mut e,row,ty,actual);assert_eq!(observed,go.snapshots[3]);
            save(retain,&format!("{}-associative",row.id),"observation.json",json!({"actual_ordinary":observed,"original_native_Go":go.snapshots[3],"actual_AST_steps":e.steps()}));outputs+=1;
        }
        total+=e.steps();
    }
    save(retain,"observation-inventory","json",json!({"whole_original_cases":cases.len(),"full_root_outputs":outputs,"shared_alias_reads":cases.len()*2,"actual_AST_steps":total,"definition_callbacks":false,"theorem_bodies":false,"observer_ceilings_changed":false}));
    println!("lens native cases={} full root outputs={outputs} actual AST steps={total}",cases.len());
}
pub(super) fn countermodel(c: &Certificate, name: &str, retain: Retain) {
    save(retain,&format!("definition-{name}"),"input.json",json!({"definition":name,"whole_bytes":vec![0u8;256],"full64_index":0,"whole64_index_bits":vec![false;64],"initial_bool":false,"written_bool":true,"initial_byte":0,"written_byte":255,"raw_model_only":true,"source_or_live_store_authority":false}));
    let mut e=Interpreter::new(c);let ty=e.invoke("Std.Bool",&[]);
    let (actual,expected)=if name=="Get" {
        let getter=e.invoke("Mpk.Go.Practical.Bool.Not",&[]);
        let identity=e.invoke(&lc("Identity"),&[ty]);let put=e.invoke(&pc("Right"),&[ty,ty,identity]);
        let l=e.invoke(&lc("Make"),&[ty,ty,getter,put]);
        let value=e.invoke(&lc("Get"),&[ty,ty,l,0]);(u64::from(e.boolean(value)),1)
    } else if name.starts_with("ArrayD") {
        let d=name.trim_start_matches("ArrayD").trim_end_matches("Element").parse::<usize>().unwrap();
        let ty=byte_type(&mut e);let r=e.raw(0,[0;256]);let mut args=vec![r];args.extend(vec![0;8-d]);
        let data=e.invoke("Mpk.Go.Practical.String.Data",&args);let i=word(&mut e,0);
        let l=e.invoke(&lc(name),&[ty,i]);let x=e.byte_input(255);
        let result=e.invoke(&lc("Put"),&[ty,ty,l,data,x]);
        let result=e.invoke(&format!("Mpk.Go.Practical.Array.D{d}.Read"),&[ty,result,i]);(u64::from(observe_byte(&mut e,ty,result)),255)
    } else if name.ends_with("Field") {
        let l=e.invoke(&lc(name),&[ty,ty]);let root=e.invoke(&pc("Make"),&[ty,ty,0,0]);
        let result=e.invoke(&lc("Put"),&[ty,ty,l,root,1]);
        let result=e.invoke(&pc(if name=="LeftField"{"Left"}else{"Right"}),&[ty,ty,result]);(u64::from(e.boolean(result)),1)
    } else {
        let identity=e.invoke(&lc("Identity"),&[ty]);
        let l=if name=="Compose" {compose(&mut e,ty,identity,identity)}else{identity};
        let result=e.invoke(&lc("Put"),&[ty,ty,l,0,1]);(u64::from(e.boolean(result)),1)
    };
    save(retain,&format!("definition-{name}"),"countermodel.json",json!({"actual":actual,"expected":expected,"different":actual!=expected,"actual_AST_steps":e.steps(),"definition_callbacks":false,"theorem_bodies":false}));
    assert_ne!(actual,expected,"{name}");
}
