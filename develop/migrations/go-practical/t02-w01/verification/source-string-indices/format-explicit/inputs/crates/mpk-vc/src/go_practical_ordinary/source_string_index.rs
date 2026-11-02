//! Full source integer widths and signs for bounded byte-string indexing.
//! These closed representation/definedness laws are not source-expression
//! evaluation, receiver-once execution, or canonical capture/context authority.
use super::syntax::*;
use super::{composition, error_layout, word, Builder, GoOrdinaryError, Result, MAX_BYTES};
use super::{
    generate_go_practical_source_string_catalogue, generate_go_practical_word_conversions,
    GoSourceStringCatalogue, GoSourceStringRow,
};
use crate::go_practical_type_keys::{GoSourceTypeKey, GoSourceValueDomain};

#[derive(Clone)]
pub struct GoSourceStringIndexCatalogue {
    strings: GoSourceStringCatalogue,
    integers: Vec<(String, u8, bool)>,
    byte: Option<String>,
    conversions: mpk_cert::encode::Certificate,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GoSourceStringIndexRow {
    prior: GoSourceStringRow,
    index: String,
    width: u8,
    signed: bool,
    byte: String,
    module: String,
    certificate: Vec<u8>,
}
impl GoSourceStringIndexRow {
    pub fn prior_string_row(&self) -> &GoSourceStringRow {
        &self.prior
    }
    pub fn source_index_type_id(&self) -> &str {
        &self.index
    }
    pub fn index_width(&self) -> u8 {
        self.width
    }
    pub fn is_signed_index(&self) -> bool {
        self.signed
    }
    pub fn source_byte_type_id(&self) -> &str {
        &self.byte
    }
    pub fn module_name(&self) -> &str {
        &self.module
    }
    pub fn certificate_bytes(&self) -> &[u8] {
        &self.certificate
    }
}
impl GoSourceStringIndexCatalogue {
    pub fn strings(&self) -> &GoSourceStringCatalogue {
        &self.strings
    }
    pub fn index_types(&self) -> &[(String, u8, bool)] {
        &self.integers
    }
    pub fn generate_row(&self, string: &str, index: &str) -> Result<GoSourceStringIndexRow> {
        let (_, width, signed) = self
            .integers
            .iter()
            .find(|r| r.0 == index)
            .ok_or(GoOrdinaryError::Linkage)?;
        let byte = self.byte.as_deref().ok_or(GoOrdinaryError::Linkage)?;
        let prior = self.strings.generate_row(string)?;
        let source = mpk_cert::decode_canonical_certificate(prior.certificate_bytes())
            .map_err(|_| GoOrdinaryError::Foundation)?;
        let mut builder =
            composition::source_string_index_prerequisites(&[source, self.conversions.clone()])?;
        let module = format!("{}.Index.T{index}", prior.module_name());
        Index {
            builder: &mut builder,
            prior: &prior,
            index,
            width: usize::from(*width),
            signed: *signed,
            byte,
            module: &module,
        }
        .append()?;
        builder.certificate.module = module.clone();
        Ok(GoSourceStringIndexRow {
            prior,
            index: index.into(),
            width: *width,
            signed: *signed,
            byte: byte.into(),
            module,
            certificate: builder.finish()?,
        })
    }
    pub fn import_row(
        &self,
        bytes: &[u8],
        string: &str,
        index: &str,
    ) -> Result<GoSourceStringIndexRow> {
        if bytes.len() > MAX_BYTES {
            return Err(GoOrdinaryError::Limit);
        }
        let expected = self.generate_row(string, index)?;
        if expected.certificate_bytes() != bytes {
            return Err(GoOrdinaryError::Linkage);
        }
        Ok(expected)
    }
}
/// All original source integer/nominal types and all original string types are
/// eligible without a smaller case/type/root/helper/parameter bound. No source
/// uint8 type is fabricated when its compiler-derived domain does not have one.
pub fn generate_go_practical_source_string_index_catalogue(
    domain: &GoSourceValueDomain,
) -> Result<GoSourceStringIndexCatalogue> {
    let strings = generate_go_practical_source_string_catalogue(domain)?;
    let mut integers = vec![];
    let mut byte = None;
    for key in strings.source_types() {
        if matches!(key,GoSourceTypeKey::Builtin{name,..} if name=="uint8") {
            byte = Some(key.id().into())
        }
        let mut base = key.id();
        while let Some(next) = strings.underlying_type_id(base) {
            base = next;
        }
        let shape = strings
            .source_types()
            .iter()
            .find(|k| k.id() == base)
            .ok_or(GoOrdinaryError::Linkage)?;
        if let GoSourceTypeKey::Builtin {
            width: Some(width),
            signed: Some(signed),
            ..
        } = shape
        {
            if ![8, 16, 32, 64].contains(width) {
                return Err(GoOrdinaryError::Linkage);
            }
            integers.push((key.id().into(), *width, *signed));
        }
    }
    let library = generate_go_practical_word_conversions()?;
    let conversions = mpk_cert::decode_canonical_certificate(library.certificate_bytes())
        .map_err(|_| GoOrdinaryError::Foundation)?;
    Ok(GoSourceStringIndexCatalogue {
        strings,
        integers,
        byte,
        conversions,
    })
}
fn numeric(n: &str, args: Vec<Expr>) -> Expr {
    call(&format!("Mpk.Go.Practical.Word.Numeric.{n}"), args)
}
fn integer(n: &str, args: Vec<Expr>) -> Expr {
    call(&format!("Mpk.Go.Practical.Int.{n}"), args)
}
fn wt(w: usize) -> Expr {
    word::word_type(w).expect("source integer width")
}
struct Index<'a> {
    builder: &'a mut Builder,
    prior: &'a GoSourceStringRow,
    index: &'a str,
    width: usize,
    signed: bool,
    byte: &'a str,
    module: &'a str,
}
impl Index<'_> {
    fn source(&self, id: &str) -> Expr {
        constant(&format!(
            "{}.T{id}.Carrier",
            self.prior.source_layout_module_name()
        ))
    }
    fn string_type(&self) -> Expr {
        self.source(self.prior.source_string_type_id())
    }
    fn index_type(&self) -> Expr {
        self.source(self.index)
    }
    fn own(&self, n: &str, args: Vec<Expr>) -> Expr {
        call(&format!("{}.{n}", self.module), args)
    }
    fn string(&self, n: &str, args: Vec<Expr>) -> Expr {
        call(&format!("{}.{n}", self.prior.module_name()), args)
    }
    fn word(&self, i: Expr) -> Expr {
        self.own("ToWord", vec![i])
    }
    fn natural(&self, i: Expr) -> Expr {
        self.own("Natural", vec![i])
    }
    fn int_value(&self, i: Expr) -> Expr {
        self.own("Integer", vec![i])
    }
    fn nonnegative(&self, i: Expr) -> Expr {
        self.own("Nonnegative", vec![i])
    }
    fn clauses(&self, s: Expr, i: Expr) -> Vec<Expr> {
        vec![
            self.string("Valid", vec![s.clone()]),
            self.nonnegative(i.clone()),
            eq(
                boolean(),
                less(self.natural(i), self.string("Size", vec![s])),
                bit(true),
            ),
        ]
    }
    fn emit(&mut self, n: &str, args: &[Binder], ty: Expr, body: Expr, law: bool) -> Result<()> {
        emit(
            self.builder,
            &format!("{}.{n}", self.module),
            args,
            ty,
            body,
            law,
        )
    }
    fn append(&mut self) -> Result<()> {
        let (s, i) = (var("s"), var("i"));
        let ia = [("i", self.index_type())];
        self.emit("ToWord", &ia, wt(self.width), i.clone(), false)?;
        self.emit(
            "To64",
            &ia,
            wt(64),
            word::wc(self.width, "UnsignedTo64", vec![self.word(i.clone())]),
            false,
        )?;
        self.emit(
            "Natural",
            &ia,
            nat(),
            numeric(
                &format!("Value{}", word::depth(self.width)?),
                vec![self.word(i.clone())],
            ),
            false,
        )?;
        self.emit(
            "Negative",
            &ia,
            boolean(),
            if self.signed {
                word::at(self.word(i.clone()), self.width, self.width - 1)?
            } else {
                bit(false)
            },
            false,
        )?;
        self.emit(
            "Nonnegative",
            &ia,
            Expr::Sort0,
            eq(boolean(), self.own("Negative", vec![i.clone()]), bit(false)),
            false,
        )?;
        let integer_type=pi("_part",boolean(),nat());
        let actual_integer=if self.signed {numeric(&format!("W{}.Signed",self.width),vec![self.word(i.clone())])} else {integer("FromNat",vec![self.natural(i.clone())])};
        self.emit("Integer",&ia,integer_type.clone(),actual_integer.clone(),false)?;
        self.emit("IntegerDefinition",&ia,eq(integer_type.clone(),self.int_value(i.clone()),actual_integer.clone()),refl(integer_type.clone(),actual_integer.clone()),true)?;
        let converted=word::wc(self.width,"UnsignedTo64",vec![self.word(i.clone())]);
        self.emit("To64Definition",&ia,eq(wt(64),self.own("To64",vec![i.clone()]),converted.clone()),refl(wt(64),converted.clone()),true)?;
        let full_value=numeric("Value6",vec![converted.clone()]);
        let e=Equality{ty:wt(64),left:self.own("To64",vec![i.clone()]),right:converted,proof:self.own("To64Definition",vec![i.clone()])}
            .mapped(lam("_word",wt(64),numeric("Value6",vec![var("_word")])),numeric("Value6",vec![self.own("To64",vec![i.clone()])]),full_value.clone(),nat())
            .then(Equality::natural(full_value,self.natural(i.clone()),call(&format!("Mpk.Go.Practical.Word.Conversions.W{}.UnsignedTo64.UnsignedExact",self.width),vec![self.word(i.clone())])))?;
        self.emit("Natural64",&ia,eq(e.ty,e.left,e.right),e.proof,true)?;
        for negative in [false,true]{
            let member=if negative {"Negative"}else{"Positive"};
            let wanted=if negative {
                if self.signed {mul(numeric(&format!("Weight{}",word::depth(self.width)?),vec![]),numeric("Bit",vec![word::at(self.word(i.clone()),self.width,self.width-1)?]))}else{zero()}
            }else{self.natural(i.clone())};
            let law=if self.signed {numeric(&format!("W{}.Signed{member}",self.width),vec![self.word(i.clone())])}else{integer(&format!("{member}OfFromNat"),vec![self.natural(i.clone())])};
            let e=Equality{ty:integer_type.clone(),left:self.int_value(i.clone()),right:actual_integer.clone(),proof:self.own("IntegerDefinition",vec![i.clone()])}
                .mapped(lam("_integer",integer_type.clone(),integer(member,vec![var("_integer")])),integer(member,vec![self.int_value(i.clone())]),integer(member,vec![actual_integer.clone()]),nat())
                .then(Equality::natural(integer(member,vec![actual_integer.clone()]),wanted,law))?;
            self.emit(&format!("{member}Value"),&ia,eq(e.ty,e.left,e.right),e.proof,true)?;
        }
        let nonnegative = [
            ("i", self.index_type()),
            ("nonnegative", self.nonnegative(i.clone())),
        ];
        let negative_zero = if self.signed {
            let sign = word::at(self.word(i.clone()), self.width, self.width - 1)?;
            let weight = numeric(&format!("Weight{}", word::depth(self.width)?), vec![]);
            let bit_eq = Equality::boolean(sign.clone(), bit(false), var("nonnegative"))
                .mapped(
                    lam("_sign", boolean(), numeric("Bit", vec![var("_sign")])),
                    numeric("Bit", vec![sign]),
                    numeric("Bit", vec![bit(false)]),
                    nat(),
                )
                .then(Equality::natural(
                    numeric("Bit", vec![bit(false)]),
                    zero(),
                    refl(nat(), zero()),
                ))?;
            Equality::natural(
                integer("Negative", vec![self.int_value(i.clone())]),
                mul(weight.clone(), bit_eq.left.clone()),
                self.own("NegativeValue",vec![i.clone()]),
            )
            .then(super::polynomial::mul_right(weight.clone(), bit_eq))?
            .then(Equality::natural(
                mul(weight.clone(), zero()),
                zero(),
                nc("MulZero", vec![weight]),
            ))?
            .proof
        } else {
            self.own("NegativeValue",vec![i.clone()])
        };
        self.emit(
            "NonnegativeNegativeZero",
            &nonnegative,
            eq(
                nat(),
                integer("Negative", vec![self.int_value(i.clone())]),
                zero(),
            ),
            negative_zero,
            true,
        )?;
        let n = self.natural(i.clone());
        let from = integer("FromNat", vec![n.clone()]);
        let left = Equality::natural(
            integer("Positive", vec![self.int_value(i.clone())]),
            n.clone(),
            self.own("PositiveValue", vec![i.clone()]),
        )
        .on_left(integer("Negative", vec![from.clone()]))
        .then(
            Equality::natural(
                integer("Negative", vec![from.clone()]),
                zero(),
                integer("NegativeOfFromNat", vec![n.clone()]),
            )
            .on_right(n.clone()),
        )?
        .then(Equality::natural(
            add(n.clone(), zero()),
            n.clone(),
            nc("AddZero", vec![n.clone()]),
        ))?;
        let right = Equality::natural(
            integer("Positive", vec![from]),
            n.clone(),
            integer("PositiveOfFromNat", vec![n.clone()]),
        )
        .on_left(integer("Negative", vec![self.int_value(i.clone())]))
        .then(
            Equality::natural(
                integer("Negative", vec![self.int_value(i.clone())]),
                zero(),
                self.own(
                    "NonnegativeNegativeZero",
                    vec![i.clone(), var("nonnegative")],
                ),
            )
            .on_right(n.clone()),
        )?
        .then(Equality::natural(
            add(n.clone(), zero()),
            n.clone(),
            nc("AddZero", vec![n]),
        ))?;
        self.emit(
            "NonnegativeInteger",
            &nonnegative,
            integer(
                "Equivalent",
                vec![
                    self.int_value(i.clone()),
                    integer("FromNat", vec![self.natural(i.clone())]),
                ],
            ),
            left.then(right.reverse())?.proof,
            true,
        )?;
        let si = [("s", self.string_type()), ("i", self.index_type())];
        self.emit(
            "Defined",
            &si,
            Expr::Sort0,
            error_layout::product(&self.clauses(s.clone(), i.clone())),
            false,
        )?;
        let mut guarded = si.to_vec();
        guarded.push(("defined", self.own("Defined", vec![s.clone(), i.clone()])));
        for (position, name) in ["DefinedValid", "DefinedNonnegative", "DefinedWithin"]
            .iter()
            .enumerate()
        {
            let clauses = self.clauses(s.clone(), i.clone());
            self.emit(
                name,
                &guarded,
                clauses[position].clone(),
                error_layout::get(&clauses, var("defined"), position),
                true,
            )?;
        }
        let mut witnesses = si.to_vec();
        witnesses.extend([
            ("valid", self.clauses(s.clone(), i.clone())[0].clone()),
            ("nonnegative", self.nonnegative(i.clone())),
            ("within", self.clauses(s.clone(), i.clone())[2].clone()),
        ]);
        self.emit(
            "DefinedOfWitnesses",
            &witnesses,
            self.own("Defined", vec![s.clone(), i.clone()]),
            error_layout::make(
                &self.clauses(s.clone(), i.clone()),
                &[var("valid"), var("nonnegative"), var("within")],
            ),
            true,
        )?;
        let ni = self.own("To64", vec![i.clone()]);
        let within = Equality::natural(
            numeric("Value6", vec![ni.clone()]),
            self.natural(i.clone()),
            self.own("Natural64", vec![i.clone()]),
        )
        .mapped(
            lam(
                "_index",
                nat(),
                less(var("_index"), self.string("Size", vec![s.clone()])),
            ),
            less(
                numeric("Value6", vec![ni.clone()]),
                self.string("Size", vec![s.clone()]),
            ),
            less(
                self.natural(i.clone()),
                self.string("Size", vec![s.clone()]),
            ),
            boolean(),
        )
        .then(Equality::boolean(
            less(
                self.natural(i.clone()),
                self.string("Size", vec![s.clone()]),
            ),
            bit(true),
            self.own("DefinedWithin", vec![s.clone(), i.clone(), var("defined")]),
        ))?;
        self.emit(
            "Within64",
            &guarded,
            eq(within.ty, within.left, within.right),
            within.proof,
            true,
        )?;
        self.emit(
            "Read",
            &si,
            self.source(self.byte),
            self.string("Byte", vec![s.clone(), ni.clone()]),
            false,
        )?;
        let address = (0..8)
            .rev()
            .map(|b| word::at(ni.clone(), 64, b))
            .collect::<Result<Vec<_>>>()?;
        self.emit(
            "ReadWithinSlot",
            &guarded,
            eq(
                wt(8),
                self.own("Read", vec![s.clone(), i.clone()]),
                app(self.string("Data", vec![s.clone()]), address),
            ),
            self.string(
                "ValidByteWithinSlot",
                vec![
                    s.clone(),
                    ni,
                    self.own("DefinedValid", vec![s.clone(), i.clone(), var("defined")]),
                    self.own("Within64", vec![s.clone(), i.clone(), var("defined")]),
                ],
            ),
            true,
        )?;
        self.emit(
            "DefinedInteger",
            &guarded,
            integer(
                "Equivalent",
                vec![
                    self.int_value(i.clone()),
                    integer("FromNat", vec![self.natural(i.clone())]),
                ],
            ),
            self.own(
                "NonnegativeInteger",
                vec![
                    i.clone(),
                    self.own(
                        "DefinedNonnegative",
                        vec![s.clone(), i.clone(), var("defined")],
                    ),
                ],
            ),
            true,
        )?;
        if self.signed {
            let mut negative = guarded;
            negative.push((
                "negative",
                eq(boolean(), self.own("Negative", vec![i.clone()]), bit(true)),
            ));
            self.emit(
                "RejectNegative",
                &negative,
                eq(boolean(), bit(false), bit(true)),
                Equality::boolean(
                    self.own("Negative", vec![i.clone()]),
                    bit(false),
                    self.own("DefinedNonnegative", vec![s, i, var("defined")]),
                )
                .reverse()
                .then(Equality::boolean(
                    self.own("Negative", vec![var("i")]),
                    bit(true),
                    var("negative"),
                ))?
                .proof,
                true,
            )?;
        }
        Ok(())
    }
}
