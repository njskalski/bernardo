/*
This is an abstract trait that will generate "labels". Label's are like tiny sticky notes you glue
to your whiteboard to annotate things. The obvious use is to enrich the output with information
like:

- type annotations that are introduced by LSP
- warnings emitted by compiler
- errors emitted by compiler
 */
use std::ops::Range;
use std::sync::Arc;

use crate::fs::path::SPath;
use crate::text::text_buffer::TextBuffer;
use crate::widgets::editor_widget::label::label::Label;

pub trait LabelsProvider: Sync + Send {
    fn query_for(&self,
                 path_op: Option<&SPath>) -> Box<dyn Iterator<Item=&'_ Label> + '_>;

    fn into_ref(self) -> LabelsProviderRef where Self: 'static + Sized {
        Arc::new(Box::new(self) as Box<dyn LabelsProvider>)
    }
}

pub type LabelsProviderRef = Arc<Box<dyn LabelsProvider + 'static>>;

