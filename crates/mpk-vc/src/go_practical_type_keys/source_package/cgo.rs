//! Validate original import-C documentation under the fixed cgo-disabled target.
//! No compiler flags are executed and no C/program semantics are admitted.
use super::*;

struct Comment {
    start: usize,
    first: usize,
    last: usize,
    text: String,
}

fn text(raw: &[u8]) -> String {
    let block = raw.starts_with(b"/*");
    let mut normalized = vec![];
    for (at, &byte) in raw.iter().enumerate() {
        if byte != b'\r'
            || block
                && normalized.len() > 2
                && normalized.last() == Some(&b'*')
                && raw.get(at + 1) == Some(&b'/')
        {
            normalized.push(byte);
        }
    }
    let s = std::str::from_utf8(&normalized).unwrap();
    let s = if block {
        &s[2..s.len() - 2]
    } else {
        let s = &s[2..];
        if let Some(s) = s.strip_prefix(' ') {
            s
        } else if directive(s) {
            return String::new();
        } else {
            s
        }
    };
    s.split('\n')
        .map(|l| l.trim_end_matches([' ', '\t', '\n', '\r']))
        .collect::<Vec<_>>()
        .join("\n")
}
fn directive(s: &str) -> bool {
    if s.starts_with("line ") || s.starts_with("extern ") || s.starts_with("export ") {
        return true;
    }
    let Some(at) = s.find(':') else {
        return false;
    };
    let b = s.as_bytes();
    at > 0
        && at + 1 < b.len()
        && b[..at]
            .iter()
            .chain(&b[at + 1..at + 2])
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit())
}

// Reconstruct native physical lead-comment groups from independently lexed
// token gaps. Literal contents and //line remapping never supply documentation.
fn lead_docs(parsed: &GoParsedSource) -> BTreeMap<usize, String> {
    let raw = parsed.source().bytes();
    let newlines = raw
        .iter()
        .enumerate()
        .filter_map(|(at, &b)| (b == b'\n').then_some(at))
        .collect::<Vec<_>>();
    let line = |offset| newlines.partition_point(|&at| at < offset) + 1;
    let mut comments = vec![];
    let mut previous = 0;
    for token in parsed.lexed().tokens().iter().filter(|t| !t.inserted()) {
        let mut at = previous;
        while at < token.start() {
            let start = at;
            let gap = &raw[at..token.start()];
            if gap.starts_with(b"//") {
                at += gap.iter().position(|&b| b == b'\n').unwrap_or(gap.len());
            } else if gap.starts_with(b"/*") {
                at += gap.windows(2).position(|s| s == b"*/").unwrap() + 2;
            } else {
                at += 1;
                continue;
            }
            comments.push(Comment {
                start,
                first: line(start),
                last: line(at - 1),
                text: text(&raw[start..at]),
            });
        }
        previous = token.end();
    }
    let mut docs = BTreeMap::new();
    let mut next = 0;
    let mut previous_line = 0;
    for token in parsed.lexed().tokens() {
        let begin = next;
        while next < comments.len() && comments[next].start < token.start() {
            next += 1;
        }
        let pending = &comments[begin..next];
        let mut first = 0;
        if !pending.is_empty() && pending[0].first == previous_line {
            let mut end = previous_line;
            while first < pending.len() && pending[first].first <= end {
                end = pending[first].last;
                first += 1;
            }
        }
        if first < pending.len() {
            let mut group = first;
            let mut last = pending[first].last;
            for (i, comment) in pending.iter().enumerate().skip(first + 1) {
                if comment.first > last + 1 {
                    group = i;
                }
                last = comment.last;
            }
            if last + 1 == line(token.start()) {
                docs.insert(
                    token.start(),
                    pending[group..]
                        .iter()
                        .map(|c| c.text.as_str())
                        .collect::<Vec<_>>()
                        .join("\n"),
                );
            }
        }
        previous_line = line(token.start());
    }
    docs
}
pub(super) fn import_docs(parsed: &GoParsedSource) -> Vec<String> {
    let docs = lead_docs(parsed);
    let mut selected = vec![];
    for decl in parsed
        .nodes()
        .iter()
        .filter(|n| n.kind() == "GenDecl" && source_signatures::attr(n, "Tok") == Some("import"))
    {
        let specs = source_signatures::list(decl, "Specs");
        for &id in specs {
            let spec = &parsed.nodes()[id];
            let Some(&path) = source_signatures::list(spec, "Path").first() else {
                continue;
            };
            if source_scopes::unquote(parsed.spelling(path).unwrap()).unwrap() != b"C" {
                continue;
            }
            let spec_doc = if source_signatures::attr(decl, "Grouped") == Some("true") {
                docs.get(&spec.start())
            } else {
                None
            };
            if let Some(doc) = spec_doc.or_else(|| {
                (specs.len() == 1)
                    .then(|| docs.get(&decl.start()))
                    .flatten()
            }) {
                selected.push(doc.clone());
            }
        }
    }
    selected
}

