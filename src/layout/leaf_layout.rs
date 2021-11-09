use std::slice::Iter;

use log::warn;

use crate::experiments::focus_group::FocusUpdate;
use crate::io::output::Output;
use crate::layout::layout::{Layout, WidgetGetter, WidgetGetterMut, WidgetIdRect};
use crate::primitives::rect::Rect;
use crate::primitives::xy::{XY, Zero};
use crate::widget::widget::{WID, Widget};

/*
This is the leaf of Layout tree. It corresponds to a single widget.
 */

pub struct LeafLayout<W: Widget> {
    wg: WidgetGetter<W>,
    wgmut: WidgetGetterMut<W>,
}

impl<W: Widget> LeafLayout<W> {
    pub fn new(wg: WidgetGetter<W>, wgmut: WidgetGetterMut<W>) -> Self {
        LeafLayout { wg, wgmut }
    }
}

impl<W: Widget> Layout<W> for LeafLayout<W> {
    fn is_leaf(&self) -> bool {
        true
    }

    fn min_size(&self, owner: &W) -> XY {
        let widget: &dyn Widget = (self.wg)(owner);
        widget.min_size()
    }

    fn sizes(&mut self, owner_mut: &mut W, output_size: XY) -> Vec<WidgetIdRect> {
        let widget: &mut dyn Widget = (self.wgmut)(owner_mut);
        let size = widget.layout(output_size);


        vec![WidgetIdRect {
            wid: widget.id(),
            rect: Rect::new(XY::new(0, 0), size),
        }]
    }

    fn render(&self, owner: &W, focused_id: Option<WID>, output: &mut Output) {
        let widget: &dyn Widget = (self.wg)(owner);

        if output.size() >= widget.min_size() {
            widget.render(focused_id == Some(widget.id()), output);
        } else {
            warn!(
                "output.size() smaller than widget.min_size() for widget {} ({:?} < {:?})",
                widget.id(),
                output.size(),
                widget.min_size(),
            );
        }
    }
}
