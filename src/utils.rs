
#[derive(Clone)]
pub struct Vec2 {
    pub x: u32,
    pub y: u32,
}

impl Vec2 {
    pub fn new() -> Self {
        Self { x: 0, y: 0 }
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
