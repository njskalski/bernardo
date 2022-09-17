/*
    Keep the provider light
 */
use std::fmt::Debug;

use crate::widgets::list_widget::list_widget_item::ListWidgetItem;

pub trait ListWidgetProvider<Item: ListWidgetItem>: Debug {
    fn len(&self) -> usize;
    fn get(&self, idx: usize) -> Option<Item>;
}

impl<Item: ListWidgetItem> ListWidgetProvider<Item> for Vec<Item> {
    fn len(&self) -> usize {
        <[Item]>::len(self)
    }

    fn get(&self, idx: usize) -> Option<Item> {
        // // Vec::get(self, idx)
        // Some(self[idx].clone())
        // let self_as_vec: &Vec<Item> = self as &Vec<Item>;
        // self_as_vec.get(idx)
        <[Item]>::get(self, idx).map(|f| f.clone())
    }
}

struct ProviderIter<'a, Item: ListWidgetItem> {
    p: &'a dyn ListWidgetProvider<Item>,
    idx: usize,
}

impl<'a, LItem: ListWidgetItem> Iterator for ProviderIter<'a, LItem> {
    type Item = LItem;

    fn next(&mut self) -> Option<Self::Item> {
        if self.idx >= self.p.len() {
            None
        } else {
            let item = self.p.get(self.idx);
            self.idx += 1;
            item
        }
    }

    fn count(self) -> usize where Self: Sized {
        self.p.len()
    }
}

impl<Item: ListWidgetItem> dyn ListWidgetProvider<Item> {
    pub fn iter(&self) -> impl std::iter::Iterator<Item=Item> + '_ {
        ProviderIter {
            p: self,
            idx: 0,
        }
    }
}