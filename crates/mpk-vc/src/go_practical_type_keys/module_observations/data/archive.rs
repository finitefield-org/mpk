//! Go1.25 ZIP/Hash1 correspondence over the whole original encoding. No
//! extraction, filesystem callbacks or replacement member inventories.
use super::*;
use miniz_oxide::{
    inflate::stream::{inflate, InflateState},
    DataFormat, MZFlush, MZStatus,
};
mod fold;

const EXPANDED: u64 = 500 << 20;
const ENTRIES: usize = 131072;
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GoModuleArchiveMember {
    pub path: String,
    pub bytes: u64,
    pub raw_sha256: String,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GoModuleArchiveInventory {
    pub(super) source: GoTypeSourceBuffer,
    pub(super) sum: String,
    pub(super) members: BTreeMap<String, GoModuleArchiveMember>,
    pub(super) namespace_entries: usize,
}
impl GoModuleArchiveInventory {
    pub fn original_source(&self) -> &GoTypeSourceBuffer {
        &self.source
    }
    pub fn checksum(&self) -> &str {
        &self.sum
    }
    pub fn members(&self) -> &BTreeMap<String, GoModuleArchiveMember> {
        &self.members
    }
    pub fn namespace_entries(&self) -> usize {
        self.namespace_entries
    }
}
fn bytes(raw: &[u8], at: i64, n: usize) -> DataResult<&[u8]> {
    let at = usize::try_from(at).map_err(|_| D::Archive)?;
    raw.get(at..at.checked_add(n).ok_or(D::Archive)?)
        .ok_or(D::Archive)
}
fn u16(b: &[u8], at: usize) -> u16 {
    u16::from_le_bytes(b[at..at + 2].try_into().unwrap())
}
fn u32(b: &[u8], at: usize) -> u32 {
    u32::from_le_bytes(b[at..at + 4].try_into().unwrap())
}
fn u64(b: &[u8], at: usize) -> u64 {
    u64::from_le_bytes(b[at..at + 8].try_into().unwrap())
}
#[derive(Debug)]
enum HeaderError {
    End,
    Stop,
}
#[derive(Clone)]
struct Header {
    name: Vec<u8>,
    flags: u16,
    method: u16,
    crc: u32,
    compressed: u64,
    expanded: u64,
    offset: i64,
}
fn header(raw: &[u8], at: usize) -> std::result::Result<(Header, usize), HeaderError> {
    // Reader.init distinguishes an immediate EOF from a partial final header.
    if at == raw.len() {
        return Err(HeaderError::End);
    }
    let b = raw
        .get(at..at.saturating_add(46))
        .ok_or(HeaderError::Stop)?;
    if u32(b, 0) != 0x02014b50 {
        return Err(HeaderError::Stop);
    }
    let (n, e, c) = (
        u16(b, 28) as usize,
        u16(b, 30) as usize,
        u16(b, 32) as usize,
    );
    let begin = at + 46;
    if begin == raw.len() && n + e + c != 0 {
        return Err(HeaderError::End);
    }
    let rest = raw
        .get(begin..begin.saturating_add(n + e + c))
        .ok_or(HeaderError::Stop)?;
    let mut h = Header {
        name: rest[..n].to_vec(),
        flags: u16(b, 8),
        method: u16(b, 10),
        crc: u32(b, 16),
        compressed: u32(b, 20) as u64,
        expanded: u32(b, 24) as u64,
        offset: u32(b, 42) as i64,
    };
    let (mut ne, mut nc, mut no) = (
        h.expanded == u32::MAX as u64,
        h.compressed == u32::MAX as u64,
        h.offset == u32::MAX as i64,
    );
    let extra = &rest[n..n + e];
    let mut x = 0;
    while x + 4 <= extra.len() {
        let (tag, size) = (u16(extra, x), u16(extra, x + 2) as usize);
        x += 4;
        let Some(field) = extra.get(x..x + size) else {
            break;
        };
        x += size;
        if tag != 1 {
            continue;
        }
        let mut p = 0;
        for (needed, slot) in [(&mut ne, &mut h.expanded), (&mut nc, &mut h.compressed)] {
            if *needed {
                let field = field.get(p..p + 8).ok_or(HeaderError::Stop)?;
                *slot = u64(field, 0);
                *needed = false;
                p += 8;
            }
        }
        if no {
            let field = field.get(p..p + 8).ok_or(HeaderError::Stop)?;
            h.offset = u64(field, 0) as i64;
            no = false;
        }
    }
    if nc || no {
        return Err(HeaderError::Stop);
    }
    Ok((h, begin + n + e + c))
}
fn signature(block: &[u8]) -> Option<usize> {
    if block.len() < 22 {
        return None;
    }
    for at in (0..=block.len() - 22).rev() {
        if u32(block, at) == 0x06054b50 {
            return (at + 22 + u16(block, at + 20) as usize <= block.len()).then_some(at);
        }
    }
    None
}
fn headers(raw: &[u8], remaining: usize) -> DataResult<Vec<Header>> {
    let mut end = None;
    for length in [1024, 65 * 1024] {
        let start = raw.len().saturating_sub(length);
        if let Some(n) = signature(&raw[start..]) {
            end = Some(start + n);
            break;
        }
        if start == 0 {
            break;
        }
    }
    let mut end = end.ok_or(D::Archive)?;
    let b = bytes(raw, end as i64, 22)?;
    let (mut count, mut size, mut offset) =
        (u16(b, 10) as u64, u32(b, 12) as u64, u32(b, 16) as u64);
    // Match Go's actual ZIP64 trigger (including directorySize == 0xffff).
    if count == 0xffff || size == 0xffff || offset == 0xffffffff {
        if let Some(loc) = end.checked_sub(20).and_then(|n| raw.get(n..n + 20)) {
            if u32(loc, 0) == 0x07064b50 && u32(loc, 4) == 0 && u32(loc, 16) == 1 {
                let at = u64(loc, 8) as i64;
                let b = bytes(raw, at, 56)?;
                need(u32(b, 0) == 0x06064b50, D::Archive)?;
                count = u64(b, 32);
                size = u64(b, 40);
                offset = u64(b, 48);
                end = usize::try_from(at).map_err(|_| D::Archive)?;
            }
        }
    }
    need(
        size <= i64::MAX as u64 && offset <= i64::MAX as u64,
        D::Archive,
    )?;
    let mut base = (end as i64)
        .wrapping_sub(size as i64)
        .wrapping_sub(offset as i64);
    let directory = base.wrapping_add(offset as i64);
    need(directory >= 0 && directory < raw.len() as i64, D::Archive)?;
    if base > 0
        && usize::try_from(offset)
            .ok()
            .and_then(|n| header(raw, n).ok())
            .is_some()
    {
        base = 0;
    }
    let mut at = usize::try_from(base.wrapping_add(offset as i64)).map_err(|_| D::Archive)?;
    let mut all = vec![];
    loop {
        match header(raw, at) {
            Ok((mut h, next)) => {
                need(all.len() < remaining, D::Limit)?;
                h.offset = h.offset.wrapping_add(base);
                all.push(h);
                at = next;
            }
            Err(HeaderError::Stop) => break,
            Err(HeaderError::End) => return Err(D::Archive),
        }
    }
    need(all.len() as u16 == count as u16, D::Archive)?;
    Ok(all)
}
const fn crc_table() -> [u32; 256] {
    let mut a = [0; 256];
    let mut i = 0;
    while i < 256 {
        let mut r = i as u32;
        let mut bit = 0;
        while bit < 8 {
            r = (r >> 1) ^ if r & 1 != 0 { 0xedb88320 } else { 0 };
            bit += 1;
        }
        a[i] = r;
        i += 1;
    }
    a
}
const CRC: [u32; 256] = crc_table();
fn member_hash(raw: &[u8], h: &Header) -> DataResult<String> {
    let local = bytes(raw, h.offset, 30)?;
    need(u32(local, 0) == 0x04034b50, D::Archive)?;
    let start = h
        .offset
        .wrapping_add(30 + u16(local, 26) as i64 + u16(local, 28) as i64);
    if h.name.ends_with(b"/") {
        need(h.expanded == 0, D::Length)?;
        return Ok(format!("{:x}", Sha256::digest([])));
    }
    let start_index = usize::try_from(start).map_err(|_| D::Archive)?;
    // SectionReader reads to EOF for an oversized section and reads no bytes
    // for a negative int64 size, matching Go's original ZIP reader.
    let len = if h.compressed as i64 <= 0 {
        0
    } else {
        usize::try_from(h.compressed).unwrap_or(usize::MAX)
    };
    let available = raw.len().saturating_sub(start_index).min(len);
    let input = if available == 0 {
        &[][..]
    } else {
        raw.get(start_index..start_index + available)
            .ok_or(D::Archive)?
    };
    let (mut sha, mut crc, mut length) = (Sha256::new(), !0u32, 0u64);
    let mut consume = |part: &[u8]| -> DataResult<()> {
        length = length.checked_add(part.len() as u64).ok_or(D::Limit)?;
        need(length <= h.expanded, D::Length)?;
        sha.update(part);
        for &b in part {
            crc = (crc >> 8) ^ CRC[((crc ^ b as u32) & 255) as usize];
        }
        Ok(())
    };
    match h.method {
        0 => consume(input)?,
        8 => {
            let mut state = InflateState::new_boxed(DataFormat::Raw);
            let mut output = vec![0; 65536];
            let mut used = 0;
            loop {
                let step = inflate(&mut state, &input[used..], &mut output, MZFlush::None);
                used = used.checked_add(step.bytes_consumed).ok_or(D::Archive)?;
                need(used <= input.len(), D::Archive)?;
                consume(&output[..step.bytes_written])?;
                match step.status {
                    Ok(MZStatus::StreamEnd) => break,
                    Ok(MZStatus::Ok) => need(
                        step.bytes_consumed != 0 || step.bytes_written != 0,
                        D::Archive,
                    )?,
                    _ => return Err(D::Archive),
                }
            }
        }
        _ => return Err(D::Archive),
    }
    need(length == h.expanded, D::Length)?;
    let crc = !crc;
    if h.flags & 8 != 0 {
        let at = start.wrapping_add(h.compressed as i64);
        let first = bytes(raw, at, 4)?;
        let descriptor = bytes(
            raw,
            if u32(first, 0) == 0x08074b50 {
                at.wrapping_add(4)
            } else {
                at
            },
            12,
        )?;
        need(u32(descriptor, 0) == h.crc && crc == h.crc, D::Crc)?;
    } else {
        need(h.crc == 0 || crc == h.crc, D::Crc)?;
    }
    Ok(format!("{:x}", sha.finalize()))
}
fn folded(s: &str) -> String {
    s.chars()
        .map(|c| {
            fold::PAIRS
                .binary_search_by_key(&(c as u32), |&(n, _)| n)
                .ok()
                .and_then(|n| char::from_u32(fold::PAIRS[n].1))
                .unwrap_or(c)
        })
        .collect()
}
/// Inspect every original ZIP header/member. This establishes content and
/// namespace consistency only; it supplies no module/root/context authority.
pub fn inspect_go_module_archive(
    path: &str,
    version: &str,
    source: &GoTypeSourceBuffer,
    expected: &str,
    remaining: usize,
) -> DataResult<GoModuleArchiveInventory> {
    identity::check(path, version)?;
    need(source.object().kind == "auxiliary", D::Source)?;
    need(remaining <= ENTRIES, D::Limit)?;
    canonical_sum(expected)?;
    let all = headers(source.bytes(), remaining)?;
    let prefix = format!("{path}@{version}/");
    let mut names: BTreeMap<(usize, String), (String, bool, usize)> = BTreeMap::new();
    let (mut ordered, mut last, mut members, mut paths) =
        (vec![], BTreeMap::new(), BTreeMap::new(), BTreeSet::new());
    let mut expanded = 0u64;
    for (index, h) in all.iter().enumerate() {
        let name = std::str::from_utf8(&h.name).map_err(|_| D::Namespace)?;
        let relative = name.strip_prefix(&prefix).ok_or(D::Namespace)?;
        need(h.expanded < i64::MAX as u64, D::Length)?;
        need(!name.ends_with('/') || h.expanded == 0, D::Length)?;
        ordered.push(name);
        last.insert(name, index);
        if relative.is_empty() {
            continue;
        }
        let dir = relative.ends_with('/');
        let relative = relative.strip_suffix('/').unwrap_or(relative);
        need(identity::file_path(relative), D::Namespace)?;
        let parts = relative.split('/').collect::<Vec<_>>();
        let mut parent = 0;
        for (at, part) in parts.iter().enumerate() {
            let directory = dir || at + 1 < parts.len();
            let key = (parent, folded(part));
            if let Some((old, is_directory, id)) = names.get(&key) {
                need(
                    old == part && *is_directory == directory && directory,
                    D::Namespace,
                )?;
                parent = *id;
            } else {
                need(names.len() < remaining, D::Limit)?;
                parent = names.len() + 1;
                names.insert(key, (part.to_string(), directory, parent));
            }
        }
        paths.insert(relative);
        if dir {
            continue;
        }
        need(
            !relative
                .rsplit('/')
                .next()
                .unwrap()
                .eq_ignore_ascii_case("go.mod")
                || relative == "go.mod",
            D::Namespace,
        )?;
        expanded = expanded.checked_add(h.expanded).ok_or(D::Limit)?;
        need(expanded <= EXPANDED, D::Limit)?;
        need(
            !matches!(relative, "go.mod" | "LICENSE") || h.expanded <= 16777216,
            D::Limit,
        )?;
        members.insert(
            relative.to_string(),
            GoModuleArchiveMember {
                path: relative.to_string(),
                bytes: h.expanded,
                raw_sha256: String::new(),
            },
        );
    }
    let count = names
        .len()
        .checked_add(all.len())
        .and_then(|n| n.checked_sub(paths.len()))
        .ok_or(D::Limit)?;
    need(count <= remaining, D::Limit)?;
    let mut summary = Sha256::new();
    ordered.sort_unstable();
    let mut hashes = BTreeMap::new();
    for name in ordered {
        let hash = if let Some(hash) = hashes.get(name) {
            hash
        } else {
            let h = &all[*last.get(name).ok_or(D::Archive)?];
            let hash = member_hash(source.bytes(), h)?;
            hashes.insert(name, hash);
            hashes.get(name).unwrap()
        };
        summary.update(format!("{hash}  {name}\n").as_bytes());
        if let Some(member) = members.get_mut(name.strip_prefix(&prefix).unwrap()) {
            member.raw_sha256 = hash.clone();
        }
    }
    let sum = format!("h1:{}", BASE64.encode(summary.finalize()));
    need(sum == expected, D::Checksum)?;
    Ok(GoModuleArchiveInventory {
        source: source.clone(),
        sum,
        members,
        namespace_entries: count,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn whole_pinned_go_unicode_fold_agrees_for_every_scalar() {
        let path=std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../develop/migrations/go-practical/t02-w01/verification/module-data/native/native-originals/module-data-oracle/original-simple-fold.json");
        let native: serde_json::Value =
            serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap();
        assert_eq!(native["unicode_version"], "15.0.0");
        let pairs = native["pairs"]
            .as_array()
            .unwrap()
            .iter()
            .map(|r| (r[0].as_u64().unwrap() as u32, r[1].as_u64().unwrap() as u32))
            .collect::<Vec<_>>();
        assert_eq!(pairs, fold::PAIRS);
        let mut count = 0;
        for value in 0..=0x10ffff {
            let Some(c) = char::from_u32(value) else {
                continue;
            };
            let expected = pairs
                .binary_search_by_key(&value, |&(r, _)| r)
                .ok()
                .map_or(value, |at| pairs[at].1);
            assert_eq!(
                folded(&c.to_string()),
                char::from_u32(expected).unwrap().to_string()
            );
            count += 1;
        }
        assert_eq!(count, 1112064);
    }
}
