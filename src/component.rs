use tcod::colors::Color;

pub struct Body {
    pub x: i32,
    pub y: i32,
    pub blocking: bool,
}

impl Body {
    pub fn coordinates(&self) -> (i32, i32) {
        (self.x, self.y)
    }
}

pub struct Renderable {
    pub char: char,
    pub color: Color,
}

pub struct Player;
