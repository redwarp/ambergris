use graphics::character::CharacterCache;
use legion::{component, Entity, IntoQuery, Read};
use piston_window::{Graphics, Key};
use std::collections::HashMap;

use crate::{
    components::Body, components::InInventory, components::Item, game::State,
    renderer::draw_window, renderer::RenderContext, renderer::Renderable,
};

struct InventoryLine {
    name: String,
    entities: Vec<Entity>,
}

pub struct Inventory {
    origin: (i32, i32),
    size: (i32, i32),
    items: HashMap<String, InventoryLine>,
    selected_line: i32,
}

impl Inventory {
    pub fn new(origin: (i32, i32), size: (i32, i32)) -> Self {
        Inventory {
            origin,
            size,
            items: HashMap::new(),
            selected_line: -1,
        }
    }

    pub fn list_items(&mut self, state: &State) {
        self.items.clear();
        for (entity, _item, body) in <(Entity, Read<Item>, Read<Body>)>::query()
            .filter(component::<InInventory>())
            .iter(&state.world)
        {
            if let Some(inventory_line) = self.items.get_mut(&body.name) {
                inventory_line.entities.push(entity.clone());
            } else {
                let inventory_line = InventoryLine {
                    name: body.name.clone(),
                    entities: vec![entity.clone()],
                };
                self.items.insert(body.name.clone(), inventory_line);
            }
        }
        if self.items.len() > 0 {
            self.selected_line = 0;
        }
    }

    pub fn set_mouse(&mut self, _mouse_position: [i32; 2]) {}

    pub fn on_keyboard(&mut self, key: &Key) -> InventoryAction {
        match key {
            Key::Up | Key::W => {
                self.selected_line = (self.selected_line - 1).max(0);
                InventoryAction::Select
            }
            Key::Down | Key::S => {
                self.selected_line = (self.selected_line + 1).min(self.items.len() as i32 - 1);
                InventoryAction::Select
            }
            Key::Escape => InventoryAction::Close,
            Key::Return | Key::NumPadEnter => self.pick(self.selected_line),
            Key::D1 | Key::NumPad1 => self.pick(0),
            Key::D2 | Key::NumPad2 => self.pick(1),
            Key::D3 | Key::NumPad3 => self.pick(2),
            Key::D4 | Key::NumPad4 => self.pick(3),
            Key::D5 | Key::NumPad5 => self.pick(4),
            Key::D6 | Key::NumPad6 => self.pick(5),
            Key::D7 | Key::NumPad7 => self.pick(6),
            Key::D8 | Key::NumPad8 => self.pick(7),
            Key::D9 | Key::NumPad9 => self.pick(8),
            _ => InventoryAction::Select,
        }
    }

    fn pick(&mut self, index: i32) -> InventoryAction {
        if index >= 0 && index < self.items.len() as i32 {
            let key = self.items.keys().nth(index as usize).unwrap().clone();
            let item = self.items.get_mut(&key).unwrap();

            if item.entities.len() > 0 {
                let entity = item.entities.pop().unwrap();
                if item.entities.len() == 0 {
                    self.items.remove(&key);
                }

                InventoryAction::Pick { entity }
            } else {
                InventoryAction::Select
            }
        } else {
            InventoryAction::Select
        }
    }
}

impl Renderable for Inventory {
    fn position(&self) -> (i32, i32) {
        self.origin
    }

    fn size(&self) -> (i32, i32) {
        self.size
    }

    fn render<'a, C, G>(&self, render_context: &mut RenderContext<'a, C, G>)
    where
        C: CharacterCache,
        G: Graphics<Texture = <C as CharacterCache>::Texture>,
    {
        draw_window(
            self.origin,
            self.size,
            "Inventory",
            render_context.grid_size,
            render_context.character_cache,
            render_context.context,
            render_context.graphics,
        );

        let mut y = 8;

        for (index, (_key, item)) in self.items.iter().enumerate() {
            if self.selected_line == index as i32 {
                crate::renderer::draw_rectangle(
                    (self.origin.0 + 1, y),
                    (self.size.0 - 2, 1),
                    crate::palette::OVERLAY.into(),
                    render_context.grid_size,
                    render_context.context,
                    render_context.graphics,
                )
            }

            let shortcut = if index < 9 {
                format!("{}", index + 1)
            } else {
                String::from(" ")
            };

            let text = match item.entities.len() {
                1 => format!("{shortcut}. {name}", shortcut = shortcut, name = item.name),
                _ => format!(
                    "{shortcut}. {name} (x{quantity})",
                    shortcut = shortcut,
                    name = item.name,
                    quantity = item.entities.len()
                ),
            };

            crate::renderer::draw_text(
                self.origin.0 + 1,
                y,
                10,
                crate::colors::WHITE.into(),
                render_context.grid_size,
                text.as_str(),
                render_context.character_cache,
                render_context.context,
                render_context.graphics,
            )
            .ok();
            y += 1;
        }
    }
}

pub enum InventoryAction {
    Close,
    Pick { entity: Entity },
    Select,
}
