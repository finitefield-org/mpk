pub fn div_i8(left: i8, right: i8) -> i8 { left / right }
pub fn div_i16(left: i16, right: i16) -> i16 { left / right }
pub fn div_i32(left: i32, right: i32) -> i32 { left / right }
pub fn div_i64(left: i64, right: i64) -> i64 { left / right }
pub fn div_u8(left: u8, right: u8) -> u8 { left / right }
pub fn div_u16(left: u16, right: u16) -> u16 { left / right }
pub fn div_u32(left: u32, right: u32) -> u32 { left / right }
pub fn div_u64(left: u64, right: u64) -> u64 { left / right }

pub fn rem_i8(left: i8, right: i8) -> i8 { left % right }
pub fn rem_i16(left: i16, right: i16) -> i16 { left % right }
pub fn rem_i32(left: i32, right: i32) -> i32 { left % right }
pub fn rem_i64(left: i64, right: i64) -> i64 { left % right }
pub fn rem_u8(left: u8, right: u8) -> u8 { left % right }
pub fn rem_u16(left: u16, right: u16) -> u16 { left % right }
pub fn rem_u32(left: u32, right: u32) -> u32 { left % right }
pub fn rem_u64(left: u64, right: u64) -> u64 { left % right }

pub fn min_div_i8(right: i8) -> i8 { -128_i8 / right }
pub fn min_rem_i8(right: i8) -> i8 { -128_i8 % right }
pub fn div_neg_one_i8(left: i8) -> i8 { left / -1_i8 }
pub fn rem_neg_one_i8(left: i8) -> i8 { left % -1_i8 }
