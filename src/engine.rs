use crate::game::State;
use crate::{
    component::{Body, Player, Renderable},
    map::Map,
};
use legion::IntoQuery;
use tcod::colors::Color;
use tcod::console::{blit, BackgroundFlag, Console, FontLayout, FontType, Offscreen, Root};
use tcod::map::FovAlgorithm;
use tcod::map::Map as FovMap;

// actual size of the window
const SCREEN_WIDTH: i32 = 80;
const SCREEN_HEIGHT: i32 = 50;

const LIMIT_FPS: i32 = 20; // 20 frames-per-second maximum

const COLOR_DARK_WALL: Color = Color { r: 0, g: 0, b: 100 };
const COLOR_LIGHT_WALL: Color = Color {
    r: 130,
    g: 110,
    b: 50,
};
const COLOR_DARK_GROUND: Color = Color {
    r: 50,
    g: 50,
    b: 150,
};
const COLOR_LIGHT_GROUND: Color = Color {
    r: 200,
    g: 180,
    b: 50,
};

const FOV_ALGO: FovAlgorithm = FovAlgorithm::Basic;
const FOV_LIGHT_WALLS: bool = true;
const TORCH_RADIUS: i32 = 10;

pub struct Engine {
    root: Root,
    console: Offscreen,
    fov: FovMap,
}

impl Engine {
    pub fn new() -> Self {
        let root = Root::initializer()
            .font("consolas_unicode_16x16.png", FontLayout::AsciiInRow)
            .font_type(FontType::Greyscale)
            .font_dimensions(32, 64)
            .size(SCREEN_WIDTH, SCREEN_HEIGHT)
            .title("Rust/libtcod tutorial")
            .init();

        Engine {
            root,
            console: Offscreen::new(1, 1),
            fov: FovMap::new(1, 1),
        }
    }

    pub fn run(&mut self, state: &mut State) {
        self.root.clear();

        render_all(self, state, true);

        self.root.flush();
        self.root.wait_for_keypress(true);
    }
}

fn render_all(engine: &mut Engine, state: &mut State, fov_recompute: bool) {
    render_map(engine, state, fov_recompute);

    let mut query = <(&Body, &Renderable)>::query();
    for (position, renderable) in query.iter(&state.world) {
        engine.root.set_default_foreground(renderable.color);
        engine.root.put_char(
            position.x,
            position.y,
            renderable.char,
            BackgroundFlag::None,
        );
    }
}

fn render_map(engine: &mut Engine, state: &mut State, fov_recompute: bool) {
    let map = &mut state.map;
    if engine.console.width() != map.width || engine.console.height() != map.height {
        engine.console = Offscreen::new(map.width, map.height);
        engine.fov = make_fov(map);
    }

    if fov_recompute {
        let mut query = <(&Player, &Body)>::query();
        for (_, body) in query.iter(&state.world) {
            engine
                .fov
                .compute_fov(body.x, body.y, TORCH_RADIUS, FOV_LIGHT_WALLS, FOV_ALGO);
        }
    }

    for y in 0..map.height {
        for x in 0..map.width {
            let visible = engine.fov.is_in_fov(x, y);
            let wall = map.tiles[x as usize][y as usize].block_sight;
            let color = match (visible, wall) {
                (false, true) => COLOR_DARK_WALL,
                (false, false) => COLOR_DARK_GROUND,
                (true, true) => COLOR_LIGHT_WALL,
                (true, false) => COLOR_LIGHT_GROUND,
            };

            let explored = &mut map.tiles[x as usize][y as usize].explored;
            if visible {
                *explored = true;
            }

            if *explored {
                engine
                    .console
                    .set_char_background(x, y, color, BackgroundFlag::Set);
            }
        }
    }

    blit(
        &engine.console,
        (0, 0),
        (map.width, map.height),
        &mut engine.root,
        (0, 0),
        1.0,
        1.0,
    );
}

fn make_fov(map: &Map) -> FovMap {
    let mut fov = FovMap::new(map.width, map.height);

    for y in 0..map.height {
        for x in 0..map.width {
            fov.set(
                x,
                y,
                !map.tiles[x as usize][y as usize].block_sight,
                !map.tiles[x as usize][y as usize].blocked,
            )
        }
    }

    fov
}
