use crate::io::output::Output;
use crate::io::style::{Effect, TextStyle};
use crate::primitives::color::Color;
use crate::primitives::rect::Rect;
use crate::primitives::xy::XY;

pub fn fill_rect(color: Color, rect: &Rect, output: &mut dyn Output) {
    let style = TextStyle::new(
        Color::new(0, 0, 0),
        color,
        Effect::None,
    );

    for x in rect.pos.x..rect.lower_right().x {
        for y in rect.pos.y..rect.lower_right().y {
            output.print_at(
                XY::new(x, y),
                style,
                " ",
            )
        }
    }
}