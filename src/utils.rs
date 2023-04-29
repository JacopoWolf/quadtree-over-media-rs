/* Copyright 2023 Comparin Jacopo
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     https://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */
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
    /// position : quad info
    pub map: HashMap<Vec2, QuadInfo>,
    /// starts with the image size and then the halved sizes based on depth
    pub sizes: Vec<Vec2>,
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

#[cfg(test)]
mod tests {
    use super::*;
    use test_case::test_case;

    #[test_case(Vec2{x:10,y:10} => (Vec2{x:5,y:5}, Vec2::ZERO); "smpl-a")]
    #[test_case(Vec2{x:1024,y:1024} => (Vec2{x:512,y:512}, Vec2::ZERO); "smpl-b")]
    #[test_case(Vec2{x:7,y:6} => (Vec2{x:3,y:3}, Vec2 { x: 1, y: 0 }); "mod-a")]
    #[test_case(Vec2{x:7,y:13} => (Vec2{x:3,y:6}, Vec2 { x: 1, y: 1 }); "mod-b")]
    fn vec2_halves(v2in: Vec2) -> (Vec2, Vec2) {
        v2in.half()
    }

    #[test]
    fn vec2_formats() {
        assert_eq!(Vec2 { x: 104, y: 6 }.to_string(), "(104,6)")
    }
}
