pub fn and_i8(left: i8, right: i8) -> i8 { left & right }
pub fn and_i16(left: i16, right: i16) -> i16 { left & right }
pub fn and_i32(left: i32, right: i32) -> i32 { left & right }
pub fn and_i64(left: i64, right: i64) -> i64 { left & right }
pub fn and_u8(left: u8, right: u8) -> u8 { left & right }
pub fn and_u16(left: u16, right: u16) -> u16 { left & right }
pub fn and_u32(left: u32, right: u32) -> u32 { left & right }
pub fn and_u64(left: u64, right: u64) -> u64 { left & right }

pub fn or_i8(left: i8, right: i8) -> i8 { left | right }
pub fn or_i16(left: i16, right: i16) -> i16 { left | right }
pub fn or_i32(left: i32, right: i32) -> i32 { left | right }
pub fn or_i64(left: i64, right: i64) -> i64 { left | right }
pub fn or_u8(left: u8, right: u8) -> u8 { left | right }
pub fn or_u16(left: u16, right: u16) -> u16 { left | right }
pub fn or_u32(left: u32, right: u32) -> u32 { left | right }
pub fn or_u64(left: u64, right: u64) -> u64 { left | right }

pub fn xor_i8(left: i8, right: i8) -> i8 { left ^ right }
pub fn xor_i16(left: i16, right: i16) -> i16 { left ^ right }
pub fn xor_i32(left: i32, right: i32) -> i32 { left ^ right }
pub fn xor_i64(left: i64, right: i64) -> i64 { left ^ right }
pub fn xor_u8(left: u8, right: u8) -> u8 { left ^ right }
pub fn xor_u16(left: u16, right: u16) -> u16 { left ^ right }
pub fn xor_u32(left: u32, right: u32) -> u32 { left ^ right }
pub fn xor_u64(left: u64, right: u64) -> u64 { left ^ right }

pub fn not_i8(value: i8) -> i8 { !value }
pub fn not_i16(value: i16) -> i16 { !value }
pub fn not_i32(value: i32) -> i32 { !value }
pub fn not_i64(value: i64) -> i64 { !value }
pub fn not_u8(value: u8) -> u8 { !value }
pub fn not_u16(value: u16) -> u16 { !value }
pub fn not_u32(value: u32) -> u32 { !value }
pub fn not_u64(value: u64) -> u64 { !value }

pub fn shl_i8_u8(value: i8, count: u8) -> i8 { value << count }
pub fn shl_i16_u8(value: i16, count: u8) -> i16 { value << count }
pub fn shl_i32_u8(value: i32, count: u8) -> i32 { value << count }
pub fn shl_i64_u8(value: i64, count: u8) -> i64 { value << count }
pub fn shl_u8_u8(value: u8, count: u8) -> u8 { value << count }
pub fn shl_u16_u8(value: u16, count: u8) -> u16 { value << count }
pub fn shl_u32_u8(value: u32, count: u8) -> u32 { value << count }
pub fn shl_u64_u8(value: u64, count: u8) -> u64 { value << count }

pub fn shr_i8_u8(value: i8, count: u8) -> i8 { value >> count }
pub fn shr_i16_u8(value: i16, count: u8) -> i16 { value >> count }
pub fn shr_i32_u8(value: i32, count: u8) -> i32 { value >> count }
pub fn shr_i64_u8(value: i64, count: u8) -> i64 { value >> count }
pub fn shr_u8_u8(value: u8, count: u8) -> u8 { value >> count }
pub fn shr_u16_u8(value: u16, count: u8) -> u16 { value >> count }
pub fn shr_u32_u8(value: u32, count: u8) -> u32 { value >> count }
pub fn shr_u64_u8(value: u64, count: u8) -> u64 { value >> count }

pub fn shl_u32_i8(value: u32, count: i8) -> u32 { value << count }
pub fn shl_u32_i16(value: u32, count: i16) -> u32 { value << count }
pub fn shl_u32_i32(value: u32, count: i32) -> u32 { value << count }
pub fn shl_u32_i64(value: u32, count: i64) -> u32 { value << count }
pub fn shl_u32_u16(value: u32, count: u16) -> u32 { value << count }
pub fn shl_u32_u32(value: u32, count: u32) -> u32 { value << count }
pub fn shl_u32_u64(value: u32, count: u64) -> u32 { value << count }

pub fn shl_u8_i16(value: u8, count: i16) -> u8 { value << count }
pub fn shl_u8_u16(value: u8, count: u16) -> u8 { value << count }
