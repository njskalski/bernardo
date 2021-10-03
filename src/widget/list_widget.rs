pub enum ListWidgetCell {
    NotAvailable,
    Loading,
    Ready(String),
}

pub trait ListWidgetItem {
    fn get_column_name(idx: usize) -> String;
    //TODO change to static str?
    fn len_columns() -> usize;
    fn get(&self, idx: usize) -> ListWidgetCell;
}

