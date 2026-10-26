//! Byte literals and Go UTF-8 wildcard/class consumption, with no host glob.
use super::decode;

enum Atom {
    Byte(u8),
    Any,
    Star,
    Class(bool, Vec<(u32, u32)>),
}
pub(super) struct Pattern(Vec<Atom>);

fn escaped(raw: &[u8], at: &mut usize) -> Option<u32> {
    if matches!(raw.get(*at), None | Some(b'-' | b']')) {
        return None;
    }
    if raw[*at] == b'\\' {
        *at += 1;
    }
    if *at == raw.len() {
        return None;
    }
    let (r, n) = decode(&raw[*at..]);
    if r == 0xfffd && n == 1 {
        return None;
    }
    *at += n;
    (*at < raw.len()).then_some(r)
}
impl Pattern {
    pub(super) fn parse(raw: &[u8]) -> Option<Self> {
        let mut atoms = vec![];
        let mut at = 0;
        while at < raw.len() {
            let b = raw[at];
            at += 1;
            atoms.push(match b {
                b'*' => Atom::Star,
                b'?' => Atom::Any,
                b'\\' => {
                    let b = *raw.get(at)?;
                    at += 1;
                    Atom::Byte(b)
                }
                b'[' => {
                    let negated = raw.get(at) == Some(&b'^');
                    at += usize::from(negated);
                    let mut ranges = vec![];
                    loop {
                        if raw.get(at) == Some(&b']') && !ranges.is_empty() {
                            at += 1;
                            break;
                        }
                        let lo = escaped(raw, &mut at)?;
                        let hi = if raw[at] == b'-' {
                            at += 1;
                            escaped(raw, &mut at)?
                        } else {
                            lo
                        };
                        ranges.push((lo, hi));
                    }
                    Atom::Class(negated, ranges)
                }
                other => Atom::Byte(other),
            });
        }
        Some(Self(atoms))
    }
    fn chunk(atoms: &[Atom], name: &[u8]) -> Option<usize> {
        let mut at = 0;
        for atom in atoms {
            let b = *name.get(at)?;
            match atom {
                Atom::Byte(expected) if *expected == b => at += 1,
                Atom::Any if b != b'/' => at += decode(&name[at..]).1,
                Atom::Class(negated, ranges) => {
                    let (r, n) = decode(&name[at..]);
                    if ranges.iter().any(|(lo, hi)| *lo <= r && r <= *hi) == *negated {
                        return None;
                    }
                    at += n;
                }
                _ => return None,
            }
        }
        Some(at)
    }
    pub(super) fn matches(&self, mut name: &[u8]) -> bool {
        let mut at = 0;
        // Follow the pinned native chunk search, including byte-wise star
        // advancement and its UTF-8 boundary behavior. No recursive backtrack.
        'parts: while at < self.0.len() {
            let mut star = false;
            while at < self.0.len() && matches!(self.0[at], Atom::Star) {
                star = true;
                at += 1;
            }
            let start = at;
            while at < self.0.len() && !matches!(self.0[at], Atom::Star) {
                at += 1;
            }
            if star && start == at {
                return !name.contains(&b'/');
            }
            let chunk = &self.0[start..at];
            let more = at < self.0.len();
            if let Some(n) = Self::chunk(chunk, name) {
                if n == name.len() || more {
                    name = &name[n..];
                    continue;
                }
            }
            if star {
                let before_slash = name.iter().position(|b| *b == b'/').unwrap_or(name.len());
                for skipped in 1..=before_slash {
                    if let Some(n) = Self::chunk(chunk, &name[skipped..]) {
                        if more || skipped + n == name.len() {
                            name = &name[skipped + n..];
                            continue 'parts;
                        }
                    }
                }
            }
            return false;
        }
        name.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::Pattern;
    use serde_json::{json, Value};
    use std::path::{Path, PathBuf};
    fn unhex(raw: &str) -> Vec<u8> {
        raw.as_bytes()
            .chunks_exact(2)
            .map(|p| u8::from_str_radix(std::str::from_utf8(p).unwrap(), 16).unwrap())
            .collect()
    }
    #[test]
    fn every_actual_original_pinned_path_match_case() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../develop/migrations/go-practical/t02-w01/verification/embedded-selection/native/native-originals/glob/original-pinned-globs.json");
        let corpus: Value = serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap();
        let mut checked = vec![];
        for row in corpus["cases"].as_array().unwrap() {
            let pattern = unhex(row["pattern_hex"].as_str().unwrap());
            let name = unhex(row["name_hex"].as_str().unwrap());
            let parsed = Pattern::parse(&pattern);
            assert_eq!(
                parsed.is_none(),
                row["bad_pattern"].as_bool().unwrap(),
                "{row}"
            );
            if let Some(parsed) = parsed {
                assert_eq!(
                    parsed.matches(&name),
                    row["matched"].as_bool().unwrap(),
                    "{row}"
                );
            }
            checked.push(row.clone());
        }
        assert!(!checked.is_empty());
        if let Some(dir) = std::env::var_os("MPK_GO_EMBEDDED_OUTPUT") {
            use std::io::Write;
            let dir = PathBuf::from(dir);
            std::fs::create_dir_all(&dir).unwrap();
            std::fs::OpenOptions::new().create_new(true).write(true).open(dir.join("native-glob-comparison.json")).unwrap()
                .write_all(&serde_json::to_vec_pretty(&json!({"status":"passed","cases":checked,"whole_original_native_test_source_retained":true})).unwrap()).unwrap();
        }
    }
}
