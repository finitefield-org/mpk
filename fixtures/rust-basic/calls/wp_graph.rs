fn a_leaf(value: u8) -> u8 {
    value
}

fn b_left(value: u8) -> u8 {
    a_leaf(value)
}

fn c_right(value: u8) -> u8 {
    a_leaf(value)
}

pub fn z_diamond(value: u8, enabled: bool) -> u8 {
    if enabled {
        b_left(value)
    } else {
        c_right(value)
    }
}

pub fn z_repeated(value: u8) -> u8 {
    value / c_right(c_right(value))
}
