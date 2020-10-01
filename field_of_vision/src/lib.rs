mod bresenham;

use crate::bresenham::Bresenham;
use std::collections::HashSet;
use std::fmt::Debug;

/// Using https://sites.google.com/site/jicenospam/visibilitydetermination
/// See http://www.roguebasin.com/index.php?title=Comparative_study_of_field_of_view_algorithms_for_2D_grid_based_worlds
#[derive(Clone)]
pub struct FovMap {
    /// Vector to store the transparent tiles.
    transparent: Vec<bool>,
    /// Vector to store the computed field of vision.
    vision: Vec<bool>,
    /// The width of the map
    width: isize,
    /// The height of the map
    height: isize,
    /// The last position where the field of view was calculated. If never calculated, initialized to (-1, -1).
    last_origin: (isize, isize),
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
            last_origin: (-1, -1),
        }
    }

    /// Returns the dimension of the map.
    pub fn size(&self) -> (isize, isize) {
        (self.width, self.height)
    }

    /// Flag a tile as transparent or visible.
    pub fn set_transparent(&mut self, x: isize, y: isize, is_transparent: bool) {
        self.assert_in_bounds(x, y);
        let index = self.index(x, y);
        self.transparent[index] = is_transparent;
    }

    /// Check whether a tile is transparent.
    pub fn is_transparent(&self, x: isize, y: isize) -> bool {
        self.assert_in_bounds(x, y);
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
        let radius_square = radius.pow(2);
        self.assert_in_bounds(x, y);
        // Reset seen to false.
        for see in self.vision.iter_mut() {
            *see = false;
        }
        self.last_origin = (x, y);

        // Self position is always visible.
        let index = self.index(x, y);
        self.vision[index] = true;

        if radius < 1 {
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
            self.cast_ray_and_mark_visible((x, y), destination, radius_square);
        }

        self.post_process_vision(x + 1, y + 1, maxx, maxy, -1, -1);
        self.post_process_vision(minx, y + 1, x - 1, maxy, 1, -1);
        self.post_process_vision(minx, miny, x - 1, y - 1, 1, 1);
        self.post_process_vision(x + 1, miny, maxx, y - 1, -1, 1);
    }

    fn assert_in_bounds(&self, x: isize, y: isize) {
        if x < 0 || y < 0 || x >= self.width || y >= self.height {
            panic!(format!(
                "(x, y) should be between (0,0) and ({}, {}), got ({}, {})",
                self.width, self.height, x, y
            ));
        }
    }

    #[inline]
    fn index(&self, x: isize, y: isize) -> usize {
        (x + y * self.width) as usize
    }

    fn cast_ray_and_mark_visible(
        &mut self,
        origin: (isize, isize),
        destination: (isize, isize),
        radius_square: isize,
    ) {
        let (origin_x, origin_y) = origin;
        let bresenham = Bresenham::new(origin, destination).skip(1);
        for (x, y) in bresenham {
            let index = self.index(x, y);
            let distance = (x - origin_x).pow(2) + (y - origin_y).pow(2);
            // If we are within radius, or if we ignore radius whatsoever.
            if distance <= radius_square || radius_square == 0 {
                self.vision[index] = true;
            }

            if !self.transparent[index] {
                return;
            }
        }
    }

    fn post_process_vision(
        &mut self,
        minx: isize,
        miny: isize,
        maxx: isize,
        maxy: isize,
        dx: isize,
        dy: isize,
    ) {
        for x in minx..=maxx {
            for y in miny..=maxy {
                let index = self.index(x, y);
                if !self.transparent[index] && !self.vision[index] {
                    // We check for walls that are not in vision only.
                    let neighboor_x = x + dx;
                    let neighboor_y = y + dy;

                    let index_0 = self.index(neighboor_x, y);
                    let index_1 = self.index(x, neighboor_y);

                    if (self.transparent[index_0] && self.vision[index_0])
                        || (self.transparent[index_1] && self.vision[index_1])
                    {
                        self.vision[index] = true;
                    }
                }
            }
        }
    }
}

impl Debug for FovMap {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let last_origin_index = if self.last_origin.0 >= 0 && self.last_origin.1 >= 0 {
            Some(self.index(self.last_origin.0, self.last_origin.1))
        } else {
            None
        };

        let mut display_string = String::from("+");
        display_string.push_str("-".repeat(self.width as usize).as_str());
        display_string.push_str("+\n");
        for index in 0..self.vision.len() {
            if index % self.width as usize == 0 {
                display_string.push('|');
            }

            let is_last_origin = if let Some(last_origin_index) = last_origin_index {
                last_origin_index == index
            } else {
                false
            };
            let tile = match (is_last_origin, self.transparent[index], self.vision[index]) {
                (true, _, _) => '*',
                (_, true, true) => ' ',
                (_, false, true) => '□',
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
        fov.assert_in_bounds(-10, 15);
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
        fov.calculate_fov(3, 2, 10);

        println!("{:?}", fov);
    }
}

pub trait Map {
    fn dimensions(&self) -> (isize, isize);
    fn is_transparent(&self, x: isize, y: isize) -> bool;

