use std::borrow::Borrow;
use std::cell::{Cell, RefCell};
use std::ops::Deref;
use std::sync::{Arc, Mutex};
use shaku::{Interface, Component, module, HasComponent};

pub trait ClipboardComponent: Interface {
    fn get(&self) -> String;
    fn set(&self, s: String) -> bool;
}

#[derive(Component)]
#[shaku(interface = ClipboardComponent)]
pub struct LocalClipboardImpl {}

impl ClipboardComponent for LocalClipboardImpl {
    fn get(&self) -> String {
        "chuj".to_owned()
    }

    fn set(&self, s: String) -> bool {
        true
    }
}

module! {
    SomeModule {
        components = [LocalClipboardImpl],
        providers = []
    }
}

pub fn x() {
    let m = SomeModule::builder().build();

    let x: &dyn ClipboardComponent = m.resolve_ref();
}

#[cfg(test)]
mod tests {
    use crate::experiments::clipboard::SomeModule;

    #[test]
    fn bst() {
        assert_eq!(std::any::type_name::<SomeModule>(), "")
    }
}