//! Private shared ordinary lexical definitions for canonical boundary JSON.
//! Enclosing schema grammar still owns delimiters, fields and typed structure.
use super::hex_codecs::{call, word};
use super::integer_format::{circuit, define};
use super::temporal::literal;
use super::*;
const NAME: &str = "Mpk.CSharp.Ordinary.JsonTokens";
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct OrdinaryJsonUnsignedDefinition {
    pub width: u32,
    /// C7 packet: prefix-valid at 0, whole-valid at 1, u64 value at 2..66,
    /// u32 consumed at 66..98, zero padding. Invalid results are entirely zero.
    /// Digits are consumed greedily; a suffix delimiter is checked by the caller.
    pub parse_definition: String,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct OrdinaryJsonSignedDefinition {
    pub width: u32,
    /// Same C7 packet as unsigned, with width-bit two's-complement value
    /// zero-extended to the 64-bit slot. Semantic i64 remains a quoted codec.
    pub parse_definition: String,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct OrdinaryJsonScalarTokenDefinition {
    /// Lexical kind, not a registered type ID. Raw and quoted kinds are kept
    /// in separate lists; the quoted list includes char and 64-bit codecs.
    pub kind: String,
    /// Arguments: document C24, absolute u32 start C5, ending C3.
    /// Ending is 0=EOF, 1=comma, 2=']', 3='}'; other values reject.
    /// C7 result: valid, whole-document-end, u64 payload, absolute u32 end,
    /// then zero padding. The delimiter is checked but not consumed.
    /// Invalid is entirely zero. Wide semantic integers remain quoted codecs.
    pub parse_definition: String,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct OrdinaryJsonQuotedCodecDefinition {
    /// Exact ordinary parse/format definitions in this same environment.
    pub codec: OrdinaryHexCodecDefinition,
    /// Arguments: C24 document, C5 absolute u32 start, C3 ending (0..3 only).
    /// C8 packet: valid at 0, whole at 1, zero-extended value at 2..130,
    /// absolute u32 end at 130..162, then zero padding. Invalid is all zero.
    pub parse_definition: String,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct OrdinaryJsonDecimalTokenDefinition {
    /// Original exact normalized/fixed-scale parser and closed configuration.
    pub codec: OrdinaryDecimalParseDefinition,
    /// Arguments: C24 document, C5 absolute start, C3 ending (0..3 only).
    /// C10 packet: valid at 0, whole at 1, full C9 decimal at 2..514,
    /// absolute u32 end at 514..546, then zero padding. Invalid is all zero.
    pub parse_definition: String,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct OrdinaryJsonCalendarTokenDefinition {
    pub codec: OrdinaryCalendarCodecDefinition,
    /// C24 document, C5 absolute start, C3 ending (0..3); C7 result:
    /// valid at 0, whole at 1, zero-extended day/ticks at 2..66,
    /// absolute u32 end at 66..98, then zero padding. Invalid is all zero.
    pub parse_definition: String,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct OrdinaryJsonTokenDefinition {
    pub strings: OrdinaryJsonStringParseDefinition,
    pub keywords: OrdinaryJsonKeywordDefinition,
    pub unsigned: Vec<OrdinaryJsonUnsignedDefinition>,
    pub signed: Vec<OrdinaryJsonSignedDefinition>,
    pub scalars: Vec<OrdinaryJsonScalarTokenDefinition>,
    /// Arguments: document C24, absolute u32 start C5, ending C3.
    /// Endings 0..3 match scalar framing; 4 requires a colon (field name).
    /// C20 result: role zero is a zero-padded C7 header (valid, at EOF,
    /// absolute u32 end, decoded u32 length); role one is the C19 string.
    /// Invalid results are entirely zero; the delimiter is not consumed.
    pub string_frame_definition: String,
    /// Project the C7 header from a C20 framed string result.
    pub string_frame_header_definition: String,
    /// Project the C19 decoded UTF-16 string from a C20 framed result.
    pub string_frame_value_definition: String,
    /// Quoted semantic char/i64/u64/duration/instant values. Each uses the
    /// same C7 result and argument layout as scalars; colon is not admitted.
    pub quoted_scalars: Vec<OrdinaryJsonScalarTokenDefinition>,
    pub quoted_codecs: Vec<OrdinaryJsonQuotedCodecDefinition>,
    pub quoted_decimals: Vec<OrdinaryJsonDecimalTokenDefinition>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub quoted_calendars: Vec<OrdinaryJsonCalendarTokenDefinition>,
    pub static_transformers: usize,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct OrdinaryJsonTokenProgram {
    schema: String,
    source_ir_sha256: String,
    foundation_sha256: String,
    boundary_program_sha256: String,
    definition: Option<OrdinaryJsonTokenDefinition>,
    certificate_sha256: String,
    #[serde(skip)]
    certificate: Vec<u8>,
}
impl OrdinaryJsonTokenProgram {
    pub fn definition(&self) -> Option<&OrdinaryJsonTokenDefinition> {
        self.definition.as_ref()
    }
    pub fn certificate_bytes(&self) -> &[u8] {
        &self.certificate
    }
    pub fn canonical_bytes(&self) -> Vec<u8> {
        serde_json::to_vec(self).expect("ordinary JSON tokens")
    }
}

fn name(s: &str) -> String {
    format!("{NAME}.{s}")
}
fn digit(c: &mut Circuit, byte: &[Bit]) -> Bit {
    let low = c.lt(byte, &literal(48, 8), false);
    let high = c.lt(&literal(57, 8), byte, false);
    let bad = c.or(low, high);
    c.not(bad)
}
// C7 scan state: u64 accumulator, five-bit count, failed, stopped, zero padding.
// Twenty concrete steps cover every admitted u64 token. Finish checks the next
// byte too, so a longer run of digits cannot be accepted as a shortened token.
fn unsigned(
    b: &mut Builder,
    document: &OrdinaryBoundaryDocumentDefinition,
) -> R<Vec<OrdinaryJsonUnsignedDefinition>> {
    // Earlier lexical circuits may already emit C7 register helpers.
    // Keep the shared declaration unique, as emit_circuit does.
    if !b.globals.contains_key(&format!("{PREFIX}.Cube.D7.Compose")) {
        b.helpers(7)?;
    }
    let c = Circuit::new(&[128]);
    let mut count = c.inputs[0][64..69].to_vec();
    count.resize(32, F);
    let count = circuit(b, &name("Count"), c, count)?;
    let mut c = Circuit::new(&[128, 8, 32]);
    let state = c.inputs[0].clone();
    let byte = c.inputs[1].clone();
    let length = c.inputs[2].clone();
    let value = &state[..64];
    let n = &state[64..69];
    let mut index = n.to_vec();
    index.resize(32, F);
    let exists = c.lt(&index, &length, false);
    let inactive = c.or(state[69], state[70]);
    let live = c.not(inactive);
    let read = c.and(live, exists);
    let is_digit = digit(&mut c, &byte);
    let take = c.and(read, is_digit);
    let raw = c.sub(&byte, &literal(48, 8)).0;
    let mut decimal = raw[..4].to_vec();
    decimal.resize(64, F);
    let zero = c.equal(value, &literal(0, 64));
    let first = c.equal(n, &literal(0, 5));
    let later = c.not(first);
    let leading_zero = c.and(later, zero);
    let limit = literal(u64::MAX as u128 / 10, 64);
    let over = c.lt(&limit, value, false);
    let equal = c.equal(value, &limit);
    let last_over = c.lt(&literal(5, 64), &decimal, false);
    let last_over = c.and(equal, last_over);
    let overflow = c.or(over, last_over);
    let bad = c.or(leading_zero, overflow);
    let bad = c.and(take, bad);
    let failed = c.or(state[69], bad);
    let mut times_two = vec![F];
    times_two.extend(&value[..63]);
    let mut times_eight = vec![F; 3];
    times_eight.extend(&value[..61]);
    let times_ten = c.add(&times_two, &times_eight, F).0;
    let next_value = c.add(&times_ten, &decimal, F).0;
    let next_count = c.add(n, &literal(1, 5), F).0;
    let mut next = c.select(take, &next_value, value);
    next.extend(c.select(take, &next_count, n));
    next.push(failed);
    let stopped = c.not(take);
    next.push(stopped);
    next.resize(128, F);
    let advance = circuit(b, &name("Advance"), c, next)?;
    let doc = b.var(1)?;
    let state = b.var(0)?;
    let index = call(b, &count, vec![state])?;
    let byte = call(b, &document.read_byte_definition, vec![doc, index])?;
    let length = call(b, &document.length_definition, vec![doc])?;
    let body = call(b, &advance, vec![state, byte, length])?;
    define(b, &name("Step"), &[24, 7], 7, body)?;
    let doc = b.var(0)?;
    let step = call(b, &name("Step"), vec![doc])?;
    let steps = b.compose(7, &[step; 20])?;
    let zero = b.constant(&format!("{PREFIX}.Cube.D7.Zero"))?;
    let body = b.app(steps, vec![zero])?;
    define(b, &name("Scan"), &[24], 7, body)?;
    let mut out = vec![];
    for width in [8, 16, 32, 64] {
        let mut c = Circuit::new(&[128, 32, 8]);
        let state = c.inputs[0].clone();
        let length = c.inputs[1].clone();
        let next = c.inputs[2].clone();
        let value = &state[..64];
        let mut consumed = state[64..69].to_vec();
        consumed.resize(32, F);
        let empty = c.equal(&consumed, &literal(0, 32));
        let nonempty = c.not(empty);
        let bounded = c.lt(&length, &literal(1_048_577, 32), false);
        let too_wide = c.lt(&literal((1u128 << width) - 1, 64), value, false);
        let fits = c.not(too_wide);
        let healthy = c.not(state[69]);
        let next_exists = c.lt(&consumed, &length, false);
        let next_digit = digit(&mut c, &next);
        let longer = c.and(next_exists, next_digit);
        let ended = c.not(longer);
        let mut valid = c.and(nonempty, bounded);
        for bit in [fits, healthy, ended] {
            valid = c.and(valid, bit);
        }
        let whole = c.equal(&consumed, &length);
        let mut packet = vec![T, whole];
        packet.extend(value);
        packet.extend(consumed);
        packet.resize(128, F);
        let packet = c.select(valid, &packet, &literal(0, 128));
        let finish = circuit(b, &name(&format!("Finish{width}")), c, packet)?;
        let doc = b.var(0)?;
        let scan = call(b, &name("Scan"), vec![doc])?;
        // Bind the scan once for state and lookahead consumption.
        let doc = b.var(1)?;
        let state = b.var(0)?;
        let index = call(b, &count, vec![state])?;
        let next = call(b, &document.read_byte_definition, vec![doc, index])?;
        let length = call(b, &document.length_definition, vec![doc])?;
        let body = call(b, &finish, vec![state, length, next])?;
        let ty = b.cube(7)?;
        let body = b.term(TermNode::Let {
            ty,
            value: scan,
            body,
        })?;
        let parse_definition = name(&format!("Unsigned{width}"));
        define(b, &parse_definition, &[24], 7, body)?;
        out.push(OrdinaryJsonUnsignedDefinition {
            width,
            parse_definition,
        });
    }
    Ok(out)
}
// Reuse the exact unsigned token parser after an optional minus. Slice validates
// the original document before shortening it, and Finish checks its bound again.
fn signed(
    b: &mut Builder,
    fragments: &OrdinaryBoundaryFragmentDefinition,
    unsigned64: &OrdinaryJsonUnsignedDefinition,
    widths: &[u32],
    helper_prefix: &str,
) -> R<Vec<OrdinaryJsonSignedDefinition>> {
    let document = &fragments.document;
    let mut c = Circuit::new(&[8]);
    let byte = c.inputs[0].clone();
    let minus = c.equal(&byte, &literal(45, 8));
    let mut offset = vec![minus];
    offset.resize(32, F);
    let offset = circuit(b, &name(&format!("{helper_prefix}SignOffset")), c, offset)?;
    let mut c = Circuit::new(&[32, 32]);
    let length = c.inputs[0].clone();
    let offset_word = c.inputs[1].clone();
    let remaining = c.sub(&length, &offset_word).0;
    let remaining = circuit(
        b,
        &name(&format!("{helper_prefix}AfterSignLength")),
        c,
        remaining,
    )?;
    let zero_index = word(b, 0, 5)?;
    let mut definitions = vec![];
    for &width in widths {
        let mut c = Circuit::new(&[128, 8, 32]);
        let parsed = c.inputs[0].clone();
        let byte = c.inputs[1].clone();
        let length = c.inputs[2].clone();
        let magnitude = &parsed[2..66];
        let minus = c.equal(&byte, &literal(45, 8));
        let limit = 1u128 << (width - 1);
        let bound = c.select(minus, &literal(limit, 64), &literal(limit - 1, 64));
        let overflow = c.lt(&bound, magnitude, false);
        let fits = c.not(overflow);
        let zero = c.equal(magnitude, &literal(0, 64));
        let negative_zero = c.and(minus, zero);
        let canonical = c.not(negative_zero);
        let bounded = c.lt(&length, &literal(1_048_577, 32), false);
        let mut valid = c.and(parsed[0], fits);
        for bit in [canonical, bounded] {
            valid = c.and(valid, bit);
        }
        let negative = c.neg(&magnitude[..width as usize]);
        let mut value = c.select(minus, &negative, &magnitude[..width as usize]);
        value.resize(64, F);
        let mut sign_count = vec![minus];
        sign_count.resize(32, F);
        let count = c.add(&parsed[66..98], &sign_count, F).0;
        let whole = c.equal(&count, &length);
        let mut packet = vec![T, whole];
        packet.extend(value);
        packet.extend(count);
        packet.resize(128, F);
        let packet = c.select(valid, &packet, &literal(0, 128));
        let finish = circuit(b, &name(&format!("SignedFinish{width}")), c, packet)?;
        let doc = b.var(0)?;
        let first = call(b, &document.read_byte_definition, vec![doc, zero_index])?;
        let offset = call(b, &offset, vec![first])?;
        let length = call(b, &document.length_definition, vec![doc])?;
        let remaining = call(b, &remaining, vec![length, offset])?;
        let digits = call(b, &fragments.slice_definition, vec![doc, offset, remaining])?;
        let parsed = call(b, &unsigned64.parse_definition, vec![digits])?;
        let doc = b.var(1)?;
        let packet = b.var(0)?;
        let first = call(b, &document.read_byte_definition, vec![doc, zero_index])?;
        let length = call(b, &document.length_definition, vec![doc])?;
        let body = call(b, &finish, vec![packet, first, length])?;
        let ty = b.cube(7)?;
        let body = b.term(TermNode::Let {
            ty,
            value: parsed,
            body,
        })?;
        let parse_definition = name(&format!("Signed{width}"));
        define(b, &parse_definition, &[24], 7, body)?;
        definitions.push(OrdinaryJsonSignedDefinition {
            width,
            parse_definition,
        });
    }
    Ok(definitions)
}

// Typed scalar grammar leaves. Enclosing array/object parsing chooses the exact
// expected delimiter and owns field order, missing/null rules and cell counts.
fn scalars(
    b: &mut Builder,
    fragments: &OrdinaryBoundaryFragmentDefinition,
    keywords: &OrdinaryJsonKeywordDefinition,
    unsigned: &[OrdinaryJsonUnsignedDefinition],
    signed: &[OrdinaryJsonSignedDefinition],
) -> R<Vec<OrdinaryJsonScalarTokenDefinition>> {
    let mut parsers = vec![];
    for (id, null) in [("bool", false), ("null", true)] {
        let mut c = Circuit::new(&[64]);
        let raw = c.inputs[0].clone();
        let kind = if null { raw[2] } else { c.not(raw[2]) };
        let valid = c.and(raw[0], kind);
        let mut payload = vec![raw[0], raw[1], if null { F } else { raw[3] }];
        payload.resize(66, F);
        payload.extend(&raw[4..36]);
        payload.resize(128, F);
        let packet = c.select(valid, &payload, &literal(0, 128));
        let convert = circuit(b, &name(&format!("Scalar.{id}.Packet")), c, packet)?;
        let doc = b.var(0)?;
        let raw = call(b, &keywords.parse_definition, vec![doc])?;
        let body = call(b, &convert, vec![raw])?;
        let parser = name(&format!("Scalar.{id}.Token"));
        define(b, &parser, &[24], 7, body)?;
        parsers.push((id.to_string(), parser));
    }
    // Raw u64 is a metadata token, not an admitted semantic u64 value.
    for d in unsigned.iter().filter(|d| d.width <= 32) {
        parsers.push((format!("u{}", d.width), d.parse_definition.clone()));
    }
    for d in signed {
        parsers.push((format!("i{}", d.width), d.parse_definition.clone()));
    }
    let mut c = Circuit::new(&[32, 32]);
    let length = c.inputs[0].clone();
    let start = c.inputs[1].clone();
    let remaining = c.sub(&length, &start).0;
    let remaining = circuit(b, &name("Scalar.Remaining"), c, remaining)?;
    let mut c = Circuit::new(&[128, 32]);
    let count = c.inputs[0][66..98].to_vec();
    let start = c.inputs[1].clone();
    let end = c.add(&start, &count, F).0;
    let end_definition = circuit(b, &name("Scalar.End"), c, end)?;
    let mut c = Circuit::new(&[128, 32, 32, 8, 8]);
    let raw = c.inputs[0].clone();
    let start = c.inputs[1].clone();
    let length = c.inputs[2].clone();
    let ending = c.inputs[3].clone();
    let next = c.inputs[4].clone();
    let (end, overflow) = c.add(&start, &raw[66..98], F);
    let no_overflow = c.not(overflow);
    let bounded = c.lt(&length, &literal(1_048_577, 32), false);
    let reversed = c.lt(&length, &start, false);
    let start_ok = c.not(reversed);
    let past = c.lt(&length, &end, false);
    let end_ok = c.not(past);
    let empty = c.equal(&raw[66..98], &literal(0, 32));
    let nonempty = c.not(empty);
    let whole = c.equal(&end, &length);
    let eof = c.equal(&ending, &literal(0, 8));
    let mut delimiter = c.and(eof, whole);
    let exists = c.lt(&end, &length, false);
    for (tag, byte) in [(1, b','), (2, b']'), (3, b'}')] {
        let selected = c.equal(&ending, &literal(tag, 8));
        let matches = c.equal(&next, &literal(byte as u128, 8));
        let accepted = c.and(selected, matches);
        let accepted = c.and(accepted, exists);
        delimiter = c.or(delimiter, accepted);
    }
    let mut valid = raw[0];
    for check in [no_overflow, bounded, start_ok, end_ok, nonempty, delimiter] {
        valid = c.and(valid, check);
    }
    let mut packet = vec![T, whole];
    packet.extend(&raw[2..66]);
    packet.extend(end);
    packet.resize(128, F);
    let packet = c.select(valid, &packet, &literal(0, 128));
    let finish = circuit(b, &name("Scalar.Finish"), c, packet)?;
    let document = &fragments.document;
    let mut definitions = vec![];
    for (kind, parser) in parsers {
        let doc = b.var(2)?;
        let start = b.var(1)?;
        let length = call(b, &document.length_definition, vec![doc])?;
        let remaining = call(b, &remaining, vec![length, start])?;
        let suffix = call(b, &fragments.slice_definition, vec![doc, start, remaining])?;
        let parsed = call(b, &parser, vec![suffix])?;
        // Bind parsed once; all enclosing arguments shift under the Let.
        let doc = b.var(3)?;
        let start = b.var(2)?;
        let ending = b.var(1)?;
        let raw = b.var(0)?;
        let end = call(b, &end_definition, vec![raw, start])?;
        let next = call(b, &document.read_byte_definition, vec![doc, end])?;
        let length = call(b, &document.length_definition, vec![doc])?;
        let body = call(b, &finish, vec![raw, start, length, ending, next])?;
        let ty = b.cube(7)?;
        let body = b.term(TermNode::Let {
            ty,
            value: parsed,
            body,
        })?;
        let parse_definition = name(&format!("Scalar.{kind}.Parse"));
        define(b, &parse_definition, &[24, 5, 3], 7, body)?;
        definitions.push(OrdinaryJsonScalarTokenDefinition {
            kind,
            parse_definition,
        });
    }
    Ok(definitions)
}
fn quoted_scalars(
    b: &mut Builder,
    fragments: &OrdinaryBoundaryFragmentDefinition,
    unsigned64: &OrdinaryJsonUnsignedDefinition,
    frame: [&str; 3],
) -> R<Vec<OrdinaryJsonScalarTokenDefinition>> {
    let signed64 = signed(b, fragments, unsigned64, &[64], "Quoted.")?
        .pop()
        .ok_or(OrdinaryCarrierError::Linkage)?;
    let mut c = Circuit::new(&[32]);
    let start = c.inputs[0].clone();
    let interior_start = c.add(&start, &literal(1, 32), F).0;
    let interior_start = circuit(b, &name("Quoted.InteriorStart"), c, interior_start)?;
    let mut c = Circuit::new(&[128, 32]);
    let end = c.inputs[0][2..34].to_vec();
    let start = c.inputs[1].clone();
    let consumed = c.sub(&end, &start).0;
    let count = c.sub(&consumed, &literal(2, 32)).0;
    let interior_count = circuit(b, &name("Quoted.InteriorCount"), c, count)?;

    // A successful canonical quoted integer consists only of literal ASCII
    // digits and an optional minus: JSON forbids their escaped spellings. The
    // exact interior slice must be wholly consumed by the existing numeric
    // parser. Frame validity owns both quotes and all original byte/cursor
    // bounds; raw whole-valid rejects whitespace, suffixes and shortened parses.
    let mut c = Circuit::new(&[128, 128, 8]);
    let head = c.inputs[0].clone();
    let raw = c.inputs[1].clone();
    let ending = c.inputs[2].clone();
    let value_ending = c.lt(&ending, &literal(4, 8), false);
    let mut valid = c.and(head[0], raw[0]);
    for check in [raw[1], value_ending] {
        valid = c.and(valid, check);
    }
    let mut result = vec![T, head[1]];
    result.extend(&raw[2..66]);
    result.extend(&head[2..34]);
    result.resize(128, F);
    let result = c.select(valid, &result, &literal(0, 128));
    let finish = circuit(b, &name("Quoted.Finish"), c, result)?;

    let mut c = Circuit::new(&[128, 16]);
    let head = c.inputs[0].clone();
    let unit = c.inputs[1].clone();
    let one = c.equal(&head[34..66], &literal(1, 32));
    let mut result = vec![one, T];
    result.extend(unit);
    result.resize(128, F);
    let char_packet = circuit(b, &name("Quoted.CharPacket"), c, result)?;

    let mut definitions = vec![];
    for (kind, parser) in [
        ("char", None),
        ("i64", Some(signed64.parse_definition.as_str())),
        ("u64", Some(unsigned64.parse_definition.as_str())),
        ("duration", Some(signed64.parse_definition.as_str())),
        ("instant", Some(signed64.parse_definition.as_str())),
    ] {
        let doc = b.var(2)?;
        let start = b.var(1)?;
        let ending = b.var(0)?;
        let framed = call(b, frame[0], vec![doc, start, ending])?;
        // Under the frame Let: document, start, ending, complete string frame.
        let doc = b.var(3)?;
        let start = b.var(2)?;
        let ending = b.var(1)?;
        let framed_value = b.var(0)?;
        let head = call(b, frame[1], vec![framed_value])?;
        let raw = if let Some(parser) = parser {
            let offset = call(b, &interior_start, vec![start])?;
            let count = call(b, &interior_count, vec![head, start])?;
            let interior = call(b, &fragments.slice_definition, vec![doc, offset, count])?;
            call(b, parser, vec![interior])?
        } else {
            let text = call(b, frame[2], vec![framed_value])?;
            let mut address = vec![super::hex_codecs::truth(b, true)?];
            address.extend(vec![super::hex_codecs::truth(b, false)?; 14]);
            let unit = b.app(text, address)?;
            call(b, &char_packet, vec![head, unit])?
        };
        let body = call(b, &finish, vec![head, raw, ending])?;
        let ty = b.cube(20)?;
        let body = b.term(TermNode::Let {
            ty,
            value: framed,
            body,
        })?;
        let parse_definition = name(&format!("Quoted.{kind}.Parse"));
        define(b, &parse_definition, &[24, 5, 3], 7, body)?;
        definitions.push(OrdinaryJsonScalarTokenDefinition {
            kind: kind.into(),
            parse_definition,
        });
    }
    Ok(definitions)
}

fn quoted_codecs(b: &mut Builder, frame: [&str; 3]) -> R<Vec<OrdinaryJsonQuotedCodecDefinition>> {
    let mut definitions = vec![];
    for (id, token, width) in [
        ("binary32", "f32", 32),
        ("binary64", "f64", 64),
        ("guid.n", "guid", 128),
        ("guid.d", "guid", 128),
    ] {
        let codec = super::hex_codecs::codec(b, id, token, width)?;
        let depth = codec.value_depth + 1;
        let mut c = Circuit::new(&[128, 1 << depth, 8]);
        let head = c.inputs[0].clone();
        let raw = c.inputs[1].clone();
        let ending = c.inputs[2].clone();
        let success = c.not(raw[0]);
        let value_ending = c.lt(&ending, &literal(4, 8), false);
        let valid = c.and(head[0], success);
        let valid = c.and(valid, value_ending);
        let mut result = vec![T, head[1]];
        // The codec result is a closed sum: zero success tag in role zero,
        // exact width-bit payload in role one. Preserve every raw float bit
        // (including NaN payloads and signed zero) and all 128 GUID bits.
        for k in 0..width as usize {
            result.push(raw[1 | (k << 1)]);
        }
        result.resize(130, F);
        result.extend(&head[2..34]);
        result.resize(256, F);
        let result = c.select(valid, &result, &vec![F; 256]);
        let finish = circuit(b, &name(&format!("QuotedCodec.{id}.Finish")), c, result)?;
        let doc = b.var(2)?;
        let start = b.var(1)?;
        let ending = b.var(0)?;
        let framed = call(b, frame[0], vec![doc, start, ending])?;
        // Bind the full string frame once. Parse only its decoded UTF-16 value;
        // both original frame validity and the codec's exact success tag are
        // mandatory. A failed frame's empty value cannot turn into success.
        let framed_value = b.var(0)?;
        let ending = b.var(1)?;
        let head = call(b, frame[1], vec![framed_value])?;
        let text = call(b, frame[2], vec![framed_value])?;
        let parsed = call(b, &codec.parse_definition, vec![text])?;
        let body = call(b, &finish, vec![head, parsed, ending])?;
        let ty = b.cube(20)?;
        let body = b.term(TermNode::Let {
            ty,
            value: framed,
            body,
        })?;
        let parse_definition = name(&format!("QuotedCodec.{id}.Parse"));
        define(b, &parse_definition, &[24, 5, 3], 8, body)?;
        definitions.push(OrdinaryJsonQuotedCodecDefinition {
            codec,
            parse_definition,
        });
    }
    Ok(definitions)
}

fn quoted_decimals(
    b: &mut Builder,
    frame: [&str; 3],
) -> R<Vec<OrdinaryJsonDecimalTokenDefinition>> {
    let codecs = super::decimal_parse::emit_parser(b)?;
    let mut c = Circuit::new(&[128, 1024, 8]);
    let head = c.inputs[0].clone();
    let raw = c.inputs[1].clone();
    let ending = c.inputs[2].clone();
    let success = c.not(raw[0]);
    let value_ending = c.lt(&ending, &literal(4, 8), false);
    let valid = c.and(head[0], success);
    let valid = c.and(valid, value_ending);
    let mut result = vec![T, head[1]];
    result.extend((0..512).map(|i| raw[1 | (i << 1)]));
    result.extend(&head[2..34]);
    result.resize(1024, F);
    let result = c.select(valid, &result, &vec![F; 1024]);
    // One shared finish circuit serves every closed scale/rounding setting.
    let finish = circuit(b, &name("QuotedDecimal.Finish"), c, result)?;
    let mut definitions = vec![];
    for codec in codecs {
        let doc = b.var(2)?;
        let start = b.var(1)?;
        let ending = b.var(0)?;
        let framed = call(b, frame[0], vec![doc, start, ending])?;
        let framed_value = b.var(0)?;
        let ending = b.var(1)?;
        let head = call(b, frame[1], vec![framed_value])?;
        let text = call(b, frame[2], vec![framed_value])?;
        let parsed = call(b, &codec.parse_definition, vec![text])?;
        let body = call(b, &finish, vec![head, parsed, ending])?;
        let ty = b.cube(20)?;
        let body = b.term(TermNode::Let {
            ty,
            value: framed,
            body,
        })?;
        let suffix = codec.scale.map_or_else(
            || "Normalized".into(),
            |scale| {
                format!(
                    "Fixed.S{scale}.{}",
                    codec.rounding.as_deref().expect("closed fixed-scale codec")
                )
            },
        );
        let parse_definition = name(&format!("QuotedDecimal.{suffix}.Parse"));
        define(b, &parse_definition, &[24, 5, 3], 10, body)?;
        definitions.push(OrdinaryJsonDecimalTokenDefinition {
            codec,
            parse_definition,
        });
    }
    Ok(definitions)
}

pub(super) fn emit(
    b: &mut Builder,
    fragments: OrdinaryBoundaryFragmentDefinition,
) -> R<OrdinaryJsonTokenDefinition> {
    let strings = super::json_string_parsers::emit(b, fragments.clone())?;
    let keywords = super::json_keywords::emit(b, fragments.clone())?;
    let unsigned = unsigned(b, &fragments.document)?;
    let signed = signed(b, &fragments, &unsigned[3], &[8, 16, 32], "")?;
    let scalars = scalars(b, &fragments, &keywords, &unsigned, &signed)?;
    let [string_frame_definition, string_frame_header_definition, string_frame_value_definition] =
        super::json_string_parsers::emit_framed(b, &strings)?;
    let quoted_scalars = quoted_scalars(
        b,
        &fragments,
        &unsigned[3],
        [
            &string_frame_definition,
            &string_frame_header_definition,
            &string_frame_value_definition,
        ],
    )?;
    let quoted_codecs = quoted_codecs(
        b,
        [
            &string_frame_definition,
            &string_frame_header_definition,
            &string_frame_value_definition,
        ],
    )?;
    let quoted_decimals = quoted_decimals(
        b,
        [
            &string_frame_definition,
            &string_frame_header_definition,
            &string_frame_value_definition,
        ],
    )?;
    Ok(OrdinaryJsonTokenDefinition {
        strings,
        keywords,
        unsigned,
        signed,
        scalars,
        string_frame_definition,
        string_frame_header_definition,
        string_frame_value_definition,
        quoted_scalars,
        quoted_codecs,
        quoted_decimals,
        quoted_calendars: vec![],
        static_transformers: b.static_transformers,
    })
}
pub(super) fn add_calendars(
    b: &mut Builder,
    definition: &mut OrdinaryJsonTokenDefinition,
    carriers: &[OrdinaryCarrier],
) -> R<()> {
    for token in ["date", "time"] {
        let Some(carrier) = carriers
            .iter()
            .find(|c| c.type_id == format!("mpk.csharp.value.{token}.v1"))
        else {
            continue;
        };
        let width = if token == "date" { 32 } else { 64 };
        if carrier.depth != address_bits(width) || carrier.shape != (OrdinaryShape::Bits { width })
        {
            return Err(OrdinaryCarrierError::Shape);
        }
        let codec = super::calendar_codecs::codec(b, token)?;
        let mut c = Circuit::new(&[128, 1 << (codec.value_depth + 1), 8]);
        let head = c.inputs[0].clone();
        let raw = c.inputs[1].clone();
        let ending = c.inputs[2].clone();
        let success = c.not(raw[0]);
        let value_ending = c.lt(&ending, &literal(4, 8), false);
        let valid = c.and(head[0], success);
        let valid = c.and(valid, value_ending);
        let mut packet = vec![T, head[1]];
        packet.extend((0..width as usize).map(|i| raw[1 | (i << 1)]));
        packet.resize(66, F);
        packet.extend(&head[2..34]);
        packet.resize(128, F);
        let packet = c.select(valid, &packet, &literal(0, 128));
        let finish = circuit(
            b,
            &name(&format!("QuotedCalendar.{token}.Finish")),
            c,
            packet,
        )?;
        let doc = b.var(2)?;
        let start = b.var(1)?;
        let ending = b.var(0)?;
        let framed = call(
            b,
            &definition.string_frame_definition,
            vec![doc, start, ending],
        )?;
        // Under the C20 frame Let, ending is Var 1 and the frame is Var 0.
        // The original frame owns canonical spelling, cursor and document bounds.
        let frame = b.var(0)?;
        let ending = b.var(1)?;
        let head = call(b, &definition.string_frame_header_definition, vec![frame])?;
        let text = call(b, &definition.string_frame_value_definition, vec![frame])?;
        let parsed = call(b, &codec.parse_definition, vec![text])?;
        let body = call(b, &finish, vec![head, parsed, ending])?;
        let ty = b.cube(20)?;
        let body = b.term(TermNode::Let {
            ty,
            value: framed,
            body,
        })?;
        let parse_definition = name(&format!("QuotedCalendar.{token}.Parse"));
        define(b, &parse_definition, &[24, 5, 3], 7, body)?;
        definition
            .quoted_calendars
            .push(OrdinaryJsonCalendarTokenDefinition {
                codec,
                parse_definition,
            });
    }
    definition.static_transformers = b.static_transformers;
    Ok(())
}

pub fn generate_csharp_practical_ordinary_json_tokens(
    vir: &ValidatedPracticalVir,
) -> R<OrdinaryJsonTokenProgram> {
    let boundary = generate_boundary_vcs(vir).map_err(|_| OrdinaryCarrierError::Linkage)?;
    let mut b = Builder::new()?;
    let definition = if boundary.contracts().is_empty() {
        None
    } else {
        let layouts = generate_csharp_practical_ordinary_carriers(vir)?;
        Some(super::emit_boundary_json_for_carriers(
            &mut b,
            layouts.carriers(),
        )?)
    };
    let certificate = b.finish()?;
    let p = OrdinaryJsonTokenProgram {
        schema: "mpk.csharp.ordinary_json_tokens.v1".into(),
        source_ir_sha256: vir.hash().into(),
        foundation_sha256: vir.construction_context().0.content_sha256().into(),
        boundary_program_sha256: boundary.hash(),
        definition,
        certificate_sha256: mpk_cert::hash_hex(&mpk_cert::certificate_hash(&certificate)),
        certificate,
    };
    if p.canonical_bytes().len() > 16 * 1024 * 1024 {
        return Err(OrdinaryCarrierError::Limit);
    }
    Ok(p)
}
pub fn import_csharp_practical_ordinary_json_tokens(
    input: &[u8],
    certificate: &[u8],
    vir: &ValidatedPracticalVir,
) -> R<OrdinaryJsonTokenProgram> {
    if input.len() > 16 * 1024 * 1024 || certificate.len() > 16 * 1024 * 1024 {
        return Err(OrdinaryCarrierError::Limit);
    }
    let p = generate_csharp_practical_ordinary_json_tokens(vir)?;
    if input != p.canonical_bytes() || certificate != p.certificate_bytes() {
        return Err(OrdinaryCarrierError::Linkage);
    }
    Ok(p)
}
