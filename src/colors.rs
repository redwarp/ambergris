pub const BLACK: Color = Color::from_rgb(0x000000);
pub const DARKER_GREEN: Color = Color::new(255, 0, 127, 0);
pub const YELLOW: Color = Color::new(255, 255, 255, 0);
pub const DESATURATED_GREEN: Color = Color::new(255, 63, 127, 63);
pub const DARK_RED: Color = Color::new(255, 191, 0, 0);

#[derive(Copy, Clone, PartialEq, Debug)]
pub struct Color {
    pub a: u8,
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Color {
    pub const fn new(a: u8, r: u8, g: u8, b: u8) -> Self {
        Color { a, r, g, b }
    }

    pub const fn from_argb(argb: u32) -> Self {
        let a = (argb >> 24) as u8;
        let r = (argb >> 16 & 0xff) as u8;
        let g = (argb >> 8 & 0xff) as u8;
        let b = (argb & 0xff) as u8;
        Color { a, r, g, b }
    }

    pub const fn from_rgb(rgb: u32) -> Self {
        let a = 255;
        let r = (rgb >> 16 & 0xff) as u8;
        let g = (rgb >> 8 & 0xff) as u8;
        let b = (rgb & 0xff) as u8;
        Color { a, r, g, b }
    }
}

#[cfg(test)]
mod tests {
    use crate::colors::Color;

    #[test]
    fn color_from_argb() {
        let color = Color::from_argb(0x33123456);

        assert_eq!(
            color,
            Color {
                a: 0x33,
                r: 0x12,
                g: 0x34,
                b: 0x56
            }
        )
    }

    #[test]
    fn color_from_rgb() {
        let color = Color::from_rgb(0x123456);

        assert_eq!(
            color,
            Color {
                a: 0xff,
                r: 0x12,
                g: 0x34,
                b: 0x56
            }
        )
    }
}
