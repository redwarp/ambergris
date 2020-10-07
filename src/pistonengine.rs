use std::time::Instant;

use crate::colors::{Color, BLACK, WHITE};
use crate::components::CombatStats;
use crate::game::{RunState, State};
use crate::resources::*;
use crate::systems;
use crate::{
    components::{Body, Player},
    map::Map,
};
use field_of_vision::FovMap;
use graphics::character::CharacterCache;
use graphics_buffer::BufferGlyphs;
use legion::*;
use piston_window::types::Color as PistonColor;
use piston_window::*;

const GRID_SIZE: u32 = 14;

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
const FONT_NAME: &str = "CourierPrime-Regular.ttf";

pub struct Engine {
    title: String,
    width: u32,
    height: u32,
    console: Console,
}

impl Engine {
    pub fn new<T: Into<String>>(title: T, width: u32, height: u32) -> Self {
        Engine {
            title: title.into(),
            width,
            height,
            console: Console::new(1, 1),
        }
    }

    pub fn run(&mut self, state: &mut State) {
        let mut window: PistonWindow = WindowSettings::new(
            &self.title,
            (self.width * GRID_SIZE, self.height * GRID_SIZE),
        )
        .exit_on_esc(false)
        .resizable(false)
        .build()
        .expect("Failed to initialize the window");
        let mut events = Events::new(EventSettings::new().max_fps(30).ups(30));

        let texture_settings = TextureSettings::new().filter(Filter::Nearest);
        let texture_context = window.create_texture_context();
        let mut glyphs = Glyphs::new(FONT_NAME, texture_context, texture_settings)
            .expect("Couldn't load the font.");

        let mut schedule = systems::game_schedule();

        let mut previous_position = state.resources.get::<SharedInfo>().unwrap().player_position;

        let mut pending_button = None;

        while let Some(event) = events.next(&mut window) {
            if let Some(button) = event.press_args() {
                pending_button = Some(button);

                if let Some(Button::Keyboard(Key::P)) = pending_button {
                    self.take_screenshot(state);
                }
            }

            if let Some(_args) = event.update_args() {
                let previous_state = state.resources.get_or_insert(RunState::Init).clone();

                let new_run_state = match previous_state {
                    RunState::Init => {
                        schedule.execute(&mut state.world, &mut state.resources);
                        RunState::WaitForPlayerInput
                    }
                    RunState::PlayerTurn => {
                        schedule.execute(&mut state.world, &mut state.resources);
                        RunState::AiTurn
                    }
                    RunState::AiTurn => {
                        schedule.execute(&mut state.world, &mut state.resources);
                        let alive = state
                            .resources
                            .get::<SharedInfo>()
                            .map_or(false, |player_info| player_info.alive);
                        if alive {
                            RunState::WaitForPlayerInput
                        } else {
                            RunState::Death
                        }
                    }
                    RunState::WaitForPlayerInput => {
                        self.consume_player_button(pending_button.take(), state)
                    }
                    RunState::Exit => break,
                    RunState::Death => self.consume_death_button(pending_button.take()),
                };

                state.resources.insert(new_run_state);

                let updated_position = state.resources.get::<SharedInfo>().unwrap().player_position;

                self.prepare_console(state, previous_position != updated_position);

                previous_position = updated_position;
            };

            if let Some(_args) = event.render_args() {
                window.draw_2d(&event, |context, graphics, device| {
                    self.render(state, graphics, context, &mut glyphs);

                    glyphs.factory.encoder.flush(device);
                });
            };
        }
    }

