use crate::components::*;
use legion::*;
use piston_window::types::Color as PistonColor;
use piston_window::Graphics;
use piston_window::*;
use piston_window::{PistonWindow, Window};

use crate::{
    components::{Body, Player},
    map::Map,
};
use crate::{
    game::{RunState, State},
    resources::PlayerInfo,
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
        let mut events = Events::new(EventSettings::new().lazy(true));
        let mut window = self.window.take().unwrap();

        while let Some(event) = events.next(&mut window) {
            window.draw_2d(&event, |context, graphics, _device| {
                self.render_all(graphics, context, state);
            });
        }

        self.window = Some(window);
    }

    pub fn render_all<G: Graphics>(
        &mut self,
        graphics: &mut G,
        context: Context,
        state: &mut State,
    ) {
        clear([0.0, 0.0, 0.0, 1.0], graphics);
        self.console.clear();
        self.render_map(graphics, context, state, true);
    }

    fn render_map<G: Graphics>(
        &mut self,
        graphics: &mut G,
        context: Context,
        state: &mut State,
        fov_recompute: bool,
    ) {
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

        self.console
            .render(graphics, context, (0, 0), (map_width, map_height), (0, 0));
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

struct Console {
    width: i32,
    height: i32,
    background: Vec<Color>,
}

impl Console {
    fn new(width: i32, height: i32) -> Self {
        Console {
            width,
            height,
            background: vec![BLACK; (width * height) as usize],
        }
    }

    fn clear(&mut self) {
        for color in self.background.iter_mut() {
            *color = BLACK;
        }
    }

    fn width(&self) -> i32 {
        self.width
    }

    fn height(&self) -> i32 {
        self.height
    }

    fn set_background(&mut self, x: i32, y: i32, color: Color) {
        self.background[(x + y * self.width) as usize] = color
    }

    fn render<G: Graphics>(
        &self,
        graphics: &mut G,
        context: Context,
        (origin_x, origin_y): (i32, i32),
        (origin_width, origin_height): (i32, i32),
        (destination_x, destination_y): (i32, i32),
    ) {
        let dx = destination_x - origin_x;
        let dy = destination_y - origin_y;

        for x in origin_x..origin_width {
            for y in origin_y..origin_height {
                let color: PistonColor = self.background[(x + y * self.width) as usize].into();
                let square = graphics::rectangle::square(
                    ((x + dx) * GRID_SIZE as i32) as f64,
                    ((y + dy) * GRID_SIZE as i32) as f64,
                    GRID_SIZE as f64,
                );
                graphics::rectangle(color, square, context.transform, graphics);
            }
        }
    }
}

const BLACK: Color = Color::from_rgb(0x000000);
