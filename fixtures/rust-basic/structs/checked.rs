pub struct Point {
    pub x: u8,
    pub y: u8,
}

pub struct Envelope {
    pub point: Point,
    pub enabled: bool,
}

pub struct Unused {
    pub ignored: u16,
}

pub fn construct(x: u8, y: u8) -> Point {
    Point { y, x }
}

pub fn read_x(point: Point) -> u8 {
    point.x
}

pub fn constructed_x(x: u8, y: u8) -> u8 {
    Point { x, y }.x
}

pub fn move_whole(point: Point) -> Point {
    let moved = point;
    moved
}

pub fn nested(x: u8, y: u8, enabled: bool) -> Envelope {
    Envelope {
        point: Point { x, y },
        enabled,
    }
}
