/// Using https://sites.google.com/site/jicenospam/visibilitydetermination
/// see http://www.roguebasin.com/index.php?title=Comparative_study_of_field_of_view_algorithms_for_2D_grid_based_worlds
pub struct FovMap {
    /// Vector to store the transparent tiles.
    transparent: Vec<bool>,
    /// Vector to store the computed field of vision.
    vision: Vec<bool>,
    /// The width of the map
    width: i32,
    /// The height of the map
    height: i32,
}

impl FovMap {
    pub fn new(width: i32, height: i32) -> Self {
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
    pub fn size(&self) -> (i32, i32) {
        (self.width, self.height)
    }

    /// Flag a tile as transparent or visible.
    pub fn set_transparent(&mut self, x: i32, y: i32, is_transparent: bool) {
        let index = self.index(x, y);
        self.transparent[index] = is_transparent;
    }

    /// Check whether a tile is transparent.
    pub fn is_transparent(&self, x: i32, y: i32) -> bool {
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
    pub fn calculate_fov(&mut self, x: i32, y: i32, radius: i32) {
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

        dbg!(&extremities);
        dbg!(&extremities.len());
    }

    fn check_in_bounds(&self, x: i32, y: i32) {
        if x < 0 || y < 0 || x >= self.width || y >= self.height {
            panic!(format!(
                "(x, y) should be between (0,0) and ({}, {}), got ({}, {})",
                self.width, self.height, x, y
            ));
        }
    }

    fn index(&self, x: i32, y: i32) -> usize {
        self.check_in_bounds(x, y);

        (x + y * self.width) as usize
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
        let mut fov = FovMap::new(3, 3);
        fov.calculate_fov(1, 1, 1);
    }
}
