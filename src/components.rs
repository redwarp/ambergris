use crate::game::{Ai, Position};
use tcod::colors::Color;

pub struct Body {
    pub name: String,
    pub x: i32,
    pub y: i32,
    pub blocking: bool,
    pub char: char,
    pub color: Color,
}

impl Body {
    pub fn coordinates(&self) -> Position {
        (self.x, self.y)
    }

    pub fn distance_to(&self, position: Position) -> f32 {
        ((self.x - position.0).pow(2) as f32 + (self.y - position.1).pow(2) as f32).sqrt()
    }
}

pub struct Player;

pub struct Monster {
    pub ai: Ai,
}