fn split_quoted(s: &str) -> std::result::Result<Vec<String>, ()> {
    let mut args = vec![];
    let mut word = String::new();
    let mut escaped = false;
    let mut quoted = false;
    let mut quote = None;
    for c in s.chars() {
        if escaped {
            escaped = false;
        } else if c == '\\' {
            escaped = true;
            continue;
        } else if let Some(q) = quote {
            if c == q {
                quote = None;
                continue;
            }
        } else if c == '"' || c == '\'' {
            quoted = true;
            quote = Some(c);
            continue;
        } else if c.is_whitespace() {
            if quoted || !word.is_empty() {
                quoted = false;
                args.push(std::mem::take(&mut word));
            }
            continue;
        }
        word.push(c);
    }
    if quoted || !word.is_empty() {
        args.push(word);
    }
    if escaped || quote.is_some() {
        Err(())
    } else {
        Ok(args)
    }
}
fn safe(s: &str) -> bool {
    const SAFE: &[u8] =
        b"+-.,/0123456789=ABCDEFGHIJKLMNOPQRSTUVWXYZ_abcdefghijklmnopqrstuvwxyz:$@%! ~^";
    !s.is_empty() && s.bytes().all(|b| b >= 128 || SAFE.contains(&b))
}
fn argument(s: &str) -> bool {
    let chunks = s.split("${SRCDIR}").collect::<Vec<_>>();
    if chunks.len() == 1 {
        safe(s)
    } else {
        chunks.iter().all(|s| s.is_empty() || safe(s))
    }
}
pub(super) fn validate(docs: &[String]) -> std::result::Result<(), ()> {
    for doc in docs {
        for original in doc.split('\n') {
            let line = original.trim();
            let Some(rest) = line.strip_prefix("#cgo") else {
                continue;
            };
            if !rest.starts_with([' ', '\t']) {
                continue;
            }
            let fields = line.split_whitespace().collect::<Vec<_>>();
            if fields.len() == 3 && matches!(fields[1], "nocallback" | "noescape") {
                continue;
            }
            let (head, args) = rest.trim().split_once(':').ok_or(())?;
            let fields = head.split_whitespace().collect::<Vec<_>>();
            let Some((verb, conditions)) = fields.split_last() else {
                return Err(());
            };
            if !conditions.is_empty() && !conditions.iter().any(|c| source_build::cgo_condition(c))
            {
                continue;
            }
            let args = split_quoted(args)?;
            if args.iter().any(|s| !argument(s)) {
                return Err(());
            }
            if !matches!(
                *verb,
                "CFLAGS" | "CPPFLAGS" | "CXXFLAGS" | "FFLAGS" | "LDFLAGS" | "pkg-config"
            ) {
                return Err(());
            }
        }
    }
    Ok(())
}
