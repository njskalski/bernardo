use crate::io::output::Metadata;
use crate::mocks::meta_frame::MetaOutputFrame;

pub struct SaveFileInterpreter<'a> {
    meta: &'a Metadata,
    output: &'a MetaOutputFrame,
}

impl<'a> SaveFileInterpreter<'a> {
    pub fn new(meta: &'a Metadata, output: &'a MetaOutputFrame) -> Self {
        Self {
            meta,
            output,
        }
    }
}