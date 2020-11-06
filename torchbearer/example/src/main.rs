use graphics::{clear, rectangle, Image, Line, Transformed};
use piston_window::{
    Button, Flip, MouseCursorEvent, PistonWindow, PressEvent, RenderEvent, Texture, TextureContext,
    TextureSettings, WindowSettings,
};
use torchbearer::{path::astar_path_fourwaygrid, Map, Point};

const MAP_WIDTH: i32 = 20;
const MAP_HEIGHT: i32 = 20;
const SCALE: i32 = 24;

struct ExampleMap {
    width: i32,
    height: i32,
    walkable: Vec<bool>,
}

impl ExampleMap {
    fn new(width: i32, height: i32) -> Self {
        ExampleMap {
            width,
            height,
            walkable: vec![true; (width * height) as usize],
        }
    }

    fn set_walkable(&mut self, x: i32, y: i32, is_walkable: bool) {
        let index = (x + y * self.width) as usize;
        self.walkable[index] = is_walkable;
    }
}

impl Map for ExampleMap {
    fn dimensions(&self) -> (i32, i32) {
        (self.width, self.height)
    }

    fn is_transparent(&self, _x: i32, _y: i32) -> bool {
        unreachable!("We don't care")
    }

    fn is_walkable(&self, x: i32, y: i32) -> bool {
        let index = (x + y * self.width) as usize;
        self.walkable[index]
    }
}

fn main() {
    let mut map = ExampleMap::new(MAP_WIDTH, MAP_HEIGHT);

    let mut window: PistonWindow = WindowSettings::new(
        "Path finding",
        [(MAP_WIDTH * SCALE) as u32, (MAP_HEIGHT * SCALE) as u32],
    )
    .exit_on_esc(true)
    .resizable(false)
    .build()
    .expect("Failed to initialize the window");

    let sprites = find_folder::Search::ParentsThenKids(3, 3)
        .for_folder("sprites")
        .unwrap();
    let mut texture_context = TextureContext {
        factory: window.factory.clone(),
        encoder: window.factory.create_command_buffer().into(),
    };
    let image = Image::new().rect([0.0, 0.0, SCALE as f64, SCALE as f64]);
    let goblin_texture = Texture::from_path(
        &mut texture_context,
        sprites.join("goblin.png"),
        Flip::None,
        &TextureSettings::new(),
    )
    .unwrap();

    let target_texture = Texture::from_path(
        &mut texture_context,
        sprites.join("target.png"),
        Flip::None,
        &TextureSettings::new(),
    )
    .unwrap();

    let rect = [0.0, 0.0, SCALE as f64, SCALE as f64];

    let from = (2, 2);
    let to = (12, 8);

    let mut dirty = true;

    let mut lines = vec![];
    let mut mouse_position = [0.0, 0.0];

    while let Some(event) = window.next() {
        if let Some(_args) = event.render_args() {
            if dirty {
                lines = if let Some(path) = astar_path_fourwaygrid(&map, from, to) {
                    path_to_line_elements(path)
                } else {
                    vec![]
                };
                dirty = false;
            }

            window.draw_2d(&event, |context, graphics, _device| {
                clear([0.9, 0.8, 0.9, 1.0], graphics);

                // Draw walls
                for x in 0..map.width {
                    for y in 0..map.height {
                        if !map.is_walkable(x, y) {
                            let translate_x = (SCALE * x) as f64;
                            let translate_y = (SCALE * y) as f64;

                            rectangle(
                                [0.3, 0.0, 0.0, 1.0],
                                rect,
                                context.transform.trans(translate_x, translate_y),
                                graphics,
                            );
                        }
                    }
                }

                // Draw path
                let line = Line::new_round([1.0, 0.0, 0.0, 1.0], (SCALE / 10) as f64);
                for &coords in lines.iter() {
                    line.draw(coords, &Default::default(), context.transform, graphics);
                }

                // Draw from position
                let translate_x = (SCALE * from.0) as f64;
                let translate_y = (SCALE * from.1) as f64;
                image.draw(
                    &goblin_texture,
                    &Default::default(),
                    context.transform.trans(translate_x, translate_y),
                    graphics,
                );

                // Draw to position
                let translate_x = (SCALE * to.0) as f64;
                let translate_y = (SCALE * to.1) as f64;
                image.draw(
                    &target_texture,
                    &Default::default(),
                    context.transform.trans(translate_x, translate_y),
                    graphics,
                );
            });
        }

        if let Some(button) = event.press_args() {
            if let Button::Mouse(_mouse) = button {
                let x = (mouse_position[0] / SCALE as f64) as i32;
                let y = (mouse_position[1] / SCALE as f64) as i32;

                if x < 0 || y < 0 || x >= MAP_WIDTH || y >= MAP_HEIGHT {
                    continue;
                }

                let is_walkable = map.is_walkable(x, y);
                map.set_walkable(x, y, !is_walkable);

                dirty = true;
            }
        }

        event.mouse_cursor(|position| {
            mouse_position = position;
        });
    }
}

fn path_to_line_elements(path: Vec<Point>) -> Vec<[f64; 4]> {
    let pair = path
        .into_iter()
        .fold((vec![], None), |(mut acc, previous), val| {
            if let Some(previous) = previous {
                acc.push((previous, val));
            }

            (acc, Some(val))
        })
        .0;
    pair.into_iter()
        .map(|(a, b)| {
            [
                (a.0 as f64 + 0.5) * SCALE as f64,
                (a.1 as f64 + 0.5) * SCALE as f64,
                (b.0 as f64 + 0.5) * SCALE as f64,
                (b.1 as f64 + 0.5) * SCALE as f64,
            ]
        })
        .collect()
}
