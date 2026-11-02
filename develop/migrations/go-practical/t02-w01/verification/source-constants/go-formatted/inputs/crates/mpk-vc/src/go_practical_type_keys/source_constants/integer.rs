//! Exact sign/magnitude arithmetic. The source evaluator separately applies the
//! pinned compiler's precision and typed-representability checks after each op.
use std::cmp::Ordering;
#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct Integer {
    negative: bool,
    limbs: Vec<u32>,
}
impl Ord for Integer {
    fn cmp(&self, b: &Self) -> Ordering {
        match (self.negative,b.negative) {
            (true,false)=>Ordering::Less,
            (false,true)=>Ordering::Greater,
            (false,false)=>self.abs_cmp(b),
            (true,true)=>self.abs_cmp(b).reverse(),
        }
    }
}
impl PartialOrd for Integer {fn partial_cmp(&self,b:&Self)->Option<Ordering>{Some(self.cmp(b))}}
impl Integer {
    fn normalized(negative:bool,mut limbs:Vec<u32>)->Self {
        while limbs.last()==Some(&0){limbs.pop();}
        Self{negative:negative&&!limbs.is_empty(),limbs}
    }
    pub(super) fn from_u64(n:u64)->Self{Self::normalized(false,vec![n as u32,(n>>32)as u32])}
    pub(super) fn is_zero(&self)->bool{self.limbs.is_empty()}
    pub(super) fn is_negative(&self)->bool{self.negative}
    pub(super) fn bits(&self)->usize{self.limbs.last().map_or(0,|n|(self.limbs.len()-1)*32+32-n.leading_zeros()as usize)}
    pub(super) fn as_u64(&self)->Option<u64>{
        if self.negative||self.limbs.len()>2{return None}
        Some(self.limbs.first().copied().unwrap_or(0)as u64|((self.limbs.get(1).copied().unwrap_or(0)as u64)<<32))
    }
    pub(super) fn neg(&self)->Self{Self::normalized(!self.negative,self.limbs.clone())}
    fn abs_cmp(&self,b:&Self)->Ordering{self.limbs.len().cmp(&b.limbs.len()).then_with(||self.limbs.iter().rev().cmp(b.limbs.iter().rev()))}
    fn abs_add(&self,b:&Self)->Vec<u32>{
        let mut out=Vec::with_capacity(self.limbs.len().max(b.limbs.len())+1);let mut carry=0u64;
        for i in 0..self.limbs.len().max(b.limbs.len()){
            let n=self.limbs.get(i).copied().unwrap_or(0)as u64+b.limbs.get(i).copied().unwrap_or(0)as u64+carry;
            out.push(n as u32);carry=n>>32;
        }
        if carry!=0{out.push(carry as u32)}out
    }
    // self >= b by magnitude; unsigned borrowing is explicit.
    fn abs_sub(&self,b:&Self)->Vec<u32>{
        let mut out=Vec::with_capacity(self.limbs.len());let mut borrow=0u64;
        for (i,&a)in self.limbs.iter().enumerate(){
            let sub=b.limbs.get(i).copied().unwrap_or(0)as u64+borrow;
            out.push((a as u64).wrapping_sub(sub)as u32);borrow=u64::from((a as u64)<sub);
        }
        debug_assert_eq!(borrow,0);out
    }
    pub(super) fn add(&self,b:&Self)->Self{
        if self.negative==b.negative{return Self::normalized(self.negative,self.abs_add(b))}
        match self.abs_cmp(b){Ordering::Less=>Self::normalized(b.negative,b.abs_sub(self)),_=>Self::normalized(self.negative,self.abs_sub(b))}
    }
    pub(super) fn sub(&self,b:&Self)->Self{self.add(&b.neg())}
    pub(super) fn mul(&self,b:&Self)->Self{
        let mut out=vec![0u32;self.limbs.len()+b.limbs.len()];
        for(i,&a)in self.limbs.iter().enumerate(){
            let mut carry=0u64;
            for(j,&v)in b.limbs.iter().enumerate(){
                let n=a as u64*v as u64+out[i+j]as u64+carry;out[i+j]=n as u32;carry=n>>32;
            }
            if !b.limbs.is_empty(){out[i+b.limbs.len()]=carry as u32;}
        }
        Self::normalized(self.negative!=b.negative,out)
    }
    pub(super) fn shl(&self,count:usize)->Self{
        if self.is_zero(){return self.clone()}
        let mut out=vec![0;count/32];let mut carry=0u64;
        for &v in &self.limbs{let n=((v as u64)<<(count%32))|carry;out.push(n as u32);carry=n>>32}
        if carry!=0{out.push(carry as u32)}Self::normalized(self.negative,out)
    }
    pub(super) fn shr(&self,count:usize)->Self{
        let words=count/32;let shift=count%32;let mut out=vec![];
        for i in words..self.limbs.len(){
            let lo=self.limbs[i]>>shift;
            let hi=if shift==0{0}else{self.limbs.get(i+1).copied().unwrap_or(0)<<(32-shift)};
            out.push(lo|hi);
        }
        let mut q=Self::normalized(false,out);
        let dropped=self.limbs.iter().take(words).any(|&v|v!=0)
            || shift!=0&&self.limbs.get(words).is_some_and(|v|v&((1u32<<shift)-1)!=0);
        if self.negative&&dropped{q=q.add(&Self::from_u64(1))}
        Self::normalized(self.negative,q.limbs)
    }
    pub(super) fn div_rem(&self,b:&Self)->Option<(Self,Self)>{
        if b.is_zero(){return None}
        let divisor=Self::normalized(false,b.limbs.clone());let mut rem=Self::from_u64(0);let mut q=vec![0u32;self.bits().div_ceil(32)];
        for bit in (0..self.bits()).rev(){
            rem=rem.shl(1);
            if self.limbs[bit/32]&(1u32<<(bit%32))!=0{rem=rem.add(&Self::from_u64(1))}
            if rem.abs_cmp(&divisor)!=Ordering::Less{rem=Self::normalized(false,rem.abs_sub(&divisor));q[bit/32]|=1u32<<(bit%32)}
        }
        Some((Self::normalized(self.negative!=b.negative,q),Self::normalized(self.negative,rem.limbs)))
    }
    fn twos(&self,n:usize)->Vec<u32>{
        let mut out=self.limbs.clone();out.resize(n,0);
        if self.negative{let mut carry=1u64;for v in &mut out{let z=(!*v)as u64+carry;*v=z as u32;carry=z>>32}}
        out
    }
    pub(super) fn bit_op(&self,b:&Self,op:&str)->Self{
        let n=self.limbs.len().max(b.limbs.len())+1;let a=self.twos(n);let b=b.twos(n);
        let mut out:Vec<_>=a.into_iter().zip(b).map(|(a,b)|match op{"&"=>a&b,"|"=>a|b,"^"=>a^b,"&^"=>a&!b,_=>unreachable!()}).collect();
        let negative=out.last().unwrap()&(1<<31)!=0;
        if negative{let mut carry=1u64;for v in &mut out{let z=(!*v)as u64+carry;*v=z as u32;carry=z>>32}}
        Self::normalized(negative,out)
    }
    pub(super) fn complement(&self,unsigned_width:Option<u8>)->Self{
        match unsigned_width{Some(w)=>Self::from_u64(1).shl(w as usize).sub(&Self::from_u64(1)).bit_op(self,"^"),None=>self.neg().sub(&Self::from_u64(1))}
    }
    pub(super) fn literal(raw:&[u8])->Option<Self>{
        if raw.is_empty()||raw.len()>10000{return None}
        let(base,start)=if raw.len()>1&&raw[0]==b'0'{match raw[1]{b'b'|b'B'=>(2,2),b'o'|b'O'=>(8,2),b'x'|b'X'=>(16,2),_=>(8,1)}}else{(10,0)};
        let mut n=Self::from_u64(0);
        for &c in &raw[start..]{if c==b'_'{continue}let digit=(c as char).to_digit(base)?;
            n=n.mul(&Self::from_u64(base as u64)).add(&Self::from_u64(digit as u64));if n.bits()>512{return None}
        }
        Some(n)
    }
    pub(super) fn decimal(&self)->String{
        if self.is_zero(){return "0".into()}
        let mut parts=vec![];let mut limbs=self.limbs.clone();
        while !limbs.is_empty(){let mut rem=0u64;for v in limbs.iter_mut().rev(){let n=(rem<<32)|*v as u64;*v=(n/1_000_000_000)as u32;rem=n%1_000_000_000}parts.push(rem as u32);while limbs.last()==Some(&0){limbs.pop();}}
        let mut out=if self.negative{"-".to_string()}else{String::new()};out.push_str(&parts.pop().unwrap().to_string());
        for p in parts.into_iter().rev(){out.push_str(&format!("{p:09}"))}out
    }
}
