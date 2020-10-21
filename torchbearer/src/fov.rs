use crate::bresenham::Bresenham;

pub trait Map {
    fn dimensions(&self) -> (i32, i32);
    fn is_opaque(&self, x: i32, y: i32) -> bool;
}

fn is_bounded<M: Map>(map: &M, x: i32, y: i32) -> bool {
    let (width, height) = map.dimensions();
    x < 0 || y < 0 || x >= width || y >= height
}

fn assert_in_bounds<M: Map>(map: &M, x: i32, y: i32) {
    let (width, height) = map.dimensions();
    if is_bounded(map, x, y) {
        panic!(format!(
            "(x, y) should be between (0,0) and ({}, {}), got ({}, {})",
            width, height, x, y
        ));
    }
}

/// Using https://sites.google.com/site/jicenospam/visibilitydetermination
/// See http://www.roguebasin.com/index.php?title=Comparative_study_of_field_of_view_algorithms_for_2D_grid_based_worlds
pub fn field_of_view<T: Map>(
    map: &T,
    x: i32,
    y: i32,
    radius: i32,
    include_walls: bool,
) -> Vec<(i32, i32)> {
    let radius_square = radius.pow(2);
    assert_in_bounds(map, x, y);

    if radius < 1 {
        return vec![(x, y)];
    }

    let (width, height) = map.dimensions();

    let minx = (x - radius).max(0);
    let miny = (y - radius).max(0);
    let maxx = (x + radius).min(width - 1);
    let maxy = (y + radius).min(height - 1);

    if maxx - minx == 0 || maxy - miny == 0 {
        // Well, no area to check.
        return vec![];
    }

    let (sub_width, sub_height) = (maxx - minx + 1, maxy - miny + 1);
    let (offset_x, offset_y) = (minx, miny);
    let sub_origin = (x - offset_x, y - offset_y);

    let mut visibles = vec![false; (sub_width * sub_height) as usize];
    // Set origin as visible.
    visibles[(x - offset_x + (y - offset_y) * sub_width) as usize] = true;

    for x in minx..maxx + 1 {
        cast_ray(
            map,
            &mut visibles,
            sub_width,
            sub_origin,
            (x - offset_x, miny - offset_y),
            radius_square,
            offset_x,
            offset_y,
        );
        cast_ray(
            map,
            &mut visibles,
            sub_width,
            sub_origin,
            (x - offset_x, maxy - offset_y),
            radius_square,
            offset_x,
            offset_y,
        );
    }
    for y in miny + 1..maxy {
        cast_ray(
            map,
            &mut visibles,
            sub_width,
            sub_origin,
            (minx - offset_x, y - offset_y),
            radius_square,
            offset_x,
            offset_y,
        );
        cast_ray(
            map,
            &mut visibles,
            sub_width,
            sub_origin,
            (maxx - offset_x, y - offset_y),
            radius_square,
            offset_x,
            offset_y,
        );
    }

    // SE
    post_process_vision(
        map,
        &mut visibles,
        sub_width,
        x - offset_x + 1,
        y - offset_y + 1,
        maxx - offset_x,
        maxy - offset_y,
        -1,
        -1,
        offset_x,
        offset_y,
    );

    // SW
    post_process_vision(
        map,
        &mut visibles,
        sub_width,
        minx - offset_x,
        y - offset_y + 1,
        x - offset_x - 1,
        maxy - offset_y,
        1,
        -1,
        offset_x,
        offset_y,
    );

    // NW
    post_process_vision(
        map,
        &mut visibles,
        sub_width,
        minx - offset_x,
        miny - offset_y,
        x - offset_x - 1,
        y - offset_y - 1,
        1,
        1,
        offset_x,
        offset_y,
    );

    // NE
    post_process_vision(
        map,
        &mut visibles,
        sub_width,
        x - offset_x + 1,
        miny - offset_y,
        maxx - offset_x,
        y - offset_y - 1,
        -1,
        1,
        offset_x,
        offset_y,
    );

    let visibles: Vec<(i32, i32)> = visibles
        .iter()
        .enumerate()
        .filter(|&(_index, visible)| *visible)
        .map(|(index, _)| {
            (
                index as i32 % sub_width + offset_x,
                index as i32 / sub_width + offset_y,
            )
        })
        .collect();

    if !include_walls {
        visibles
            .into_iter()
            .filter(|&(x, y)| !map.is_opaque(x, y))
            .collect()
    } else {
        visibles
    }
}

