// Copyright 2018 The Go Authors. All rights reserved.
// BSD license: ../../GO-BUILDINFO-LICENSE; see the repository NOTICE.
//! Pinned debug/elf section-table and section-data observations.
use super::*;
use std::borrow::Cow;

mod compression;

struct Section {
    name: u64,
    kind: u64,
    flags: u64,
    addr: u64,
    offset: u64,
    file_size: u64,
    size: u64,
    compression: u64,
    header: u64,
}
fn signed(value: u64) -> CompilerResult<u64> {
    need(value <= i64::MAX as u64, C::BuildInfo)?;
    Ok(value)
}
fn available(raw: &[u8], offset: u64, size: u64) -> &[u8] {
    let Ok(offset) = usize::try_from(offset) else {
        return &[];
    };
    let Some(raw) = raw.get(offset..) else {
        return &[];
    };
    &raw[..raw.len().min(usize::try_from(size).unwrap_or(usize::MAX))]
}
impl Section {
    fn data<'a>(&self, raw: &'a [u8]) -> CompilerResult<Cow<'a, [u8]>> {
        need(self.kind != 8, C::BuildInfo)?;
        let data = available(raw, self.offset, self.file_size);
        if self.flags & 0x800 == 0 {
            return Ok(Cow::Borrowed(
                data.get(..usize::try_from(self.size).map_err(|_| C::BuildInfo)?)
                    .ok_or(C::BuildInfo)?,
            ));
        }
        need(self.flags & 2 == 0, C::BuildInfo)?;
        let data = data.get(self.header as usize..).ok_or(C::BuildInfo)?;
        Ok(Cow::Owned(compression::prefix(
            data,
            self.size,
            self.compression,
        )?))
    }
}
impl<'a> Image<'a> {
    pub(super) fn elf(raw: &'a [u8]) -> CompilerResult<Self> {
        let wide = match raw[4] {
            1 => false,
            2 => true,
            _ => return Err(C::BuildInfo),
        };
        let big = match raw[5] {
            1 => false,
            2 => true,
            _ => return Err(C::BuildInfo),
        };
        let n = |at, width| number(raw, at, width, big);
        slice(raw, 0, if wide { 64 } else { 52 })?;
        need(raw[6] == 1 && n(20, 4)? == 1, C::BuildInfo)?;
        let (word, po, so, psize, pcount, ssize, mut scount, mut names) = if wide {
            (
                8,
                n(32, 8)?,
                n(40, 8)?,
                n(54, 2)?,
                n(56, 2)?,
                n(58, 2)?,
                n(60, 2)?,
                n(62, 2)?,
            )
        } else {
            (
                4,
                n(28, 4)?,
                n(32, 4)?,
                n(42, 2)?,
                n(44, 2)?,
                n(46, 2)?,
                n(48, 2)?,
                n(50, 2)?,
            )
        };
        signed(po)?;
        signed(so)?;
        need(so != 0 || scount == 0, C::BuildInfo)?;
        need(scount == 0 || names < scount, C::BuildInfo)?;
        let mut image = Self::new(raw);
        let mut fallback = None;
        if pcount != 0 {
            need(psize >= if wide { 56 } else { 32 }, C::BuildInfo)?;
            for p in table(raw, po, psize, pcount)?.chunks_exact(psize as usize) {
                let n = |at, w| number(p, at, w, big);
                let (flags, offset, addr, filesz, memsz) = if wide {
                    (n(4, 4)?, n(8, 8)?, n(16, 8)?, n(32, 8)?, n(40, 8)?)
                } else {
                    (n(24, 4)?, n(4, 4)?, n(8, 4)?, n(16, 4)?, n(20, 4)?)
                };
                signed(offset)?;
                signed(filesz)?;
                image.add(addr, offset, filesz);
                if fallback.is_none() && n(0, 4)? == 1 && flags & 3 == 2 {
                    fallback = Some((addr, memsz));
                }
            }
        }
        if so != 0 && scount == 0 {
            let first = slice(raw, so, if wide { 64 } else { 40 })?;
            need(number(first, 4, 4, big)? == 0, C::BuildInfo)?;
            scount = signed(number(first, if wide { 32 } else { 20 }, word, big)?)?;
            need(scount >= 0xff00, C::BuildInfo)?;
            if names == 0xffff {
                names = number(first, if wide { 40 } else { 24 }, 4, big)?;
                need(names >= 0xff00, C::BuildInfo)?;
            }
        }
        let mut sections = Vec::new();
        if scount != 0 {
            need(ssize >= if wide { 64 } else { 40 }, C::BuildInfo)?;
            for s in table(raw, so, ssize, scount)?.chunks_exact(ssize as usize) {
                let n = |at, w| number(s, at, w, big);
                let flags = n(8, word)?;
                let offset = signed(n(if wide { 24 } else { 16 }, word)?)?;
                let file_size = signed(n(if wide { 32 } else { 20 }, word)?)?;
                let (compression, size, header) = if flags & 0x800 != 0 {
                    let size = if wide { 24 } else { 12 };
                    let header = available(raw, offset, file_size)
                        .get(..size)
                        .ok_or(C::BuildInfo)?;
                    (
                        number(header, 0, 4, big)?,
                        number(header, if wide { 8 } else { 4 }, word, big)?,
                        size as u64,
                    )
                } else {
                    (0, file_size, 0)
                };
                sections.push(Section {
                    name: n(0, 4)?,
                    kind: n(4, 4)?,
                    flags,
                    addr: n(if wide { 16 } else { 12 }, word)?,
                    offset,
                    file_size,
                    size,
                    compression,
                    header,
                });
            }
        }
        if names != 0 && !sections.is_empty() {
            let names = sections
                .get(usize::try_from(names).map_err(|_| C::BuildInfo)?)
                .ok_or(C::BuildInfo)?;
            need(names.kind == 3, C::BuildInfo)?;
            let names = names.data(raw)?;
            let mut found = None;
            // Go resolves every name before buildinfo selects its first section.
            // A later malformed name cannot be hidden behind an earlier match.
            for s in &sections {
                let name = names
                    .get(usize::try_from(s.name).map_err(|_| C::BuildInfo)?..)
                    .ok_or(C::BuildInfo)?;
                let end = name.iter().position(|&v| v == 0).ok_or(C::BuildInfo)?;
                if found.is_none() && &name[..end] == b".go.buildinfo" {
                    found = Some((s.addr, s.size));
                }
            }
            if let Some((start, size)) = found {
                image.start = start;
                image.size = size;
                return Ok(image);
            }
        }
        (image.start, image.size) = fallback.unwrap_or_default();
        Ok(image)
    }
}
