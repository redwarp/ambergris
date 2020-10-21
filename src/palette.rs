use crate::colors::Color;

pub const WINDOW_BACKGROUND: Color = Color::from_argb(0xaa000000);
pub const SELECTED: Color = Color::from_argb(0x66ff0000);
pub const OVERLAY: Color = Color::from_argb(0x3300ff00);
pub const HEALTH: Color = Color::new(255, 191, 0, 0);
pub const DARK_WALL: Color = Color::new(255, 0, 0, 100);
pub const LIGHT_WALL: Color = Color {
    a: 255,
    r: 130,
    g: 110,
    b: 50,
};
pub const DARK_GROUND: Color = Color {
    a: 255,
    r: 50,
    g: 50,
    b: 150,
};
pub const LIGHT_GROUND: Color = Color {
    a: 255,
    r: 200,
    g: 180,
    b: 50,
};
