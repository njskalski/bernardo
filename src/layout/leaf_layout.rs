use log::{debug, error};

use crate::experiments::subwidget_pointer::SubwidgetPointer;
use crate::layout::layout::{Layout, LayoutResult};
use crate::layout::widget_with_rect::WidgetWithRect;
use crate::primitives::rect::Rect;
use crate::primitives::xy::XY;
use crate::widget::fill_policy::{DeterminedBy, SizePolicy};
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

    fn exact_size(&self, root: &W, output_size: XY) -> XY {
        let widget = self.widget.get(root);

        let size_policy = widget.size_policy();
        if size_policy == SizePolicy::MATCH_LAYOUT {
            return output_size;
        }

        let widget_full_size = widget.full_size();
        let widget_output_size = XY::new(
            if size_policy.x == DeterminedBy::Widget {
                widget_full_size.x
            } else {
                output_size.x
            },
            if size_policy.y == DeterminedBy::Widget {
                widget_full_size.y
            } else {
                output_size.y
            },
        );
        widget_output_size
    }

    fn layout(&self, root: &mut W, output_size: XY, visible_rect: Rect) -> LayoutResult<W> {
        let widget_output_size = self.exact_size(root, output_size);

        let root_id = root.id();
        let root_desc = format!("{:?}", root as &dyn Widget);
        let widget = self.widget.get_mut(root);
        let skip = root_id == widget.id();

        if skip {
            return LayoutResult::new(vec![], XY::ZERO);
        }

        let widget_desc = format!("W{}{}", widget.typename(), widget.id());
        // let widget_full_size = widget.full_size();
        //
        // if !(widget_full_size <= output_size) {
        //     error!("can't fit widget {}, required {} scaled to {} and got max {}", &widget_desc, widget_full_size, widget_output_size, output_size);
        //     return LayoutResult::new(vec![], widget_output_size);
        // }

        debug_assert!(widget_output_size <= output_size);

        if let Some(widget_visible_rect) = visible_rect.capped_at(widget_output_size) {
            widget.layout(widget_output_size, widget_visible_rect);

            debug!("leaf layout for {}, returning {}", &widget_desc, widget_output_size);

            LayoutResult::new(
                vec![WidgetWithRect::new(
                    self.widget.clone(),
                    Rect::new(XY::ZERO, widget_output_size),
                    true,
                )],
                widget_output_size)
        } else {
            debug!("leaf layout for {} CULLED, returning {}", &widget_desc, widget_output_size);

            LayoutResult::new(
                vec![],
                widget_output_size)
        }
    }
}

