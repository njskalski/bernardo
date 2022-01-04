use crate::io::output::Output;
use crate::io::style::{Effect, TextStyle};
use crate::primitives::color::Color;
use crate::primitives::rect::Rect;
use crate::primitives::xy::XY;

pub fn fill_output(color: Color, output: &mut dyn Output) {
    let style = TextStyle::new(
        Color::new(0, 0, 0),
        color,
        Effect::None,
    );

    let rect = output.size_constraint().hint().clone();

    for x in rect.upper_left().x..rect.lower_right().x {
        for y in rect.upper_left().y..rect.lower_right().y {
            output.print_at(
                XY::new(x, y),
                style,
                " ",
            )
        }
    }
}