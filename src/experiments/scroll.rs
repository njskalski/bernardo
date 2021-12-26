use crate::primitives::xy::{XY, ZERO};
use crate::{Output, Theme, Widget};
use crate::io::over_output::OverOutput;

pub enum ScrollDirection {
    Horizontal,
    Vertical,
    Both
}

pub struct Scroll {
    max_size : XY,
    offset : XY,
    direction : ScrollDirection
}

impl Scroll {
    pub fn new(max_size : XY) -> Self {
        Scroll {
            max_size,
            offset : ZERO,
            direction : ScrollDirection::Vertical
        }
    }

    pub fn render_within<W : Widget>(&self, output : &mut dyn Output, widget : &W, theme: &Theme, focused: bool) {
        let mut over_output = OverOutput::new(output, self.offset, self.max_size);
        widget.render(theme, focused, &mut over_output)
    }
}