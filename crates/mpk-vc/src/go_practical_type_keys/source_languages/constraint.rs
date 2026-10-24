//! Iterative dual-polarity minima, not SAT solving. The 1,000 operand/group
//! entries are the actual pinned Go1.25 build-expression parser bound.
use super::super::validate::unicode;
type Minimum = Option<i64>;
type Pair = (Minimum, Minimum);
#[derive(Clone, Copy)]
enum Op {
    And,
    Or,
    Group(bool),
}
fn apply<T>(values: &mut Vec<T>, op: Op, combine: &impl Fn(T, T, bool) -> T) -> Result<(), ()> {
    let b = values.pop().ok_or(())?;
    let a = values.pop().ok_or(())?;
    values.push(match op {
        Op::And => combine(a, b, true),
        Op::Or => combine(a, b, false),
        Op::Group(_) => return Err(()),
    });
    Ok(())
}
struct Tokens<'a> {
    text: &'a str,
    at: usize,
}
impl<'a> Tokens<'a> {
    fn next(&mut self) -> Result<&'a str, ()> {
        while self.at < self.text.len() && matches!(self.text.as_bytes()[self.at], b' ' | b'\t') {
            self.at += 1;
        }
        let start = self.at;
        let Some(c) = self.text[self.at..].chars().next() else {
            return Ok("");
        };
        if matches!(c, '(' | ')' | '!') {
            self.at += 1;
            return Ok(&self.text[start..self.at]);
        }
        if c == '&' || c == '|' {
            self.at += 1;
            if self.text.as_bytes().get(self.at) != Some(&(c as u8)) {
                return Err(());
            }
            self.at += 1;
            return Ok(&self.text[start..self.at]);
        }
        while let Some(c) = self.text[self.at..].chars().next() {
            if !unicode::letter(c) && !unicode::digit(c) && c != '_' && c != '.' {
                break;
            }
            self.at += c.len_utf8();
        }
        if self.at == start {
            return Err(());
        }
        Ok(&self.text[start..self.at])
    }
}
fn parse<T>(
    text: &str,
    tag: impl Fn(&str) -> T,
    negate: impl Fn(T) -> T,
    combine: impl Fn(T, T, bool) -> T,
) -> Result<T, ()> {
    let mut tokens = Tokens { text, at: 0 };
    let mut values: Vec<T> = vec![];
    let mut ops = vec![];
    let mut operand = true;
    let mut size = 0;
    loop {
        let mut token = tokens.next()?;
        if operand {
            size += 1;
            if size > 1000 {
                return Err(());
            }
            let negative = token == "!";
            if negative {
                token = tokens.next()?;
            }
            if token == "(" {
                ops.push(Op::Group(negative));
                continue;
            }
            if matches!(token, "" | "!" | ")" | "&&" | "||") {
                return Err(());
            }
            let value = tag(token);
            values.push(if negative { negate(value) } else { value });
            operand = false;
            continue;
        }
        if token == "&&" || token == "||" {
            while let Some(op) = ops.last().copied() {
                if matches!(op, Op::Group(_)) || token == "&&" && matches!(op, Op::Or) {
                    break;
                }
                ops.pop();
                apply(&mut values, op, &combine)?;
            }
            ops.push(if token == "&&" { Op::And } else { Op::Or });
            operand = true;
            continue;
        }
        if token == ")" {
            let mut group = None;
            while let Some(op) = ops.pop() {
                if let Op::Group(negative) = op {
                    group = Some(negative);
                    break;
                }
                apply(&mut values, op, &combine)?;
            }
            let negative = group.ok_or(())?;
            if negative {
                let value = values.pop().ok_or(())?;
                values.push(negate(value));
            }
            continue;
        }
        if !token.is_empty() {
            return Err(());
        }
        while let Some(op) = ops.pop() {
            apply(&mut values, op, &combine)?;
        }
        return if values.len() == 1 {
            Ok(values.pop().unwrap())
        } else {
            Err(())
        };
    }
}

pub(super) fn minimum(text: &str) -> Result<Minimum, ()> {
    let pair = parse(
        text,
        |token| {
            let n = if token == "go1" {
                Some(0)
            } else {
                token
                    .split_once("go1.")
                    .and_then(|(_, v)| v.parse::<i64>().ok())
                    .filter(|n| *n >= 0)
            };
            (n, None)
        },
        |p: Pair| (p.1, p.0),
        |a: Pair, b: Pair, and| {
            if and {
                (a.0.max(b.0), a.1.min(b.1))
            } else {
                (a.0.min(b.0), a.1.max(b.1))
            }
        },
    )?;
    Ok(pair.0)
}

pub(crate) fn evaluate(text: &str, tag: impl Fn(&str) -> bool) -> Result<bool, ()> {
    parse(
        text,
        tag,
        |value| !value,
        |a, b, and| if and { a && b } else { a || b },
    )
}
