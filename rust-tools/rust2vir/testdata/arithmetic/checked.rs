pub fn add_i8(left: i8, right: i8) -> i8 { left + right }
pub fn add_i16(left: i16, right: i16) -> i16 { left + right }
pub fn add_i32(left: i32, right: i32) -> i32 { left + right }
pub fn add_i64(left: i64, right: i64) -> i64 { left + right }
pub fn add_u8(left: u8, right: u8) -> u8 { left + right }
pub fn add_u16(left: u16, right: u16) -> u16 { left + right }
pub fn add_u32(left: u32, right: u32) -> u32 { left + right }
pub fn add_u64(left: u64, right: u64) -> u64 { left + right }

pub fn sub_i8(left: i8, right: i8) -> i8 { left - right }
pub fn sub_i16(left: i16, right: i16) -> i16 { left - right }
pub fn sub_i32(left: i32, right: i32) -> i32 { left - right }
pub fn sub_i64(left: i64, right: i64) -> i64 { left - right }
pub fn sub_u8(left: u8, right: u8) -> u8 { left - right }
pub fn sub_u16(left: u16, right: u16) -> u16 { left - right }
pub fn sub_u32(left: u32, right: u32) -> u32 { left - right }
pub fn sub_u64(left: u64, right: u64) -> u64 { left - right }

pub fn mul_i8(left: i8, right: i8) -> i8 { left * right }
pub fn mul_i16(left: i16, right: i16) -> i16 { left * right }
pub fn mul_i32(left: i32, right: i32) -> i32 { left * right }
pub fn mul_i64(left: i64, right: i64) -> i64 { left * right }
pub fn mul_u8(left: u8, right: u8) -> u8 { left * right }
pub fn mul_u16(left: u16, right: u16) -> u16 { left * right }
pub fn mul_u32(left: u32, right: u32) -> u32 { left * right }
pub fn mul_u64(left: u64, right: u64) -> u64 { left * right }

pub fn neg_i8(value: i8) -> i8 { -value }
pub fn neg_i16(value: i16) -> i16 { -value }
pub fn neg_i32(value: i32) -> i32 { -value }
pub fn neg_i64(value: i64) -> i64 { -value }

pub fn min_i8() -> i8 { -128_i8 }
pub fn min_i16() -> i16 { -32768_i16 }
pub fn min_i32() -> i32 { -2147483648_i32 }
pub fn min_i64() -> i64 { -9223372036854775808_i64 }
pub fn above_min_i8() -> i8 { -127_i8 }

pub fn add_below_i8(value: i8) -> i8 { value + -1_i8 }
pub fn add_at_i8(value: i8) -> i8 { value + 0_i8 }
pub fn add_above_i8(value: i8) -> i8 { value + 1_i8 }
