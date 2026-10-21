//! Function documentation groups from original token gaps. Comments are never
//! found inside a literal, and physical lines ignore //line redirection.
use super::*;

struct Comment {
    start: usize,
    end: usize,
    first_line: usize,
    last_line: usize,
    directive: bool,
}

pub(super) fn function_docs(parsed: &GoParsedSource) -> BTreeMap<usize, (usize, usize)> {
    let raw = parsed.source().bytes();
    let newlines: Vec<_> = raw
        .iter()
        .enumerate()
        .filter_map(|(i, &b)| (b == b'\n').then_some(i))
        .collect();
    let line = |offset| newlines.partition_point(|&at| at < offset) + 1;
    let mut comments = vec![];
    let mut previous_end = 0;
    for token in parsed.lexed().tokens().iter().filter(|t| !t.inserted()) {
        let mut at = previous_end;
        while at < token.start() {
            let start = at;
            let gap = &raw[at..token.start()];
            let block = gap.starts_with(b"/*");
            if gap.starts_with(b"//") {
                at += gap.iter().position(|&b| b == b'\n').unwrap_or(gap.len());
            } else if block {
                // The independent lexer already established a complete comment.
                at += gap.windows(2).position(|p| p == b"*/").unwrap() + 2;
            } else {
                at += 1;
                continue;
            }
            let text = &raw[start..at];
            let directive = !block
                && text
                    .iter()
                    .copied()
                    .filter(|&b| b != b'\r')
                    .take(5)
                    .eq(b"//go:".iter().copied());
            comments.push(Comment {
                start,
                // Go's ast.Comment.End uses its CR-stripped comment text length.
                end: start + text.iter().filter(|&&b| b != b'\r').count(),
                first_line: line(start),
                last_line: line(at - 1),
                directive,
            });
        }
        previous_end = token.end();
    }

    let mut result = BTreeMap::new();
    let mut next_comment = 0;
    let mut previous_line = 0;
    for token in parsed.lexed().tokens() {
        let begin = next_comment;
        while next_comment < comments.len() && comments[next_comment].start < token.start() {
            next_comment += 1;
        }
        let pending = &comments[begin..next_comment];
        if token.kind() == "func" && !pending.is_empty() {
            let mut first = 0;
            if pending[0].first_line == previous_line {
                // Comments on the preceding token's line form a trailing group.
                let mut end_line = previous_line;
                while first < pending.len() && pending[first].first_line <= end_line {
                    end_line = pending[first].last_line;
                    first += 1;
                }
            }
            if first < pending.len() {
                let mut group = first;
                let mut last_line = pending[first].last_line;
                for (i, comment) in pending.iter().enumerate().skip(first + 1) {
                    if comment.first_line > last_line + 1 {
                        group = i;
                    }
                    last_line = comment.last_line;
                }
                if last_line + 1 == line(token.start()) {
                    if let Some(comment) = pending[group..].iter().find(|c| c.directive) {
                        result.insert(token.start(), (comment.start, comment.end));
                    }
                }
            }
        }
        // Inserted semicolons also terminate comment groups, including a
        // semicolon whose physical newline lies inside a block comment.
        previous_line = line(token.start());
    }
    result
}
