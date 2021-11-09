use crate::layout::layout::{Layout, WidgetIdRect};
use crate::primitives::xy::XY;
use crate::widget::widget::Widget;

pub struct CachedSizes {
    pub for_size : XY,
    pub widget_sizes : Vec<WidgetIdRect>
}

impl CachedSizes {
    pub fn new<W:Widget>(for_size : XY, widget_sizes : Vec<WidgetIdRect>) -> Self {
        CachedSizes {
            for_size,
            widget_sizes,
        }
    }
}