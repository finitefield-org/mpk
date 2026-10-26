// Copyright 2023 The Go Authors. All rights reserved.
// BSD license: ../../../GO-BUILDINFO-LICENSE; see the repository NOTICE.
//! The exact section-data prefix consumed by pinned Go1.25, without inventing
//! a decompressed-section budget or interpreting unread trailing data.
use super::*;
use miniz_oxide::{
    inflate::stream::{inflate, InflateState},
    DataFormat, MZFlush, MZStatus,
};
use mpk_go_zstd_blocks::mpk_go125_blocks::Go125Blocks;

mod hash;

pub(super) fn prefix(raw: &[u8], size: u64, kind: u64) -> CompilerResult<Vec<u8>> {
    signed(size)?;
    let size = usize::try_from(size).map_err(|_| C::BuildInfo)?;
    if size == 0 {
        return Ok(vec![]);
    }
    match kind {
        1 => zlib(raw, size),
        2 => zstd(raw, size),
        _ => Err(C::BuildInfo),
    }
}
fn append(out: &mut Vec<u8>, part: &[u8]) -> CompilerResult<()> {
    out.try_reserve(part.len()).map_err(|_| C::BuildInfo)?;
    out.extend_from_slice(part);
    Ok(())
}
fn zlib(raw: &[u8], size: usize) -> CompilerResult<Vec<u8>> {
    let header = number(raw, 0, 2, true)?;
    need(
        header >> 8 & 15 == 8 && header >> 12 <= 7 && header % 31 == 0,
        C::BuildInfo,
    )?;
    let start = if header & 0x20 != 0 {
        // zlib.NewReader permits an advertised dictionary only when its Adler
        // identity equals that of the actual empty dictionary.
        need(number(raw, 2, 4, true)? == 1, C::BuildInfo)?;
        6
    } else {
        2
    };
    let mut state = InflateState::new_boxed(DataFormat::Raw);
    let mut input = raw.get(start..).ok_or(C::BuildInfo)?;
    let mut out = Vec::new();
    let mut buffer = vec![0; 65536];
    while out.len() < size {
        let limit = buffer.len().min(size - out.len());
        let step = inflate(&mut state, input, &mut buffer[..limit], MZFlush::None);
        input = input.get(step.bytes_consumed..).ok_or(C::BuildInfo)?;
        append(&mut out, &buffer[..step.bytes_written])?;
        // io.ReadFull succeeds once n bytes arrive, even when the same read
        // also reports EOF/checksum failure. An unread suffix is not a gate.
        if out.len() == size {
            return Ok(out);
        }
        need(
            matches!(step.status, Ok(MZStatus::Ok))
                && (step.bytes_written != 0 || step.bytes_consumed != 0),
            C::BuildInfo,
        )?;
    }
    Ok(out)
}
fn take<'a>(input: &mut &'a [u8], size: usize) -> CompilerResult<&'a [u8]> {
    let part = input.get(..size).ok_or(C::BuildInfo)?;
    *input = &input[size..];
    Ok(part)
}
fn little(input: &mut &[u8], width: usize) -> CompilerResult<u64> {
    number(take(input, width)?, 0, width, false)
}
fn zstd(mut input: &[u8], size: usize) -> CompilerResult<Vec<u8>> {
    let mut out = Vec::new();
    while out.len() < size {
        let magic = little(&mut input, 4)?;
        if (0x184d2a50..=0x184d2a5f).contains(&magic) {
            let count = little(&mut input, 4)?;
            take(
                &mut input,
                usize::try_from(count).map_err(|_| C::BuildInfo)?,
            )?;
            continue;
        }
        need(magic == 0xfd2fb528, C::BuildInfo)?;
        let descriptor = little(&mut input, 1)?;
        need(descriptor & 8 == 0, C::BuildInfo)?;
        let single = descriptor & 0x20 != 0;
        let width = if descriptor >> 6 == 0 && !single {
            0
        } else {
            1 << (descriptor >> 6)
        };
        let mut window = if single {
            0
        } else {
            let d = little(&mut input, 1)?;
            let base = 1u64 << (10 + (d >> 3));
            base + base / 8 * (d & 7)
        };
        let dictionary = descriptor & 3;
        if dictionary != 0 {
            need(
                little(&mut input, 1 << (dictionary - 1))? == 0,
                C::BuildInfo,
            )?;
        }
        let mut remaining = if width == 0 {
            None
        } else {
            let n = little(&mut input, width)?;
            Some(n + if width == 2 { 256 } else { 0 })
        };
        if single {
            window = remaining.ok_or(C::BuildInfo)?;
        }
        // This is Go1.25's decoder history, not a source/certificate/resource
        // limit. Large advertised windows are clamped, never rejected by name.
        let mut blocks = Go125Blocks::new(window.min(8 << 20) as usize);
        let mut hash = hash::Hash::new();
        loop {
            let (block, last) = blocks.next(&mut input).map_err(|_| C::BuildInfo)?;
            if let Some(n) = &mut remaining {
                *n = n.checked_sub(block.len() as u64).ok_or(C::BuildInfo)?;
            }
            if descriptor & 4 != 0 {
                hash.update(&block);
            }
            if last {
                need(remaining.is_none_or(|n| n == 0), C::BuildInfo)?;
                if descriptor & 4 != 0 {
                    need(
                        little(&mut input, 4)? == u64::from(hash.digest() as u32),
                        C::BuildInfo,
                    )?;
                }
            }
            let count = block.len().min(size - out.len());
            append(&mut out, &block[..count])?;
            if out.len() == size {
                return Ok(out);
            }
            if last {
                break;
            }
        }
    }
    Ok(out)
}
