use std::iter::Iterator;

use crate::experiments::focus_group::FocusUpdate;
use crate::io::output::Output;
use crate::primitives::rect::Rect;
use crate::primitives::xy::XY;
use crate::widget::widget::{WID, Widget};

pub type WidgetGetter<T: Widget> = Box<dyn Fn(&'_ T) -> &'_ dyn Widget>;
pub type WidgetGetterMut<T: Widget> = Box<dyn Fn(&'_ mut T) -> &'_ mut dyn Widget>;


#[derive(Clone, Copy, Debug)]
pub struct WidgetIdRect {
    pub wid: WID,
    pub rect: Rect,
}

#[derive(Clone, Copy)]
pub struct WidgetRect<'a> {
    pub widget: &'a dyn Widget,
    pub rect: Rect,
}

impl<'a> WidgetRect<'a> {
    pub fn new(widget: &'a dyn Widget, rect: Rect) -> Self {
        WidgetRect { widget, rect }
    }
}

pub trait Layout<'a> {
    fn is_leaf(&self) -> bool {
        false
    }

    fn min_size(&self) -> XY;

    /*
    This only calculates the rects under current constraints. The widgets themselves should
    receive information about their new sizes before render.
     */
    fn calc_sizes(&'a mut self, output_size: XY) -> Vec<WidgetRect<'a>>;


    // fn render(&self, owner: &W, focused_id: Option<WID>, output: &mut Output);
    //
    // fn boxed(self) -> Box<dyn Layout<W>> where Self: Sized {
    //     Box::new(self)
    // }
}
