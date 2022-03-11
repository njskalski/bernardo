use log::{error, warn};

use crate::io::sub_output::SubOutput;
use crate::layout::layout::{Layout, WidgetIdRect};
use crate::primitives::rect::Rect;
use crate::primitives::size_constraint::SizeConstraint;
use crate::primitives::xy::{XY, ZERO};
use crate::widget::widget::Widget;

pub struct FrameLayout<'a> {
    layout: &'a mut dyn Layout,
    margins: XY,
}

impl<'a> FrameLayout<'a> {
    pub fn new(layout: &'a mut dyn Layout, margins: XY) -> Self {
        Self {
            layout,
            margins,
        }
    }

    pub fn sub_rect(&self, size: XY) -> Option<Rect> {
        if size > self.margins * 2 {
            Some(Rect::new(self.margins, size - self.margins))
        } else {
            None
        }
    }
}

impl<'a> Layout for FrameLayout<'a> {
    fn is_leaf(&self) -> bool {
        true
    }

    fn min_size(&self) -> XY {
        self.layout.min_size() + self.margins * 2
    }


    fn calc_sizes(&mut self, output_size: XY) -> Vec<WidgetIdRect> {
        let rect = match self.sub_rect(output_size) {
            Some(rect) => rect,
            None => {
                error!("too small output to render with margins");
                return vec![];
            }
        };

        let subs = self.layout.calc_sizes(rect.size);

        subs.iter().map(|wir| {
            wir.shifted(self.margins)
        }).collect()
    }
}
