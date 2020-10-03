use graphics::character::CharacterCache;
use legion::*;
use piston_window::types::Color as PistonColor;
use piston_window::Graphics;
use piston_window::PistonWindow;
use piston_window::*;

use crate::game::{RunState, State};
use crate::resources::*;
use crate::systems;
use crate::{
    components::{Body, Player},
    map::Map,
};
use field_of_vision::FovMap;

const GRID_SIZE: u32 = 16;

const COLOR_DARK_WALL: Color = Color {
    a: 255,
    r: 0,
    g: 0,
    b: 100,
};
const COLOR_LIGHT_WALL: Color = Color {
    a: 255,
    r: 130,
    g: 110,
    b: 50,
};
const COLOR_DARK_GROUND: Color = Color {
    a: 255,
    r: 50,
    g: 50,
    b: 150,
};
const COLOR_LIGHT_GROUND: Color = Color {
    a: 255,
    r: 200,
    g: 180,
    b: 50,
};
const TORCH_RADIUS: isize = 10;

pub struct Engine {
    window: Option<PistonWindow>,
    console: Console,
}

impl Engine {
    pub fn new<T: Into<String>>(title: T, width_in_squares: u32, height_in_squares: u32) -> Self {
        let window: PistonWindow = piston_window::WindowSettings::new(
            title,
            (width_in_squares * GRID_SIZE, height_in_squares * GRID_SIZE),
        )
        .exit_on_esc(true)
        .build()
        .expect("Failed to initialize the window");

        Engine {
            window: Some(window),
            console: Console::new(1, 1),
        }
    }

    pub fn run(&mut self, state: &mut State) {
        let mut events = Events::new(EventSettings::new().lazy(true).ups(2));
        let mut window = self.window.take().unwrap();

        let texture_settings = TextureSettings::new().filter(Filter::Nearest);
        let texture_context = window.create_texture_context();
        let mut glyphs = Glyphs::new(
            "CourierPrime-Regular.ttf",
            texture_context,
            texture_settings,
        )
        .expect("Couldn't load the font.");

        let mut schedule = systems::game_schedule();

        let mut previous_position = state.resources.get::<PlayerInfo>().unwrap().position;

        while let Some(event) = events.next(&mut window) {
            let previous_state = state.resources.get_or_insert(RunState::Init).clone();
            let new_run_state = match previous_state {
                RunState::Init => {
                    schedule.execute(&mut state.world, &mut state.resources);
                    RunState::WaitForInput
                }
                RunState::PlayerTurn => {
                    schedule.execute(&mut state.world, &mut state.resources);
                    RunState::AiTurn
                }
                RunState::AiTurn => {
                    schedule.execute(&mut state.world, &mut state.resources);
                    RunState::WaitForInput
                }
                RunState::WaitForInput => RunState::WaitForInput,
                RunState::Exit => break,
            };

            if let Some(_args) = event.render_args() {
                let updated_position = state.resources.get::<PlayerInfo>().unwrap().position;

                self.prepare_console(state, previous_position != updated_position);
                window.draw_2d(&event, |context, graphics, device| {
                    clear(BLACK.into(), graphics);
                    self.console.render(
                        graphics,
                        context,
                        &mut glyphs,
                        (0, 0),
                        (self.console.width, self.console.height),
                        (0, 0),
                    );

                    glyphs.factory.encoder.flush(device);
                });

                previous_position = updated_position;
            }

            state.resources.insert(new_run_state);
        }

        self.window = Some(window);
    }

    pub fn prepare_console(&mut self, state: &mut State, compute_fov: bool) {
        self.console.clear();
        self.prepare_map(state, compute_fov);

        let fov = state.resources.get::<FovMap>().unwrap();

        let mut query = <&Body>::query();
        for body in query.iter(&state.world) {
            if fov.is_in_fov(body.x as isize, body.y as isize) {
                self.console
                    .set_foreground(body.x, body.y, body.char, body.color);
            }
        }
    }

