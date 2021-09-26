use std::iter;

pub trait Buffer {
    fn len_lines(&self) -> usize;
    fn lines(&self) -> Box<dyn iter::Iterator<Item=&str> + '_>;
}