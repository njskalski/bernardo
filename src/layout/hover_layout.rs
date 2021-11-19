// some would call this a "modal" layout, that is:
// there's a background layout and foreground layout.
// Background is visible but inactive, foreground is visible.

use crate::io::output::Output;
use crate::layout::layout::{Layout, WidgetIdRect};
use crate::primitives::theme::Theme;
use crate::primitives::xy::XY;

struct HoverLayout {}

impl Layout for HoverLayout {
    fn min_size(&self) -> XY {
        todo!()
    }

    fn calc_sizes(&mut self, output_size: XY) -> Vec<WidgetIdRect> {
        todo!()
    }

    fn draw_border(&self, theme: &Theme, focused: bool, output: &mut Output) {
        todo!()
    }
}