    fn prepare_map(&mut self, state: &mut State, fov_recompute: bool) {
        let mut map = state.resources.get_mut::<Map>().unwrap();
        let mut fov = state.resources.get_mut::<FovMap>().unwrap();

        if self.console.width() != map.width || self.console.height() != map.height {
            self.console = Console::new(map.width, map.height);
        }

        if fov_recompute {
            let mut query = <(&Player, &Body)>::query();
            for (_, body) in query.iter(&state.world) {
                fov.calculate_fov(body.x as isize, body.y as isize, TORCH_RADIUS);
            }
        }

        let map_width = map.width;
        let map_height = map.height;
        for y in 0..map_height {
            for x in 0..map_width {
                let visible = fov.is_in_fov(x as isize, y as isize);
                let wall = map.tiles[x as usize + y as usize * map_width as usize].block_sight;
                let color = match (visible, wall) {
                    (false, true) => COLOR_DARK_WALL,
                    (false, false) => COLOR_DARK_GROUND,
                    (true, true) => COLOR_LIGHT_WALL,
                    (true, false) => COLOR_LIGHT_GROUND,
                };

                let explored =
                    &mut map.explored_tiles[x as usize + y as usize * map_width as usize];
                if visible {
                    *explored = true;
                }

                if *explored {
                    self.console.set_background(x, y, color);
                }
            }
        }
    }
}

#[derive(Copy, Clone)]
struct Color {
    a: u8,
    r: u8,
    g: u8,
    b: u8,
}

impl Color {
    const fn new(a: u8, r: u8, g: u8, b: u8) -> Self {
        Color { a, r, g, b }
    }

    const fn from_argb(argb: u32) -> Self {
        let a = (argb >> 24) as u8;
        let r = (argb >> 16 & 0xff) as u8;
        let g = (argb >> 8 & 0xff) as u8;
        let b = (argb & 0xff) as u8;
        Color { a, r, g, b }
    }

    const fn from_rgb(rgb: u32) -> Self {
        let a = 255;
        let r = (rgb >> 16 & 0xff) as u8;
        let g = (rgb >> 8 & 0xff) as u8;
        let b = (rgb & 0xff) as u8;
        Color { a, r, g, b }
    }
}

impl Into<PistonColor> for Color {
    fn into(self) -> PistonColor {
        [
            self.r as f32 / 255.0,
            self.g as f32 / 255.0,
            self.b as f32 / 255.0,
            self.a as f32 / 255.0,
        ]
    }
}

impl Into<Color> for tcod::Color {
    fn into(self) -> Color {
        Color {
            a: 255,
            r: self.r,
            g: self.g,
            b: self.b,
        }
    }
}

struct Console {
    width: i32,
    height: i32,
    background: Vec<Option<Color>>,
    foreground: Vec<Option<(char, Color)>>,
}

impl Console {
    fn new(width: i32, height: i32) -> Self {
        Console {
            width,
            height,
            background: vec![None; (width * height) as usize],
            foreground: vec![None; (width * height) as usize],
        }
    }

    fn clear(&mut self) {
        for color in self.background.iter_mut() {
            *color = None;
        }
    }

    fn width(&self) -> i32 {
        self.width
    }

    fn height(&self) -> i32 {
        self.height
    }

    fn set_background(&mut self, x: i32, y: i32, color: Color) {
        self.background[(x + y * self.width) as usize] = Some(color);
    }

    fn set_foreground<C: Into<Color>>(&mut self, x: i32, y: i32, glyph: char, color: C) {
        self.foreground[(x + y * self.width) as usize] = Some((glyph, color.into()));
    }

    fn render(
        &self,
        graphics: &mut G2d,
        context: Context,
        glyphs: &mut Glyphs,
        (origin_x, origin_y): (i32, i32),
        (origin_width, origin_height): (i32, i32),
        (destination_x, destination_y): (i32, i32),
    ) {
        let dx = destination_x - origin_x;
        let dy = destination_y - origin_y;

        for x in origin_x..origin_width {
            for y in origin_y..origin_height {
                let (draw_x, draw_y) = (
                    ((x + dx) * GRID_SIZE as i32) as f64,
                    ((y + dy) * GRID_SIZE as i32) as f64,
                );
                if let Some(color) = self.background[(x + y * self.width) as usize] {
                    let color: PistonColor = color.into();

                    let square = graphics::rectangle::square(draw_x, draw_y, GRID_SIZE as f64);
                    graphics::rectangle(color, square, context.transform, graphics);
                }

                if let Some((glyph, color)) = self.foreground[(x + y * self.width) as usize] {
                    text::Text::new_color(color.into(), GRID_SIZE)
                        .draw(
                            &format!("{}", glyph),
                            glyphs,
                            &context.draw_state,
                            context.transform.trans(draw_x, draw_y),
                            graphics,
                        )
                        .expect("Could not draw glyph");
                }
            }
        }
    }
}

const BLACK: Color = Color::from_rgb(0x000000);
