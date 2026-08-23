fn signed_max(left: i8, right: i8) -> i8 {
    if left > right { left } else { right }
}

pub fn max_values(
    unsigned_left: u8,
    unsigned_right: u8,
    signed_left: i8,
    signed_right: i8,
) -> u8 {
    let signed = signed_max(signed_left, signed_right);
    if signed >= 0_i8 {
        if unsigned_left > unsigned_right {
            unsigned_left
        } else {
            unsigned_right
        }
    } else if unsigned_left > unsigned_right {
        unsigned_left
    } else {
        unsigned_right
    }
}
