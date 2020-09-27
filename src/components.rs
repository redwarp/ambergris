use crate::game::{Ai, Position};
use tcod::colors::Color;

pub struct Body {
    pub x: i32,
    pub y: i32,
    pub blocking: bool,
}

impl Body {
    pub fn coordinates(&self) -> Position {
        (self.x, self.y)
    }
}

pub struct Renderable {
    pub char: char,
    pub color: Color,
}

pub struct Player;

pub struct Monster {
    pub ai: Ai,
}
