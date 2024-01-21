use crate::io::input_source::InputSource;

pub trait Input {
    fn source(&self) -> &InputSource;
}
