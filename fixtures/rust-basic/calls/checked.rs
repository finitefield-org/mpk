mod helpers {
    fn z_private_leaf(value: u8) -> u8 {
        value
    }

    pub fn a_public_helper(value: u8) -> u8 {
        z_private_leaf(value)
    }
}

pub fn selected(value: u8) -> u8 {
    helpers::a_public_helper(value)
}

pub fn call_through_local(value: u8) -> u8 {
    let result = helpers::a_public_helper(value);
    result
}

pub fn repeated_call(value: u8) -> u8 {
    helpers::a_public_helper(helpers::a_public_helper(value))
}

pub fn bool_identity(value: bool) -> bool {
    value
}

pub fn short_circuit(enabled: bool) -> bool {
    enabled && bool_identity(enabled)
}

pub fn usize_identity(value: usize) -> usize {
    value
}

pub fn usize_call(value: usize) -> usize {
    usize_identity(value)
}

mod dead_helpers {
    pub fn z_source_dead(value: u8) -> u8 {
        value
    }
}

pub fn a_source_dead(value: u8) -> u8 {
    if false {
        dead_helpers::z_source_dead(value)
    } else {
        value
    }
}