    fn field_of_view(&mut self, x: isize, y: isize, radius: isize) -> Vec<(isize, isize)> {
        let radius_square = radius.pow(2);
        self.assert_in_bounds(x, y);

        let mut visibles: Vec<(isize, isize)> = Vec::with_capacity((4 * radius * radius) as usize);

        // let mut visibles = Vec::with_capacity((4 * radius * radius) as usize);

        // Self position is always visible.
        visibles.push((x, y));

        if radius < 1 {
            return visibles;
        }

        let (width, height) = self.dimensions();

        let minx = (x - radius).max(0);
        let miny = (y - radius).max(0);
        let maxx = (x + radius).min(width - 1);
        let maxy = (y + radius).min(height - 1);

        if maxx - minx == 0 || maxy - miny == 0 {
            // Well, no area to check.
            return visibles;
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
            visibles.append(&mut self.cast_ray((x, y), destination, radius_square));
        }

        visibles.sort();
        visibles.dedup();

        visibles.append(&mut self.post_process_vision(&visibles, x + 1, y + 1, maxx, maxy, -1, -1));
        visibles.append(&mut self.post_process_vision(&visibles, minx, y + 1, x - 1, maxy, 1, -1));
        visibles.append(&mut self.post_process_vision(&visibles, minx, miny, x - 1, y - 1, 1, 1));
        visibles.append(&mut self.post_process_vision(&visibles, x + 1, miny, maxx, y - 1, -1, 1));

        visibles
    }

    fn cast_ray(
        &self,
        origin: (isize, isize),
        destination: (isize, isize),
        radius_square: isize,
    ) -> Vec<(isize, isize)> {
        let (origin_x, origin_y) = origin;
        let bresenham = Bresenham::new(origin, destination).skip(1);
        let mut visibles = Vec::new();
        for (x, y) in bresenham {
            let distance = (x - origin_x).pow(2) + (y - origin_y).pow(2);
            // If we are within radius, or if we ignore radius whatsoever.
            if distance <= radius_square || radius_square == 0 {
                visibles.push((x, y));
            }

            if !self.is_transparent(x, y) {
                return visibles;
            }
        }
        visibles
    }

    fn assert_in_bounds(&self, x: isize, y: isize) {
        let (width, height) = self.dimensions();
        if self.is_bounded(x, y) {
            panic!(format!(
                "(x, y) should be between (0,0) and ({}, {}), got ({}, {})",
                width, height, x, y
            ));
        }
    }

    fn is_bounded(&self, x: isize, y: isize) -> bool {
        let (width, height) = self.dimensions();
        x < 0 || y < 0 || x >= width || y >= height
    }

    fn post_process_vision(
        &mut self,
        visibles: &Vec<(isize, isize)>,
        minx: isize,
        miny: isize,
        maxx: isize,
        maxy: isize,
        dx: isize,
        dy: isize,
    ) -> Vec<(isize, isize)> {
        let mut extras = Vec::new();
        for x in minx..=maxx {
            for y in miny..=maxy {
                if !self.is_transparent(x, y) && !visibles.contains(&(x, y)) {
                    // We check for walls that are not in vision only.
                    let neighboor_x = x + dx;
                    let neighboor_y = y + dy;

                    if (self.is_transparent(neighboor_x, y) && visibles.contains(&(neighboor_x, y)))
                        || (self.is_transparent(x, neighboor_y)
                            && visibles.contains(&(x, neighboor_y)))
                    {
                        extras.push((x, y));
                    }
                }
            }
        }
        extras
    }
}

pub struct SampleMap {
    /// Vector to store the transparent tiles.
    transparent: Vec<bool>,
    /// Vector to store the computed field of vision.
    vision: Vec<bool>,
    /// The width of the map
    width: isize,
    /// The height of the map
    height: isize,
    /// The last position where the field of view was calculated. If never calculated, initialized to (-1, -1).
    last_origin: (isize, isize),
}

impl Map for SampleMap {
    fn dimensions(&self) -> (isize, isize) {
        (self.width, self.height)
    }

    fn is_transparent(&self, x: isize, y: isize) -> bool {
        let index = (x + y * self.width) as usize;
        self.transparent[index]
    }
}

impl SampleMap {
    pub fn new(width: isize, height: isize) -> Self {
        if width <= 0 && height <= 0 {
            panic!(format!(
                "Width and height should be > 0, got ({},{})",
                width, height
            ));
        }
        SampleMap {
            transparent: vec![true; (width * height) as usize],
            vision: vec![false; (width * height) as usize],
            width,
            height,
            last_origin: (-1, -1),
        }
    }
    /// Flag a tile as transparent or visible.
    pub fn set_transparent(&mut self, x: isize, y: isize, is_transparent: bool) {
        self.transparent[(x + y * self.width) as usize] = is_transparent;
    }

    pub fn calculate_fov(&mut self, x: isize, y: isize, radius: isize) {
        for see in self.vision.iter_mut() {
            *see = false;
        }

        let visibles = self.field_of_view(x, y, radius);

        for (x, y) in visibles {
            self.vision[(x + y * self.width) as usize] = true
        }
        self.last_origin = (x, y);
    }
}
