pub fn add_u8(left: u8, right: u8) -> u8 {
    left + right
}

pub fn div_i8(left: i8, right: i8) -> i8 {
    left / right
}

pub fn shl_u32_i64(value: u32, count: i64) -> u32 {
    value << count
}

pub fn read(values: [u8; 4], index: usize) -> u8 {
    values[index]
}

pub fn guarded_div(enabled: bool, left: i8, right: i8) -> i8 {
    if enabled {
        left / right
    } else {
        0
    }
}