fn cast_ray<T: Map>(
    map: &T,
    visibles: &mut Vec<bool>,
    width: i32,
    origin: (i32, i32),
    destination: (i32, i32),
    radius_square: i32,
    offset_x: i32,
    offset_y: i32,
) {
    let (origin_x, origin_y) = origin;
    let bresenham = Bresenham::new(origin, destination).skip(1);
    for (x, y) in bresenham {
        let distance = (x - origin_x).pow(2) + (y - origin_y).pow(2);
        // If we are within radius, or if we ignore radius whatsoever.
        if distance <= radius_square || radius_square == 0 {
            visibles[(x + y * width) as usize] = true;
        }

        if map.is_opaque(x + offset_x, y + offset_y) {
            return;
        }
    }
}

fn post_process_vision<T: Map>(
    map: &T,
    visibles: &mut Vec<bool>,
    width: i32,
    minx: i32,
    miny: i32,
    maxx: i32,
    maxy: i32,
    dx: i32,
    dy: i32,
    offset_x: i32,
    offset_y: i32,
) {
    for x in minx..=maxx {
        for y in miny..=maxy {
            let index = (x + y * width) as usize;
            let opaque = map.is_opaque(x + offset_x, y + offset_y);
            if opaque && !visibles[index] {
                // We check for walls that are not in vision only.
                let neighboor_x = x + dx;
                let neighboor_y = y + dy;

                let index_0 = (neighboor_x + y * width) as usize;
                let index_1 = (x + neighboor_y * width) as usize;

                if (!map.is_opaque(neighboor_x + offset_x, y + offset_y) && visibles[index_0])
                    || (!map.is_opaque(x + offset_x, neighboor_y + offset_y) && visibles[index_1])
                {
                    visibles[index] = true;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use rand::{prelude::StdRng, Rng, SeedableRng};
    use std::fmt::Debug;

    use super::{field_of_view, Map};
    const WIDTH: i32 = 45;
    const HEIGHT: i32 = 45;
    const POSITION_X: i32 = 22;
    const POSITION_Y: i32 = 22;
    const RADIUS: i32 = 24;
    const RANDOM_WALLS: i32 = 10;

    pub struct SampleMap {
        /// Vector to store the transparent tiles.
        transparent: Vec<bool>,
        /// Vector to store the computed field of vision.
        vision: Vec<bool>,
        /// The width of the map
        width: i32,
        /// The height of the map
        height: i32,
        /// The last position where the field of view was calculated. If never calculated, initialized to (-1, -1).
        last_origin: (i32, i32),
    }

    impl Map for SampleMap {
        fn dimensions(&self) -> (i32, i32) {
            (self.width, self.height)
        }

        fn is_opaque(&self, x: i32, y: i32) -> bool {
            let index = (x + y * self.width) as usize;
            !self.transparent[index]
        }
    }

    impl SampleMap {
        pub fn new(width: i32, height: i32) -> Self {
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
        pub fn set_transparent(&mut self, x: i32, y: i32, is_transparent: bool) {
            self.transparent[(x + y * self.width) as usize] = is_transparent;
        }

        pub fn calculate_fov(&mut self, x: i32, y: i32, radius: i32) {
            for see in self.vision.iter_mut() {
                *see = false;
            }

            let visibles = field_of_view(self, x, y, radius, true);

            for (x, y) in visibles {
                self.vision[(x + y * self.width) as usize] = true
            }
            self.last_origin = (x, y);
        }
    }

    impl Debug for SampleMap {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            let (width, _height) = self.dimensions();

            let last_origin_index = if self.last_origin.0 >= 0 && self.last_origin.1 >= 0 {
                Some((self.last_origin.0 + self.last_origin.1 * width) as usize)
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
    #[test]
    fn fov_with_sample_map() {
        let mut fov = SampleMap::new(10, 10);
        for x in 1..10 {
            fov.set_transparent(x, 3, false);
        }
        for y in 0..10 {
            fov.set_transparent(9, y, false);
        }
        fov.calculate_fov(3, 2, 10);

        println!("{:?}", fov);
    }

    #[test]
    fn fov_to_vector() {
        let mut fov = SampleMap::new(WIDTH, HEIGHT);

        fov.calculate_fov(POSITION_X, POSITION_Y, RADIUS);
    }

    #[test]
    fn fov_with_wall_to_vector() {
        let mut fov = SampleMap::new(WIDTH, HEIGHT);
        let mut rng = StdRng::seed_from_u64(42);
        for _ in 0..RANDOM_WALLS {
            let (x, y) = (rng.gen_range(0, WIDTH), rng.gen_range(0, HEIGHT));
            fov.set_transparent(x, y, false);
        }
        fov.set_transparent(POSITION_X, POSITION_Y, true);

        fov.calculate_fov(POSITION_X, POSITION_Y, RADIUS);

        println!("{:?}", fov);
    }
}
