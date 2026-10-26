// Copyright 2018 The Go Authors. All rights reserved.
// BSD license: ../GO-BUILDINFO-LICENSE; see the repository NOTICE.
//! Only the object mappings needed by debug/buildinfo are interpreted. These
//! facts do not certify loadability, relocations, machine code or signatures.
use super::{need, CompilerResult, C};

mod elf;

pub(super) fn number(b: &[u8], at: usize, width: usize, big: bool) -> CompilerResult<u64> {
    let b = b
        .get(at..at.checked_add(width).ok_or(C::BuildInfo)?)
        .ok_or(C::BuildInfo)?;
    let mut out = 0u64;
    if big {
        for &v in b {
            out = (out << 8) | v as u64;
        }
    } else {
        for &v in b.iter().rev() {
            out = (out << 8) | v as u64;
        }
    }
    Ok(out)
}
fn slice(b: &[u8], at: u64, size: u64) -> CompilerResult<&[u8]> {
    let end = at.checked_add(size).ok_or(C::BuildInfo)?;
    b.get(
        usize::try_from(at).map_err(|_| C::BuildInfo)?
            ..usize::try_from(end).map_err(|_| C::BuildInfo)?,
    )
    .ok_or(C::BuildInfo)
}
fn table(b: &[u8], at: u64, size: u64, count: u64) -> CompilerResult<&[u8]> {
    slice(b, at, size.checked_mul(count).ok_or(C::BuildInfo)?)
}
fn cstring(b: &[u8]) -> &[u8] {
    &b[..b.iter().position(|&v| v == 0).unwrap_or(b.len())]
}
struct Mapping {
    addr: u64,
    offset: u64,
    size: u64,
}
pub(super) struct Image<'a> {
    raw: &'a [u8],
    mappings: Vec<Mapping>,
    start: u64,
    size: u64,
}
impl<'a> Image<'a> {
    fn new(raw: &'a [u8]) -> Self {
        Self {
            raw,
            mappings: vec![],
            start: 0,
            size: 0,
        }
    }
    fn add(&mut self, addr: u64, offset: u64, size: u64) {
        self.mappings.push(Mapping { addr, offset, size });
    }
    fn remaining(&self, addr: u64) -> CompilerResult<&'a [u8]> {
        for m in &self.mappings {
            if let Some(delta) = addr.checked_sub(m.addr).filter(|&n| n < m.size) {
                let offset = m.offset.checked_add(delta).ok_or(C::BuildInfo)?;
                let start = usize::try_from(offset).map_err(|_| C::BuildInfo)?;
                let available = self.raw.get(start..).ok_or(C::BuildInfo)?;
                let count = (m.size - delta).min(available.len() as u64) as usize;
                return Ok(&available[..count]);
            }
        }
        Err(C::BuildInfo)
    }
    pub(super) fn exact(&self, addr: u64, size: u64) -> CompilerResult<&'a [u8]> {
        self.remaining(addr)?
            .get(..usize::try_from(size).map_err(|_| C::BuildInfo)?)
            .ok_or(C::BuildInfo)
    }
    pub(super) fn string(&self, addr: u64) -> CompilerResult<(Vec<u8>, u64)> {
        let b = self.remaining(addr)?;
        let mut length = 0u64;
        for (i, &v) in b.iter().take(10).enumerate() {
            need(i != 9 || v <= 1, C::BuildInfo)?;
            length |= (v as u64 & 127) << (7 * i);
            if v < 128 {
                let start = addr.checked_add(i as u64 + 1).ok_or(C::BuildInfo)?;
                return Ok((
                    self.exact(start, length)?.to_vec(),
                    start.checked_add(length).ok_or(C::BuildInfo)?,
                ));
            }
        }
        Err(C::BuildInfo)
    }
    pub(super) fn search_magic(&self) -> CompilerResult<u64> {
        let end = self.start.checked_add(self.size).ok_or(C::BuildInfo)?;
        let mut start = self.start.checked_add(15).ok_or(C::BuildInfo)? & !15;
        while start < end {
            let remaining = end - start;
            let chunk = remaining.min(1 << 20);
            let data = self.remaining(start)?;
            need(!data.is_empty(), C::BuildInfo)?;
            let mut data = &data[..data.len().min(chunk as usize)];
            while let Some(index) = data.windows(14).position(|s| s == b"\xff Go buildinf:") {
                need(remaining >= index as u64 + 32, C::BuildInfo)?;
                if index % 16 == 0 {
                    return start.checked_add(index as u64).ok_or(C::BuildInfo);
                }
                // Preserve Go1.25's chunk/slice-relative search, including its
                // returned address after a misaligned match, rather than scan
                // for an arbitrary magic anywhere in executable bytes.
                let next = (index + 15) & !15;
                data = data.get(next..).ok_or(C::BuildInfo)?;
            }
            start = start.checked_add(chunk).ok_or(C::BuildInfo)?;
        }
        Err(C::BuildInfo)
    }
    pub(super) fn parse(raw: &'a [u8]) -> CompilerResult<Self> {
        need(raw.len() >= 16, C::BuildInfo)?;
        match &raw[..4] {
            b"\x7fELF" => Self::elf(raw),
            b"\xfe\xed\xfa\xce" | b"\xce\xfa\xed\xfe" | b"\xfe\xed\xfa\xcf"
            | b"\xcf\xfa\xed\xfe" => Self::macho(raw),
            b"\xca\xfe\xba\xbe" => Self::fat(raw),
            _ if raw.starts_with(b"MZ") => Self::pe(raw),
            _ if raw.starts_with(b"\x01\xdf") || raw.starts_with(b"\x01\xf7") => Self::xcoff(raw),
            _ if matches!(number(raw, 0, 4, true)?, 0x1eb | 0x8a97 | 0x647) => Self::plan9(raw),
            _ => Err(C::BuildInfo),
        }
    }
    fn macho(raw: &'a [u8]) -> CompilerResult<Self> {
        let big = raw[0] == 0xfe;
        let wide = matches!(&raw[..4], b"\xfe\xed\xfa\xcf" | b"\xcf\xfa\xed\xfe");
        let mut image = Self::new(raw);
        let mut fallback = None;
        let mut section = None;
        let count = number(raw, 16, 4, big)?;
        let mut commands = slice(raw, if wide { 32 } else { 28 }, number(raw, 20, 4, big)?)?;
        for _ in 0..count {
            let cmd = number(commands, 0, 4, big)?;
            let size = number(commands, 4, 4, big)?;
            need(size >= 8, C::BuildInfo)?;
            let current = slice(commands, 0, size)?;
            commands = &commands[size as usize..];
            if !matches!(cmd, 1 | 0x19) {
                continue;
            }
            let w = if cmd == 0x19 { 8 } else { 4 };
            let head = if w == 8 { 72 } else { 56 };
            let n = |at, width| number(current, at, width, big);
            let name = cstring(current.get(8..24).ok_or(C::BuildInfo)?);
            let addr = n(24, w)?;
            let memsz = n(24 + w, w)?;
            let offset = n(24 + 2 * w, w)?;
            let filesz = n(24 + 3 * w, w)?;
            if name != b"__PAGEZERO" {
                image.add(addr, offset, filesz);
            }
            if fallback.is_none()
                && addr != 0
                && filesz != 0
                && n(24 + 4 * w, 4)? == 3
                && n(28 + 4 * w, 4)? == 3
            {
                fallback = Some((addr, memsz));
            }
            let nsect = n(32 + 4 * w, 4)?;
            for s in table(current, head, if w == 8 { 80 } else { 68 }, nsect)?
                .chunks_exact(if w == 8 { 80 } else { 68 })
            {
                if section.is_none() && cstring(&s[..16]) == b"__go_buildinfo" {
                    section = Some((number(s, 32, w, big)?, number(s, 32 + w, w, big)?));
                }
            }
        }
        (image.start, image.size) = section.or(fallback).unwrap_or_default();
        Ok(image)
    }
    fn fat(raw: &'a [u8]) -> CompilerResult<Self> {
        let count = number(raw, 4, 4, true)?;
        need(count != 0, C::BuildInfo)?;
        let mut first = None;
        let mut arches = std::collections::BTreeSet::new();
        let mut kind = None;
        for row in table(raw, 8, 20, count)?.chunks_exact(20) {
            need(arches.insert(number(row, 0, 8, true)?), C::BuildInfo)?;
            let part = slice(raw, number(row, 8, 4, true)?, number(row, 12, 4, true)?)?;
            need(
                part.len() >= 16
                    && matches!(
                        &part[..4],
                        b"\xfe\xed\xfa\xce"
                            | b"\xce\xfa\xed\xfe"
                            | b"\xfe\xed\xfa\xcf"
                            | b"\xcf\xfa\xed\xfe"
                    ),
                C::BuildInfo,
            )?;
            let current = number(part, 12, 4, part[0] == 0xfe)?;
            need(kind.is_none_or(|old| old == current), C::BuildInfo)?;
            kind = Some(current);
            let image = Self::macho(part)?;
            if first.is_none() {
                first = Some(image);
            }
        }
        first.ok_or(C::BuildInfo)
    }
    fn pe(raw: &'a [u8]) -> CompilerResult<Self> {
        let start = number(raw, 60, 4, false)?;
        need(slice(raw, start, 4)? == b"PE\0\0", C::BuildInfo)?;
        let head = slice(raw, start + 4, 20)?;
        let optional = number(head, 16, 2, false)?;
        let oh = slice(raw, start + 24, optional)?;
        let base = if optional == 0 {
            0
        } else {
            match number(oh, 0, 2, false)? {
                0x10b => number(oh, 28, 4, false)?,
                0x20b => number(oh, 24, 8, false)?,
                _ => return Err(C::BuildInfo),
            }
        };
        let mut image = Self::new(raw);
        let mut found = false;
        for s in table(raw, start + 24 + optional, 40, number(head, 2, 2, false)?)?.chunks_exact(40)
        {
            let va = number(s, 12, 4, false)?;
            let size = number(s, 16, 4, false)?;
            image.add(
                base.checked_add(va).ok_or(C::BuildInfo)?,
                number(s, 20, 4, false)?,
                size,
            );
            if !found && va != 0 && size != 0 && number(s, 36, 4, false)? & !0x600000 == 0xc0000040
            {
                image.start = base.checked_add(va).ok_or(C::BuildInfo)?;
                image.size = number(s, 8, 4, false)?;
                found = true;
            }
        }
        Ok(image)
    }
    fn xcoff(raw: &'a [u8]) -> CompilerResult<Self> {
        let wide = raw[1] == 0xf7;
        let word = if wide { 8 } else { 4 };
        let mut image = Self::new(raw);
        let mut found = false;
        let at = if wide { 24 } else { 20 } + number(raw, 16, 2, true)?;
        for s in table(
            raw,
            at,
            if wide { 72 } else { 40 },
            number(raw, 2, 2, true)?,
        )?
        .chunks_exact(if wide { 72 } else { 40 })
        {
            let addr = number(s, 8 + word, word, true)?;
            let size = number(s, 8 + 2 * word, word, true)?;
            image.add(addr, number(s, 8 + 3 * word, word, true)?, size);
            if !found && number(s, if wide { 64 } else { 36 }, 4, true)? == 0x40 {
                image.start = addr;
                image.size = size;
                found = true;
            }
        }
        Ok(image)
    }
    fn plan9(raw: &'a [u8]) -> CompilerResult<Self> {
        let mut image = Self::new(raw);
        let mut offset = if number(raw, 0, 4, true)? & 0x8000 != 0 {
            40u32
        } else {
            32
        };
        slice(raw, 0, offset as u64)?;
        for (index, field) in [4, 8, 16, 24, 28].into_iter().enumerate() {
            let size = number(raw, field, 4, true)? as u32;
            image.add(offset as u64, offset as u64, size as u64);
            if index == 1 {
                image.start = offset as u64;
                image.size = size as u64;
            }
            offset = offset.wrapping_add(size);
        }
        Ok(image)
    }
}
