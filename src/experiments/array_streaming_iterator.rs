use streaming_iterator::StreamingIterator;

pub struct ArrayStreamingIt<'a, T> {
    array: &'a [T],
    pos: usize,
}

impl<'a, T> ArrayStreamingIt<'a, T> {
    pub fn new(array: &'a [T]) -> Self {
        Self {
            array,
            pos: 0,
        }
    }
}

impl<'a, T> StreamingIterator for ArrayStreamingIt<'a, T> {
    type Item = T;

    fn advance(&mut self) {
        self.pos += 1;
    }

    fn get(&self) -> Option<&Self::Item> {
        if self.pos < self.array.len() {
            Some(&self.array[self.pos])
        } else {
            None
        }
    }
}