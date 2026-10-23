//! Source-specific adapters for the frozen bounded byte-string representation.
//! Length/byte observations here use encoded unsigned words, not Go expression
//! evaluation or Go's signed int result/index typing. Context authority is due.
use super::syntax::*;
use super::{composition, error_layout, source_error_layout, string, word, word_index_builder};
use super::{Builder, GoOrdinaryError, Result, MAX_BYTES};
use crate::go_practical_type_keys::{GoSourceTypeKey, GoSourceValueDomain};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone)]
pub struct GoSourceStringCatalogue {
    builder: Builder,
    source: String,
    types: Vec<GoSourceTypeKey>,
    underlying: BTreeMap<String, String>,
    strings: Vec<String>,
    cases: Vec<String>,
    prior_layout: Vec<u8>,
    generic_string: Vec<u8>,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GoSourceStringRow {
    source: String,
    source_type: String,
    module: String,
    certificate: Vec<u8>,
}
impl GoSourceStringRow {
    pub fn source_layout_module_name(&self) -> &str {
        &self.source
    }
    pub fn source_string_type_id(&self) -> &str {
        &self.source_type
    }
    pub fn module_name(&self) -> &str {
        &self.module
    }
    pub fn certificate_bytes(&self) -> &[u8] {
        &self.certificate
    }
}
impl GoSourceStringCatalogue {
    pub fn source_layout_module_name(&self) -> &str {
        &self.source
    }
    pub fn source_types(&self) -> &[GoSourceTypeKey] {
        &self.types
    }
    pub fn underlying_type_id(&self, id: &str) -> Option<&str> {
        self.underlying.get(id).map(String::as_str)
    }
    pub fn source_string_type_ids(&self) -> &[String] {
        &self.strings
    }
    pub fn case_type_ids(&self) -> &[String] {
        &self.cases
    }
    pub fn prior_layout_certificate_bytes(&self) -> &[u8] {
        &self.prior_layout
    }
    pub fn generic_string_certificate_bytes(&self) -> &[u8] {
        &self.generic_string
    }
    pub fn generate_row(&self, id: &str) -> Result<GoSourceStringRow> {
        if self.strings.binary_search_by(|s| s.as_str().cmp(id)).is_err() {
            return Err(GoOrdinaryError::Linkage);
        }
        let module = format!("{}.Strings.T{id}", self.source);
        let mut builder = self.builder.clone();
        Adapter {
            builder: &mut builder,
            source: &self.source,
            id,
            module: &module,
        }
        .append()?;
        builder.certificate.module = module.clone();
        Ok(GoSourceStringRow {
            source: self.source.clone(),
            source_type: id.into(),
            module,
            certificate: builder.finish()?,
        })
    }
    pub fn import_row(&self, bytes: &[u8], id: &str) -> Result<GoSourceStringRow> {
        if bytes.len() > MAX_BYTES {
            return Err(GoOrdinaryError::Limit);
        }
        let expected = self.generate_row(id)?;
        if bytes != expected.certificate_bytes() {
            return Err(GoOrdinaryError::Linkage);
        }
        Ok(expected)
    }
}

/// Every original source type and the complete source error universe remain
/// in each row. Only the row's string type is selected; U is never reduced.
pub fn generate_go_practical_source_string_catalogue(
    domain: &GoSourceValueDomain,
) -> Result<GoSourceStringCatalogue> {
    let (source_builder, layout, _) = source_error_layout::source_builder(domain)?;
    let prior_layout = source_builder.clone().finish()?;
    let mut generic = word_index_builder()?;
    string::append(&mut generic)?;
    let generic_string = generic.clone().finish()?;
    let builder = composition::source_string_prerequisites(
        &[source_builder.certificate, generic.certificate],
        layout.module_name(),
    )?;
    let keys = domain
        .types()
        .map(|k| (k.id().to_string(), k.clone()))
        .collect::<BTreeMap<_, _>>();
    let underlying = keys
        .keys()
        .filter_map(|id| domain.underlying(id).map(|u| (id.clone(), u.to_string())))
        .collect::<BTreeMap<_, _>>();
    let mut strings = vec![];
    for id in keys.keys() {
        let mut current = id.as_str();
        let mut visited = BTreeSet::new();
        loop {
            if !visited.insert(current) {
                return Err(GoOrdinaryError::Linkage);
            }
            match keys.get(current).ok_or(GoOrdinaryError::Linkage)? {
                GoSourceTypeKey::Nominal { .. } => {
                    current = underlying.get(current).ok_or(GoOrdinaryError::Linkage)?;
                }
                GoSourceTypeKey::Builtin { name, .. } if name == "string" => {
                    strings.push(id.clone());
                    break;
                }
                _ => break,
            }
        }
    }
    Ok(GoSourceStringCatalogue {
        builder,
        source: layout.module_name().into(),
        types: keys.into_values().collect(),
        underlying,
        strings,
        cases: layout.case_type_ids().to_vec(),
        prior_layout,
        generic_string,
    })
}

fn generic(n: &str, args: Vec<Expr>) -> Expr {
    call(&format!("Mpk.Go.Practical.String.{n}"), args)
}
fn word_type(width: usize) -> Expr {
    word::word_type(width).expect("frozen source string word")
}
fn slots() -> Expr {
    (0..8).rev().fold(word_type(8), |ty, i| {
        pi(&format!("_string_slot{i}"), boolean(), ty)
    })
}
fn raw_string() -> Expr {
    error_layout::product(&[word_type(64), slots()])
}
struct Adapter<'a> {
    builder: &'a mut Builder,
    source: &'a str,
    id: &'a str,
    module: &'a str,
}
impl Adapter<'_> {
    fn carrier(&self) -> Expr {
        constant(&format!("{}.T{}.Carrier", self.source, self.id))
    }
    fn zero(&self) -> Expr {
        constant(&format!("{}.T{}.Zero", self.source, self.id))
    }
    fn own(&self, name: &str, args: Vec<Expr>) -> Expr {
        call(&format!("{}.{name}", self.module), args)
    }
    fn math(&self, value: Expr) -> Expr {
        self.own("ToMath", vec![value])
    }
    fn emit(&mut self, name: &str, args: &[Binder], ty: Expr, body: Expr, law: bool) -> Result<()> {
        emit(self.builder, &format!("{}.{name}", self.module), args, ty, body, law)
    }
    fn append(&mut self) -> Result<()> {
        let (carrier, raw) = (self.carrier(), raw_string());
        self.emit("ToMath", &[("s", carrier.clone())], raw.clone(), var("s"), false)?;
        self.emit("ToSource", &[("m", raw.clone())], carrier.clone(), var("m"), false)?;
        let (s, m) = (var("s"), var("m"));
        self.emit(
            "SourceRoundTrip", &[("s", carrier.clone())],
            eq(carrier.clone(), self.own("ToSource", vec![self.math(s.clone())]), s.clone()),
            refl(carrier.clone(), s.clone()), true,
        )?;
        self.emit(
            "MathRoundTrip", &[("m", raw.clone())],
            eq(raw.clone(), self.math(self.own("ToSource", vec![m.clone()])), m.clone()),
            refl(raw.clone(), m), true,
        )?;
        let math_zero = self.math(self.zero());
        self.emit("ZeroToMath", &[], eq(raw.clone(), math_zero.clone(), generic("Zero", vec![])),
            refl(raw.clone(), generic("Zero", vec![])), true)?;
        let make_args = [("length", word_type(64)), ("data", slots())];
        let made = self.own("Make", vec![var("length"), var("data")]);
        self.emit("Make", &make_args, carrier.clone(),
            self.own("ToSource", vec![generic("Make", vec![var("length"), var("data")])]), false)?;
        self.emit("MakeToMath", &make_args,
            eq(raw, self.math(made.clone()), generic("Make", vec![var("length"), var("data")])),
            self.own("MathRoundTrip", vec![generic("Make", vec![var("length"), var("data")])]), true)?;
        let sequence_type = error_layout::product(&[nat(), pi("_index", nat(), word_type(8))]);
        for (name, ty) in [
            ("Length", word_type(64)), ("Data", slots()), ("Size", nat()),
            ("Sequence", sequence_type), ("LengthFits", Expr::Sort0),
            ("Padding", Expr::Sort0), ("Valid", Expr::Sort0),
        ] {
            self.emit(name, &[("s", carrier.clone())], ty,
                generic(name, vec![self.math(s.clone())]), false)?;
        }
        self.emit("Byte", &[("s", carrier.clone()), ("i", word_type(64))], word_type(8),
            generic("Byte", vec![self.math(s.clone()), var("i")]), false)?;
        for (name, slot_count, bit_count) in [("LengthBit", 0, 6), ("SlotBit", 8, 3)] {
            let names = (0..slot_count).map(|i| format!("slot{i}"))
                .chain((0..bit_count).map(|i| format!("bit{i}"))).collect::<Vec<_>>();
            let mut args = vec![("s", carrier.clone())];
            args.extend(names.iter().map(|n| (n.as_str(), boolean())));
            let mut values = vec![self.math(s.clone())];
            values.extend(names.iter().map(|n| var(n)));
            self.emit(name, &args, boolean(), generic(name, values), false)?;
            let mut args = make_args.to_vec();
            args.extend(names.iter().map(|n| (n.as_str(), boolean())));
            let mut values = vec![made.clone()];
            values.extend(names.iter().map(|n| var(n)));
            let wanted = if slot_count == 0 {
                app(var("length"), names.iter().map(|n| var(n)).collect())
            } else {
                app(app(var("data"), names[..slot_count].iter().map(|n| var(n)).collect()),
                    names[slot_count..].iter().map(|n| var(n)).collect())
            };
            let mut proof_args = vec![var("length"), var("data")];
            proof_args.extend(names.iter().map(|n| var(n)));
            self.emit(&format!("Make{name}"), &args,
                eq(boolean(), self.own(name, values), wanted),
                generic(&format!("Make{name}"), proof_args), true)?;
        }
        for (name, ty, want) in [("Length", word_type(64), var("length")),
            ("Data", slots(), var("data")),
            ("Size", nat(), call("Mpk.Go.Practical.Word.Numeric.Value6", vec![var("length")]))] {
            self.emit(&format!("Make{name}"), &make_args,
                eq(ty, self.own(name, vec![made.clone()]), want),
                generic(&format!("Make{name}"), vec![var("length"), var("data")]), true)?;
        }
        let mut valid_args = make_args.to_vec();
        valid_args.extend([("upper", self.own("LengthFits", vec![made.clone()])),
            ("padding", self.own("Padding", vec![made.clone()]))]);
        self.emit("MakeValid", &valid_args, self.own("Valid", vec![made]),
            generic("MakeValid", vec![var("length"), var("data"), var("upper"), var("padding")]), true)?;
        let valid_args = [("s", carrier.clone()), ("valid", self.own("Valid", vec![s.clone()]))];
        for (name, predicate) in [("ValidUpper", "LengthFits"), ("ValidPadding", "Padding")] {
            self.emit(name, &valid_args, self.own(predicate, vec![s.clone()]),
                generic(name, vec![self.math(s.clone()), var("valid")]), true)?;
        }
        // Transport the checked generic zero witness along the complete value
        // equality, rather than treating a pointwise relation as function Eq.
        for (name, predicate) in [("ZeroValid", "Valid"), ("ZeroPadding", "Padding"), ("ZeroFits", "LengthFits")] {
            let zero = generic("Zero", vec![]);
            let same = call("Std.Eq.symm", vec![raw_string(), math_zero.clone(), zero.clone(), self.own("ZeroToMath", vec![])]);
            let proof = call("Std.Eq.rewrite", vec![raw_string(), zero, math_zero.clone(),
                lam("_zero_string", raw_string(), generic(predicate, vec![var("_zero_string")])),
                same, generic(name, vec![])]);
            self.emit(name, &[], self.own(predicate, vec![self.zero()]), proof, true)?;
        }
        for (name, generic_law, ty, want) in [
            ("ZeroLength", "ZeroLength", word_type(64), generic("Length", vec![generic("Zero", vec![])])),
            ("ZeroData", "ZeroData", slots(), generic("Data", vec![generic("Zero", vec![])])),
            ("ZeroSize", "ZeroValue", nat(), zero()),
        ] {
            // Length/Data preserve the whole function-valued observation. The
            // separate ZeroSize law decodes the complete 64-bit length.
            let observation = if name == "ZeroLength" { "Length" } else if name == "ZeroData" { "Data" } else { "Size" };
            let proof = if name == "ZeroSize" {
                generic(generic_law, vec![])
            } else {
                refl(ty.clone(), want.clone())
            };
            self.emit(name, &[], eq(ty, self.own(observation, vec![self.zero()]), want), proof, true)?;
        }
        let i = var("i");
        let within = eq(boolean(), less(call("Mpk.Go.Practical.Word.Numeric.Value6", vec![i.clone()]),
            self.own("Size", vec![s.clone()])), bit(true));
        let guarded = [("s", carrier), ("i", word_type(64)),
            ("valid", self.own("Valid", vec![s.clone()])), ("within", within)];
        let proof_args = vec![self.math(s.clone()), i.clone(), var("valid"), var("within")];
        self.emit("ValidWithinCapacity", &guarded,
            eq(boolean(), less(call("Mpk.Go.Practical.Word.Numeric.Value6", vec![i.clone()]),
                call("Mpk.Go.Practical.FiniteVector.D8.Capacity", vec![])), bit(true)),
            generic("ValidWithinCapacity", proof_args.clone()), true)?;
        let address = (0..8).rev().map(|b| word::at(i.clone(), 64, b)).collect::<Result<Vec<_>>>()?;
        self.emit("ValidByteWithinSlot", &guarded,
            eq(word_type(8), self.own("Byte", vec![s.clone(), i]), app(self.own("Data", vec![s]), address)),
            generic("ValidByteWithinSlot", proof_args), true)
    }
}
