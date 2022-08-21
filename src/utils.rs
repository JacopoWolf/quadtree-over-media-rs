use image::Rgba;

use crate::DEFAULT_COLOR;

#[derive(Clone)]
pub struct Vec2 {
    pub x: u32,
    pub y: u32,
}

impl Vec2 {
    pub fn new() -> Self {
        Self { x:0, y:0 }
    }
    pub fn smaller(&self, other: &Vec2) -> bool {
        self.x < other.x || self.y < other.y
    }
}
impl std::fmt::Display for Vec2 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{};{}]", self.x, self.y)
    }
}

pub(super) fn rgba_from_colorname(name: &str) -> Rgba<u8>{
    match name {
        "red" => Rgba([255,0,0,255]),
        "green" => Rgba([0,255,0,255]),
        "blue" => Rgba([0,0,255,255]),
        "purple" => DEFAULT_COLOR,
        "black" |
        &_ => Rgba([0,0,0,255])
    }
}