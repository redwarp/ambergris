use crate::systems;
use crate::{
    colors::{Color, BLACK, DARK_GREY, WHITE},
    components::{Body, CombatStats, Coordinates, Player},
    game::{Journal, RunState, State, Targeting},
    inventory::InventoryAction,
    map::Map,
    palette,
    renderer::RenderContext,
    renderer::Renderable,
};
use crate::{inventory::Inventory, resources::SharedInfo};
use graphics::character::CharacterCache;
use graphics_buffer::BufferGlyphs;
use legion::*;
use piston_window::*;
use std::{collections::VecDeque, time::Instant};

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
const TORCH_RADIUS: i32 = 10;
const FONT_NAME: &str = "fonts/CourierPrime-Regular.ttf";

pub struct Engine {
    title: String,
    width: i32,
    height: i32,
    console: Console,
    hud: Hud,
    inventory: Option<Inventory>,
    mouse_position: [i32; 2],
    target_area: Option<Vec<(i32, i32)>>,
}

impl Engine {
    pub fn new<T: Into<String>>(title: T, width: i32, height: i32) -> Self {
        Engine {
            title: title.into(),
            width,
            height,
            console: Console::new(0, 0, 1, 1),
            hud: Hud::new(width, height),
            inventory: None,
            mouse_position: [0, 0],
            target_area: None,
        }
    }

    pub fn run(&mut self, state: &mut State) {
        let mut window: PistonWindow = WindowSettings::new(
            &self.title,
            (
                self.width as u32 * GRID_SIZE,
                self.height as u32 * GRID_SIZE,
            ),
        )
        .exit_on_esc(false)
        .resizable(false)
        .build()
        .expect("Failed to initialize the window");

        println!("{:?}", window.size());

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

            event.mouse_cursor(|position| {
                self.mouse_position[0] = (position[0] / GRID_SIZE as f64) as i32;
                self.mouse_position[1] = (position[1] / GRID_SIZE as f64) as i32;
            });

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
                    RunState::ShowInventory => {
                        self.consume_inventory_button(pending_button.take(), state)
                    }
                    RunState::ShowTargeting { item, range, burst } => self.consume_targeting(
                        state,
                        Targeting { item, range, burst },
                        pending_button.take(),
                    ),
                };

                state.resources.insert(new_run_state);

                if previous_state != new_run_state {
                    if new_run_state == RunState::ShowInventory {
                        let mut inventory =
                            Inventory::new((5, 5), (self.width - 10, self.height - 10));
                        inventory.list_items(state);
                        self.inventory = Some(inventory);
                    } else {
                        self.inventory = None;
                    }

                    let updated_position =
                        state.resources.get::<SharedInfo>().unwrap().player_position;

                    self.prepare_console(state, previous_position != updated_position);

                    let (current, max) = current_player_life(state).unwrap_or((0, 0));
                    self.hud.health_bar.update(current, max);

                    {
                        let journal = state.resources.get::<Journal>().unwrap();
                        self.hud.update_journal(&journal);
                    }

                    if let RunState::ShowTargeting {
                        item: _,
                        range,
                        burst: _,
                    } = new_run_state
                    {
                        self.show_targeting_overlay_on_console(state, range);
                    }

                    previous_position = updated_position;
                }

                // Mouse stuff.
                if let Some(inventory) = &mut self.inventory {
                    inventory.set_mouse(self.mouse_position);
                } else {
                    match new_run_state {
                        RunState::ShowTargeting {
                            item: _,
                            range: _,
                            burst,
                        } => {
                            self.show_targeting_ring_on_console(state, burst);
                        }
                        _ => {
                            self.prepare_tooltip(state);
                        }
                    }
                }
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

        let map = state.resources.get::<Map>().unwrap();

        let mut query = <(&Body, &Coordinates)>::query();
        let mut bodies: Vec<_> = query.iter(&state.world).collect();
        bodies.sort_by(|&(body_0, _), &(body_1, _)| body_0.blocking.cmp(&body_1.blocking));

