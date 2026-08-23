pub fn read(values: [u8; 4], index: usize) -> u8 {
    values[index]
}

pub fn zero(values: [u8; 4]) -> u8 {
    values[0]
}

pub fn last(values: [u8; 4]) -> u8 {
    values[3]
}

pub fn length(values: [u8; 4], index: usize) -> u8 {
    values[index]
}

pub fn independent(
    left: [u8; 4],
    right: [u8; 4],
    left_index: usize,
    right_index: usize,
) -> u8 {
    left[left_index] ^ right[right_index]
}
