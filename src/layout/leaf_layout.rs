use log::warn;

use crate::experiments::focus_group::FocusUpdate;
use crate::io::output::Output;
use crate::layout::layout::{Layout, WidgetGetter, WidgetGetterMut, WidgetIdRect};
use crate::primitives::border;
use crate::primitives::border::SingleBorderStyle;
use crate::primitives::rect::Rect;
use crate::primitives::theme::Theme;
use crate::primitives::xy::{XY, ZERO};
use crate::widget::widget::{WID, Widget};

pub struct LeafLayout<'a> {
    widget: &'a mut dyn Widget,
    with_border: bool,
}

impl<'a> LeafLayout<'a> {
    pub fn new(widget: &'a mut dyn Widget) -> Self {
        LeafLayout { widget, with_border: false }
    }

    pub fn with_border(self) -> Self {
        LeafLayout {
            with_border: true,
            ..self
        }
    }
}

impl<'a> Layout for LeafLayout<'a> {
    fn is_leaf(&self) -> bool {
        true
    }

    fn min_size(&self) -> XY {
        self.widget.min_size()
    }

    fn calc_sizes(&mut self, output_size: XY) -> Vec<WidgetIdRect> {
        let wid = self.widget.id();

        if self.with_border {
            if output_size > (2, 2).into() {
                let limited_output = output_size - (2.2).into();
                let size = self.widget.layout(limited_output);
                let rect = Rect::new(XY::new(1, 1), size);

                vec![WidgetIdRect {
                    wid,
                    rect,
                }]
            } else {
                warn!("too small LeafLayout to draw the view.");
                vec![]
            }
        } else {
            let size = self.widget.layout(output_size);
            let rect = Rect::new(ZERO, size);

            vec![WidgetIdRect {
                wid,
                rect,
            }]
        }
    }

    fn draw_border(&self, theme: &Theme, focused: bool, output: &mut Output) {
        border::draw_full_rect(theme.default_text(),
                               &SingleBorderStyle,
                               output);
    }
}
