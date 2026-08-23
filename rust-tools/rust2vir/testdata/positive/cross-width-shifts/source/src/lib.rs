pub fn cross_width_shifts(
    value: u32,
    narrow_unsigned: u8,
    wide_signed: i64,
    narrow_signed: i8,
    wide_unsigned: u64,
) -> u32 {
    let left_narrow = value << narrow_unsigned;
    let right_wide = left_narrow >> wide_signed;
    let left_signed = right_wide << narrow_signed;
    left_signed >> wide_unsigned
}
