use std::marker::PhantomData;
use std::sync::Arc;

use streaming_iterator::StreamingIterator;

use crate::widgets::list_widget::list_widget_provider::ListWidgetProvider;

#[derive(Clone, Debug)]
pub struct ArcVecIter<S> {
    arc_vec: Arc<Vec<S>>,
    pos: usize,
}

impl<S> ArcVecIter<S> {
    pub fn new(arc_vec: Arc<Vec<S>>) -> Self {
        Self {
            arc_vec,
            pos: 0,
        }
    }
}

impl<S> StreamingIterator for ArcVecIter<S> {
    type Item = S;

    fn advance(&mut self) {
        if self.pos < self.arc_vec.len() {
            self.pos += 1;
        }
    }

    fn get(&self) -> Option<&Self::Item> {
        self.arc_vec.get(self.pos)
    }
}

#[derive(Clone, Debug)]
pub struct RefVecIter<'a, S> {
    vec: &'a Vec<S>,
    pos: usize,
}

impl<'a, S> RefVecIter<'a, S> {
    pub fn new(vec: &'a Vec<S>) -> Self {
        Self {
            vec,
            pos: 0,
        }
    }
}

impl<'a, S> StreamingIterator for RefVecIter<'a, S> {
    type Item = S;

    fn advance(&mut self) {
        if self.pos < self.vec.len() {
            self.pos += 1;
        }
    }

    fn get(&self) -> Option<&Self::Item> {
        self.vec.get(self.pos)
    }
}