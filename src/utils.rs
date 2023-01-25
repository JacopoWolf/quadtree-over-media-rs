use std::{cmp::Ordering, collections::HashMap, hash::Hash};

use image::Rgba;

/* data structures */

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct Vec2 {
    pub x: u32,
    pub y: u32,
}

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct QuadInfo {
    pub depth: u8,
    pub color: Option<Rgba<u8>>,
}

pub struct QuadStructure {
    pub quads: HashMap<Vec2, QuadInfo>,
    pub sizes: HashMap<u8, Vec2>,
}

/* implementations */

impl Vec2 {
    pub const ZERO: Vec2 = Vec2 { x: 0, y: 0 };
    /// returns the halfed Vec2 and the remainder for x and y
    pub fn half(&self) -> (Vec2, Vec2) {
        (
            Vec2 {
                x: self.x / 2,
                y: self.y / 2,
            },
            Vec2 {
                x: self.x & 0x1,
                y: self.y & 0x1,
            },
        )
    }
}
impl From<(u32, u32)> for Vec2 {
    fn from(src: (u32, u32)) -> Self {
        Vec2 { x: src.0, y: src.1 }
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
        write!(f, "({},{})", self.x, self.y)
    }
}
impl QuadInfo {
    pub fn new(d: u8) -> Self {
        Self {
            depth: d,
            color: None,
        }
    }
}
