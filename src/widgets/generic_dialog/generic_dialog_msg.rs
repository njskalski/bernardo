use crate::AnyMsg;

#[derive(Debug)]
pub enum GenericDialogMsg {
    Left,
    Right,
    Cancel,
    Hit(usize),
    JustPassMessage(Box<dyn AnyMsg>),
}

impl AnyMsg for GenericDialogMsg {}
