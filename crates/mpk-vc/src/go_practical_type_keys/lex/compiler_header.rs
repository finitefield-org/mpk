//! Native ImportsOnly token demand, without a fabricated EOF source, an AST
//! node budget, or inspecting unused body tokens after the first lookahead.
#[derive(Clone, Copy)]
enum Phase {
    Package,
    Name,
    PackageSemi,
    BetweenImports,
    ImportStart,
    Spec(bool),
    Path(bool),
    SpecSemi(bool),
    Group,
    GroupSemi,
    Done,
}
pub(super) struct State(Phase);
impl State {
    pub(super) fn new() -> Self {
        Self(Phase::Package)
    }
    pub(super) fn done(&self) -> bool {
        matches!(self.0, Phase::Done)
    }
    pub(super) fn accept(&mut self, token: &str) {
        if token == "COMMENT" {
            return;
        }
        loop {
            self.0 = match self.0 {
                Phase::Package if token == "package" => Phase::Name,
                Phase::Name if token == "IDENT" => Phase::PackageSemi,
                Phase::PackageSemi | Phase::GroupSemi => {
                    if token == ";" {
                        Phase::BetweenImports
                    } else if matches!(token, ")" | "}") {
                        self.0 = Phase::BetweenImports;
                        continue;
                    } else {
                        Phase::Done
                    }
                }
                Phase::BetweenImports if token == "import" => Phase::ImportStart,
                Phase::ImportStart => {
                    if token == "(" {
                        Phase::Group
                    } else {
                        self.0 = Phase::Spec(false);
                        continue;
                    }
                }
                Phase::Group => {
                    if token == ")" {
                        Phase::GroupSemi
                    } else {
                        self.0 = Phase::Spec(true);
                        continue;
                    }
                }
                Phase::Spec(group) => match token {
                    "IDENT" | "." => Phase::Path(group),
                    "STRING" => Phase::SpecSemi(group),
                    _ => Phase::Done,
                },
                Phase::Path(group) if token == "STRING" => Phase::SpecSemi(group),
                Phase::SpecSemi(group) => {
                    let next = if group {
                        Phase::Group
                    } else {
                        Phase::BetweenImports
                    };
                    if token == ";" {
                        next
                    } else if matches!(token, ")" | "}") {
                        self.0 = next;
                        continue;
                    } else {
                        Phase::Done
                    }
                }
                _ => Phase::Done,
            };
            return;
        }
    }
}
