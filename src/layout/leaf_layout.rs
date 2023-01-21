use log::{debug, error};

use crate::experiments::subwidget_pointer::SubwidgetPointer;
use crate::layout::layout::{Layout, LayoutResult};
use crate::layout::widget_with_rect::WidgetWithRect;
use crate::primitives::rect::Rect;
use crate::primitives::size_constraint::SizeConstraint;
use crate::primitives::xy::XY;
use crate::widget::widget::Widget;

pub struct LeafLayout<W: Widget> {
    widget: SubwidgetPointer<W>,
}

impl<W: Widget> LeafLayout<W> {
    pub fn new(widget: SubwidgetPointer<W>) -> Self {
        LeafLayout { widget }
    }
}

impl<W: Widget> Layout<W> for LeafLayout<W> {
    fn prelayout(&self, root: &mut W) {
        let widget = self.widget.get_mut(root);
        widget.prelayout();
    }

    fn min_size(&self, root: &W) -> XY {
        self.widget.get(root).size()
    }

    fn layout(&self, root: &mut W, sc: SizeConstraint) -> LayoutResult<W> {
        let root_id = root.id();
        let widget = self.widget.get_mut(root);
        let skip = root_id == widget.id();

        let widget_min_size = widget.size();
        let properly_sized = sc.bigger_equal_than(widget_min_size);
        let _widget_name = widget.typename();

        if !skip {
            if properly_sized {
                let xy = widget.layout(sc);

                debug_assert!(sc.bigger_equal_than(xy),
                              "widget {} #{} violated invariant",
                              widget.typename(), widget.id());

                debug!("leaf layout for {:?}, returning {}", widget.typename(), xy);

                LayoutResult::new(
                    vec![WidgetWithRect::new(
                        self.widget.clone(),
                        Rect::new(XY::ZERO, xy),
                        true,
                    )],
                    xy)
            } else {
                error!("got layout too small (sc: {:?}) to draw widget [{:?}] min_size {}, returning min_size instead, but this will lead to incorrect layouting", sc, widget, widget_min_size);
                LayoutResult::new(
                    vec![],
                    widget_min_size)
            }
        } else {
            LayoutResult::new(vec![], XY::ZERO)
        }
    }
}

