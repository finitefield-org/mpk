//! Contract adapters reuse the exact ordinary scalar codec bodies.
use super::hex_codecs::{call, truth};
use super::integer_format::{circuit, define};
use super::*;

#[derive(Default)]
pub(in super::super) struct ContractCodecCache {
    integer_formats: Option<Vec<OrdinaryIntegerFormatDefinition>>,
    integer_parsers: Option<Vec<OrdinaryIntegerParseDefinition>>,
    hex: Option<Vec<OrdinaryHexCodecDefinition>>,
    calendar: Option<Vec<OrdinaryCalendarCodecDefinition>>,
    decimal_digits: Option<decimal_format::DecimalDigitHelpers>,
    decimal_normalized: Option<OrdinaryDecimalFormatDefinition>,
    decimal_fixed: Option<Vec<OrdinaryDecimalFixedFormatDefinition>>,
    decimal_parsers: Option<Vec<OrdinaryDecimalParseDefinition>>,
}

impl ContractCodecCache {
    pub(in super::super) fn emit(
        &mut self,
        b: &mut Builder,
        layouts: &OrdinaryCarrierProgram,
        vir: &ValidatedPracticalVir,
        codec: &BoundaryCodec,
        parse: bool,
    ) -> R<(String, Option<OrdinaryShape>)> {
        let (id, scale, rounding) = codec.ordinary_configuration();
        if matches!(id, "decimal.normalized" | "decimal.fixed") {
            return self.decimal(b, layouts, vir, codec, parse);
        }
        if scale.is_some() || rounding.is_some() {
            return Err(OrdinaryCarrierError::Shape);
        }
        match id {
            "binary32" | "binary64" | "guid.n" | "guid.d" => {
                if self.hex.is_none() {
                    self.hex = Some(hex_codecs::emit_codecs(layouts, b)?);
                }
                let d = self
                    .hex
                    .as_ref()
                    .unwrap()
                    .iter()
                    .find(|d| d.codec_id == id && d.value_type_id == codec.type_id())
                    .ok_or(OrdinaryCarrierError::Linkage)?;
                return Ok(if parse {
                    (
                        d.parse_definition.clone(),
                        Some(d.parse_result_shape.clone()),
                    )
                } else {
                    (d.format_definition.clone(), None)
                });
            }
            "date" | "time" => {
                if self.calendar.is_none() {
                    self.calendar = Some(calendar_codecs::emit_codecs(layouts, b)?);
                }
                let d = self
                    .calendar
                    .as_ref()
                    .unwrap()
                    .iter()
                    .find(|d| d.codec_id == id && d.value_type_id == codec.type_id())
                    .ok_or(OrdinaryCarrierError::Linkage)?;
                return Ok(if parse {
                    (
                        d.parse_definition.clone(),
                        Some(d.parse_result_shape.clone()),
                    )
                } else {
                    (d.format_definition.clone(), None)
                });
            }
            _ if id.starts_with("integer.")
                || matches!(id, "duration_ticks" | "unix_milliseconds") => {}
            _ => return Err(OrdinaryCarrierError::Shape),
        }
        if parse {
            if self.integer_parsers.is_none() {
                self.integer_parsers = Some(integer_parse::emit_parsers(layouts, b)?);
            }
            let d = self
                .integer_parsers
                .as_ref()
                .unwrap()
                .iter()
                .find(|d| d.codec_id == id && d.value_type_id == codec.type_id())
                .ok_or(OrdinaryCarrierError::Linkage)?;
            Ok((
                d.parse_definition.clone(),
                Some(d.parse_result_shape.clone()),
            ))
        } else {
            if self.integer_formats.is_none() {
                self.integer_formats = Some(integer_format::emit_formats(layouts, b)?);
            }
            let d = self
                .integer_formats
                .as_ref()
                .unwrap()
                .iter()
                .find(|d| d.codec_id == id && d.value_type_id == codec.type_id())
                .ok_or(OrdinaryCarrierError::Linkage)?;
            Ok((d.format_definition.clone(), None))
        }
    }

