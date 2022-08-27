use std::{hash::Hash, cmp::Ordering};

use image::Rgba;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Vec2 {
    pub x: u32,
    pub y: u32,
}

#[derive(Clone)]
pub struct QuadInfo{
    pub depth: u8,
    pub color: Option<Rgba<u8>>
}


impl Vec2 {
    pub fn is_odd (&self) -> bool {
        self.x&0xFF_FE > 1 || self.y&0xFF_FE > 1
    }
}
impl PartialOrd for Vec2 {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match self.x.partial_cmp(&other.x) {
            Some(Ordering::Equal) => {}
            ord => return ord,
        }
        self.y.partial_cmp(&other.y)
    }
}
impl std::fmt::Display for Vec2 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{},{}]", self.x, self.y)
    }
}
impl QuadInfo {
    pub fn new( d: u8 ) -> Self{
        Self { depth: d, color: None }
    }
}
