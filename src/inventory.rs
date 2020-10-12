use graphics::{character::CharacterCache, Context, Graphics};
use legion::{component, IntoQuery, Read};

use crate::{
    components::Body, components::InInventory, components::Item, game::State, renderer::draw_window,
};

pub struct Inventory {}

impl Inventory {
    pub fn list_items(&self, state: &State) {
        let items: Vec<(&Item, &Body)> = <(Read<Item>, Read<Body>)>::query()
            .filter(component::<InInventory>())
            .iter(&state.world)
            .collect::<Vec<_>>();
    }

    pub fn render<G, C>(
        &self,
        state: &State,
        window_size: (i32, i32),
        grid_size: u32,
        graphics: &mut G,
        context: Context,
        glyph_cache: &mut C,
    ) where
        C: CharacterCache,
        G: Graphics<Texture = <C as CharacterCache>::Texture>,
    {
        let (width, height) = window_size;
        draw_window(
            (5, 5),
            (width - 10, height - 10),
            "Inventory",
            grid_size,
            glyph_cache,
            context,
            graphics,
        );

        let items: Vec<(&Item, &Body)> = <(Read<Item>, Read<Body>)>::query()
            .filter(component::<InInventory>())
            .iter(&state.world)
            .collect::<Vec<_>>();

        let mut y = 8;
        for (_item, body) in items {
            crate::renderer::draw_text(
                6,
                y,
                width as u32,
                crate::colors::WHITE.into(),
                grid_size,
                body.name.as_str(),
                glyph_cache,
                context,
                graphics,
            )
            .ok();
            y += 1;
        }
    }
}
