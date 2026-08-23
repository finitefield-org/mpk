pub fn minimum_or_negate(value: i8, choose_minimum: bool) -> i8 {
    if choose_minimum { -128_i8 } else { -value }
}