    fn decimal(
        &mut self,
        b: &mut Builder,
        layouts: &OrdinaryCarrierProgram,
        vir: &ValidatedPracticalVir,
        codec: &BoundaryCodec,
        parse: bool,
    ) -> R<(String, Option<OrdinaryShape>)> {
        let (id, scale, rounding) = codec.ordinary_configuration();
        let rounding = rounding.map(|r| format!("{r:?}"));
        if codec.type_id() != "mpk.csharp.value.decimal.v1"
            || (id == "decimal.normalized" && (scale.is_some() || rounding.is_some()))
            || (id == "decimal.fixed" && (!scale.is_some_and(|s| s <= 28) || rounding.is_none()))
        {
            return Err(OrdinaryCarrierError::Linkage);
        }
        let carrier = layouts
            .carriers()
            .iter()
            .find(|c| c.type_id == codec.type_id())
            .ok_or(OrdinaryCarrierError::Linkage)?;
        if carrier.depth != 9 || carrier.shape != primitive("decimal", vir)? {
            return Err(OrdinaryCarrierError::Shape);
        }
        if parse {
            if self.decimal_parsers.is_none() {
                self.decimal_parsers = Some(decimal_parse::emit_parser(b)?);
            }
            let d = self
                .decimal_parsers
                .as_ref()
                .unwrap()
                .iter()
                .find(|d| {
                    d.codec_id == id
                        && d.value_type_id == codec.type_id()
                        && d.scale == scale
                        && d.rounding == rounding
                })
                .ok_or(OrdinaryCarrierError::Linkage)?;
            return Ok((
                d.parse_definition.clone(),
                Some(d.parse_result_shape.clone()),
            ));
        }
        if self.decimal_digits.is_none() {
            self.decimal_digits = Some(decimal_format::emit_decimal_digit_helpers(b)?);
        }
        let digits = self.decimal_digits.as_ref().unwrap();
        if id == "decimal.normalized" {
            if self.decimal_normalized.is_none() {
                self.decimal_normalized =
                    Some(decimal_format::emit_with_digits(b, digits.clone())?);
            }
            Ok((
                self.decimal_normalized
                    .as_ref()
                    .unwrap()
                    .format_definition
                    .clone(),
                None,
            ))
        } else {
            if self.decimal_fixed.is_none() {
                self.decimal_fixed =
                    Some(decimal_fixed_format::emit_with_digits(b, digits.clone())?);
            }
            let d = self
                .decimal_fixed
                .as_ref()
                .unwrap()
                .iter()
                .find(|d| {
                    d.codec_id == id
                        && d.value_type_id == codec.type_id()
                        && Some(d.scale) == scale
                        && Some(&d.rounding) == rounding.as_ref()
                })
                .ok_or(OrdinaryCarrierError::Linkage)?;
            Ok((d.format_definition.clone(), None))
        }
    }

    /// Output length is a W03 condition. Input value domains remain separate
    /// obligations, as for the standalone codec relations.
    pub(in super::super) fn format_defined(
        b: &mut Builder,
        format: &str,
        value_depth: u32,
        name: &str,
    ) -> R<()> {
        let bounded = format!("{PREFIX}.ContractCodec.OutputBound");
        if !b.globals.contains_key(&bounded) {
            let mut c = Circuit::new(&[32]);
            let length = c.inputs[0].clone();
            let limit = (0..32)
                .map(|i| if i == 14 { T } else { F })
                .collect::<Word>();
            let over = c.lt(&limit, &length, false);
            let valid = c.not(over);
            let predicate = circuit(b, &format!("{bounded}.Circuit"), c, vec![valid])?;
            let source = b.var(5)?;
            let zero = truth(b, false)?;
            let mut args = vec![zero; 14];
            args.extend(b.selectors(5)?);
            let length = b.app(source, args)?;
            let length = b.wrap_selectors(5, length)?;
            let body = call(b, &predicate, vec![length])?;
            define(b, &bounded, &[19], 0, body)?;
        }
        let value = b.var(0)?;
        let text = call(b, format, vec![value])?;
        let body = call(b, &bounded, vec![text])?;
        define(b, name, &[value_depth], 0, body)
    }
}

#[cfg(test)]
mod tests {
    use super::super::super::test_eval::{bit, run, sparse_cube};
    use super::*;

    #[test]
    fn ordinary_codec_format_definedness_checks_full_length_word() {
        let mut b = Builder::new().unwrap();
        let body = b.var(0).unwrap();
        define(&mut b, "Test.IdentityText", &[19], 19, body).unwrap();
        ContractCodecCache::format_defined(&mut b, "Test.IdentityText", 19, "Test.FormatDefined")
            .unwrap();
        let c = decode_canonical_certificate(&b.finish().unwrap()).unwrap();
        for length in [0u32, 1, 16383, 16384, 16385, 32768, 1 << 31, u32::MAX] {
            let cells = (0..32)
                .filter(|i| length & (1 << i) != 0)
                .map(|i| i << 14)
                .collect();
            assert_eq!(
                bit(run(&c, "Test.FormatDefined", vec![sparse_cube(19, cells)])),
                length <= 16384,
                "length {length}"
            );
        }
    }
}
