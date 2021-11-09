use crate::layout::layout::{Layout, WidgetIdRect};
use crate::primitives::xy::XY;
use crate::widget::widget::Widget;

//TODO: more advanced option would store references to widgets instead of their WIDs.
// I'll consider that in a next step.

#[derive(Clone, Debug)]
pub struct CachedSizes {
    pub for_size : XY,
    pub widget_sizes : Vec<WidgetIdRect>
}

impl CachedSizes {
    pub fn new(for_size : XY, widget_sizes : Vec<WidgetIdRect>) -> Self {
        CachedSizes {
            for_size,
            widget_sizes,
        }
    }
}