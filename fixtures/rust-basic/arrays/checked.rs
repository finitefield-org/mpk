pub const LENGTH: usize = 3;
pub const FIRST: u8 = 7;
pub const ENABLED: bool = true;
pub const UNUSED: u16 = 11;
pub const NEGATIVE: i8 = -7;
pub const LOCAL_LENGTH: usize = 2;

pub fn construct(middle: u8) -> [u8; LENGTH] {
    [FIRST, middle, 9]
}

pub fn read_constructed(middle: u8, index: usize) -> u8 {
    let values = [FIRST, middle, 9];
    values[index]
}

pub fn copy_array(values: [u8; LENGTH]) -> [u8; LENGTH] {
    values
}

pub fn nested() -> [[u8; 2]; 2] {
    [[1, 2], [3, 4]]
}

pub fn enabled() -> bool {
    ENABLED
}

pub fn negative() -> i8 {
    NEGATIVE
}

pub fn local_length() -> u8 {
    let values: [u8; LOCAL_LENGTH] = [4, 5];
    values[0]
}
