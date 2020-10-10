use graphics::{
    character::CharacterCache, types::ColorComponent, Context, Graphics, Image, Transformed,
};

/// Draw a character, and center it in the grid.
pub fn draw_char<C, G>(
    x: i32,
    y: i32,
    color: [ColorComponent; 4],
    grid_size: u32,
    character: char,
    glyph_cache: &mut C,
    context: Context,
    graphics: &mut G,
) -> Result<(), C::Error>
where
    C: CharacterCache,
    G: Graphics<Texture = <C as CharacterCache>::Texture>,
{
    let character = glyph_cache.character(grid_size, character)?;
    let font_adjust_x = (grid_size as f64 - character.atlas_size[0]) / 2.0;
    let font_adjust_y = (grid_size as f64 - character.atlas_size[1]) / 2.0;

    let mut image = Image::new_color(color);

    image = image.src_rect([
        character.atlas_offset[0],
        character.atlas_offset[1],
        character.atlas_size[0],
        character.atlas_size[1],
    ]);
    image.draw(
        character.texture,
        &Default::default(),
        context.transform.trans(
            x as f64 * grid_size as f64 + font_adjust_x,
            y as f64 * grid_size as f64 + font_adjust_y,
        ),
        graphics,
    );

    Ok(())
}

/// Draw a string.
pub fn draw_text<C, G>(
    x: i32,
    y: i32,
    _max_width: u32,
    color: [ColorComponent; 4],
    grid_size: u32,
    text: &str,
    glyph_cache: &mut C,
    context: Context,
    graphics: &mut G,
) -> Result<(), C::Error>
where
    C: CharacterCache,
    G: Graphics<Texture = <C as CharacterCache>::Texture>,
{
    let mut x = x as f64;
    let font_size = (grid_size as f64 * 0.9) as u32;

    // Get tallest char for vertical centering.
    let i = glyph_cache.character(font_size, 'I')?;
    let i_font_top = i.top() + (grid_size as f64 - i.atlas_size[1]) / 2.0;

    for (index, ch) in text.chars().enumerate() {
        let character = glyph_cache.character(font_size, ch)?;
        let font_adjust_x = (grid_size as f64 - character.atlas_size[0]) / 2.0;
        let font_adjust_y = i_font_top - character.top();

        if index == 0 {
            x = x as f64 * grid_size as f64 + font_adjust_x;
        }

        let mut image = Image::new_color(color);

        image = image.src_rect([
            character.atlas_offset[0],
            character.atlas_offset[1],
            character.atlas_size[0],
            character.atlas_size[1],
        ]);
        image.draw(
            character.texture,
            &Default::default(),
            context
                .transform
                .trans(x, y as f64 * grid_size as f64 + font_adjust_y),
            graphics,
        );

        x += character.advance_width();
    }

    Ok(())
}

pub fn draw_square<G>(
    x: i32,
    y: i32,
    color: [ColorComponent; 4],
    grid_size: u32,
    context: Context,
    graphics: &mut G,
) where
    G: Graphics,
{
    let x = x as f64 * grid_size as f64;
    let y = y as f64 * grid_size as f64;
    let square = graphics::rectangle::square(0.0, 0.0, grid_size as f64);
    graphics::rectangle(color, square, context.transform.trans(x, y), graphics);
}

pub fn draw_rectangle<G>(
    origin: (i32, i32),
    size: (i32, i32),
    color: [ColorComponent; 4],
    grid_size: u32,
    context: Context,
    graphics: &mut G,
) where
    G: Graphics,
{
    let x = origin.0 as f64 * grid_size as f64;
    let y = origin.1 as f64 * grid_size as f64;
    let width = size.0 as f64 * grid_size as f64;
    let height = size.1 as f64 * grid_size as f64;

    graphics::rectangle(color, [x, y, width, height], context.transform, graphics);
}