        for (body, coordinates) in bodies {
            if map.is_in_player_fov(coordinates.x, coordinates.y) {
                self.console
                    .set_foreground(coordinates.x, coordinates.y, body.char, body.color);
            }
        }
    }

    pub fn show_targeting_overlay_on_console(&mut self, state: &mut State, range: i32) {
        let map = state.resources.get::<Map>().unwrap();
        let shared_info = state.resources.get::<SharedInfo>().unwrap();

        let selected = field_of_vision::field_of_view(
            &*map,
            shared_info.player_position.0,
            shared_info.player_position.1,
            range,
            false,
        );
        self.console.overlay(&selected[..]);
        self.target_area = Some(selected);
    }
    pub fn show_targeting_ring_on_console(&mut self, state: &mut State, burst: i32) {
        let map = state.resources.get::<Map>().unwrap();

        let (x, y) = (self.mouse_position[0], self.mouse_position[1] - 3);
        if let Some(target_area) = &self.target_area {
            if target_area.contains(&(x, y)) {
                if burst <= 0 {
                    self.console.select(x, y)
                } else {
                    let burst_area = field_of_vision::field_of_view(&*map, x, y, burst, false);
                    self.console.select_multiple(&burst_area[..]);
                }

                // Let's also display the tooltip, because why not.
                self.hud.set_tooltip::<String>(None);
                let target_coordinates = Coordinates { x, y };
                for (position, body) in <(&Coordinates, &Body)>::query().iter(&state.world) {
                    if target_coordinates == *position {
                        self.hud.set_tooltip(Some(body.name.clone()));
                        break;
                    }
                }
            }
        }
    }

    pub fn prepare_tooltip(&mut self, state: &mut State) {
        self.hud.set_tooltip::<String>(None);

        let x = self.mouse_position[0];
        let y = self.mouse_position[1] - 3;

        let map = state.resources.get::<Map>().unwrap();
        if !map.is_in_bounds(x, y) || !map.is_in_player_fov(x, y) {
            // No tooltip for stuff we can't see!
            return;
        }

        self.console.select(x, y);
        let target_coordinates = Coordinates { x, y };
        for (position, body) in <(&Coordinates, &Body)>::query().iter(&state.world) {
            if target_coordinates == *position {
                self.hud.set_tooltip(Some(body.name.clone()));
                break;
            }
        }
    }

    fn prepare_map(&mut self, state: &mut State, fov_recompute: bool) {
        let mut map = state.resources.get_mut::<Map>().unwrap();

        if self.console.width() != map.width || self.console.height() != map.height {
            self.console = Console::new(0, 3, map.width, map.height);
        }

        if fov_recompute {
            let mut query = <&Coordinates>::query().filter(component::<Player>());
            for coordinates in query.iter(&state.world) {
                map.calculate_player_fov(coordinates.x, coordinates.y, TORCH_RADIUS);
            }
        }

        let map_width = map.width;
        let map_height = map.height;
        for y in 0..map_height {
            for x in 0..map_width {
                let visible = map.is_in_player_fov(x, y);
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
                    Key::G => {
                        if state.grab_item() {
                            RunState::PlayerTurn
                        } else {
                            RunState::WaitForPlayerInput
                        }
                    }
                    Key::I => RunState::ShowInventory,
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

    fn consume_inventory_button(&mut self, button: Option<Button>, state: &mut State) -> RunState {
        if let Some(Button::Keyboard(key)) = button {
            if let Some(inventory) = &mut self.inventory {
                match inventory.on_keyboard(&key) {
                    InventoryAction::Selecting => RunState::ShowInventory,
                    InventoryAction::Pick { entity } => state.use_item(entity),
                    InventoryAction::Close => RunState::PlayerTurn,
                    InventoryAction::Drop { entity } => state.drop_item(entity),
                }
            } else {
                RunState::ShowInventory
            }
        } else {
            RunState::ShowInventory
        }
    }

    fn consume_targeting(
        &mut self,
        state: &mut State,
        targeting: Targeting,
        button: Option<Button>,
    ) -> RunState {
        match button {
            Some(Button::Mouse(_mouse)) => {
                println!("Clicked on {:?}", self.mouse_position);

                let current_state = RunState::ShowTargeting {
                    item: targeting.item,
                    range: targeting.range,
                    burst: targeting.burst,
                };
                state.use_range_item_with_targeting(
                    current_state,
                    targeting.item,
                    (self.mouse_position[0], self.mouse_position[1] - 3),
                )
            }
            Some(Button::Keyboard(key)) if key == Key::Escape => RunState::WaitForPlayerInput,
            _ => RunState::ShowTargeting {
                item: targeting.item,
                range: targeting.range,
                burst: targeting.burst,
            },
        }
    }

    fn take_screenshot(&self, state: &mut State) {
        let now = Instant::now();
        let mut glyph_cache = BufferGlyphs::new(
            FONT_NAME,
            (),
            TextureSettings::new().filter(Filter::Nearest),
        )
        .expect("Couldn't load the font.");
        let mut buffer = graphics_buffer::RenderBuffer::new(
            self.width as u32 * GRID_SIZE,
            self.height as u32 * GRID_SIZE,
        );
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
        clear(BLACK.into(), graphics);
        let run_state = state
            .resources
            .get::<RunState>()
            .map_or(RunState::Init, |fetched| *fetched);

        let mut render_context = RenderContext {
            grid_size: GRID_SIZE,
            character_cache: glyph_cache,
            context,
            graphics,
        };

        match run_state {
            RunState::ShowInventory => {
                self.render_map_and_hud(&mut render_context);
                self.render_inventory(&mut render_context);
            }
            _ => {
                self.render_map_and_hud(&mut render_context);
            }
        }
    }

    fn render_map_and_hud<C, G>(&self, render_context: &mut RenderContext<C, G>)
    where
        C: CharacterCache,
        G: Graphics<Texture = <C as CharacterCache>::Texture>,
    {
        clear(BLACK.into(), render_context.graphics);

        self.console.render(render_context);
        self.hud.render(render_context);
    }

    fn render_inventory<C, G>(&self, render_context: &mut RenderContext<C, G>)
    where
        C: CharacterCache,
        G: Graphics<Texture = <C as CharacterCache>::Texture>,
    {
        if let Some(inventory) = &self.inventory {
            inventory.render(render_context);
        }
    }
}

fn current_player_life(state: &State) -> Option<(i32, i32)> {
    let player = <(&Player, &CombatStats)>::query().get(&state.world, state.player_entity);
    player.map_or(None, |(_, combat_stats)| {
        Some((combat_stats.hp, combat_stats.max_hp))
    })
}

struct Console {
    origin: (i32, i32),
    width: i32,
    height: i32,
    background: Vec<Option<Color>>,
    foreground: Vec<Option<(char, Color)>>,
    overlay: Vec<(i32, i32, Color)>,
    selection: Vec<(i32, i32, Color)>,
}

impl Console {
    fn new(x: i32, y: i32, width: i32, height: i32) -> Self {
        Console {
            origin: (x, y),
            width,
            height,
            background: vec![None; (width * height) as usize],
            foreground: vec![None; (width * height) as usize],
            overlay: vec![],
            selection: vec![],
        }
    }

    fn clear(&mut self) {
        for background in self.background.iter_mut() {
            *background = None;
        }
        for foreground in self.foreground.iter_mut() {
            *foreground = None;
        }
        self.overlay.clear();
        self.selection.clear();
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

    fn select(&mut self, x: i32, y: i32) {
        self.selection.clear();
        if x >= 0 && x < self.width && y >= 0 && y < self.height {
            self.selection.push((x, y, palette::SELECTED));
        }
    }

    fn select_multiple(&mut self, selection: &[(i32, i32)]) {
        self.selection.clear();
        for position in selection.iter().map(|&(x, y)| (x, y, palette::SELECTED)) {
            self.selection.push(position);
        }
    }

    fn overlay(&mut self, overlay: &[(i32, i32)]) {
        self.overlay.clear();
        for position in overlay.iter().map(|&(x, y)| (x, y, palette::OVERLAY)) {
            self.overlay.push(position);
        }
    }
}

impl Renderable for Console {
    fn position(&self) -> (i32, i32) {
        self.origin
    }

    fn size(&self) -> (i32, i32) {
        (self.width, self.height)
    }

    fn render<'a, C, G>(&self, render_context: &mut RenderContext<'a, C, G>)
    where
        C: CharacterCache,
        G: Graphics<Texture = <C as CharacterCache>::Texture>,
    {
        let (dx, dy) = self.position();
        let (width, height) = self.size();

        for x in 0..width {
            for y in 0..height {
                if let Some(color) = self.background[(x + y * self.width) as usize] {
                    crate::renderer::draw_square(
                        x + dx,
                        y + dy,
                        color.into(),
                        GRID_SIZE,
                        render_context.context,
                        render_context.graphics,
                    );
                }

                if let Some((glyph, color)) = self.foreground[(x + y * self.width) as usize] {
                    crate::renderer::draw_char(
                        x + dx,
                        y + dy,
                        color.into(),
                        GRID_SIZE,
                        glyph,
                        render_context.character_cache,
                        render_context.context,
                        render_context.graphics,
                    )
                    .ok();
                }
            }
        }

        for (x, y, color) in self.overlay.iter() {
            crate::renderer::draw_square(
                x + dx,
                y + dy,
                color.into(),
                GRID_SIZE,
                render_context.context,
                render_context.graphics,
            );
        }
        for (x, y, color) in self.selection.iter() {
            crate::renderer::draw_square(
                x + dx,
                y + dy,
                color.into(),
                GRID_SIZE,
                render_context.context,
                render_context.graphics,
            );
        }
    }
}

struct StatBar {
    name: String,
    current: i32,
    max: i32,
    color: Color,
}

impl StatBar {
    fn update(&mut self, current: i32, max: i32) {
        self.current = current.max(0);
        self.max = max;
    }

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
        if self.max <= 0 {
            return;
        }

        let text = format!("{} ({}/{})", self.name, self.current, self.max);
        let max_width = (GRID_SIZE * 10) as f64;
        let origin_x = (origin.0 * GRID_SIZE as i32) as f64;
        let origin_y = (origin.1 * GRID_SIZE as i32) as f64;
        let ratio = self.current as f64 / self.max as f64;

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

        crate::renderer::draw_text(
            origin.0 + 11,
            origin.1,
            0,
            WHITE.into(),
            GRID_SIZE,
            &text.as_str(),
            glyph_cache,
            context,
            graphics,
        )
        .ok();
    }
}

struct Hud {
    width: i32,
    height: i32,
    health_bar: StatBar,
    tooltip: Option<String>,
    journal_entries: VecDeque<String>,
}

impl Hud {
    pub fn new(width: i32, height: i32) -> Self {
        Hud {
            width,
            height,
            health_bar: StatBar {
                name: String::from("Health"),
                color: palette::HEALTH,
                current: 0,
                max: 0,
            },
            tooltip: None,
            journal_entries: VecDeque::new(),
        }
    }

    pub fn set_tooltip<S: Into<String>>(&mut self, tooltip: Option<S>) {
        self.tooltip = tooltip.map(|tooltip| tooltip.into());
    }

    pub fn update_journal(&mut self, journal: &Journal) {
        self.journal_entries.clear();
        for entry in journal.get_entries().iter().take(5) {
            self.journal_entries.push_front(entry.clone());
        }
    }
}

impl Renderable for Hud {
    fn position(&self) -> (i32, i32) {
        (0, 0)
    }

    fn size(&self) -> (i32, i32) {
        (self.width, self.height)
    }

    fn render<'a, C, G>(&self, render_context: &mut RenderContext<'a, C, G>)
    where
        C: CharacterCache,
        G: Graphics<Texture = <C as CharacterCache>::Texture>,
    {
        crate::renderer::draw_rectangle(
            (0, 0),
            (self.width, 3),
            DARK_GREY.into(),
            GRID_SIZE,
            render_context.context,
            render_context.graphics,
        );

        crate::renderer::draw_rectangle(
            (0, self.height - 7),
            (self.width, 7),
            DARK_GREY.into(),
            GRID_SIZE,
            render_context.context,
            render_context.graphics,
        );

        self.health_bar.render(
            render_context.graphics,
            render_context.character_cache,
            render_context.context,
            (1, 1),
        );

        if let Some(tooltip) = &self.tooltip {
            crate::renderer::draw_text(
                self.width / 2,
                1,
                10,
                WHITE.into(),
                GRID_SIZE,
                tooltip.as_str(),
                render_context.character_cache,
                render_context.context,
                render_context.graphics,
            )
            .ok();
        }

        let max_log = 5;
        let mut y = self.height as i32 - max_log - 1;

        for log in self.journal_entries.iter() {
            crate::renderer::draw_text(
                1,
                y,
                50,
                WHITE.into(),
                GRID_SIZE,
                log.as_str(),
                render_context.character_cache,
                render_context.context,
                render_context.graphics,
            )
            .ok();

            y += 1;
        }
    }
}
