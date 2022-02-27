pub trait IsDefault {
    fn is_default(&self) -> bool;
}

impl<D: Default + Eq> IsDefault for D {
    fn is_default(&self) -> bool {
        self == &D::default()
    }
}