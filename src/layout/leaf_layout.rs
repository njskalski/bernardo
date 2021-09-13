use crate::experiments::focus_group::FocusUpdate;
use crate::io::output::Output;
use crate::layout::layout::{Layout, WidgetGetter, WidgetGetterMut, WidgetIdRect};
use crate::primitives::rect::Rect;
use crate::primitives::xy::{Zero, XY};
use crate::widget::widget::{Widget, WID};
use log::warn;
use std::slice::Iter;

/*
This is the leaf of Layout tree. It corresponds to a single widget.

To understand the docs below, see Widget::render docs first.

// the stuff below is not implemented yet, may be moved to scroll layout cause I might
want scrolling over split layout or sth like this.

If a fixed size is declared, the widget is drawn *as if given output of that size*.
The frame_offset remains unchanged.
This is to facilitate scrolling.

If no fixed size is provided, the widget is given the size of the output (again it has to care
for frame_offset as well).

Now a widget when given a size can decide whether to draw itself, draw itself partially or
just draw a placeholder indicating that without more space it's not drawing itself properly.
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
    fn get_focused<'a>(&self, parent: &'a W) -> &'a dyn Widget {
        (self.wg)(parent)
    }

    fn get_focused_mut<'a>(&self, parent: &'a mut W) -> &'a mut dyn Widget {
        (self.wgmut)(parent)
    }

    fn update_focus(&mut self, focus_update: FocusUpdate) -> bool {
        false
    }

    fn is_leaf(&self) -> bool {
        true
    }

    fn min_size(&self, owner: &W) -> XY {
        let widget: &dyn Widget = (self.wg)(owner);
        widget.min_size()
    }

    fn get_rects(&self, owner: &W, output_size: XY) -> Vec<WidgetIdRect> {
        let widget: &dyn Widget = (self.wg)(owner);
        let size = widget.size(output_size);

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