    pub fn prepare_console(&mut self, state: &mut State, compute_fov: bool) {
        self.console.clear();
        self.prepare_map(state, compute_fov);

        let fov = state.resources.get::<FovMap>().unwrap();

        let mut query = <&Body>::query();
        let mut bodies: Vec<_> = query.iter(&state.world).collect();
        bodies.sort_by(|&body_0, &body_1| body_0.blocking.cmp(&body_1.blocking));
        for body in bodies {
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

    fn consume_death_button(&self, button: Option<Button>) -> RunState {
        if let Some(button) = button {
            match button {
                Button::Keyboard(key) => match key {
                    Key::Escape => RunState::Exit,
                    _ => RunState::Death,
                },
                _ => RunState::Death,
            }
        } else {
            RunState::Death
        }
    }

    fn consume_player_button(&self, button: Option<Button>, state: &mut State) -> RunState {
        if let Some(button) = button {
            match button {
                Button::Keyboard(key) => match key {
                    Key::W | Key::Up => {
                        state.move_player(0, -1);
                        RunState::PlayerTurn
                    }
                    Key::A | Key::Left => {
                        state.move_player(-1, 0);
                        RunState::PlayerTurn
                    }
                    Key::S | Key::Down => {
                        state.move_player(0, 1);
                        RunState::PlayerTurn
                    }
                    Key::D | Key::Right => {
                        state.move_player(1, 0);
                        RunState::PlayerTurn
                    }
                    Key::Escape => RunState::Exit,
                    Key::Space => RunState::PlayerTurn,
                    _ => RunState::WaitForPlayerInput,
                },
                _ => RunState::WaitForPlayerInput,
            }
        } else {
            RunState::WaitForPlayerInput
        }
    }

    fn take_screenshot(&self, state: &State) {
        let now = Instant::now();
        let mut glyph_cache = BufferGlyphs::new(
            FONT_NAME,
            (),
            TextureSettings::new().filter(Filter::Nearest),
        )
        .expect("Couldn't load the font.");
        let mut buffer =
            graphics_buffer::RenderBuffer::new(self.width * GRID_SIZE, self.height * GRID_SIZE);
        let context = Context::new();
        self.render(state, &mut buffer, context, &mut glyph_cache);

        buffer.save("screenshot.png").ok();

        println!("Taking screenshot took {} ms", now.elapsed().as_millis());
    }

    fn render<G, C>(&self, state: &State, graphics: &mut G, context: Context, glyph_cache: &mut C)
    where
        C: CharacterCache,
        G: Graphics<Texture = <C as CharacterCache>::Texture>,
    {
        let player_life = current_player_life(state);
        clear(BLACK.into(), graphics);

        self.console.render(
            graphics,
            context,
            glyph_cache,
            (0, 0),
            (self.console.width, self.console.height),
            (0, 0),
        );

        if let Some((hp, max_hp)) = player_life {
            let health_bar = StatBar {
                name: String::from("Health"),
                color: crate::colors::DARK_RED,
                current: hp as u32,
                max: max_hp as u32,
            };
            health_bar.render(graphics, glyph_cache, context, (1, self.height as i32 - 3));
        }
    }
}

fn current_player_life(state: &State) -> Option<(i32, i32)> {
    let player = <(&Player, &CombatStats)>::query().get(&state.world, state.player_entity);
    player.map_or(None, |(_, combat_stats)| {
        Some((combat_stats.hp, combat_stats.max_hp))
    })
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
        for background in self.background.iter_mut() {
            *background = None;
        }
        for foreground in self.foreground.iter_mut() {
            *foreground = None;
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

    fn render<C, G>(
        &self,
        graphics: &mut G,
        context: Context,
        glyph_cache: &mut C,
        (origin_x, origin_y): (i32, i32),
        (origin_width, origin_height): (i32, i32),
        (destination_x, destination_y): (i32, i32),
    ) where
        C: CharacterCache,
        G: Graphics<Texture = <C as CharacterCache>::Texture>,
    {
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
                    let square = graphics::rectangle::square(0.0, 0.0, GRID_SIZE as f64);
                    graphics::rectangle(
                        color,
                        square,
                        context.transform.trans(draw_x, draw_y),
                        graphics,
                    );
                }

                if let Some((glyph, color)) = self.foreground[(x + y * self.width) as usize] {
                    let character = glyph_cache
                        .character(GRID_SIZE, glyph)
                        .ok()
                        .expect("Could not get glyph");
                    let font_adjust_x =
                        character.left() + (GRID_SIZE as f64 - character.atlas_size[0]) / 2.0;
                    let font_adjust_y =
                        character.top() + (GRID_SIZE as f64 - character.atlas_size[1]) / 2.0;

                    text::Text::new_color(color.into(), GRID_SIZE)
                        .draw(
                            &format!("{}", glyph),
                            glyph_cache,
                            &context.draw_state,
                            context
                                .transform
                                .trans(draw_x + font_adjust_x, draw_y + font_adjust_y),
                            graphics,
                        )
                        .ok()
                        .expect("Could not draw glyph");
                }
            }
        }
    }
}

struct StatBar {
    name: String,
    current: u32,
    max: u32,
    color: Color,
}

impl StatBar {
    fn render<C, G>(
        &self,
        graphics: &mut G,
        glyph_cache: &mut C,
        context: Context,
        origin: (i32, i32),
    ) where
        C: CharacterCache,
        G: Graphics<Texture = <C as CharacterCache>::Texture>,
    {
        let text = format!("{} [{}/{}]", self.name, self.current, self.max);
        let max_width = (GRID_SIZE * 10) as f64;
        let origin_x = (origin.0 * GRID_SIZE as i32) as f64;
        let text_origin_y = (origin.1 as f64 + 0.7) * GRID_SIZE as f64;
        let origin_y = ((origin.1 + 1) * GRID_SIZE as i32) as f64;
        let ratio = self.current as f64 / self.max as f64;

        graphics::text(
            WHITE.into(),
            GRID_SIZE,
            text.as_str(),
            glyph_cache,
            context.transform.trans(origin_x, text_origin_y),
            graphics,
        )
        .ok()
        .unwrap();

        graphics::rectangle(
            self.color.into(),
            [origin_x, origin_y, max_width * ratio, GRID_SIZE as f64],
            context.transform,
            graphics,
        );
        graphics::rectangle(
            self.color.darker().into(),
            [
                origin_x + max_width * ratio,
                origin_y,
                max_width * (1.0 - ratio),
                GRID_SIZE as f64,
            ],
            context.transform,
            graphics,
        );
    }
}
