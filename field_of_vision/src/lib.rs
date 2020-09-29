use crate::bresenham::Bresenham;
use std::fmt::{Debug, Display};
mod bresenham;

/// Using https://sites.google.com/site/jicenospam/visibilitydetermination
/// See http://www.roguebasin.com/index.php?title=Comparative_study_of_field_of_view_algorithms_for_2D_grid_based_worlds
pub struct FovMap {
    /// Vector to store the transparent tiles.
    transparent: Vec<bool>,
    /// Vector to store the computed field of vision.
    vision: Vec<bool>,
    /// The width of the map
    width: isize,
    /// The height of the map
    height: isize,
}

impl FovMap {
    pub fn new(width: isize, height: isize) -> Self {
        if width <= 0 && height <= 0 {
            panic!(format!(
                "Width and height should be > 0, got ({},{})",
                width, height
            ));
        }
        FovMap {
            transparent: vec![true; (width * height) as usize],
            vision: vec![false; (width * height) as usize],
            width,
            height,
        }
    }

    /// Returns the dimension of the map.
    pub fn size(&self) -> (isize, isize) {
        (self.width, self.height)
    }

    /// Flag a tile as transparent or visible.
    pub fn set_transparent(&mut self, x: isize, y: isize, is_transparent: bool) {
        let index = self.index(x, y);
        self.transparent[index] = is_transparent;
    }

    /// Check whether a tile is transparent.
    pub fn is_transparent(&self, x: isize, y: isize) -> bool {
        let index = self.index(x, y);
        self.transparent[index]
    }

    /// Recaculate the visible tiles, based on a location, and a radius.
    ///
    /// # Arguments
    ///
    /// * `x` - The x coordinate where the field of vision will be centered.
    /// * `y` - The x coordinate where the field of vision will be centered.
    /// * `radius` - How far the eye can see, in squares.
    pub fn calculate_fov(&mut self, x: isize, y: isize, radius: isize) {
        self.check_in_bounds(x, y);
        for see in self.vision.iter_mut() {
            *see = false;
        }

        if radius < 1 {
            let index = self.index(x, y);
            self.vision[index] = true;
            return;
        }

        let minx = (x - radius).max(0);
        let miny = (y - radius).max(0);
        let maxx = (x + radius).min(self.width - 1);
        let maxy = (y + radius).min(self.height - 1);

        if maxx - minx == 0 || maxy - miny == 0 {
            // Well, no area to check.
            return;
        }

        let mut extremities = Vec::new();
        for x in minx..maxx + 1 {
            extremities.push((x, miny));
            extremities.push((x, maxy));
        }
        for y in miny + 1..maxy {
            extremities.push((minx, y));
            extremities.push((maxx, y));
        }

        for destination in extremities {
            self.cast_ray((x, y), destination, radius);
        }
    }

    fn check_in_bounds(&self, x: isize, y: isize) {
        if x < 0 || y < 0 || x >= self.width || y >= self.height {
            panic!(format!(
                "(x, y) should be between (0,0) and ({}, {}), got ({}, {})",
                self.width, self.height, x, y
            ));
        }
    }

    fn index(&self, x: isize, y: isize) -> usize {
        self.check_in_bounds(x, y);

        (x + y * self.width) as usize
    }

    fn cast_ray(&mut self, origin: (isize, isize), destination: (isize, isize), radius: isize) {
        let (origin_x, origin_y) = origin;
        println!("Casting ray from {:?} to {:?}", origin, destination);
        let bresenham = Bresenham::new(origin, destination);
        for (x, y) in bresenham {
            let distance = ((x - origin_x).pow(2) as f32 + (y - origin_y).pow(2) as f32).sqrt();
            if distance <= radius as f32 {
                let index = self.index(x, y);
                self.vision[index] = true;
            }

            if !self.is_transparent(x, y) {
                return;
            }
        }
    }
}

impl Debug for FovMap {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut display_string = String::from("+");
        display_string.push_str("-".repeat(self.width as usize).as_str());
        display_string.push_str("+\n");
        for index in 0..self.vision.len() {
            if index % self.width as usize == 0 {
                display_string.push('|');
            }

            let tile = match (self.transparent[index], self.vision[index]) {
                (true, true) => ' ',
                (false, true) => '□',
                _ => '?',
            };
            display_string.push(tile);
            if index > 0 && (index + 1) % self.width as usize == 0 {
                display_string.push_str("|\n");
            }
        }
        display_string.truncate(display_string.len() - 1);
        display_string.push('\n');
        display_string.push('+');
        display_string.push_str("-".repeat(self.width as usize).as_str());
        display_string.push('+');

        write!(f, "{}", display_string)
    }
}

#[cfg(test)]
mod test {
    use crate::*;

    #[test]
    fn size() {
        let fov = FovMap::new(20, 40);

        assert_eq!((20, 40), fov.size());
    }

    #[test]
    fn new_fov_map_all_is_transparent() {
        let fov = FovMap::new(10, 20);
        for is_transparent in fov.transparent.iter() {
            assert!(is_transparent);
        }
    }

    #[test]
    fn set_tranparent() {
        let mut fov = FovMap::new(10, 20);
        fov.set_transparent(5, 5, false);

        assert!(!fov.is_transparent(5, 5));
    }

    #[test]
    #[should_panic(expected = "Width and height should be > 0, got (0,0)")]
    fn newfov_size_zero_panic() {
        FovMap::new(0, 0);
    }

    #[test]
    #[should_panic(expected = "(x, y) should be between (0,0) and (10, 10), got (-10, 15)")]
    fn check_in_bounds_out_of_bounds_panic() {
        let fov = FovMap::new(10, 10);
        fov.check_in_bounds(-10, 15);
    }

    #[test]
    fn fov() {
        let mut fov = FovMap::new(10, 10);
        for x in 1..10 {
            fov.set_transparent(x, 3, false);
        }
        for y in 0..10 {
            fov.set_transparent(9, y, false);
        }
        fov.calculate_fov(0, 5, 10);

        println!("{:?}", fov);
    }
}
