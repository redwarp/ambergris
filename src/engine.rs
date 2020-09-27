use crate::game::{RunState, State};
use crate::systems;
use crate::{
    components::{Body, Player},
    map::Map,
};
use input::Event;
use input::Key;
use input::KeyCode;
use input::Mouse;
use legion::IntoQuery;
use legion::Schedule;
use tcod::console::{blit, BackgroundFlag, Console, FontLayout, FontType, Offscreen, Root};
use tcod::map::FovAlgorithm;
use tcod::map::Map as FovMap;
use tcod::{colors::Color, input};

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

        tcod::system::set_fps(LIMIT_FPS);
        Engine {
            root,
            console: Offscreen::new(1, 1),
            fov: FovMap::new(1, 1),
        }
    }

    pub fn run(&mut self, state: &mut State) {
        let mut schedule = Schedule::builder()
            .add_system(systems::new_turn_system())
            .add_system(systems::monster_move_system())
            .build();

        while !self.root.window_closed() {
            let (_mouse, key) = check_for_event();
            state.run_state = self.consume_key(state, key);
            match state.run_state {
                RunState::Exit => break,
                RunState::Running => {
                    schedule.execute(&mut state.world, &mut state.resources);
                }
                RunState::Paused => {}
            }

            self.root.clear();

            self.render_all(state, true);

            self.root.flush();
        }
    }

    fn render_all(&mut self, state: &mut State, fov_recompute: bool) {
        self.console.clear();
        self.render_map(state, fov_recompute);

        let mut query = <&Body>::query();
        for body in query.iter(&state.world) {
            if self.fov.is_in_fov(body.x, body.y) {
                self.root.set_default_foreground(body.color);
                self.root
                    .put_char(body.x, body.y, body.char, BackgroundFlag::None);
            }
        }
    }

    fn render_map(&mut self, state: &mut State, fov_recompute: bool) {
        let map = &mut state.map;
        if self.console.width() != map.width || self.console.height() != map.height {
            self.console = Offscreen::new(map.width, map.height);
            self.fov = make_fov(map);
        }

        if fov_recompute {
            let mut query = <(&Player, &Body)>::query();
            for (_, body) in query.iter(&state.world) {
                self.fov
                    .compute_fov(body.x, body.y, TORCH_RADIUS, FOV_LIGHT_WALLS, FOV_ALGO);
            }
        }

        for y in 0..map.height {
            for x in 0..map.width {
                let visible = self.fov.is_in_fov(x, y);
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
                    self.console
                        .set_char_background(x, y, color, BackgroundFlag::Set);
                }
            }
        }

        blit(
            &self.console,
            (0, 0),
            (map.width, map.height),
            &mut self.root,
            (0, 0),
            1.0,
            1.0,
        );
    }

    fn consume_key(&mut self, state: &mut State, key: Key) -> RunState {
        match (key, key.text()) {
            (
                Key {
                    code: KeyCode::Up, ..
                },
                _,
            ) => {
                state.move_player(0, -1);
                RunState::Running
            }
            (
                Key {
                    code: KeyCode::Down,
                    ..
                },
                _,
            ) => {
                state.move_player(0, 1);
                RunState::Running
            }
            (
                Key {
                    code: KeyCode::Left,
                    ..
                },
                _,
            ) => {
                state.move_player(-1, 0);
                RunState::Running
            }
            (
                Key {
                    code: KeyCode::Right,
                    ..
                },
                _,
            ) => {
                state.move_player(1, 0);
                RunState::Running
            }
            (
                Key {
                    code: KeyCode::Escape,
                    ..
                },
                _,
            ) => RunState::Exit,
            _ => RunState::Paused,
        }
    }
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

fn check_for_event() -> (Mouse, Key) {
    match input::check_for_event(input::MOUSE | input::KEY_PRESS) {
        Some((_, Event::Mouse(mouse))) => (mouse, Default::default()),
        Some((_, Event::Key(key))) => (Default::default(), key),
        _ => (Default::default(), Default::default()),
    }
}
