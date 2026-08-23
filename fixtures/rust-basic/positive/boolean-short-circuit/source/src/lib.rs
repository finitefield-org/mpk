fn bool_identity(value: bool) -> bool {
    value
}

fn bool_negation(value: bool) -> bool {
    !value
}

fn guarded_and(enabled: bool, numerator: u32, denominator: u32) -> bool {
    enabled && ((numerator / denominator) > 0_u32)
}

fn guarded_or(enabled: bool, numerator: u32, denominator: u32) -> bool {
    enabled || ((numerator / denominator) > 0_u32)
}

pub fn boolean_short_circuit(enabled: bool, left: u32, right: u32) -> bool {
    let identity = bool_identity(enabled);
    let negated = bool_negation(enabled);
    if identity {
        guarded_and(identity, left, right)
    } else {
        guarded_or(negated, left, right)
    }
}
