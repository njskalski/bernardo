use crate::experiments::subwidget_pointer::SubwidgetPointer;
use crate::layout::widget_with_rect::WidgetWithRect;
use crate::primitives::rect::Rect;
use crate::primitives::size_constraint::SizeConstraint;
use crate::primitives::xy::XY;
use crate::widget::widget::{WID, Widget};

pub type WidgetGetter<T> = Box<dyn Fn(&'_ T) -> &'_ dyn Widget>;
pub type WidgetGetterMut<T> = Box<dyn Fn(&'_ mut T) -> &'_ mut dyn Widget>;

/* TODO I want to get to the point where all layout is generated from macros, and then
    depending on whether root is mut or not, we get mut layout or not-mut layout. */

pub struct LayoutResult<W: Widget> {
    pub wwrs: Vec<WidgetWithRect<W>>,
    pub total_size: XY,
}

impl<W: Widget> LayoutResult<W> {
    pub fn new(wwrs: Vec<WidgetWithRect<W>>, total_size: XY) -> LayoutResult<W> {
        for w in wwrs.iter() {
            debug_assert!(total_size >= w.rect().lower_right());
        }

        LayoutResult {
            wwrs,
            total_size,
        }
    }
}

/*
 Layout will SKIP a widget, if it's widget.id() == root.id()!
 */
pub trait Layout<W: Widget> {
    fn min_size(&self, root: &W) -> XY;

    /*
    Current semantics: returns wwrs that intersect with "SizeConstraint" and have non empty visible
     intersection. So it does culling and layouting.

     Why culling? Because I need "visible rect" calculation for centering anyways, so it was free.
     Why not caching? Because contents of widget may change without "update" method call.
     */
    fn layout(&self, root: &mut W, sc: SizeConstraint) -> LayoutResult<W>;

    fn boxed(self) -> Box<dyn Layout<W>> where Self: Sized, Self: 'static {
        Box::new(self)
    }
}
