use std::fmt::Debug;
use std::rc::Rc;

pub trait ListWidgetItem: Debug + Clone {
    //TODO change to static str?
    fn get_column_name(idx: usize) -> &'static str;
    fn get_min_column_width(idx: usize) -> u16;
    fn len_columns() -> usize;
    fn get(&self, idx: usize) -> Option<Rc<String>>;
}
