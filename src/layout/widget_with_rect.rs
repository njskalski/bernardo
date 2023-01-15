use crate::experiments::subwidget_pointer::SubwidgetPointerOp;
use crate::primitives::rect::Rect;
use crate::primitives::xy::XY;
use crate::widget::widget::Widget;

pub struct WidgetWithRect<W: Widget> {
    widget: SubwidgetPointerOp<W>,
    rect: Rect,
    focusable: bool,
}

impl<W: Widget> Clone for WidgetWithRect<W> {
    fn clone(&self) -> Self {
        Self {
            widget: self.widget.clone(),
            rect: self.rect.clone(),
            focusable: self.focusable,
        }
    }
}

impl<W: Widget> WidgetWithRect<W> {
    pub fn new(widget: SubwidgetPointerOp<W>, rect: Rect, focusable: bool) -> Self {
        Self {
            widget,
            rect,
            focusable,
        }
    }

    pub fn rect(&self) -> &Rect {
        &self.rect
    }

    pub fn widget(&self) -> &SubwidgetPointerOp<W> {
        &self.widget
    }

    pub fn shifted(self, offset: XY) -> Self {
        Self {
            rect: self.rect.shifted(offset),
            ..self
        }
    }

    pub fn set_focusable(&mut self, focusable: bool) {
        self.focusable = focusable;
    }

    pub fn focusable(&self) -> bool {
        self.focusable
    }
}
