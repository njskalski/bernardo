use std::marker::PhantomData;

use crate::io::output::Metadata;
use crate::mocks::meta_frame::MetaOutputFrame;
use crate::widget::widget::Widget;
use crate::widgets::with_scroll::with_scroll::WithScroll;

pub struct WithScrollWidgetInterpreter<'a, T: Widget> {
    meta: &'a Metadata,
    output: &'a MetaOutputFrame,
    _t: PhantomData<T>,
}

impl<'a, T: Widget> WithScrollWidgetInterpreter<'a, T> {
    pub fn new(output: &'a MetaOutputFrame, meta: &'a Metadata) -> Self {
        debug_assert!(meta.typename == WithScroll::TYPENAME);

        // Checking if we have this thing inside
        debug_assert!(output.get_meta_by_type(T::typename()).find(|item| {
            meta.rect.contains_rect(item.rect)
        }).is_some());

        Self {
            meta,
            output,
            _t: Default::default(),
        }
    }

    pub fn is_focused(&self) -> bool {
        self.meta.focused
    }
}