use bracket_pathfinding::prelude::{Algorithm2D, SmallVec};
use criterion::{criterion_group, criterion_main, Criterion};
use torchbearer::{bresenham::Bresenham, path::astar_path, Map, Position};

struct TestMap {
    width: i32,
    height: i32,
    tiles: Vec<bool>,
}

impl TestMap {
    fn new(width: i32, height: i32) -> Self {
        TestMap {
            width,
            height,
            tiles: vec![false; (width * height) as usize],
        }
    }

    fn build_wall(&mut self, from: Position, to: Position) {
        let bresenham = Bresenham::new(from, to);
        for (x, y) in bresenham {
            self.tiles[(x + y * self.width) as usize] = true;
        }
    }

    fn is_exit_valid(&self, x: i32, y: i32) -> bool {
        if x < 1 || x > self.width - 1 || y < 1 || y > self.height - 1 {
            return false;
        }
        let idx = x + y * self.width;
        !self.tiles[idx as usize]
    }
}

impl Map for TestMap {
    fn dimensions(&self) -> (i32, i32) {
        (self.width, self.height)
    }

    fn is_opaque(&self, x: i32, y: i32) -> bool {
        self.tiles[(x + y * self.width) as usize]
    }
}

/// Implementing the BaseMap like
/// https://bfnightly.bracketproductions.com/rustbook/chapter_7.html?highlight=pathfin#chasing-the-player
impl bracket_pathfinding::prelude::BaseMap for TestMap {
    fn is_opaque(&self, index: usize) -> bool {
        self.tiles[index]
    }

    fn get_available_exits(&self, idx: usize) -> SmallVec<[(usize, f32); 10]> {
        let mut exits = SmallVec::new();
        let x = idx as i32 % self.width;
        let y = idx as i32 / self.width;
        let w = self.width as usize;

        // Cardinal directions
        if self.is_exit_valid(x - 1, y) {
            exits.push((idx - 1, 1.0))
        };
        if self.is_exit_valid(x + 1, y) {
            exits.push((idx + 1, 1.0))
        };
        if self.is_exit_valid(x, y - 1) {
            exits.push((idx - w, 1.0))
        };
        if self.is_exit_valid(x, y + 1) {
            exits.push((idx + w, 1.0))
        };

        exits
    }

    fn get_pathing_distance(&self, idx1: usize, idx2: usize) -> f32 {
        let w = self.width as usize;
        let p1 = bracket_pathfinding::prelude::Point::new(idx1 % w, idx1 / w);
        let p2 = bracket_pathfinding::prelude::Point::new(idx2 % w, idx2 / w);
        bracket_pathfinding::prelude::DistanceAlg::Pythagoras.distance2d(p1, p2)
    }
}

impl bracket_pathfinding::prelude::Algorithm2D for TestMap {
    fn dimensions(&self) -> bracket_pathfinding::prelude::Point {
        (self.width, self.height).into()
    }

    fn point2d_to_index(&self, pt: bracket_pathfinding::prelude::Point) -> usize {
        (pt.x + pt.y * self.width) as usize
    }
}

pub fn astar_benchmark(c: &mut Criterion) {
    let mut map = TestMap::new(20, 20);
    map.build_wall((0, 3), (3, 3));
    map.build_wall((3, 3), (3, 10));

    c.bench_function("astar_benchmark", |bencher| {
        bencher.iter(|| astar_path(&map, (1, 4), (10, 4)));
    });
}

pub fn rltk_astar_benchmark(c: &mut Criterion) {
    let mut map = TestMap::new(20, 20);
    map.build_wall((0, 3), (3, 3));
    map.build_wall((3, 3), (3, 10));
    let start = map.point2d_to_index((1, 4).into());
    let end = map.point2d_to_index((10, 4).into());

    c.bench_function("rltk_astar_benchmark", |bencher| {
        bencher.iter(|| bracket_pathfinding::prelude::a_star_search(start, end, &map));
    });
}

criterion_group!(benches, astar_benchmark, rltk_astar_benchmark);
criterion_main!(benches);
