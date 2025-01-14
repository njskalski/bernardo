/*
Handler is a wrapper that translates language specific project definition into common project
items, like run configurations, test targets, and LSP clients.
 */

use std::ops::Deref;
use std::sync::Arc;

use crate::tsw::lang_id::LangId;
use crate::w7e::navcomp_provider::NavCompProvider;

// TODO this might become a more complex type, so all methods on it can be sync, but they are
// executed asynchronously by affiliated task. Though it does sound like just another layer of
// thread over LSP thread, so NOT SURE.
#[derive(Clone, Debug)]
pub struct NavCompRef(Arc<Box<dyn NavCompProvider>>);

impl NavCompRef {
    pub fn new(inner: Box<dyn NavCompProvider>) -> Self {
        NavCompRef(Arc::new(inner))
    }
}

pub trait Handler {
    fn lang_id(&self) -> LangId;
    fn handler_id(&self) -> &'static str;
    fn project_name(&self) -> &str;

    fn navcomp(&self) -> Option<NavCompRef> {
        None
    }
}

impl Deref for NavCompRef {
    type Target = Box<dyn NavCompProvider>;

    fn deref(&self) -> &Self::Target {
        self.0.deref()
    }
}
