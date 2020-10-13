use graphics::character::CharacterCache;
use legion::{component, IntoQuery, Read};
use piston_window::Graphics;
use std::collections::HashMap;

use crate::{
    components::Body, components::InInventory, components::Item, game::State,
    renderer::draw_window, renderer::RenderContext, renderer::Renderable,
};

struct InventoryLine {
    name: String,
    quantity: u32,
}

pub struct Inventory {
    origin: (i32, i32),
    size: (i32, i32),
    items: HashMap<String, InventoryLine>,
}

impl Inventory {
    pub fn new(origin: (i32, i32), size: (i32, i32)) -> Self {
        Inventory {
            origin,
            size,
            items: HashMap::new(),
        }
    }

    pub fn list_items(&mut self, state: &State) {
        self.items.clear();
        for (_item, body) in <(Read<Item>, Read<Body>)>::query()
            .filter(component::<InInventory>())
            .iter(&state.world)
        {
            if let Some(inventory_line) = self.items.get_mut(&body.name) {
                inventory_line.quantity += 1;
            } else {
                let inventory_line = InventoryLine {
                    name: body.name.clone(),
                    quantity: 1,
                };
                self.items.insert(body.name.clone(), inventory_line);
            }
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
            let text = match item.quantity {
                1 => format!("{index}. {name}", index = index, name = item.name),
                _ => format!(
                    "{index}. {name} (x{quantity})",
                    index = index,
                    name = item.name,
                    quantity = item.quantity
                ),
            };

            crate::renderer::draw_text(
                6,
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
