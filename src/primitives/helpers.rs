use crate::io::output::Output;
use crate::io::style::{Effect, TextStyle};
use crate::primitives::color::Color;
use crate::primitives::xy::XY;

pub fn fill_background(color: Color, output: &mut dyn Output) {
    let style = TextStyle::new(
        Color::new(0, 0, 0),
        color,
        Effect::None,
    );

    for x in 0..output.size_constraint().hint().x {
        for y in 0..output.size_constraint().hint().y {
            output.print_at(
                XY::new(x, y),
                style,
                " ",
            )
        }
    }
}