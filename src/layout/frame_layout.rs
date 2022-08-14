use log::error;

use crate::{Output, Theme, Widget};
use crate::layout::layout::{Layout, WidgetIdRect, WidgetWithRect};
use crate::primitives::rect::Rect;
use crate::primitives::xy::XY;

pub struct FrameLayout<W: Widget> {
    layout: Box<dyn Layout<W>>,
    margins: XY,
}

impl<W: Widget> FrameLayout<W> {
    pub fn new(layout: Box<dyn Layout<W>>, margins: XY) -> Self {
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

impl<W: Widget> Layout<W> for FrameLayout<W> {
    fn min_size(&self, root: &W) -> XY {
        self.layout.min_size(root) + self.margins * 2
    }

    fn calc_sizes(&self, root: &mut W, output_size: XY) -> Vec<WidgetIdRect> {
        let rect = match self.sub_rect(output_size) {
            Some(rect) => rect,
            None => {
                error!("too small output to render with margins");
                return vec![];
            }
        };

        let subs = self.layout.calc_sizes(root, rect.size);

        subs.iter().map(|wir| {
            wir.shifted(self.margins)
        }).collect()
    }

    fn layout(&self, root: &mut W, output_size: XY) -> Vec<WidgetWithRect<W>> {
        let rect = match self.sub_rect(output_size) {
            Some(rect) => rect,
            None => {
                error!("too small output to render with margins");
                return vec![];
            }
        };

        let subs = self.layout.layout(root, rect.size);

        subs.into_iter().map(|wir| {
            wir.shifted(self.margins)
        }).collect()
    }
}
