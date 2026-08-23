pub struct Point {
    pub x: u8,
    pub y: u8,
}

pub fn construct_move_read(x: u8, y: u8) -> u8 {
    let sum = x + y;
    let point = Point { x: sum, y };
    let moved = point;
    moved.x
}
