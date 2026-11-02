use super::*;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GoSourceRuntimeValue { typ:String, data:GoSourceRuntimeData }
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum GoSourceRuntimeData {
    Nil, Bool(bool), Word(u64), Bytes(Vec<u8>), Aggregate(Vec<GoSourceRuntimeValue>),
    Slice(GoSourceRuntimeSlice), Error(Option<Box<GoSourceRuntimeValue>>),
}
#[derive(Clone, Debug)]
pub struct GoSourceRuntimeSlice {
    storage:Option<Rc<RefCell<Vec<GoSourceRuntimeValue>>>>,
    offset:usize, length:usize, capacity:usize,
}
impl PartialEq for GoSourceRuntimeSlice {
    fn eq(&self,other:&Self)->bool {
        self.offset==other.offset && self.length==other.length && self.capacity==other.capacity && match (&self.storage,&other.storage) { (None,None)=>true,(Some(a),Some(b))=>Rc::ptr_eq(a,b),_=>false }
    }
}
impl Eq for GoSourceRuntimeSlice {}
impl GoSourceRuntimeSlice {
    pub fn nil()->Self { Self{storage:None,offset:0,length:0,capacity:0} }
    /// Includes the complete initialized capacity tail. Clones share backing
    /// storage; no caller owner ID or live-root assertion is accepted.
    pub fn from_backing(values:Vec<GoSourceRuntimeValue>,length:usize)->Option<Self> {
        (length<=values.len()).then(||Self{capacity:values.len(),storage:Some(Rc::new(RefCell::new(values))),offset:0,length})
    }
    pub fn is_nil(&self)->bool { self.storage.is_none() }
    pub fn length(&self)->usize { self.length }
    pub fn capacity(&self)->usize { self.capacity }
    pub fn elements(&self)->Vec<GoSourceRuntimeValue> {
        self.storage.as_ref().map_or_else(Vec::new,|s|s.borrow()[self.offset..self.offset+self.length].to_vec())
    }
    pub(super) fn read(&self,i:usize)->Option<GoSourceRuntimeValue> {
        if i>=self.length { return None; } self.storage.as_ref()?.borrow().get(self.offset+i).cloned()
    }
    pub(super) fn write(&self,i:usize,v:GoSourceRuntimeValue)->Run<()> {
        if i>=self.length { return Err(Halt::Fault(GoSourceRuntimeFault::IndexBounds)); }
        self.storage.as_ref().ok_or_else(source)?.borrow_mut()[self.offset+i]=v;Ok(())
    }
    pub(super) fn append(&self,extra:Vec<GoSourceRuntimeValue>,_zero:&GoSourceRuntimeValue)->Run<Self> {
        if extra.is_empty(){return Ok(self.clone());}
        let length=self.length.checked_add(extra.len()).ok_or(Halt::Error(GoSourceExecutionErrorKind::HostCapacity))?;
        if length<=self.capacity {
            let mut data=self.storage.as_ref().ok_or_else(source)?.borrow_mut();
            for (i,value) in extra.into_iter().enumerate(){data[self.offset+self.length+i]=value;}
            return Ok(Self{storage:self.storage.clone(),offset:self.offset,length,capacity:self.capacity});
        }
        // Go permits any new capacity sufficient for the result. This observer
        // chooses the exact required capacity, not a pinned-runtime growth law.
        let mut data=self.elements();data.try_reserve_exact(extra.len()).map_err(|_|Halt::Error(GoSourceExecutionErrorKind::HostCapacity))?;data.extend(extra);
        Self::from_backing(data,length).ok_or_else(source)
    }
    pub(super) fn subslice(&self,lo:usize,hi:usize,cap:usize)->Run<Self> {
        if lo>hi || hi>cap || cap>self.capacity { return Err(Halt::Fault(GoSourceRuntimeFault::SliceBounds)); }
        Ok(Self{storage:self.storage.clone(),offset:self.offset+lo,length:hi-lo,capacity:cap-lo})
    }
}
impl GoSourceRuntimeValue {
    pub fn new(type_id:impl Into<String>,data:GoSourceRuntimeData)->Self { Self{typ:type_id.into(),data} }
    pub fn type_id(&self)->&str { &self.typ }
    pub fn data(&self)->&GoSourceRuntimeData { &self.data }
}
impl Machine<'_> {
    pub(super) fn shape(&self,typ:&str)->Run<&GoSourceTypeKey> {
        let base=self.domain.underlying(typ).unwrap_or(typ);
        self.domain.types().find(|t|t.id()==base).ok_or_else(source)
    }
    pub(super) fn word_shape(&self,typ:&str)->Run<(u8,bool)> {
        match self.shape(typ)? { GoSourceTypeKey::Builtin{width:Some(w),signed:Some(s),..}=>Ok((*w,*s)),_=>Err(source()) }
    }
    pub(super) fn builtin(&self,name:&str)->Run<String> {
        self.domain.types().find_map(|t|match t { GoSourceTypeKey::Builtin{id,name:n,..} if n==name=>Some(id.clone()),_=>None }).ok_or_else(source)
    }
    pub(super) fn valid(&self,v:&GoSourceRuntimeValue)->bool {
        match (self.shape(v.type_id()),v.data()) {
            (Ok(GoSourceTypeKey::Builtin{name,..}),GoSourceRuntimeData::Bool(_))=>name=="bool",
            (Ok(GoSourceTypeKey::Builtin{width:Some(w),..}),GoSourceRuntimeData::Word(n))=>*w==64 || *n<(1u64<<w),
            (Ok(GoSourceTypeKey::Builtin{name,..}),GoSourceRuntimeData::Bytes(_))=>name=="string",
            (Ok(GoSourceTypeKey::Builtin{name,..}),GoSourceRuntimeData::Error(e))=>name=="error" && e.as_ref().is_none_or(|v|self.domain.error_cases().iter().any(|c|c.value_type_id()==v.type_id()) && self.valid(v)),
            (Ok(GoSourceTypeKey::Array{element,length,..}),GoSourceRuntimeData::Aggregate(xs))=>xs.len() as u64==*length && xs.iter().all(|v|v.type_id()==element && self.valid(v)),
            (Ok(GoSourceTypeKey::Struct{fields,..}),GoSourceRuntimeData::Aggregate(xs))=>xs.len()==fields.len() && xs.iter().zip(fields).all(|(v,f)|v.type_id()==f.type_id && self.valid(v)),
            (Ok(GoSourceTypeKey::Slice{element,..}),GoSourceRuntimeData::Slice(s))=>match &s.storage {
                None=>s.offset==0 && s.length==0 && s.capacity==0,
                Some(storage)=>{let xs=storage.borrow();s.length<=s.capacity && s.offset.checked_add(s.capacity).is_some_and(|end|end<=xs.len()) && xs[s.offset..s.offset+s.capacity].iter().all(|v|v.type_id()==element && self.valid(v))}
            },
            _=>false,
        }
    }
    pub(super) fn zero(&self,typ:&str)->Run<GoSourceRuntimeValue> {
        use GoSourceRuntimeData::*;
        let data=match self.shape(typ)? {
            GoSourceTypeKey::Builtin{name,width,..}=>match name.as_str() { "bool"=>Bool(false),"string"=>Bytes(vec![]),"error"=>Error(None),_ if width.is_some()=>Word(0),_=>return Err(source()) },
            GoSourceTypeKey::Array{element,length,..}=>{
                let n=usize::try_from(*length).map_err(|_|Halt::Error(GoSourceExecutionErrorKind::HostCapacity))?;
                let mut xs=Vec::new();xs.try_reserve_exact(n).map_err(|_|Halt::Error(GoSourceExecutionErrorKind::HostCapacity))?;
                for _ in 0..n { xs.push(self.zero(element)?); } Aggregate(xs)
            },
            GoSourceTypeKey::Struct{fields,..}=>Aggregate(fields.iter().map(|f|self.zero(&f.type_id)).collect::<Run<Vec<_>>>()?),
            GoSourceTypeKey::Slice{..}=>Slice(GoSourceRuntimeSlice::nil()),
            _=>return Err(source()),
        };Ok(GoSourceRuntimeValue::new(typ,data))
    }
    pub(super) fn constant(&self,typ:&str,c:&GoEvaluatedConstant)->Run<GoSourceRuntimeValue> {
        let data=if let Some(b)=c.boolean() { GoSourceRuntimeData::Bool(b) }
        else if let Some(b)=c.bytes() { GoSourceRuntimeData::Bytes(b.to_vec()) }
        else { let n=c.integer_decimal().ok_or_else(source)?.parse::<i128>().map_err(|_|source())?;let(w,_)=self.word_shape(typ)?;GoSourceRuntimeData::Word((n as u64)&mask(w)) };
        let v=GoSourceRuntimeValue::new(typ,data);if !self.valid(&v) { return Err(source()); }Ok(v)
    }
    pub(super) fn assigned(&self,mut v:GoSourceRuntimeValue,target:&str)->Run<GoSourceRuntimeValue> {
        if matches!(v.data(),GoSourceRuntimeData::Nil) {
            if !matches!(self.shape(target)?,GoSourceTypeKey::Slice{..}) && !matches!(self.shape(target)?,GoSourceTypeKey::Builtin{name,..} if name=="error") {return Err(source());}
            return self.zero(target);
        }
        if v.type_id()!=target && matches!(self.shape(target)?,GoSourceTypeKey::Builtin{name,..} if name=="error") {
            if !self.domain.error_cases().iter().any(|c|c.value_type_id()==v.type_id()) {return Err(source());}
            return Ok(GoSourceRuntimeValue::new(target,GoSourceRuntimeData::Error(Some(Box::new(v)))));
        }
        v.typ=target.into();if !self.valid(&v) {return Err(source());}Ok(v)
    }
    pub(super) fn integer(&self,v:&GoSourceRuntimeValue)->Run<i128> {
        let GoSourceRuntimeData::Word(n)=v.data else { return Err(source()); };let(w,s)=self.word_shape(v.type_id())?;
        Ok(if s && n&(1u64<<(w-1))!=0 {n as i128-(1i128<<w)} else {n as i128})
    }
    pub(super) fn index(&self,v:&GoSourceRuntimeValue,fault:GoSourceRuntimeFault)->Run<usize> {
        let n=self.integer(v)?;usize::try_from(n).map_err(|_|Halt::Fault(fault))
    }
}
pub(super) fn mask(w:u8)->u64 { if w==64 {u64::MAX} else {(1u64<<w)-1} }
