use criterion::{criterion_group, criterion_main, Criterion};
use rand::{rngs::StdRng, Rng, SeedableRng};
use rltk::Algorithm2D;
use tcod::Map;
use torchbearer::FovMap;
use torchbearer::SampleMap;

const WIDTH: i32 = 45;
const HEIGHT: i32 = 45;
const POSITION_X: i32 = 22;
const POSITION_Y: i32 = 22;
const RADIUS: i32 = 12;
const RANDOM_WALLS: i32 = 10;

struct RltkMap {
    width: i32,
    height: i32,
    transparents: Vec<bool>,
}

impl RltkMap {
    fn set_transparent(&mut self, x: i32, y: i32, transparent: bool) {
        self.transparents[(x + y * self.width) as usize] = transparent;
    }

    fn compute_fov(&mut self, x: i32, y: i32, radius: i32) {
        rltk::field_of_view((x as i32, y as i32).into(), radius as i32, self);
    }
}

impl rltk::BaseMap for RltkMap {
    fn is_opaque(&self, index: usize) -> bool {
        !self.transparents[index]
    }
}

impl Algorithm2D for RltkMap {
    fn dimensions(&self) -> rltk::Point {
        (self.width as i32, self.height as i32).into()
    }
}

pub fn fov_benchmark_no_walls(c: &mut Criterion) {
    let mut fov = FovMap::new(WIDTH, HEIGHT);

    c.bench_function("fov_benchmark_no_walls", |bencher| {
        bencher.iter(|| fov.calculate_fov(POSITION_X, POSITION_Y, RADIUS));
    });
}

pub fn fov_benchmark_random_walls(c: &mut Criterion) {
    let mut fov = FovMap::new(WIDTH, HEIGHT);
    let mut rng = StdRng::seed_from_u64(42);
    for _ in 0..RANDOM_WALLS {
        let (x, y) = (rng.gen_range(0, WIDTH), rng.gen_range(0, HEIGHT));
        fov.set_transparent(x, y, false);
    }
    fov.set_transparent(POSITION_X, POSITION_Y, true);

    c.bench_function("fov_benchmark_random_walls", |bencher| {
        bencher.iter(|| fov.calculate_fov(POSITION_X, POSITION_Y, RADIUS));
    });
}

pub fn vec_fov_benchmark_no_walls(c: &mut Criterion) {
    let mut fov = SampleMap::new(WIDTH, HEIGHT);

    c.bench_function("vec_fov_benchmark_no_walls", |bencher| {
        bencher.iter(|| fov.calculate_fov(POSITION_X, POSITION_Y, RADIUS));
    });
}

pub fn vec_fov_benchmark_random_walls(c: &mut Criterion) {
    let mut fov = SampleMap::new(WIDTH, HEIGHT);
    let mut rng = StdRng::seed_from_u64(42);
    for _ in 0..RANDOM_WALLS {
        let (x, y) = (rng.gen_range(0, WIDTH), rng.gen_range(0, HEIGHT));
        fov.set_transparent(x, y, false);
    }
    fov.set_transparent(POSITION_X, POSITION_Y, true);

    c.bench_function("vec_fov_benchmark_random_walls", |bencher| {
        bencher.iter(|| fov.calculate_fov(POSITION_X, POSITION_Y, RADIUS));
    });
}

pub fn tcod_benchmark_no_walls(c: &mut Criterion) {
    let mut map = Map::new(WIDTH as i32, HEIGHT as i32);
    for x in 0..WIDTH as i32 {
        for y in 0..HEIGHT as i32 {
            map.set(x, y, true, true);
        }
    }

    let x = POSITION_X as i32;
    let y = POSITION_Y as i32;
    let radius = RADIUS as i32;
    c.bench_function("tcod_benchmark_no_walls", |bencher| {
        bencher.iter(|| map.compute_fov(x, y, radius, true, tcod::map::FovAlgorithm::Basic));
    });
}

pub fn tcod_benchmark_random_walls(c: &mut Criterion) {
    let mut map = Map::new(WIDTH as i32, HEIGHT as i32);
    for x in 0..WIDTH as i32 {
        for y in 0..HEIGHT as i32 {
            map.set(x, y, true, true);
        }
    }

    let mut rng = StdRng::seed_from_u64(42);
    for _ in 0..RANDOM_WALLS {
        let (x, y) = (rng.gen_range(0, WIDTH), rng.gen_range(0, HEIGHT));
        map.set(x as i32, y as i32, false, false);
    }
    map.set(POSITION_X as i32, POSITION_Y as i32, true, true);

    let x = POSITION_X as i32;
    let y = POSITION_Y as i32;
    let radius = RADIUS as i32;
    c.bench_function("tcod_benchmark_random_walls", |bencher| {
        bencher.iter(|| map.compute_fov(x, y, radius, true, tcod::map::FovAlgorithm::Basic));
    });
}

pub fn rltk_benchmark_no_walls(c: &mut Criterion) {
    let mut map = RltkMap {
        width: WIDTH,
        height: HEIGHT,
        transparents: vec![true; (WIDTH * HEIGHT) as usize],
    };

    c.bench_function("rltk_benchmark_no_walls", |bencher| {
        bencher.iter(|| map.compute_fov(POSITION_X, POSITION_Y, RADIUS));
    });
}

pub fn rltk_benchmark_random_walls(c: &mut Criterion) {
    let mut map = RltkMap {
        width: WIDTH,
        height: HEIGHT,
        transparents: vec![true; (WIDTH * HEIGHT) as usize],
    };

    let mut rng = StdRng::seed_from_u64(42);
    for _ in 0..RANDOM_WALLS {
        let (x, y) = (rng.gen_range(0, WIDTH), rng.gen_range(0, HEIGHT));
        map.set_transparent(x, y, false);
    }
    map.set_transparent(POSITION_X, POSITION_Y, true);

    c.bench_function("rltk_benchmark_random_walls", |bencher| {
        bencher.iter(|| map.compute_fov(POSITION_X, POSITION_Y, RADIUS));
    });
}

criterion_group!(
    benches,
    fov_benchmark_no_walls,
    fov_benchmark_random_walls,
    vec_fov_benchmark_no_walls,
    vec_fov_benchmark_random_walls,
    tcod_benchmark_no_walls,
    tcod_benchmark_random_walls,
    rltk_benchmark_no_walls,
    rltk_benchmark_random_walls
);
criterion_main!(benches);
