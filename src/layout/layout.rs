use crate::experiments::subwidget_pointer::SubwidgetPointer;
use crate::layout::widget_with_rect::WidgetWithRect;
use crate::primitives::rect::Rect;
use crate::primitives::size_constraint::SizeConstraint;
use crate::primitives::xy::XY;
use crate::widget::widget::{WID, Widget};

pub type WidgetGetter<T> = Box<dyn Fn(&'_ T) -> &'_ dyn Widget>;
pub type WidgetGetterMut<T> = Box<dyn Fn(&'_ mut T) -> &'_ mut dyn Widget>;

// TODO I want to get to the point where all layout is generated from macros, and then
// depending on whether root is mut or not, we get mut layout or not-mut layout.


/*
 Layouts do not work on infinite planes (scrolling of layouted view will fail).
 I might one day extend the definition, but it would require additional type to filter out layouts
 like "split".

 Layout will SKIP a widget, if it's widget.id() == root.id()!
 */
pub trait Layout<W: Widget> {
    fn min_size(&self, root: &W) -> XY;

    fn layout(&self, root: &mut W, sc: SizeConstraint) -> Vec<WidgetWithRect<W>>;

    fn boxed(self) -> Box<dyn Layout<W>> where Self: Sized, Self: 'static {
        Box::new(self)
    }
}
